<script lang="ts">
  let {
    question,
    answer,
    onanswer,
    onretry,
  }: {
    question: string;
    answer: string;
    onanswer: (value: string) => void;
    onretry: () => void;
  } = $props();
</script>

<section class="card">
  <div class="kicker">needs a sharper target</div>
  <p class="q">{question}</p>
  <textarea
    rows="2"
    placeholder="answer, then re-refine"
    value={answer}
    oninput={(e) => onanswer((e.currentTarget as HTMLTextAreaElement).value)}
  ></textarea>
  <div class="row">
    <span class="sub">the refiner will not invent scope.</span>
    <button class="primary" type="button" onclick={onretry} disabled={!answer.trim()}>
      re-refine
    </button>
  </div>
</section>

<style>
  .card {
    background: var(--craft-surface);
    border-radius: 18px;
    padding: 16px 16px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .kicker {
    font-family: var(--font-sf-rounded);
    font-size: 0.92rem;
    font-weight: 600;
    letter-spacing: -0.02em;
    color: var(--ink);
  }
  .q {
    margin: 0;
    font-size: 14px;
    font-weight: 500;
    letter-spacing: -0.02em;
    color: color-mix(in srgb, var(--ink) 78%, transparent);
    text-wrap: pretty;
  }
  textarea {
    border: 0;
    background: var(--bg);
    border-radius: 10px;
    padding: 8px 10px;
    resize: vertical;
    box-shadow: var(--shadow-border);
  }
  .row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 10px;
  }
  .sub {
    font-size: 11px;
    color: var(--muted);
  }
</style>
