<script lang="ts">
  let { name, active = false }: { name: string; active?: boolean } = $props();
</script>

<span class="chip {name}" class:active>{name}</span>

<style>
  .chip {
    display: inline-flex;
    align-items: center;
    height: 20px;
    padding: 0 8px;
    border-radius: 999px;
    font-family: var(--mono);
    font-size: 10.5px;
    background: var(--craft-surface);
    color: var(--muted);
    letter-spacing: 0.02em;
    transition: background-color 180ms ease, color 180ms ease;
  }
  .chip.active {
    color: var(--ink);
  }
  @media (hover: hover) {
    .chip.git:hover,
    .chip.git.active {
      background: var(--pastel-peach);
      color: var(--ink);
    }
    .chip.claude:hover,
    .chip.claude.active {
      background: var(--pastel-sage);
      color: var(--ink);
    }
    .chip.gh:hover,
    .chip.gh.active {
      background: var(--pastel-sky);
      color: var(--ink);
    }
  }
</style>
