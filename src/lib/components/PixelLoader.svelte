<script lang="ts">
  let { elapsed = 0, label = "refining" }: { elapsed?: number; label?: string } = $props();

  const cells = $derived(Array.from({ length: 20 }, (_, i) => i));
</script>

<div class="loader" role="status" aria-live="polite">
  <div class="grid" aria-hidden="true">
    {#each cells as i}
      <span class="cell" style="animation-delay: {(i % 5) * 80 + Math.floor(i / 5) * 40}ms"></span>
    {/each}
  </div>
  <div class="meta">
    <span class="label">{label}</span>
    <span class="time">{elapsed.toFixed(1)}s</span>
  </div>
</div>

<style>
  .loader {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(5, 8px);
    grid-template-rows: repeat(4, 8px);
    gap: 3px;
  }
  .cell {
    width: 8px;
    height: 8px;
    border-radius: 2px;
    background: var(--craft-heat-0);
    animation: heat 1.2s ease-in-out infinite;
  }
  .meta {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .label {
    font-size: 12px;
    font-weight: 550;
    letter-spacing: -0.015em;
  }
  .time {
    font-variant-numeric: tabular-nums;
    font-size: 11px;
    color: var(--muted);
    font-family: var(--mono);
  }
  @keyframes heat {
    0%,
    100% {
      background: var(--craft-heat-0);
    }
    50% {
      background: var(--craft-heat-2);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .cell {
      animation: none;
      background: var(--craft-heat-1);
    }
  }
</style>
