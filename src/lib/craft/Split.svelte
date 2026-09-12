<script lang="ts">
  import { onMount } from "svelte";

  export type SplitItem = {
    id: string;
    label: string;
    mark?: string;
    onSelect?: () => void;
  };

  let { items, label = "open" }: { items: SplitItem[]; label?: string } = $props();

  let open = $state(false);
  let root: HTMLDivElement | undefined = $state();

  function arc(index: number, count: number) {
    if (count <= 1) return { fx: "0px", fy: "-64px" };
    const t = index / (count - 1);
    const a = Math.PI * (0.78 - 0.56 * t);
    const r = 64;
    return {
      fx: `${Math.round(Math.cos(a) * r)}px`,
      fy: `${Math.round(-Math.sin(a) * r)}px`,
    };
  }

  onMount(() => {
    const onPointer = (event: PointerEvent) => {
      if (!root?.contains(event.target as Node)) open = false;
    };
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") open = false;
    };
    document.addEventListener("pointerdown", onPointer);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("pointerdown", onPointer);
      document.removeEventListener("keydown", onKey);
    };
  });
</script>

<div bind:this={root} class="craft-split" data-open={open}>
  {#each items as item, i}
    {@const pos = arc(i, items.length)}
    <button
      type="button"
      class="craft-split__sat"
      tabindex={open ? 0 : -1}
      aria-label={item.label}
      style="--fx: {pos.fx}; --fy: {pos.fy}; --i: {i}"
      onclick={() => {
        item.onSelect?.();
        open = false;
      }}
    >
      {item.mark ?? item.label.slice(0, 1)}
    </button>
  {/each}
  <button
    type="button"
    class="craft-split__plus"
    aria-expanded={open}
    aria-label={open ? "close" : label}
    onclick={() => (open = !open)}
  >
    <svg width="14" height="14" viewBox="0 0 14 14" aria-hidden="true">
      <path
        d="M7 2v10M2 7h10"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
      />
    </svg>
  </button>
</div>
