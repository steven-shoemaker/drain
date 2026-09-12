<script lang="ts">
  export type SpreadItem = { href: string; label: string };

  let { items }: { items: SpreadItem[] } = $props();

  const SHUFFLE_Y = [4, -3, -5, 2, 5, -4, 3];

  function fanZ(index: number, count: number) {
    if (count <= 1) return 1;
    const mid = (count - 1) / 2;
    return 1 + Math.round((1 - Math.abs(index - mid) / mid) * 4);
  }
</script>

{#if items.length}
  <div class="craft-spread" style="--spread-slots: {Math.max(items.length, 2)}">
    <div class="craft-spread__row">
      {#each items as item, i}
        {@const t = items.length <= 1 ? 0.5 : i / (items.length - 1)}
        <a
          href={item.href}
          class="craft-spread__hit"
          style="
            --spread-t: {t};
            --spread-z: {fanZ(i, items.length)};
            --spread-y: {SHUFFLE_Y[i % SHUFFLE_Y.length]}px;
          "
        >
          <span class="craft-spread__face">{item.label}</span>
        </a>
      {/each}
    </div>
  </div>
{/if}
