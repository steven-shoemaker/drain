<script lang="ts">
  import QueueRow from "$lib/components/QueueRow.svelte";
  import HeatMap from "$lib/craft/HeatMap.svelte";
  import Spread from "$lib/craft/Spread.svelte";
  import {
    pauseQueue,
    reorderTasks,
    retryTask,
    skipFailed,
    startQueue,
  } from "$lib/api";
  import { store } from "$lib/stores.svelte";
  import { goto } from "$app/navigation";
  import { STATUS_HEAT, type Task } from "$lib/types";

  let err = $state<string | null>(null);
  let filter = $state<"all" | "ready" | "waiting" | "done" | "failed">("all");

  const idle = $derived(!store.running);

  const shown = $derived.by(() => {
    return store.tasks.filter((t) => {
      switch (filter) {
        case "all":
          return true;
        case "ready":
          return t.status === "ready" || t.status === "running";
        case "waiting":
          return t.status === "awaiting_pr" || t.status === "awaiting_approve";
        case "done":
          return t.status === "done";
        case "failed":
          return t.status === "failed";
        default: {
          const _x: never = filter;
          return _x;
        }
      }
    });
  });

  const counts = $derived({
    all: store.tasks.length,
    ready: store.tasks.filter((t) => t.status === "ready" || t.status === "running").length,
    waiting: store.tasks.filter(
      (t) => t.status === "awaiting_pr" || t.status === "awaiting_approve",
    ).length,
    done: store.tasks.filter((t) => t.status === "done").length,
    failed: store.tasks.filter((t) => t.status === "failed").length,
  });

  async function start() {
    err = null;
    try {
      await startQueue();
      await store.refresh();
    } catch (e) {
      err = e instanceof Error ? e.message : String(e);
    }
  }

  async function pause() {
    await pauseQueue();
    await store.refresh();
  }

  async function move(task: Task, dir: -1 | 1) {
    const ready = store.tasks.filter((t) => t.status === "ready");
    const i = ready.findIndex((t) => t.id === task.id);
    const j = i + dir;
    if (i < 0 || j < 0 || j >= ready.length) return;
    const ids = ready.map((t) => t.id);
    [ids[i], ids[j]] = [ids[j], ids[i]];
    try {
      store.tasks = await reorderTasks(ids);
    } catch (e) {
      err = e instanceof Error ? e.message : String(e);
    }
  }

  async function retry(id: number) {
    try {
      await retryTask(id);
      await store.refresh();
    } catch (e) {
      err = e instanceof Error ? e.message : String(e);
    }
  }

  async function skip(id: number) {
    try {
      await skipFailed(id);
      await store.refresh();
    } catch (e) {
      err = e instanceof Error ? e.message : String(e);
    }
  }

  function openRun(task: Task) {
    store.activeTaskId = task.id;
    void store.loadLog(task.id);
    void goto("/run");
  }

  const heat = $derived(store.tasks.map((t) => STATUS_HEAT[t.status]));
  const spread = $derived(
    store.tasks
      .filter((t) => t.status === "ready" || t.status === "running")
      .map((t) => ({ href: "/run", label: t.title })),
  );

  const filters: { id: typeof filter; label: string }[] = [
    { id: "all", label: "all" },
    { id: "ready", label: "ready" },
    { id: "waiting", label: "waiting" },
    { id: "done", label: "done" },
    { id: "failed", label: "failed" },
  ];
</script>

<div class="screen">
  <h1>queue</h1>
  <p class="lede">one agent. next starts only after done, skip, or fail handled.</p>

  {#if err}
    <p class="err">{err}</p>
  {/if}

  <div class="toolbar">
    {#if store.paused}
      <button class="craft-btn craft-btn--primary" type="button" onclick={() => void start()}>start</button>
    {:else}
      <button class="craft-btn craft-btn--ghost" type="button" onclick={() => void pause()}>pause</button>
    {/if}
    <span class="hint">
      {#if store.running}
        running — reorder locked
      {:else if store.paused}
        paused — start drains the next ready task
      {:else}
        live — will pick up the next ready task
      {/if}
    </span>
  </div>

  <HeatMap weeks={18} levels={heat} />
  <Spread items={spread} />

  <div class="filters" role="tablist">
    {#each filters as f}
      <button
        type="button"
        class="craft-btn {filter === f.id ? 'craft-btn--primary' : 'craft-btn--ghost'}"
        onclick={() => (filter = f.id)}
      >
        {f.label}
        <em>{counts[f.id]}</em>
      </button>
    {/each}
  </div>

  <div class="stack">
    {#if shown.length === 0}
      <p class="craft-empty">
        {store.tasks.length === 0
          ? "queue is empty. refine something in inbox, then enqueue."
          : "nothing in this filter."}
      </p>
    {/if}
    {#each shown as task (task.id)}
      <div>
        <QueueRow
          {task}
          idle={idle}
          expanded={task.status === "running" || task.status === "awaiting_approve"}
          onup={() => void move(task, -1)}
          ondown={() => void move(task, 1)}
          onopen={() => openRun(task)}
        />
        {#if task.status === "failed"}
          <div class="failbar">
            <span>{task.failNote ?? "Failed"}</span>
            <button class="craft-btn craft-btn--ghost" type="button" onclick={() => void retry(task.id)}>retry</button>
            <button class="craft-btn craft-btn--ghost" type="button" onclick={() => void skip(task.id)}>skip / next</button>
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .hint {
    font-size: 12px;
    color: var(--muted);
  }
  .filters {
    display: flex;
    gap: 6px;
    margin: 8px 0 16px;
    flex-wrap: wrap;
  }
  em {
    font-style: normal;
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    color: var(--muted);
  }
  .failbar {
    display: flex;
    gap: 8px;
    align-items: center;
    padding: 6px 4px 2px 12px;
    font-size: 12px;
    color: var(--muted);
  }
  .failbar span {
    flex: 1;
  }
</style>
