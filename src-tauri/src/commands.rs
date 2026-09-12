use crate::db::{Db, Idea, LogLine, Settings, Task};
use crate::process::{self, ProcessHub};
use crate::runner::{self, QueueCtl};
use drain_core::{
    parse_refine_output, refine_system_prompt, refine_user_prompt, can_reorder, RefineOutcome,
    RefinedTask, Status,
};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

pub struct AppState {
    pub db: Db,
    pub hub: ProcessHub,
    pub queue: Arc<QueueCtl>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryStatus {
    pub git: Option<String>,
    pub claude: Option<String>,
    pub gh: Option<String>,
}

#[tauri::command]
pub fn list_ideas(state: State<AppState>) -> Result<Vec<Idea>, String> {
    state.db.list_ideas()
}

#[tauri::command]
pub fn create_idea(state: State<AppState>, body: String) -> Result<Idea, String> {
    if body.trim().is_empty() {
        return Err("idea is empty".into());
    }
    state.db.create_idea(&body)
}

#[tauri::command]
pub fn get_idea(state: State<AppState>, id: i64) -> Result<Idea, String> {
    state.db.get_idea(id)
}

#[tauri::command]
pub fn update_idea_drafts(
    state: State<AppState>,
    id: i64,
    drafts: Vec<RefinedTask>,
) -> Result<Idea, String> {
    state.db.set_idea_drafts(id, &drafts)?;
    state.db.get_idea(id)
}

#[tauri::command]
pub fn update_idea_body(state: State<AppState>, id: i64, body: String) -> Result<Idea, String> {
    if body.trim().is_empty() {
        return Err("idea is empty".into());
    }
    state.db.set_idea_body(id, &body)?;
    state.db.get_idea(id)
}

#[tauri::command]
pub fn refine_idea(app: AppHandle, state: State<AppState>, id: i64) -> Result<(), String> {
    let idea = state.db.get_idea(id)?;
    state
        .db
        .set_idea_status(id, Status::Refining, None)?;
    runner::emit_status(&app, None, "idea_refining", Some(id.to_string()));

    let settings = state.db.settings()?;
    let claude = settings.claude_bin();
    let repo = PathBuf::from(settings.default_repo_path.trim());
    let context = if repo.exists() {
        process::repo_context(&repo)
    } else {
        String::new()
    };
    let prompt = format!(
        "{}\n\n{}",
        refine_system_prompt(),
        refine_user_prompt(&idea.body, &context)
    );

    let handle = app.clone();
    thread::spawn(move || {
        let Some(st) = handle.try_state::<AppState>() else {
            return;
        };
        runner::emit_status(&handle, None, "idea_trace", Some(format!("{id}:0")));
        let cwd = {
            let s = st.db.settings().ok();
            s.and_then(|s| {
                let p = PathBuf::from(s.default_repo_path.trim());
                if p.exists() {
                    Some(p)
                } else {
                    None
                }
            })
        };
        runner::emit_status(&handle, None, "idea_trace", Some(format!("{id}:1")));
        runner::emit_status(&handle, None, "idea_trace", Some(format!("{id}:2")));
        let result = process::run_capture(
            &claude,
            &["-p", &prompt],
            cwd.as_deref(),
        );
        runner::emit_status(&handle, None, "idea_trace", Some(format!("{id}:3")));
        match result {
            Ok(out) => {
                let combined = format!("{}\n{}", out.stdout, out.stderr);
                if out.code != 0 {
                    let note = format!(
                        "claude exited {} — {}",
                        out.code,
                        out.stderr.lines().next().unwrap_or("refine failed")
                    );
                    let _ = st.db.set_idea_status(id, Status::Idea, Some(&note));
                    runner::emit_status(&handle, None, "idea_failed", Some(note));
                    return;
                }
                match parse_refine_output(&combined) {
                    Ok(RefineOutcome::Tasks(tasks)) => {
                        let _ = st.db.set_idea_drafts(id, &tasks);
                        runner::emit_status(&handle, None, "idea_refined", Some(id.to_string()));
                    }
                    Ok(RefineOutcome::Refusal(reason)) => {
                        let _ = st.db.set_idea_status(id, Status::Idea, Some(&reason));
                        runner::emit_status(&handle, None, "idea_refused", Some(reason));
                    }
                    Err(e) => {
                        let _ = st.db.set_idea_status(id, Status::Idea, Some(&e));
                        runner::emit_status(&handle, None, "idea_failed", Some(e));
                    }
                }
            }
            Err(e) => {
                let _ = st.db.set_idea_status(id, Status::Idea, Some(&e));
                runner::emit_status(&handle, None, "idea_failed", Some(e));
            }
        }
    });
    Ok(())
}

#[tauri::command]
pub fn enqueue_idea(
    state: State<AppState>,
    id: i64,
    drafts: Vec<RefinedTask>,
    repo_path: Option<String>,
) -> Result<Vec<Task>, String> {
    if drafts.is_empty() {
        return Err("no tasks to enqueue".into());
    }
    let settings = state.db.settings()?;
    let repo = repo_path
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(settings.default_repo_path.clone());
    if repo.trim().is_empty() {
        return Err("set a default repo path in Settings before enqueueing".into());
    }
    let mut pos = state.db.max_position()? + 1;
    for d in &drafts {
        if d.acceptance_criteria.is_empty() {
            return Err(format!("`{}` needs at least one acceptance criterion", d.title));
        }
        state.db.insert_task(id, d, repo.trim(), pos)?;
        pos += 1;
    }
    state.db.set_idea_drafts(id, &drafts)?;
    state.db.list_tasks()
}

#[tauri::command]
pub fn list_tasks(state: State<AppState>) -> Result<Vec<Task>, String> {
    state.db.list_tasks()
}

#[tauri::command]
pub fn get_task(state: State<AppState>, id: i64) -> Result<Task, String> {
    state.db.get_task(id)
}

#[tauri::command]
pub fn reorder_tasks(state: State<AppState>, ids: Vec<i64>) -> Result<Vec<Task>, String> {
    can_reorder(state.db.running_task_id()?.is_some()).map_err(|e| e.to_string())?;
    state.db.reorder(&ids)?;
    state.db.list_tasks()
}

#[tauri::command]
pub fn start_queue(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    state.queue.paused.store(false, std::sync::atomic::Ordering::SeqCst);
    kick_queue(app, &state)
}

#[tauri::command]
pub fn pause_queue(state: State<AppState>) -> Result<(), String> {
    state.queue.paused.store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn queue_paused(state: State<AppState>) -> Result<bool, String> {
    Ok(state.queue.paused.load(std::sync::atomic::Ordering::SeqCst))
}

#[tauri::command]
pub fn cancel_run(state: State<AppState>) -> Result<(), String> {
    state.queue.cancel.store(true, std::sync::atomic::Ordering::SeqCst);
    let _ = state.hub.cancel();
    Ok(())
}

#[tauri::command]
pub fn approve_task(app: AppHandle, state: State<AppState>, id: i64) -> Result<Task, String> {
    let task = state.db.get_task(id)?;
    if task.status != Status::AwaitingApprove.as_str()
        && task.status != Status::AwaitingPr.as_str()
    {
        return Err("task is not waiting for approve".into());
    }
    let settings = state.db.settings()?;
    let repo = PathBuf::from(if task.repo_path.trim().is_empty() {
        settings.default_repo_path.clone()
    } else {
        task.repo_path.clone()
    });
    match settings.approve() {
        drain_core::ApproveAction::Merge => {
            let url = task
                .pr_url
                .clone()
                .ok_or_else(|| "no pull request URL stored".to_string())?;
            let gh = settings.gh_bin();
            let result = process::run_logged(
                &state.hub,
                &app,
                id,
                &gh,
                &["pr", "merge", &url, "--merge"],
                Some(&repo),
                &[],
            )?;
            if result.code != 0 {
                return Err(format!(
                    "gh pr merge failed: {}",
                    result.stderr.trim()
                ));
            }
        }
        drain_core::ApproveAction::OpenBrowser => {
            if let Some(url) = &task.pr_url {
                app.opener()
                    .open_url(url, None::<&str>)
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    state.db.set_task_status(id, Status::Done, None, None)?;
    runner::emit_status(&app, Some(id), Status::Done.as_str(), None);
    let settings = state.db.settings()?;
    let paused = state.queue.paused.load(std::sync::atomic::Ordering::SeqCst);
    if drain_core::should_auto_advance_on(Status::Done, settings.auto_run, paused) {
        let _ = kick_queue(app.clone(), &state);
    }
    state.db.get_task(id)
}

#[tauri::command]
pub fn reject_task(
    app: AppHandle,
    state: State<AppState>,
    id: i64,
    note: String,
) -> Result<Task, String> {
    let n = if note.trim().is_empty() {
        "rejected"
    } else {
        note.trim()
    };
    state
        .db
        .set_task_status(id, Status::Failed, None, Some(n))?;
    runner::emit_status(&app, Some(id), Status::Failed.as_str(), Some(n.into()));
    state.db.get_task(id)
}

#[tauri::command]
pub fn retry_task(state: State<AppState>, id: i64) -> Result<Task, String> {
    let task = state.db.get_task(id)?;
    if task.status != Status::Failed.as_str() {
        return Err("only a failed task can be retried".into());
    }
    let pos = state.db.max_position()? + 1;
    {
        let conn = state.db.0.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE tasks SET status = 'ready', fail_note = NULL, position = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![pos, chrono_now(), id],
        )
        .map_err(|e| e.to_string())?;
    }
    state.db.get_task(id)
}

fn chrono_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
}

#[tauri::command]
pub fn skip_failed(app: AppHandle, state: State<AppState>, id: i64) -> Result<(), String> {
    let task = state.db.get_task(id)?;
    if task.status != Status::Failed.as_str() {
        return Err("only a failed task can be skipped".into());
    }
    // leave as failed; start next if Start is on
    if !state.queue.paused.load(std::sync::atomic::Ordering::SeqCst) {
        kick_queue(app, &state)?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<Settings, String> {
    state.db.settings()
}

#[tauri::command]
pub fn save_settings(state: State<AppState>, settings: Settings) -> Result<Settings, String> {
    state.db.save_settings(&settings)?;
    state.db.settings()
}

#[tauri::command]
pub fn get_run_log(state: State<AppState>, task_id: i64) -> Result<Vec<LogLine>, String> {
    state.db.logs_for(task_id)
}

#[tauri::command]
pub fn binary_status(state: State<AppState>) -> Result<BinaryStatus, String> {
    let settings = state.db.settings()?;
    Ok(BinaryStatus {
        git: process::which("git"),
        claude: process::which(&settings.claude_bin()),
        gh: process::which(&settings.gh_bin()),
    })
}

#[tauri::command]
pub async fn pick_folder(app: AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_folder(move |folder| {
        let mapped = folder.and_then(|p| p.into_path().ok()).map(|pb| {
            pb.to_string_lossy().into_owned()
        });
        let _ = tx.send(mapped);
    });
    rx.await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_url(app: AppHandle, url: String) -> Result<(), String> {
    app.opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| e.to_string())
}

pub fn kick_queue(app: AppHandle, state: &AppState) -> Result<(), String> {
    if state.queue.paused.load(std::sync::atomic::Ordering::SeqCst) {
        return Ok(());
    }
    if state.db.running_task_id()?.is_some() {
        return Ok(());
    }
    if state.hub.is_busy() {
        return Ok(());
    }
    let Some(task) = state.db.next_ready()? else {
        runner::emit_status(&app, None, "idle", None);
        return Ok(());
    };
    if !state.db.claim_ready(task.id)? {
        return Err("could not claim task (another run is active)".into());
    }
    let handle = app.clone();
    let task_id = task.id;
    thread::spawn(move || {
        let Some(st) = handle.try_state::<AppState>() else {
            return;
        };
        let Ok(task) = st.db.get_task(task_id) else {
            return;
        };
        runner::drain_one(&handle, &st.db, &st.hub, &st.queue, task);
        if let Ok(settings) = st.db.settings() {
            if let Ok(t) = st.db.get_task(task_id) {
                if t.status == Status::Done.as_str()
                    && settings.auto_run
                    && !st.queue.paused.load(std::sync::atomic::Ordering::SeqCst)
                {
                    let _ = kick_queue(handle.clone(), st.inner());
                } else {
                    runner::emit_status(&handle, None, "idle", None);
                }
            }
        }
    });
    Ok(())
}
