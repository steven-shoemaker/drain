<script lang="ts">
  let {
    items,
    spread = "hover",
  }: {
    items: string[];
    spread?: "hover" | "always";
  } = $props();

  function pose(index: number, count: number) {
    if (count <= 1) {
      return {
        rest: { rotate: 0, y: 0, gap: 0 },
        open: { rotate: 0, y: 0, gap: 0 },
        z: 1,
      };
    }
    const t = index / (count - 1);
    const mid = (count - 1) / 2;
    const z = 1 + Math.round((1 - Math.abs(index - mid) / Math.max(mid, 1)) * 2);
    return {
      rest: { rotate: -4 + t * 8, y: index * 2, gap: index === 0 ? 0 : -68 },
      open: {
        rotate: -11 + t * 22,
        y: 4 + 10 * (2 * t - 1) ** 2,
        gap: index === 0 ? 0 : -22,
      },
      z,
    };
  }
</script>

<div class="craft-fan" data-spread={spread} role="list">
  {#each items as item, i}
    {@const p = pose(i, items.length)}
    <div
      class="craft-fan__item"
      role="listitem"
      tabindex="0"
      style="
        --fan-i: {i};
        --fan-rot-rest: {p.rest.rotate}deg;
        --fan-rot-open: {p.open.rotate}deg;
        --fan-y-rest: {p.rest.y}px;
        --fan-y-open: {p.open.y}px;
        --fan-gap-rest: {p.rest.gap}px;
        --fan-gap-open: {p.open.gap}px;
        --fan-delay: {i * 40}ms;
        z-index: {p.z};
      "
    >
      {item}
    </div>
  {/each}
</div>
