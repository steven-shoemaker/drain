<script lang="ts">
  import type { ThinkStep } from "$lib/types";

  let {
    open = $bindable(true),
    steps,
  }: {
    open?: boolean;
    steps: ThinkStep[];
  } = $props();

  const running = $derived(steps.some((s) => s.state === "running"));
  const doneCount = $derived(steps.filter((s) => s.state === "done").length);
</script>

<div class="think">
  <button type="button" class="head" onclick={() => (open = !open)} aria-expanded={open}>
    <span class="dot" class:live={running}></span>
    <span class="label">{running ? "thinking" : "trace"}</span>
    <span class="count">{doneCount}/{steps.length}</span>
    <span class="chev">{open ? "▾" : "▸"}</span>
  </button>
  {#if open}
    <ol>
      {#each steps as step}
        <li class={step.state}>
          <i></i>
          {step.label}
        </li>
      {/each}
    </ol>
  {/if}
</div>

<style>
  .think {
    background: var(--bg);
    border-radius: 12px;
    box-shadow: var(--shadow-border);
    overflow: hidden;
  }
  .head {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    height: 34px;
    padding: 0 10px;
    background: transparent;
    border: none;
    font-size: 12px;
    font-weight: 550;
    cursor: pointer;
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 99px;
    background: var(--craft-heat-0);
  }
  .dot.live {
    background: var(--craft-heat-3);
  }
  .count,
  .chev {
    color: var(--muted);
    font-weight: 400;
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    font-family: var(--mono);
  }
  .label {
    flex: 1;
    text-align: left;
  }
  ol {
    margin: 0;
    padding: 0 10px 10px 28px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  li {
    position: relative;
    font-size: 12px;
    color: var(--muted);
    list-style: none;
  }
  li i {
    position: absolute;
    left: -16px;
    top: 6px;
    width: 6px;
    height: 6px;
    border-radius: 99px;
    background: var(--craft-heat-0);
  }
  li.running {
    color: var(--ink);
  }
  li.running i {
    background: var(--craft-heat-2);
  }
  li.done {
    color: var(--ink);
  }
  li.done i {
    background: var(--craft-heat-4);
  }
</style>
