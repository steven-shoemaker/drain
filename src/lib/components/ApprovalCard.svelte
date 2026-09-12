<script lang="ts">
  import Check from "$lib/craft/Check.svelte";
  import Hold from "$lib/craft/Hold.svelte";
  import type { Task } from "$lib/types";

  let {
    task,
    note = $bindable(""),
    onapprove,
    onreject,
    onopen,
  }: {
    task: Task;
    note: string;
    onapprove: () => void;
    onreject: () => void;
    onopen: () => void;
  } = $props();
</script>

<section class="card">
  <div class="kicker">human gate · 1 of 1</div>
  <h2>merge this work?</h2>
  <p class="title">{task.title}</p>
  {#if task.prUrl}
    <button class="link" type="button" onclick={onopen}>{task.prUrl}</button>
  {/if}
  <ul>
    {#each task.acceptanceCriteria as c}
      <li>
        <Check label={c} />
        <span>{c}</span>
      </li>
    {/each}
  </ul>
  <div class="row">
    <input placeholder="reject note" bind:value={note} />
    <Hold label="hold to reject" doneLabel="rejected" oncommit={onreject} />
    <button class="craft-btn craft-btn--primary" type="button" onclick={onapprove}>approve</button>
  </div>
</section>

<style>
  .card {
    background: var(--craft-surface);
    border-radius: 18px;
    padding: 18px 18px 14px;
    max-width: 720px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .kicker {
    font-family: var(--font-sf-mono);
    font-size: 0.68rem;
    font-weight: 500;
    letter-spacing: 0.08em;
    color: var(--muted);
  }
  h2 {
    margin: 0;
    font-family: var(--font-sf-rounded);
    font-size: 1.15rem;
    font-weight: 700;
    letter-spacing: -0.03em;
  }
  .title {
    margin: 0;
    color: var(--muted);
    font-size: 13px;
  }
  .link {
    align-self: flex-start;
    background: none;
    border: none;
    padding: 0;
    color: var(--ink);
    font-family: var(--mono);
    font-size: 12px;
    text-decoration: underline;
    text-decoration-thickness: 1px;
    text-underline-offset: 3px;
    text-decoration-color: color-mix(in srgb, var(--ink) 28%, transparent);
    cursor: pointer;
  }
  @media (hover: hover) {
    .link:hover {
      text-decoration-color: var(--ink);
    }
  }
  ul {
    margin: 4px 0 6px;
    padding: 0;
    list-style: none;
    color: var(--craft-muted);
    font-size: 12.5px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  li {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .row {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  input {
    flex: 1;
    height: 28px;
    border-radius: 8px;
    border: 0;
    background: var(--bg);
    padding: 0 10px;
    box-shadow: var(--shadow-border);
  }
</style>
