<script lang="ts">
  import ApprovalCard from "$lib/components/ApprovalCard.svelte";
  import StatusChip from "$lib/components/StatusChip.svelte";
  import ToolChip from "$lib/components/ToolChip.svelte";
  import Check from "$lib/craft/Check.svelte";
  import Hold from "$lib/craft/Hold.svelte";
  import { approveTask, cancelRun, openUrl, rejectTask } from "$lib/api";
  import { store } from "$lib/stores.svelte";
  import { onMount } from "svelte";

  let note = $state("");
  let err = $state<string | null>(null);
  let logEl: HTMLElement | undefined = $state();

  const task = $derived(
    store.tasks.find((t) => t.id === store.focusedTaskId) ??
      store.running ??
      store.awaiting ??
      store.tasks[0] ??
      null,
  );

  const tools = $derived.by(() => {
    const names = new Set<string>();
    for (const line of store.liveLog) {
      if (/\bgit\b/.test(line.line)) names.add("git");
      if (/\bclaude\b/.test(line.line)) names.add("claude");
      if (/\bgh\b/.test(line.line)) names.add("gh");
    }
    return [...names];
  });

  $effect(() => {
    store.liveLog.length;
    if (logEl) logEl.scrollTop = logEl.scrollHeight;
  });

  onMount(() => {
    const id = store.focusedTaskId;
    if (id) void store.loadLog(id);
  });

  async function approve() {
    if (!task) return;
    err = null;
    try {
      await approveTask(task.id);
      await store.refresh();
      store.notice("approved");
    } catch (e) {
      err = e instanceof Error ? e.message : String(e);
    }
  }

  async function reject() {
    if (!task) return;
    err = null;
    try {
      await rejectTask(task.id, note);
      note = "";
      await store.refresh();
    } catch (e) {
      err = e instanceof Error ? e.message : String(e);
    }
  }

  async function cancel() {
    err = null;
    try {
      await cancelRun();
      await store.refresh();
    } catch (e) {
      err = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<div class="screen run">
  <header>
    <div>
      <h1>run</h1>
      {#if task}
        <p class="lede">{task.title}</p>
      {:else}
        <p class="lede">idle. start the queue when a task is ready.</p>
      {/if}
    </div>
    {#if task}
      <StatusChip status={task.status} />
    {/if}
  </header>

  {#if err}
    <p class="err">{err}</p>
  {/if}

  {#if tools.length}
    <div class="chips">
      {#each tools as name}
        <ToolChip {name} active={task?.status === "running"} />
      {/each}
    </div>
  {/if}

  {#if task?.status === "awaiting_approve" || task?.status === "awaiting_pr"}
    <ApprovalCard
      {task}
      bind:note
      onapprove={() => void approve()}
      onreject={() => void reject()}
      onopen={() => void openUrl(task.prUrl ?? "")}
    />
  {:else if task}
    <ul class="criteria">
      {#each task.acceptanceCriteria as c}
        <li>
          <Check label={c} />
          <span>{c}</span>
        </li>
      {/each}
    </ul>
  {/if}

  <div class="toolbar">
    {#if task?.status === "running"}
      <Hold label="hold to cancel" doneLabel="cancelled" oncommit={() => void cancel()} />
    {/if}
    {#if task?.status === "failed"}
      <span class="err" style="margin:0">{task.failNote}</span>
    {/if}
  </div>

  <pre class="log" bind:this={logEl}>
{#if store.liveLog.length === 0}<span class="dim">no log yet.</span>{/if}{#each store.liveLog as line}<span class={line.stream}>{line.line}{"\n"}</span>{/each}</pre>
</div>

<style>
  .run {
    display: flex;
    flex-direction: column;
    max-width: none;
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 16px;
  }
  .chips {
    display: flex;
    gap: 6px;
    margin: 0 0 12px;
  }
  .criteria {
    margin: 0 0 10px;
    padding: 0;
    list-style: none;
    color: var(--craft-muted);
    font-size: 12.5px;
    max-width: 680px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .criteria li {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .log {
    flex: 1;
    min-height: 280px;
    margin: 8px 0 0;
    padding: 14px 16px;
    border-radius: 18px;
    background: var(--ink);
    color: #f6f6f6;
    font-family: var(--mono);
    font-size: 11.5px;
    line-height: 1.55;
    overflow: auto;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .dim {
    color: #9e9e9e;
  }
  .log :global(.system) {
    color: var(--craft-heat-1);
  }
  .log :global(.stderr) {
    color: var(--pastel-peach);
  }
  .log :global(.stdout) {
    color: #f6f6f6;
  }
</style>
