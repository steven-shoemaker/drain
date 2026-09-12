<script lang="ts">
  import Check from "$lib/craft/Check.svelte";
  import type { RefinedTask } from "$lib/types";

  let {
    draft = $bindable(),
    onremove,
  }: {
    draft: RefinedTask;
    onremove: () => void;
  } = $props();

  function listText(items: string[]) {
    return items.join("\n");
  }
  function parseList(raw: string) {
    return raw
      .split("\n")
      .map((s) => s.replace(/^[-*]\s*/, "").trim())
      .filter(Boolean);
  }
</script>

<article class="card">
  <header>
    <input
      class="title"
      maxlength="80"
      bind:value={draft.title}
      placeholder="imperative title"
    />
    <button class="craft-btn craft-btn--ghost" type="button" onclick={onremove}>remove</button>
  </header>
  <p class="branch">{draft.suggestedBranch}</p>

  <label>
    acceptance criteria
    {#if draft.acceptanceCriteria.length}
      <ul class="checks">
        {#each draft.acceptanceCriteria as c, i}
          <li>
            <Check label={c} />
            <input
              value={c}
              oninput={(e) => {
                draft.acceptanceCriteria[i] = (e.currentTarget as HTMLInputElement).value;
                draft.acceptanceCriteria = draft.acceptanceCriteria;
              }}
            />
          </li>
        {/each}
      </ul>
    {/if}
    <textarea
      rows="3"
      value={listText(draft.acceptanceCriteria)}
      oninput={(e) =>
        (draft.acceptanceCriteria = parseList((e.currentTarget as HTMLTextAreaElement).value))}
    ></textarea>
  </label>
  <label>
    out of scope
    <textarea
      rows="3"
      value={listText(draft.outOfScope)}
      oninput={(e) =>
        (draft.outOfScope = parseList((e.currentTarget as HTMLTextAreaElement).value))}
    ></textarea>
  </label>
  <label>
    branch
    <input bind:value={draft.suggestedBranch} />
  </label>
</article>

<style>
  .card {
    background: var(--bg);
    border-radius: 18px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    box-shadow: var(--shadow-border);
  }
  header {
    display: flex;
    gap: 10px;
    align-items: flex-start;
  }
  .title {
    flex: 1;
    font-family: var(--font-sf-rounded);
    font-size: 1.05rem;
    font-weight: 600;
    letter-spacing: -0.02em;
    border: none;
    background: transparent;
    padding: 0;
    color: var(--ink);
  }
  .branch {
    margin: -4px 0 0;
    font-size: 11px;
    color: var(--muted);
    font-family: var(--mono);
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 5px;
    font-size: 11px;
    font-weight: 500;
    letter-spacing: -0.01em;
    color: var(--muted);
  }
  .checks {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .checks li {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .checks input {
    flex: 1;
    font-size: 13px;
    font-weight: 400;
    background: transparent;
    border: 0;
    padding: 0;
    color: var(--craft-fg);
  }
  textarea,
  input:not(.title) {
    font-size: 13px;
    font-weight: 400;
    letter-spacing: -0.15px;
    color: var(--ink);
    background: var(--craft-surface);
    border: 0;
    border-radius: 10px;
    padding: 8px 10px;
    font-family: inherit;
    resize: vertical;
  }
  .checks input {
    background: transparent;
    padding: 0;
    border-radius: 0;
  }
</style>
