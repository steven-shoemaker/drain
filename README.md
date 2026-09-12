# Drain

A Mac desktop app where you dump messy ideas, refine them into super-defined agent tasks, then Claude Code drains the queue **one-by-one**.

Overnight MVP. Not an App Store build. Not a swarm dashboard.

To publish this folder as a GitHub repo (create failed from the agent token):

```bash
gh repo create drain --private --source . --remote origin --push
```

## What it does

1. **Inbox** — capture a messy idea, run Refine (`claude -p` with a fixed system prompt), edit the resulting task cards, enqueue.
2. **Queue** — ordered tasks with status chips. Reorder when idle. Start / Pause.
3. **Run** — live stdout/stderr from the child process. Cancel. Approve / Reject when a PR is ready.
4. **Settings** — default repo, `claude` / `gh` paths, auto-run, completion mode, approve action, permission mode.

State machine:

`idea` → `refining` → `ready` → `running` → `awaiting_pr` → `awaiting_approve` → `done` | `failed`

Rules:

- Exactly one task `running` at a time.
- Next starts only when the previous is `done` (or you skip / retry a failure).
- Failed runs surface an error and **do not** silently eat the queue.
- No parallel agents.

## Stack

- Tauri 2 (macOS first)
- Svelte 5 + TypeScript
- SQLite via `rusqlite` (bundled), stored in the app data dir
- Local processes spawned **from Rust** (`git`, `claude`, `gh`) — the webview never shells out

## Prerequisites

On the Mac that will run Drain:

- [Rust](https://rustup.rs) + Xcode CLT
- Node 22+
- `git`, [Claude Code CLI](https://code.claude.com/docs/en/cli-reference) (`claude`, already logged in), [GitHub CLI](https://cli.github.com) (`gh auth login`)

```bash
npm install
npm run tauri dev
```

Release (unsigned, local):

```bash
npm run tauri build
```

## Settings (defaults)

| Key | Default | Notes |
| --- | --- | --- |
| Default repo | empty | Required before enqueue / drain |
| Path to `claude` | `claude` on PATH | Absolute path if needed |
| Path to `gh` | `gh` on PATH | Absolute path if needed |
| Auto-run | off | After **Approve → done**, start the next `ready` task |
| On success | Create PR, wait for Approve | Matches the drain contract. Alternative: **push and merge** (skips the approve gate after a successful run) |
| Approve click | `gh pr merge` | Alternative: open the PR in a browser, then mark done |
| Permission mode | `bypassPermissions` | Passed as `claude -p --permission-mode …` so the queue does not hang on prompts |

Auth: local Claude Code CLI session. No API key UI.

## Drain contract

For each `ready` task, Rust runs (in the repo):

1. `git checkout <default-branch> && git pull && git checkout -b drain/<slug>`
2. `claude -p` with the task prompt + acceptance criteria
3. commit leftovers, `git push`, `gh pr create` (body = criteria checklist)
4. status → `awaiting_approve`, store `pr_url`
5. **Stop.** Do not merge. Do not start next. (Unless completion mode is push-and-merge.)

Approve is an explicit click. Reject marks `failed` with a note; Retry puts it back on the queue; Skip / next starts the following `ready` task without clearing the failure.

## Browser preview

`npm run dev` opens the Svelte UI in a browser with a **mock** backend (no real `claude` / `git`). The sidebar says so. Real spawning only happens inside the Tauri app.

## Non-goals (this MVP)

Swarms, cloud sync, iOS, billing, Jira/Linear, in-app PR diffs, Windows/Linux packaging, notarization, auto-updates, app icon polish.
