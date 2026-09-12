use crate::db::{Db, Task};
use crate::process::{self, ProcessHub};
use drain_core::{
    drain_task_prompt, extract_pr_url, normalize_branch, pr_body, CompletionMode, Status,
};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusEvent {
    pub task_id: Option<i64>,
    pub status: String,
    pub message: Option<String>,
}

pub struct QueueCtl {
    pub paused: AtomicBool,
    pub cancel: AtomicBool,
}

impl QueueCtl {
    pub fn new() -> Self {
        Self {
            paused: AtomicBool::new(true),
            cancel: AtomicBool::new(false),
        }
    }
}

pub fn emit_status(app: &AppHandle, task_id: Option<i64>, status: &str, message: Option<String>) {
    let _ = app.emit(
        "drain://status",
        StatusEvent {
            task_id,
            status: status.to_string(),
            message,
        },
    );
}

fn fail_task(db: &Db, app: &AppHandle, task_id: i64, note: &str) {
    let _ = db.set_task_status(task_id, Status::Failed, None, Some(note));
    let _ = db.append_log(task_id, "system", &format!("failed: {note}"));
    emit_status(app, Some(task_id), Status::Failed.as_str(), Some(note.into()));
}

fn unique_branch(repo: &Path, wanted: &str) -> String {
    let check = process::run_capture("git", &["rev-parse", "--verify", wanted], Some(repo));
    if check.map(|r| r.code != 0).unwrap_or(true) {
        return wanted.to_string();
    }
    for i in 2..30 {
        let candidate = format!("{wanted}-{i}");
        let check = process::run_capture("git", &["rev-parse", "--verify", &candidate], Some(repo));
        if check.map(|r| r.code != 0).unwrap_or(true) {
            return candidate;
        }
    }
    format!("{wanted}-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0))
}

fn default_branch(repo: &Path) -> String {
    if let Ok(r) = process::run_capture(
        "git",
        &["symbolic-ref", "refs/remotes/origin/HEAD"],
        Some(repo),
    ) {
        if r.code == 0 {
            if let Some(name) = r.stdout.trim().rsplit('/').next() {
                if !name.is_empty() {
                    return name.to_string();
                }
            }
        }
    }
    for cand in ["main", "master"] {
        if let Ok(r) = process::run_capture("git", &["rev-parse", "--verify", cand], Some(repo)) {
            if r.code == 0 {
                return cand.to_string();
            }
        }
    }
    "main".into()
}

fn require_ok(result: process::CmdResult, label: &str) -> Result<process::CmdResult, String> {
    if result.code != 0 {
        let err = if result.stderr.trim().is_empty() {
            result.stdout.trim().to_string()
        } else {
            result.stderr.trim().to_string()
        };
        Err(format!("{label} failed (exit {}): {err}", result.code))
    } else {
        Ok(result)
    }
}

fn commits_ahead(repo: &Path, base: &str) -> Result<i32, String> {
    let r = process::run_capture(
        "git",
        &["rev-list", "--count", &format!("{base}..HEAD")],
        Some(repo),
    )?;
    Ok(r.stdout.trim().parse().unwrap_or(0))
}

fn is_dirty(repo: &Path) -> Result<bool, String> {
    let r = process::run_capture("git", &["status", "--porcelain"], Some(repo))?;
    Ok(!r.stdout.trim().is_empty())
}

pub fn drain_one(
    app: &AppHandle,
    db: &Db,
    hub: &ProcessHub,
    ctl: &Arc<QueueCtl>,
    task: Task,
) {
    ctl.cancel.store(false, Ordering::SeqCst);
    let settings = match db.settings() {
        Ok(s) => s,
        Err(e) => {
            fail_task(db, app, task.id, &e);
            return;
        }
    };

    let repo = if !task.repo_path.trim().is_empty() {
        PathBuf::from(task.repo_path.trim())
    } else {
        PathBuf::from(settings.default_repo_path.trim())
    };
    if repo.as_os_str().is_empty() || !repo.exists() {
        fail_task(
            db,
            app,
            task.id,
            "no repo path — set default_repo_path in Settings or enqueue with a path",
        );
        return;
    }

    let _ = db.clear_logs(task.id);
    emit_status(app, Some(task.id), Status::Running.as_str(), None);

    let log = |line: &str| {
        let _ = db.append_log(task.id, "system", line);
        let _ = app.emit(
            "drain://log",
            process::LogEvent {
                task_id: task.id,
                stream: "system".into(),
                line: line.to_string(),
            },
        );
    };

    let run = |bin: &str, args: &[&str]| -> Result<process::CmdResult, String> {
        if ctl.cancel.load(Ordering::SeqCst) {
            return Err("cancelled".into());
        }
        let result = process::run_logged(hub, app, task.id, bin, args, Some(&repo), &[])?;
        for line in result.stdout.lines() {
            let _ = db.append_log(task.id, "stdout", line);
        }
        for line in result.stderr.lines() {
            let _ = db.append_log(task.id, "stderr", line);
        }
        Ok(result)
    };

    let git = "git";
    let gh = settings.gh_bin();
    let claude = settings.claude_bin();
    let permission = settings.permission();
    let mode = settings.completion();

    log("detect default branch");
    let base = default_branch(&repo);
    log(&format!("base branch: {base}"));

    let branch = unique_branch(
        &repo,
        &normalize_branch(&task.suggested_branch, &task.title),
    );
    log(&format!("working branch: {branch}"));

    let steps: Result<(), String> = (|| {
        if is_dirty(&repo)? {
            return Err("working tree is dirty — commit or stash before Drain starts".into());
        }
        require_ok(run(git, &["checkout", &base])?, "git checkout base")?;
        let pull = run(git, &["pull", "--ff-only"])?;
        if pull.code != 0 {
            log("git pull failed (continuing with local base)");
        }
        require_ok(run(git, &["checkout", "-b", &branch])?, "git checkout -b")?;

        let prompt = drain_task_prompt(
            &task.title,
            &task.acceptance_criteria,
            &task.out_of_scope,
            &branch,
            permission,
        );
        log(&format!(
            "claude -p  (permission-mode {})",
            permission.as_cli()
        ));
        let claude_result = process::run_logged(
            hub,
            app,
            task.id,
            &claude,
            &[
                "-p",
                &prompt,
                "--permission-mode",
                permission.as_cli(),
            ],
            Some(&repo),
            &[],
        )?;
        for line in claude_result.stdout.lines() {
            let _ = db.append_log(task.id, "stdout", line);
        }
        for line in claude_result.stderr.lines() {
            let _ = db.append_log(task.id, "stderr", line);
        }
        if ctl.cancel.load(Ordering::SeqCst) {
            return Err("cancelled".into());
        }
        if claude_result.code != 0 {
            return Err(format!(
                "claude exited {} — {}",
                claude_result.code,
                claude_result.stderr.lines().next().unwrap_or("see log")
            ));
        }

        require_ok(run(git, &["add", "-A"])?, "git add")?;
        if is_dirty(&repo)? {
            let commit = run(git, &["commit", "-m", &task.title])?;
            if commit.code != 0 {
                log("git commit failed — retrying with Drain identity");
                require_ok(
                    process::run_logged(
                        hub,
                        app,
                        task.id,
                        git,
                        &[
                            "-c",
                            "user.name=Drain",
                            "-c",
                            "user.email=drain@localhost",
                            "commit",
                            "-m",
                            &task.title,
                        ],
                        Some(&repo),
                        &[],
                    )?,
                    "git commit",
                )?;
            }
        }
        let ahead = commits_ahead(&repo, &base)?;
        if ahead == 0 && !is_dirty(&repo)? {
            return Err("no changes produced — nothing to push".into());
        }

        require_ok(
            run(git, &["push", "-u", "origin", "HEAD"])?,
            "git push",
        )?;

        let body = pr_body(&task.title, &task.acceptance_criteria, &task.out_of_scope);
        let body_path = std::env::temp_dir().join(format!("drain-pr-{}.md", task.id));
        fs::write(&body_path, &body).map_err(|e| e.to_string())?;
        let body_file = body_path.to_string_lossy().to_string();

        let _ = db.set_task_status(task.id, Status::AwaitingPr, None, None);
        emit_status(
            app,
            Some(task.id),
            Status::AwaitingPr.as_str(),
            None,
        );

        log("gh pr create");
        let pr = require_ok(
            process::run_logged(
                hub,
                app,
                task.id,
                &gh,
                &[
                    "pr",
                    "create",
                    "--title",
                    &task.title,
                    "--body-file",
                    &body_file,
                    "--head",
                    &branch,
                    "--base",
                    &base,
                ],
                Some(&repo),
                &[],
            )?,
            "gh pr create",
        )?;
        let combined = format!("{}\n{}", pr.stdout, pr.stderr);
        let url = extract_pr_url(&combined)
            .ok_or_else(|| format!("gh pr create succeeded but no URL found:\n{combined}"))?;
        log(&format!("pr: {url}"));

        match mode {
            CompletionMode::PrThenApprove => {
                let _ = db.set_task_status(
                    task.id,
                    Status::AwaitingApprove,
                    Some(&url),
                    None,
                );
                emit_status(
                    app,
                    Some(task.id),
                    Status::AwaitingApprove.as_str(),
                    Some(url),
                );
                log("stopped — waiting for Approve (will not merge, will not start next)");
            }
            CompletionMode::PushAndMerge => {
                log("completion mode: push_and_merge");
                require_ok(
                    process::run_logged(
                        hub,
                        app,
                        task.id,
                        &gh,
                        &["pr", "merge", &url, "--merge", "--delete-branch"],
                        Some(&repo),
                        &[],
                    )?,
                    "gh pr merge",
                )?;
                let _ = db.set_task_status(task.id, Status::Done, Some(&url), None);
                emit_status(app, Some(task.id), Status::Done.as_str(), Some(url));
            }
        }
        let _ = fs::remove_file(body_path);
        Ok(())
    })();

    if let Err(e) = steps {
        if e == "cancelled" {
            fail_task(db, app, task.id, "cancelled");
        } else {
            fail_task(db, app, task.id, &e);
        }
    }
}
