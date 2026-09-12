//! Spawn local binaries from Rust only. The webview never shells out.

use serde::Serialize;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEvent {
    pub task_id: i64,
    pub stream: String,
    pub line: String,
}

pub struct LiveProcess {
    pub pid: u32,
    pub cancel: Arc<AtomicBool>,
}

#[derive(Clone)]
pub struct ProcessHub {
    current: Arc<Mutex<Option<LiveProcess>>>,
}

impl ProcessHub {
    pub fn new() -> Self {
        Self {
            current: Arc::new(Mutex::new(None)),
        }
    }

    pub fn cancel(&self) -> bool {
        if let Ok(guard) = self.current.lock() {
            if let Some(live) = guard.as_ref() {
                live.cancel.store(true, Ordering::SeqCst);
                let _ = Command::new("kill")
                    .arg("-TERM")
                    .arg(live.pid.to_string())
                    .status();
                return true;
            }
        }
        false
    }

    pub fn is_busy(&self) -> bool {
        self.current.lock().map(|g| g.is_some()).unwrap_or(false)
    }
}

pub fn which(bin: &str) -> Option<String> {
    let p = Path::new(bin);
    if bin.contains('/') || bin.starts_with('~') {
        let expanded = if let Some(rest) = bin.strip_prefix("~/") {
            let home = std::env::var("HOME").ok()?;
            PathBuf::from(home).join(rest)
        } else {
            p.to_path_buf()
        };
        return if expanded.exists() {
            Some(expanded.to_string_lossy().into_owned())
        } else {
            None
        };
    }
    let output = Command::new("/usr/bin/which").arg(bin).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}

pub struct CmdResult {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

fn emit_log(app: &AppHandle, task_id: i64, stream: &str, line: &str) {
    let _ = app.emit(
        "drain://log",
        LogEvent {
            task_id,
            stream: stream.to_string(),
            line: line.to_string(),
        },
    );
}

/// Run a command, stream stdout/stderr as `drain://log` events, return combined result.
pub fn run_logged(
    hub: &ProcessHub,
    app: &AppHandle,
    task_id: i64,
    bin: &str,
    args: &[&str],
    cwd: Option<&Path>,
    extra_env: &[(&str, &str)],
) -> Result<CmdResult, String> {
    let display = format!(
        "$ {} {}",
        bin,
        args.iter()
            .map(|a| {
                if a.contains(' ') || a.contains('\n') {
                    "…".to_string()
                } else {
                    a.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    );
    emit_log(app, task_id, "system", &display);

    let mut cmd = Command::new(bin);
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GH_PROMPT_DISABLED", "1")
        .env("CLICOLOR", "0");
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    for (k, v) in extra_env {
        cmd.env(k, v);
    }

    let mut child = cmd.spawn().map_err(|e| format!("failed to spawn {bin}: {e}"))?;
    let pid = child.id();
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut slot = hub.current.lock().map_err(|e| e.to_string())?;
        *slot = Some(LiveProcess {
            pid,
            cancel: cancel.clone(),
        });
    }

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let out_buf = Arc::new(Mutex::new(String::new()));
    let err_buf = Arc::new(Mutex::new(String::new()));
    let mut handles = Vec::new();

    if let Some(out) = stdout {
        let app = app.clone();
        let buf = out_buf.clone();
        handles.push(thread::spawn(move || {
            let reader = BufReader::new(out);
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        emit_log(&app, task_id, "stdout", &line);
                        if let Ok(mut b) = buf.lock() {
                            b.push_str(&line);
                            b.push('\n');
                        }
                    }
                    Err(_) => break,
                }
            }
        }));
    }
    if let Some(err) = stderr {
        let app = app.clone();
        let buf = err_buf.clone();
        handles.push(thread::spawn(move || {
            let reader = BufReader::new(err);
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        emit_log(&app, task_id, "stderr", &line);
                        if let Ok(mut b) = buf.lock() {
                            b.push_str(&line);
                            b.push('\n');
                        }
                    }
                    Err(_) => break,
                }
            }
        }));
    }

    let status = child.wait().map_err(|e| e.to_string())?;
    for h in handles {
        let _ = h.join();
    }
    {
        let mut slot = hub.current.lock().map_err(|e| e.to_string())?;
        *slot = None;
    }

    if cancel.load(Ordering::SeqCst) {
        return Err("cancelled".into());
    }

    Ok(CmdResult {
        code: status.code().unwrap_or(-1),
        stdout: out_buf.lock().map(|s| s.clone()).unwrap_or_default(),
        stderr: err_buf.lock().map(|s| s.clone()).unwrap_or_default(),
    })
}

pub fn run_capture(
    bin: &str,
    args: &[&str],
    cwd: Option<&Path>,
) -> Result<CmdResult, String> {
    let mut cmd = Command::new(bin);
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GH_PROMPT_DISABLED", "1");
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let output = cmd
        .output()
        .map_err(|e| format!("failed to spawn {bin}: {e}"))?;
    Ok(CmdResult {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

pub fn repo_context(path: &Path) -> String {
    let mut out = String::new();
    for name in ["README.md", "readme.md", "README"] {
        let p = path.join(name);
        if let Ok(text) = std::fs::read_to_string(&p) {
            let clipped: String = text.chars().take(6000).collect();
            out.push_str(&format!("### {name}\n{clipped}\n\n"));
            break;
        }
    }
    out.push_str("### tree (depth 2)\n");
    if let Ok(entries) = walk_shallow(path, 2) {
        for e in entries.into_iter().take(120) {
            out.push_str(&e);
            out.push('\n');
        }
    }
    out
}

fn walk_shallow(root: &Path, max_depth: usize) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    fn rec(
        dir: &Path,
        root: &Path,
        depth: usize,
        max_depth: usize,
        out: &mut Vec<String>,
    ) {
        if depth > max_depth {
            return;
        }
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        let mut names: Vec<_> = rd.filter_map(|e| e.ok()).collect();
        names.sort_by_key(|e| e.file_name());
        for entry in names {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.')
                || name == "node_modules"
                || name == "target"
                || name == "dist"
                || name == "build"
                || name == ".git"
            {
                continue;
            }
            let rel = entry
                .path()
                .strip_prefix(root)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| name.to_string());
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            if is_dir {
                out.push(format!("{rel}/"));
                rec(&entry.path(), root, depth + 1, max_depth, out);
            } else {
                out.push(rel);
            }
        }
    }
    rec(root, root, 1, max_depth, &mut out);
    Ok(out)
}
