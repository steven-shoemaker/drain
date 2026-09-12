<script lang="ts">
  const LEVELS = [
    "var(--craft-heat-0)",
    "var(--craft-heat-1)",
    "var(--craft-heat-2)",
    "var(--craft-heat-3)",
    "var(--craft-heat-4)",
  ] as const;

  let {
    weeks = 18,
    levels,
  }: {
    weeks?: number;
    levels?: number[];
  } = $props();

  function seed(week: number, day: number) {
    const n = (week * 7 + day) * 17;
    const r = (n % 11) / 10;
    if (r < 0.28) return 0;
    if (r < 0.48) return 1;
    if (r < 0.68) return 2;
    if (r < 0.86) return 3;
    return 4;
  }

  function levelAt(week: number, day: number) {
    const i = week * 7 + day;
    if (levels && i < levels.length) return Math.max(0, Math.min(4, levels[i] ?? 0));
    if (levels) return 0;
    return seed(week, day);
  }
</script>

<div class="craft-contrib" aria-hidden="true">
  <div class="craft-contrib__grid" style="--contrib-weeks: {weeks}">
    {#each Array.from({ length: weeks }, (_, w) => w) as w}
      <div class="craft-contrib__week">
        {#each Array.from({ length: 7 }, (_, d) => d) as d}
          <span class="craft-contrib__day" style="background: {LEVELS[levelAt(w, d)]}"></span>
        {/each}
      </div>
    {/each}
  </div>
</div>
