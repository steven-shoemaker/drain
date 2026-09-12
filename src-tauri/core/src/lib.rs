//! Pure domain logic for Drain — no Tauri, no process spawn, no SQLite.
//! Keep this crate testable on any host (including Linux CI without WebKit).

use serde::{Deserialize, Serialize};

pub const MAX_TITLE_LEN: usize = 80;
pub const BRANCH_PREFIX: &str = "drain/";

/// Task / idea pipeline. Exactly one task may be `Running` at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Idea,
    Refining,
    Ready,
    Running,
    AwaitingPr,
    AwaitingApprove,
    Done,
    Failed,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Idea => "idea",
            Status::Refining => "refining",
            Status::Ready => "ready",
            Status::Running => "running",
            Status::AwaitingPr => "awaiting_pr",
            Status::AwaitingApprove => "awaiting_approve",
            Status::Done => "done",
            Status::Failed => "failed",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "idea" => Some(Status::Idea),
            "refining" => Some(Status::Refining),
            "ready" => Some(Status::Ready),
            "running" => Some(Status::Running),
            "awaiting_pr" => Some(Status::AwaitingPr),
            "awaiting_approve" => Some(Status::AwaitingApprove),
            "done" => Some(Status::Done),
            "failed" => Some(Status::Failed),
            _ => None,
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Status::Done | Status::Failed)
    }

    pub fn is_active_queue(self) -> bool {
        matches!(
            self,
            Status::Ready
                | Status::Running
                | Status::AwaitingPr
                | Status::AwaitingApprove
        )
    }

    pub fn blocks_reorder(self) -> bool {
        matches!(self, Status::Running)
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionMode {
    /// Create a PR, stop, wait for an explicit Approve click. Default.
    PrThenApprove,
    /// After a successful agent run: push, create PR, merge, mark done.
    PushAndMerge,
}

impl CompletionMode {
    pub fn as_str(self) -> &'static str {
        match self {
            CompletionMode::PrThenApprove => "pr_then_approve",
            CompletionMode::PushAndMerge => "push_and_merge",
        }
    }

    pub fn parse(raw: &str) -> Self {
        match raw {
            "push_and_merge" => CompletionMode::PushAndMerge,
            _ => CompletionMode::PrThenApprove,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApproveAction {
    /// `gh pr merge` after explicit Approve click.
    Merge,
    /// Open the PR in a browser; still marks the task done.
    OpenBrowser,
}

impl ApproveAction {
    pub fn as_str(self) -> &'static str {
        match self {
            ApproveAction::Merge => "merge",
            ApproveAction::OpenBrowser => "open_browser",
        }
    }

    pub fn parse(raw: &str) -> Self {
        match raw {
            "open_browser" => ApproveAction::OpenBrowser,
            _ => ApproveAction::Merge,
        }
    }
}

/// Claude Code permission mode passed to `claude --permission-mode`.
/// Bypass is the overnight default so the queue does not hang on prompts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionMode {
    BypassPermissions,
    AcceptEdits,
    Default,
}

impl PermissionMode {
    pub fn as_cli(self) -> &'static str {
        match self {
            PermissionMode::BypassPermissions => "bypassPermissions",
            PermissionMode::AcceptEdits => "acceptEdits",
            PermissionMode::Default => "default",
        }
    }

    pub fn parse(raw: &str) -> Self {
        match raw {
            "acceptEdits" => PermissionMode::AcceptEdits,
            "default" => PermissionMode::Default,
            _ => PermissionMode::BypassPermissions,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefinedTask {
    pub title: String,
    pub acceptance_criteria: Vec<String>,
    pub out_of_scope: Vec<String>,
    pub suggested_branch: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefineOutcome {
    Tasks(Vec<RefinedTask>),
    Refusal(String),
}

#[derive(Debug, Deserialize)]
struct RefineJson {
    #[serde(default)]
    tasks: Vec<RefineJsonTask>,
    #[serde(default)]
    refusal: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RefineJsonTask {
    title: String,
    #[serde(default)]
    acceptance_criteria: Vec<String>,
    #[serde(default)]
    out_of_scope: Vec<String>,
    #[serde(default)]
    suggested_branch: Option<String>,
}

/// After a run succeeds, what status should the task take before the approve gate?
pub fn status_after_agent_success(mode: CompletionMode) -> Status {
    match mode {
        CompletionMode::PrThenApprove => Status::AwaitingPr,
        CompletionMode::PushAndMerge => Status::AwaitingPr,
    }
}

/// Can we start this task given the current running task (if any)?
pub fn can_start(task: Status, currently_running: bool) -> Result<(), &'static str> {
    if currently_running {
        return Err("exactly one task may run at a time");
    }
    if task != Status::Ready {
        return Err("only a ready task can start");
    }
    Ok(())
}

/// Reorder is allowed only when nothing is running.
pub fn can_reorder(has_running: bool) -> Result<(), &'static str> {
    if has_running {
        Err("cannot reorder while a task is running")
    } else {
        Ok(())
    }
}

/// Failed runs must not auto-advance the queue.
pub fn should_auto_advance_on(status: Status, auto_run: bool, paused: bool) -> bool {
    auto_run && !paused && status == Status::Done
}

pub fn slugify(title: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in title.chars().flat_map(|c| c.to_lowercase()) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
        if out.len() >= 40 {
            break;
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "task".to_string()
    } else {
        out
    }
}

pub fn suggested_branch(title: &str) -> String {
    format!("{BRANCH_PREFIX}{}", slugify(title))
}

pub fn normalize_branch(raw: &str, title: &str) -> String {
    let trimmed = raw.trim().trim_start_matches('/').to_string();
    if trimmed.is_empty() {
        return suggested_branch(title);
    }
    if trimmed.starts_with(BRANCH_PREFIX) {
        let slug = slugify(trimmed.trim_start_matches(BRANCH_PREFIX));
        format!("{BRANCH_PREFIX}{slug}")
    } else {
        format!("{BRANCH_PREFIX}{}", slugify(&trimmed))
    }
}

pub fn clamp_title(title: &str) -> String {
    let t = title.trim();
    if t.chars().count() <= MAX_TITLE_LEN {
        t.to_string()
    } else {
        t.chars().take(MAX_TITLE_LEN).collect()
    }
}

pub fn extract_json_blob(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if let Some(start) = trimmed.find("```json") {
        let after = &trimmed[start + 7..];
        if let Some(end) = after.find("```") {
            return Some(after[..end].trim().to_string());
        }
    }
    if let Some(start) = trimmed.find("```") {
        let after = &trimmed[start + 3..];
        let after = after.strip_prefix('\n').unwrap_or(after);
        if let Some(end) = after.find("```") {
            let inner = after[..end].trim();
            if inner.starts_with('{') {
                return Some(inner.to_string());
            }
        }
    }
    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    if end > start {
        Some(trimmed[start..=end].to_string())
    } else {
        None
    }
}

pub fn parse_refine_output(raw: &str) -> Result<RefineOutcome, String> {
    let blob = extract_json_blob(raw)
        .ok_or_else(|| "refine output did not contain a JSON object".to_string())?;
    let parsed: RefineJson = serde_json::from_str(&blob)
        .map_err(|e| format!("could not parse refine JSON: {e}"))?;

    if let Some(reason) = parsed
        .refusal
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        return Ok(RefineOutcome::Refusal(reason));
    }

    if parsed.tasks.is_empty() {
        return Err("refine returned no tasks and no refusal".to_string());
    }

    let mut tasks = Vec::new();
    for t in parsed.tasks {
        let title = clamp_title(&t.title);
        if title.is_empty() {
            return Err("a refined task is missing a title".to_string());
        }
        let criteria: Vec<String> = t
            .acceptance_criteria
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if criteria.is_empty() {
            return Err(format!(
                "task `{title}` has no testable acceptance criteria"
            ));
        }
        let out_of_scope: Vec<String> = t
            .out_of_scope
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let branch = normalize_branch(t.suggested_branch.as_deref().unwrap_or(""), &title);
        tasks.push(RefinedTask {
            title,
            acceptance_criteria: criteria,
            out_of_scope,
            suggested_branch: branch,
        });
    }
    Ok(RefineOutcome::Tasks(tasks))
}

pub fn refine_system_prompt() -> &'static str {
    r#"You are Drain's task refiner — the step that turns a messy human thought into super-defined coding agent tasks.

Return ONLY a JSON object (optionally in a ```json fence) with this shape:
{
  "tasks": [
    {
      "title": "imperative, <=80 chars",
      "acceptance_criteria": ["testable bullet", "..."],
      "out_of_scope": ["bullet"],
      "suggested_branch": "drain/short-slug"
    }
  ],
  "refusal": null
}

You MUST:
- Split independent jobs into separate tasks (1–5). "Fix settings and polish the empty state" is TWO tasks.
- Titles are imperative ("Add X", "Fix Y"), max 80 characters.
- Every acceptance criterion is checkable by a stranger (pass/fail, no vibes).
- Each task's out_of_scope lists the sibling tasks plus unrelated work.
- If the idea is too vague to check, set "tasks" to [] and "refusal" to ONE short question that would make it checkable. Do not invent scope. Do not guess the product.
- suggested_branch always starts with drain/ and uses a short kebab slug.
- Do not include implementation code. Do not mention being Claude.
- Use optional repo context only to name real files; never hallucinate paths."#
}

pub fn refine_user_prompt(idea: &str, repo_context: &str) -> String {
    let mut s = String::from("## Messy idea\n\n");
    s.push_str(idea.trim());
    s.push('\n');
    if !repo_context.trim().is_empty() {
        s.push_str("\n## Repo context\n\n");
        s.push_str(repo_context.trim());
        s.push('\n');
    }
    s.push_str("\nRespond with JSON only.\n");
    s
}

pub fn drain_task_prompt(
    title: &str,
    acceptance_criteria: &[String],
    out_of_scope: &[String],
    branch: &str,
    permission_mode: PermissionMode,
) -> String {
    let mut s = String::new();
    s.push_str("You are draining a single queued task in this git checkout.\n");
    s.push_str(&format!("You are on branch `{branch}`.\n\n"));
    s.push_str(&format!("## Task\n{title}\n\n"));
    s.push_str("## Acceptance criteria\n");
    for c in acceptance_criteria {
        s.push_str(&format!("- [ ] {c}\n"));
    }
    if !out_of_scope.is_empty() {
        s.push_str("\n## Out of scope\n");
        for c in out_of_scope {
            s.push_str(&format!("- {c}\n"));
        }
    }
    s.push_str("\n");
    s.push_str("Implement until the acceptance criteria are met.\n");
    s.push_str("Do not open a pull request. Do not push. Do not merge.\n");
    s.push_str("Leave work in the working tree and/or local commits; Drain will commit leftovers, push, and open the PR.\n");
    s.push_str(&format!(
        "Permission mode for this run: {}.\n",
        permission_mode.as_cli()
    ));
    s
}

pub fn pr_body(
    title: &str,
    acceptance_criteria: &[String],
    out_of_scope: &[String],
) -> String {
    let mut s = String::new();
    s.push_str(&format!("# {title}\n\n"));
    s.push_str("## Acceptance criteria\n\n");
    for c in acceptance_criteria {
        s.push_str(&format!("- [ ] {c}\n"));
    }
    if !out_of_scope.is_empty() {
        s.push_str("\n## Out of scope\n\n");
        for c in out_of_scope {
            s.push_str(&format!("- {c}\n"));
        }
    }
    s.push_str("\n---\nOpened by Drain. Merge only after an explicit Approve click in the app (unless completion mode is push-and-merge).\n");
    s
}

pub fn extract_pr_url(output: &str) -> Option<String> {
    for token in output.split_whitespace() {
        if token.starts_with("https://github.com/") && token.contains("/pull/") {
            return Some(token.trim_end_matches(['.', ')']).to_string());
        }
    }
    for line in output.lines() {
        let line = line.trim();
        if line.starts_with("https://") && line.contains("/pull/") {
            return Some(line.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Add dark mode toggle"), "add-dark-mode-toggle");
        assert_eq!(slugify("  Hello, World!  "), "hello-world");
        assert_eq!(slugify("***"), "task");
        assert_eq!(suggested_branch("Fix login"), "drain/fix-login");
    }

    #[test]
    fn parse_fenced_json() {
        let raw = r#"
Here you go.

```json
{
  "tasks": [
    {
      "title": "Add a settings persistence test",
      "acceptance_criteria": ["Saving auto_run round-trips after relaunch"],
      "out_of_scope": ["iCloud"],
      "suggested_branch": "drain/settings-persist"
    }
  ],
  "refusal": null
}
```
"#;
        let RefineOutcome::Tasks(tasks) = parse_refine_output(raw).unwrap() else {
            panic!("expected tasks");
        };
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].suggested_branch, "drain/settings-persist");
        assert_eq!(tasks[0].acceptance_criteria.len(), 1);
    }

    #[test]
    fn parse_refusal() {
        let raw = r#"{"tasks":[],"refusal":"Which repo and what should the button do?"}"#;
        match parse_refine_output(raw).unwrap() {
            RefineOutcome::Refusal(r) => assert!(r.contains("repo")),
            _ => panic!("expected refusal"),
        }
    }

    #[test]
    fn rejects_empty_criteria() {
        let raw = r#"{"tasks":[{"title":"Do stuff","acceptance_criteria":[],"out_of_scope":[]}]}"#;
        assert!(parse_refine_output(raw).is_err());
    }

    #[test]
    fn can_start_rules() {
        assert!(can_start(Status::Ready, false).is_ok());
        assert!(can_start(Status::Ready, true).is_err());
        assert!(can_start(Status::Done, false).is_err());
        assert!(can_reorder(true).is_err());
        assert!(can_reorder(false).is_ok());
        assert!(!should_auto_advance_on(Status::Failed, true, false));
        assert!(should_auto_advance_on(Status::Done, true, false));
        assert!(!should_auto_advance_on(Status::Done, true, true));
    }

    #[test]
    fn extract_pr_url_from_gh() {
        let out = "Creating pull request\nhttps://github.com/acme/drain/pull/12\n";
        assert_eq!(
            extract_pr_url(out).as_deref(),
            Some("https://github.com/acme/drain/pull/12")
        );
    }

    #[test]
    fn parse_split_tasks() {
        let raw = r#"{
          "tasks": [
            {
              "title": "Persist auto_run in Settings",
              "acceptance_criteria": ["Value survives relaunch"],
              "out_of_scope": ["Inbox empty state"],
              "suggested_branch": "drain/settings-autorun"
            },
            {
              "title": "Polish inbox empty state",
              "acceptance_criteria": ["Empty copy shows when Inbox has zero ideas"],
              "out_of_scope": ["Settings persistence"],
              "suggested_branch": "drain/inbox-empty"
            }
          ],
          "refusal": null
        }"#;
        let RefineOutcome::Tasks(tasks) = parse_refine_output(raw).unwrap() else {
            panic!("expected tasks");
        };
        assert_eq!(tasks.len(), 2);
        assert!(refine_system_prompt().contains("Split independent"));
    }

    #[test]
    fn title_clamp() {
        let long = "A".repeat(90);
        assert_eq!(clamp_title(&long).chars().count(), 80);
    }
}
