<script lang="ts">
  import StatusChip from "$lib/components/StatusChip.svelte";
  import type { Task } from "$lib/types";

  let {
    task,
    idle,
    expanded = false,
    onup,
    ondown,
    onopen,
  }: {
    task: Task;
    idle: boolean;
    expanded?: boolean;
    onup: () => void;
    ondown: () => void;
    onopen: () => void;
  } = $props();

  let open = $state(false);

  $effect(() => {
    if (task.status === "running" || task.status === "awaiting_approve" || expanded) {
      open = true;
    }
  });
</script>

<article class="row" class:live={task.status === "running"}>
  <button class="hit" type="button" onclick={onopen}>
    <div class="main">
      <div class="title">{task.title}</div>
      <div class="meta">
        <span class="branch">{task.suggestedBranch}</span>
        {#if task.prUrl}<span class="pr">pr</span>{/if}
        {#if task.failNote}<span class="fail">{task.failNote}</span>{/if}
      </div>
    </div>
    <StatusChip status={task.status} />
  </button>
  {#if idle && task.status === "ready"}
    <div class="move">
      <button type="button" onclick={onup} aria-label="Move up">↑</button>
      <button type="button" onclick={ondown} aria-label="Move down">↓</button>
    </div>
  {/if}
  <button class="toggle" type="button" onclick={() => (open = !open)} aria-expanded={open}>
    {open ? "hide" : `${task.acceptanceCriteria.length} checks`}
  </button>
  {#if open}
    <ul>
      {#each task.acceptanceCriteria as c}
        <li class={task.status}>{c}</li>
      {/each}
    </ul>
  {/if}
</article>

<style>
  .row {
    display: grid;
    grid-template-columns: 1fr auto auto;
    gap: 4px 8px;
    align-items: center;
    padding: 6px 4px 10px;
    margin: 0 -8px;
    border-radius: 12px;
    transition: background-color 160ms ease;
  }
  @media (hover: hover) {
    .row:hover {
      background: color-mix(in srgb, var(--ink) 4%, transparent);
    }
  }
  .row.live {
    background: var(--craft-heat-0);
  }
  .hit {
    grid-column: 1;
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    padding: 8px 8px;
  }
  .main {
    flex: 1;
    min-width: 0;
  }
  .title {
    font-size: 0.98rem;
    font-weight: 500;
    letter-spacing: -0.015em;
    overflow-wrap: anywhere;
  }
  .meta {
    display: flex;
    gap: 8px;
    margin-top: 3px;
    font-size: 11px;
    color: var(--muted);
    min-width: 0;
  }
  .branch {
    font-family: var(--mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .pr {
    color: var(--ink);
    font-weight: 600;
  }
  .fail {
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .move {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .move button,
  .toggle {
    border-radius: 8px;
    background: var(--bg);
    border: 0;
    box-shadow: var(--shadow-border);
    color: var(--muted);
    font-size: 11px;
    cursor: pointer;
  }
  .move button {
    width: 22px;
    height: 18px;
    padding: 0;
  }
  .toggle {
    height: 22px;
    padding: 0 8px;
  }
  .move button:active,
  .toggle:active {
    transform: scale(0.96);
  }
  ul {
    grid-column: 1 / -1;
    margin: 0 8px 4px;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  li {
    font-size: 12px;
    color: var(--muted);
    padding-left: 14px;
    position: relative;
  }
  li::before {
    content: "";
    position: absolute;
    left: 2px;
    top: 6px;
    width: 6px;
    height: 6px;
    border-radius: 99px;
    background: var(--craft-heat-0);
  }
  li.running::before,
  li.awaiting_approve::before,
  li.awaiting_pr::before {
    background: var(--craft-heat-2);
  }
  li.done::before {
    background: var(--craft-heat-4);
  }
  li.failed::before {
    background: var(--muted);
  }
</style>
