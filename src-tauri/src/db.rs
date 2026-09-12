use drain_core::{ApproveAction, CompletionMode, PermissionMode, RefinedTask, Status};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;

pub struct Db(pub Mutex<Connection>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub default_repo_path: String,
    pub path_to_claude: String,
    pub path_to_gh: String,
    pub auto_run: bool,
    pub completion_mode: String,
    pub approve_action: String,
    pub permission_mode: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_repo_path: String::new(),
            path_to_claude: String::new(),
            path_to_gh: String::new(),
            auto_run: false,
            completion_mode: CompletionMode::PrThenApprove.as_str().to_string(),
            approve_action: ApproveAction::Merge.as_str().to_string(),
            permission_mode: PermissionMode::BypassPermissions.as_cli().to_string(),
        }
    }
}

impl Settings {
    pub fn completion(&self) -> CompletionMode {
        CompletionMode::parse(&self.completion_mode)
    }
    pub fn approve(&self) -> ApproveAction {
        ApproveAction::parse(&self.approve_action)
    }
    pub fn permission(&self) -> PermissionMode {
        PermissionMode::parse(&self.permission_mode)
    }
    pub fn claude_bin(&self) -> String {
        if self.path_to_claude.trim().is_empty() {
            "claude".into()
        } else {
            self.path_to_claude.trim().to_string()
        }
    }
    pub fn gh_bin(&self) -> String {
        if self.path_to_gh.trim().is_empty() {
            "gh".into()
        } else {
            self.path_to_gh.trim().to_string()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Idea {
    pub id: i64,
    pub body: String,
    pub status: String,
    pub error: Option<String>,
    pub drafts: Vec<RefinedTask>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: i64,
    pub idea_id: Option<i64>,
    pub title: String,
    pub acceptance_criteria: Vec<String>,
    pub out_of_scope: Vec<String>,
    pub suggested_branch: String,
    pub repo_path: String,
    pub status: String,
    pub position: i64,
    pub pr_url: Option<String>,
    pub fail_note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogLine {
    pub id: i64,
    pub task_id: i64,
    pub stream: String,
    pub line: String,
    pub ts: String,
}

pub fn open(path: &Path) -> Result<Connection, String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;

        CREATE TABLE IF NOT EXISTS settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            default_repo_path TEXT NOT NULL DEFAULT '',
            path_to_claude TEXT NOT NULL DEFAULT '',
            path_to_gh TEXT NOT NULL DEFAULT '',
            auto_run INTEGER NOT NULL DEFAULT 0,
            completion_mode TEXT NOT NULL DEFAULT 'pr_then_approve',
            approve_action TEXT NOT NULL DEFAULT 'merge',
            permission_mode TEXT NOT NULL DEFAULT 'bypassPermissions'
        );

        INSERT OR IGNORE INTO settings (id) VALUES (1);

        CREATE TABLE IF NOT EXISTS ideas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            body TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'idea',
            error TEXT,
            drafts_json TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            idea_id INTEGER,
            title TEXT NOT NULL,
            acceptance_criteria TEXT NOT NULL DEFAULT '[]',
            out_of_scope TEXT NOT NULL DEFAULT '[]',
            suggested_branch TEXT NOT NULL,
            repo_path TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL,
            position INTEGER NOT NULL DEFAULT 0,
            pr_url TEXT,
            fail_note TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (idea_id) REFERENCES ideas(id)
        );

        CREATE TABLE IF NOT EXISTS run_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL,
            stream TEXT NOT NULL,
            line TEXT NOT NULL,
            ts TEXT NOT NULL,
            FOREIGN KEY (task_id) REFERENCES tasks(id)
        );
        "#,
    )
    .map_err(|e| e.to_string())?;
    Ok(conn)
}

fn now() -> String {
    // RFC3339-ish UTC without extra crates — good enough for local sort.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

fn parse_json_list(raw: &str) -> Vec<String> {
    serde_json::from_str(raw).unwrap_or_default()
}

fn parse_drafts(raw: &str) -> Vec<RefinedTask> {
    serde_json::from_str(raw).unwrap_or_default()
}

impl Db {
    pub fn settings(&self) -> Result<Settings, String> {
        let conn = self.0.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT default_repo_path, path_to_claude, path_to_gh, auto_run,
                    completion_mode, approve_action, permission_mode
             FROM settings WHERE id = 1",
            [],
            |row| {
                Ok(Settings {
                    default_repo_path: row.get(0)?,
                    path_to_claude: row.get(1)?,
                    path_to_gh: row.get(2)?,
                    auto_run: row.get::<_, i64>(3)? != 0,
                    completion_mode: row.get(4)?,
                    approve_action: row.get(5)?,
                    permission_mode: row.get(6)?,
                })
            },
        )
        .map_err(|e| e.to_string())
    }

    pub fn save_settings(&self, s: &Settings) -> Result<(), String> {
        let conn = self.0.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE settings SET
                default_repo_path = ?1,
                path_to_claude = ?2,
                path_to_gh = ?3,
                auto_run = ?4,
                completion_mode = ?5,
                approve_action = ?6,
                permission_mode = ?7
             WHERE id = 1",
            params![
                s.default_repo_path,
                s.path_to_claude,
                s.path_to_gh,
                if s.auto_run { 1 } else { 0 },
                s.completion_mode,
                s.approve_action,
                s.permission_mode,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_ideas(&self) -> Result<Vec<Idea>, String> {
        let conn = self.0.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, body, status, error, drafts_json, created_at, updated_at
                 FROM ideas ORDER BY id DESC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Idea {
                    id: row.get(0)?,
                    body: row.get(1)?,
                    status: row.get(2)?,
                    error: row.get(3)?,
                    drafts: parse_drafts(&row.get::<_, String>(4)?),
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn get_idea(&self, id: i64) -> Result<Idea, String> {
        let conn = self.0.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT id, body, status, error, drafts_json, created_at, updated_at
             FROM ideas WHERE id = ?1",
            params![id],
            |row| {
                Ok(Idea {
                    id: row.get(0)?,
                    body: row.get(1)?,
                    status: row.get(2)?,
                    error: row.get(3)?,
                    drafts: parse_drafts(&row.get::<_, String>(4)?),
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )
        .map_err(|e| e.to_string())
    }

    pub fn create_idea(&self, body: &str) -> Result<Idea, String> {
        let ts = now();
        let conn = self.0.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO ideas (body, status, drafts_json, created_at, updated_at)
             VALUES (?1, 'idea', '[]', ?2, ?2)",
            params![body.trim(), ts],
        )
        .map_err(|e| e.to_string())?;
        let id = conn.last_insert_rowid();
        drop(conn);
        self.get_idea(id)
    }

    pub fn set_idea_status(
        &self,
        id: i64,
        status: Status,
        error: Option<&str>,
    ) -> Result<(), String> {
        let ts = now();
        let conn = self.0.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE ideas SET status = ?1, error = ?2, updated_at = ?3 WHERE id = ?4",
            params![status.as_str(), error, ts, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn set_idea_body(&self, id: i64, body: &str) -> Result<(), String> {
        let ts = now();
        let conn = self.0.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE ideas SET body = ?1, error = NULL, updated_at = ?2 WHERE id = ?3",
            params![body.trim(), ts, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn set_idea_drafts(&self, id: i64, drafts: &[RefinedTask]) -> Result<(), String> {
        let ts = now();
        let json = serde_json::to_string(drafts).map_err(|e| e.to_string())?;
        let conn = self.0.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE ideas SET drafts_json = ?1, status = 'idea', error = NULL, updated_at = ?2
             WHERE id = ?3",
            params![json, ts, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_tasks(&self) -> Result<Vec<Task>, String> {
        let conn = self.0.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, idea_id, title, acceptance_criteria, out_of_scope,
                        suggested_branch, repo_path, status, position, pr_url,
                        fail_note, created_at, updated_at
                 FROM tasks ORDER BY
                    CASE status
                        WHEN 'running' THEN 0
                        WHEN 'awaiting_pr' THEN 1
                        WHEN 'awaiting_approve' THEN 2
                        WHEN 'ready' THEN 3
                        WHEN 'failed' THEN 4
                        ELSE 5
                    END,
                    position ASC, id ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], map_task)
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn get_task(&self, id: i64) -> Result<Task, String> {
        let conn = self.0.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT id, idea_id, title, acceptance_criteria, out_of_scope,
                    suggested_branch, repo_path, status, position, pr_url,
                    fail_note, created_at, updated_at
             FROM tasks WHERE id = ?1",
            params![id],
            map_task,
        )
        .map_err(|e| e.to_string())
    }

    pub fn running_task_id(&self) -> Result<Option<i64>, String> {
        let conn = self.0.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT id FROM tasks WHERE status = 'running' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())
    }

    pub fn next_ready(&self) -> Result<Option<Task>, String> {
        let conn = self.0.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT id, idea_id, title, acceptance_criteria, out_of_scope,
                    suggested_branch, repo_path, status, position, pr_url,
                    fail_note, created_at, updated_at
             FROM tasks WHERE status = 'ready' ORDER BY position ASC, id ASC LIMIT 1",
            [],
            map_task,
        )
        .optional()
        .map_err(|e| e.to_string())
    }

    pub fn max_position(&self) -> Result<i64, String> {
        let conn = self.0.lock().map_err(|e| e.to_string())?;
        let v: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(position), 0) FROM tasks WHERE status IN
                 ('ready','running','awaiting_pr','awaiting_approve')",
                [],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        Ok(v)
    }

    pub fn insert_task(
        &self,
        idea_id: i64,
        draft: &RefinedTask,
        repo_path: &str,
        position: i64,
    ) -> Result<i64, String> {
        let ts = now();
        let criteria = serde_json::to_string(&draft.acceptance_criteria).map_err(|e| e.to_string())?;
        let oos = serde_json::to_string(&draft.out_of_scope).map_err(|e| e.to_string())?;
        let conn = self.0.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO tasks
                (idea_id, title, acceptance_criteria, out_of_scope, suggested_branch,
                 repo_path, status, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'ready', ?7, ?8, ?8)",
            params![
                idea_id,
                draft.title,
                criteria,
                oos,
                draft.suggested_branch,
                repo_path,
                position,
                ts
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }

    pub fn set_task_status(
        &self,
        id: i64,
        status: Status,
        pr_url: Option<&str>,
        fail_note: Option<&str>,
    ) -> Result<(), String> {
        let ts = now();
        let conn = self.0.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE tasks SET status = ?1, pr_url = COALESCE(?2, pr_url),
                    fail_note = ?3, updated_at = ?4
             WHERE id = ?5",
            params![status.as_str(), pr_url, fail_note, ts, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn claim_ready(&self, id: i64) -> Result<bool, String> {
        if self.running_task_id()?.is_some() {
            return Ok(false);
        }
        let ts = now();
        let conn = self.0.lock().map_err(|e| e.to_string())?;
        let n = conn
            .execute(
                "UPDATE tasks SET status = 'running', updated_at = ?1
                 WHERE id = ?2 AND status = 'ready'",
                params![ts, id],
            )
            .map_err(|e| e.to_string())?;
        Ok(n == 1)
    }

    pub fn reorder(&self, ids: &[i64]) -> Result<(), String> {
        let mut conn = self.0.lock().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for (i, id) in ids.iter().enumerate() {
            tx.execute(
                "UPDATE tasks SET position = ?1 WHERE id = ?2 AND status = 'ready'",
                params![i as i64, id],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn append_log(&self, task_id: i64, stream: &str, line: &str) -> Result<(), String> {
        let ts = now();
        let conn = self.0.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO run_logs (task_id, stream, line, ts) VALUES (?1, ?2, ?3, ?4)",
            params![task_id, stream, line, ts],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn logs_for(&self, task_id: i64) -> Result<Vec<LogLine>, String> {
        let conn = self.0.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, task_id, stream, line, ts FROM run_logs
                 WHERE task_id = ?1 ORDER BY id ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![task_id], |row| {
                Ok(LogLine {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    stream: row.get(2)?,
                    line: row.get(3)?,
                    ts: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn clear_logs(&self, task_id: i64) -> Result<(), String> {
        let conn = self.0.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM run_logs WHERE task_id = ?1", params![task_id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn map_task(row: &rusqlite::Row) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get(0)?,
        idea_id: row.get(1)?,
        title: row.get(2)?,
        acceptance_criteria: parse_json_list(&row.get::<_, String>(3)?),
        out_of_scope: parse_json_list(&row.get::<_, String>(4)?),
        suggested_branch: row.get(5)?,
        repo_path: row.get(6)?,
        status: row.get(7)?,
        position: row.get(8)?,
        pr_url: row.get(9)?,
        fail_note: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}
