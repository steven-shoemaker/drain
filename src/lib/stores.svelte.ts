import { getRunLog, listIdeas, listTasks, queuePaused } from "$lib/api";
import { REFINE_STEPS } from "$lib/refine";
import type { Idea, LogEvent, Task, ThinkStep } from "$lib/types";

function freshSteps(): ThinkStep[] {
  return REFINE_STEPS.map((s) => ({ ...s, state: "pending" as const }));
}

class DrainStore {
  ideas = $state<Idea[]>([]);
  tasks = $state<Task[]>([]);
  paused = $state(true);
  liveLog = $state<LogEvent[]>([]);
  activeTaskId = $state<number | null>(null);
  flash = $state<string | null>(null);
  error = $state<string | null>(null);
  refiningId = $state<number | null>(null);
  refineStartedAt = $state<number | null>(null);
  refineSteps = $state<ThinkStep[]>(freshSteps());

  get running() {
    return this.tasks.find((t) => t.status === "running") ?? null;
  }

  get awaiting() {
    return (
      this.tasks.find((t) => t.status === "awaiting_approve" || t.status === "awaiting_pr") ??
      null
    );
  }

  get readyCount() {
    return this.tasks.filter((t) => t.status === "ready").length;
  }

  get focusedTaskId() {
    return this.running?.id ?? this.awaiting?.id ?? this.activeTaskId;
  }

  startRefine(id: number) {
    this.refiningId = id;
    this.refineStartedAt = Date.now();
    this.refineSteps = freshSteps();
  }

  advanceTrace(index: number) {
    this.refineSteps = this.refineSteps.map((s, i) => {
      if (i < index) return { ...s, state: "done" };
      if (i === index) return { ...s, state: "running" };
      return { ...s, state: "pending" };
    });
  }

  endRefine() {
    this.refineSteps = this.refineSteps.map((s) => ({ ...s, state: "done" }));
    this.refiningId = null;
    this.refineStartedAt = null;
  }

  async refresh() {
    try {
      const [ideas, tasks, paused] = await Promise.all([
        listIdeas(),
        listTasks(),
        queuePaused(),
      ]);
      this.ideas = ideas;
      this.tasks = tasks;
      this.paused = paused;
      const running = tasks.find((t) => t.status === "running");
      if (running) this.activeTaskId = running.id;
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    }
  }

  async loadLog(taskId: number) {
    const lines = await getRunLog(taskId);
    this.liveLog = lines.map((l) => ({
      taskId: l.taskId,
      stream: l.stream,
      line: l.line,
    }));
    this.activeTaskId = taskId;
  }

  pushLog(ev: LogEvent) {
    this.activeTaskId = ev.taskId;
    this.liveLog = [...this.liveLog, ev];
    if (this.liveLog.length > 4000) {
      this.liveLog = this.liveLog.slice(-3000);
    }
  }

  notice(msg: string) {
    this.flash = msg;
    setTimeout(() => {
      if (this.flash === msg) this.flash = null;
    }, 3200);
  }
}

export const store = new DrainStore();
