<script lang="ts">
  let {
    value = $bindable(""),
    disabled = false,
    onrefine,
    oncapture,
  }: {
    value: string;
    disabled?: boolean;
    onrefine: () => void;
    oncapture: () => void;
  } = $props();

  function key(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      onrefine();
    }
  }
</script>

<div class="bar">
  <textarea
    rows="3"
    placeholder="dump a messy idea. refine splits it into checkable tasks."
    bind:value
    onkeydown={key}
    {disabled}
  ></textarea>
  <div class="row">
    <span class="hint">⌘↩ refine</span>
    <div class="actions">
      <button class="ghost" type="button" onclick={oncapture} disabled={disabled || !value.trim()}>
        capture
      </button>
      <button class="primary" type="button" onclick={onrefine} disabled={disabled || !value.trim()}>
        refine
      </button>
    </div>
  </div>
</div>

<style>
  .bar {
    background: var(--craft-surface);
    border-radius: 18px;
    padding: 12px 12px 10px;
  }
  textarea {
    width: 100%;
    resize: none;
    border: none;
    background: transparent;
    padding: 4px 6px;
    min-height: 72px;
    line-height: 1.5;
    font-size: 1.02rem;
    letter-spacing: -0.015em;
  }
  textarea:focus {
    outline: none;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 4px 2px;
  }
  .hint {
    font-family: var(--font-sf-mono);
    font-size: 11px;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
  .actions {
    display: flex;
    gap: 6px;
  }
</style>
