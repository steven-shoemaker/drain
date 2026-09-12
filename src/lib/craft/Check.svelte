<script lang="ts">
  let {
    label,
    checked = $bindable(false),
    onchange,
  }: {
    label: string;
    checked?: boolean;
    onchange?: (checked: boolean) => void;
  } = $props();

  function toggle() {
    checked = !checked;
    onchange?.(checked);
  }
</script>

<button
  type="button"
  class="craft-check"
  role="checkbox"
  aria-checked={checked}
  aria-label={label}
  onclick={toggle}
>
  {#if checked}
    <span class="craft-check__tick">
      <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
        <path
          d="M2 6.2 4.7 9 10 3"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </span>
  {/if}
</button>
