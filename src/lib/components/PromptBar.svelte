<script lang="ts">
  import Split from "$lib/craft/Split.svelte";

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

  const empty = $derived(!value.trim() || disabled);
</script>

<div class="craft-cta">
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
      <button
        class="craft-btn craft-btn--ghost"
        type="button"
        onclick={oncapture}
        disabled={empty}
      >
        capture
      </button>
      <button
        class="craft-btn craft-btn--primary"
        type="button"
        onclick={onrefine}
        disabled={empty}
      >
        refine
      </button>
      <Split
        label="compose"
        items={[
          { id: "capture", label: "capture", mark: "c", onSelect: empty ? undefined : oncapture },
          { id: "refine", label: "refine", mark: "r", onSelect: empty ? undefined : onrefine },
        ]}
      />
    </div>
  </div>
</div>

<style>
  textarea {
    width: 100%;
    resize: none;
    border: none;
    background: transparent;
    padding: 0 0 12px;
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
    gap: 12px;
  }
  .hint {
    font-family: var(--font-sf-mono);
    font-size: 11px;
    color: var(--craft-muted);
    font-variant-numeric: tabular-nums;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .craft-btn:disabled {
    opacity: 0.45;
    cursor: default;
  }
</style>
