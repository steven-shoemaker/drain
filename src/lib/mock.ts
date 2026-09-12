import { refineMessyIdea, REFINE_STEPS } from "$lib/refine";
import type { Idea, LogEvent, RefinedTask, Settings, StatusEvent, Task } from "$lib/types";

type Listener<T> = (payload: T) => void;

const ideas: Idea[] = [];
let tasks: Task[] = [];
let logs: LogEvent[] = [];
let nextIdea = 1;
let nextTask = 1;
let paused = true;
let running = false;
let settings: Settings = loadSettings();

const logListeners = new Set<Listener<LogEvent>>();
const statusListeners = new Set<Listener<StatusEvent>>();

function loadSettings(): Settings {
  try {
    const raw = localStorage.getItem("drain.mock.settings");
    if (raw) return JSON.parse(raw) as Settings;
  } catch {
    /* ignore */
  }
  return {
    defaultRepoPath: "/Users/you/src/example",
    pathToClaude: "",
    pathToGh: "",
    autoRun: false,
    completionMode: "pr_then_approve",
    approveAction: "merge",
    permissionMode: "bypassPermissions",
  };
}

function persistSettings() {
  localStorage.setItem("drain.mock.settings", JSON.stringify(settings));
}

function now() {
  return String(Date.now());
}

function emitLog(taskId: number, stream: string, line: string) {
  const ev: LogEvent = { taskId, stream, line };
  logs.push(ev);
  for (const l of logListeners) l(ev);
}

function emitStatus(taskId: number | null, status: string, message: string | null = null) {
  const ev: StatusEvent = { taskId, status, message };
  for (const l of statusListeners) l(ev);
}

export const mock = {
  onLog(fn: Listener<LogEvent>) {
    logListeners.add(fn);
    return () => logListeners.delete(fn);
  },
  onStatus(fn: Listener<StatusEvent>) {
    statusListeners.add(fn);
    return () => statusListeners.delete(fn);
  },
  async listIdeas() {
    return ideas.slice().sort((a, b) => b.id - a.id);
  },
  async createIdea(body: string) {
    const idea: Idea = {
      id: nextIdea++,
      body: body.trim(),
      status: "idea",
      error: null,
      drafts: [],
      createdAt: now(),
      updatedAt: now(),
    };
    ideas.unshift(idea);
    return idea;
  },
  async getIdea(id: number) {
    const idea = ideas.find((i) => i.id === id);
    if (!idea) throw new Error("idea not found");
    return idea;
  },
  async updateIdeaDrafts(id: number, drafts: RefinedTask[]) {
    const idea = ideas.find((i) => i.id === id);
    if (!idea) throw new Error("idea not found");
    idea.drafts = drafts;
    idea.status = "idea";
    idea.error = null;
    idea.updatedAt = now();
    return idea;
  },
  async updateIdeaBody(id: number, body: string) {
    const idea = ideas.find((i) => i.id === id);
    if (!idea) throw new Error("idea not found");
    idea.body = body.trim();
    idea.error = null;
    idea.updatedAt = now();
    return idea;
  },
  async refineIdea(id: number) {
    const idea = ideas.find((i) => i.id === id);
    if (!idea) throw new Error("idea not found");
    idea.status = "refining";
    idea.error = null;
    emitStatus(null, "idea_refining", String(id));
    for (let i = 0; i < REFINE_STEPS.length; i++) {
      emitStatus(null, "idea_trace", `${id}:${i}`);
      await delay(220);
    }
    const outcome = refineMessyIdea(idea.body);
    idea.status = "idea";
    idea.updatedAt = now();
    if (outcome.kind === "refusal") {
      idea.drafts = [];
      idea.error = outcome.question;
      emitStatus(null, "idea_refused", outcome.question);
      return;
    }
    idea.drafts = outcome.tasks;
    idea.error = null;
    emitStatus(null, "idea_refined", String(id));
  },
  async enqueueIdea(id: number, drafts: RefinedTask[], repoPath?: string) {
    const idea = ideas.find((i) => i.id === id);
    if (!idea) throw new Error("idea not found");
    const repo = repoPath?.trim() || settings.defaultRepoPath;
    if (!repo) throw new Error("set a default repo path in Settings before enqueueing");
    idea.drafts = drafts;
    let pos = tasks.reduce((m, t) => Math.max(m, t.position), 0) + 1;
    for (const d of drafts) {
      tasks.push({
        id: nextTask++,
        ideaId: id,
        title: d.title,
        acceptanceCriteria: d.acceptanceCriteria,
        outOfScope: d.outOfScope,
        suggestedBranch: d.suggestedBranch,
        repoPath: repo,
        status: "ready",
        position: pos++,
        prUrl: null,
        failNote: null,
        createdAt: now(),
        updatedAt: now(),
      });
    }
    return mock.listTasks();
  },
  async listTasks() {
    const rank = (s: Task["status"]) =>
      ({
        running: 0,
        awaiting_pr: 1,
        awaiting_approve: 2,
        ready: 3,
        failed: 4,
        done: 5,
        idea: 6,
        refining: 7,
      })[s];
    return tasks.slice().sort((a, b) => rank(a.status) - rank(b.status) || a.position - b.position);
  },
  async getTask(id: number) {
    const t = tasks.find((x) => x.id === id);
    if (!t) throw new Error("task not found");
    return t;
  },
  async reorderTasks(ids: number[]) {
    if (tasks.some((t) => t.status === "running")) {
      throw new Error("cannot reorder while a task is running");
    }
    ids.forEach((id, i) => {
      const t = tasks.find((x) => x.id === id);
      if (t && t.status === "ready") t.position = i;
    });
    return mock.listTasks();
  },
  async startQueue() {
    paused = false;
    void runNext();
  },
  async pauseQueue() {
    paused = true;
  },
  async queuePaused() {
    return paused;
  },
  async cancelRun() {
    running = false;
    const t = tasks.find((x) => x.status === "running");
    if (t) {
      t.status = "failed";
      t.failNote = "cancelled";
      emitLog(t.id, "system", "cancelled");
      emitStatus(t.id, "failed", "cancelled");
    }
  },
  async approveTask(id: number) {
    const t = tasks.find((x) => x.id === id);
    if (!t) throw new Error("task not found");
    t.status = "done";
    emitStatus(id, "done", t.prUrl);
    if (settings.autoRun && !paused) void runNext();
    return t;
  },
  async rejectTask(id: number, note: string) {
    const t = tasks.find((x) => x.id === id);
    if (!t) throw new Error("task not found");
    t.status = "failed";
    t.failNote = note.trim() || "rejected";
    emitStatus(id, "failed", t.failNote);
    return t;
  },
  async retryTask(id: number) {
    const t = tasks.find((x) => x.id === id);
    if (!t) throw new Error("task not found");
    t.status = "ready";
    t.failNote = null;
    t.position = tasks.reduce((m, x) => Math.max(m, x.position), 0) + 1;
    return t;
  },
  async skipFailed() {
    if (!paused) void runNext();
  },
  async getSettings() {
    return { ...settings };
  },
  async saveSettings(next: Settings) {
    settings = { ...next };
    persistSettings();
    return { ...settings };
  },
  async getRunLog(taskId: number) {
    return logs
      .filter((l) => l.taskId === taskId)
      .map((l, i) => ({
        id: i + 1,
        taskId: l.taskId,
        stream: l.stream,
        line: l.line,
        ts: now(),
      }));
  },
  async binaryStatus() {
    return { git: "/usr/bin/git", claude: "claude", gh: "gh" };
  },
  async pickFolder() {
    return settings.defaultRepoPath || "/Users/you/src/example";
  },
  async openUrl(url: string) {
    window.open(url, "_blank");
  },
};

async function runNext() {
  if (paused || running) return;
  const next = tasks
    .filter((t) => t.status === "ready")
    .sort((a, b) => a.position - b.position)[0];
  if (!next) {
    emitStatus(null, "idle", null);
    return;
  }
  running = true;
  next.status = "running";
  emitStatus(next.id, "running", null);
  const lines = [
    ["system", "$ git checkout main"],
    ["stdout", "Already on 'main'"],
    ["system", `$ git checkout -b ${next.suggestedBranch}`],
    ["stdout", `Switched to a new branch '${next.suggestedBranch}'`],
    ["system", "claude -p  (permission-mode bypassPermissions)"],
    ["stdout", `Working on: ${next.title}`],
    ["stdout", "Implementing acceptance criteria…"],
    ["stdout", "Done."],
    ["system", "$ git add -A"],
    ["system", "$ git commit"],
    ["stdout", `[${next.suggestedBranch} abcdef1] ${next.title}`],
    ["system", "$ git push -u origin HEAD"],
    ["system", "gh pr create"],
  ] as const;
  for (const [stream, line] of lines) {
    if (!running) return;
    await delay(280);
    emitLog(next.id, stream, line);
  }
  const url = `https://github.com/example/repo/pull/${next.id}`;
  next.prUrl = url;
  next.status = "awaiting_approve";
  running = false;
  emitLog(next.id, "system", `pr: ${url}`);
  emitLog(next.id, "system", "stopped — waiting for Approve");
  emitStatus(next.id, "awaiting_approve", url);
}

function delay(ms: number) {
  return new Promise((r) => setTimeout(r, ms));
}
