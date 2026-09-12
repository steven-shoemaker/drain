<script lang="ts">
  const HOLD_MS = 700;

  let {
    label,
    doneLabel = "done",
    accent = "var(--craft-fg)",
    tint = "var(--craft-surface)",
    oncommit,
  }: {
    label: string;
    doneLabel?: string;
    accent?: string;
    tint?: string;
    oncommit?: () => void;
  } = $props();

  let fill = $state(0);
  let done = $state(false);
  let frame = 0;
  let started = 0;

  function stop() {
    if (frame) cancelAnimationFrame(frame);
    frame = 0;
  }

  function startHold() {
    if (done) return;
    stop();
    started = performance.now();
    const tick = (now: number) => {
      fill = Math.min(1, (now - started) / HOLD_MS);
      if (fill >= 1) {
        done = true;
        fill = 1;
        oncommit?.();
        return;
      }
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
  }

  function cancelHold() {
    if (done) return;
    stop();
    fill = 0;
  }

  function key(e: KeyboardEvent) {
    if (done || e.repeat) return;
    if (e.key !== "Enter" && e.key !== " ") return;
    e.preventDefault();
    done = true;
    fill = 1;
    oncommit?.();
  }
</script>

<button
  type="button"
  class="craft-hold"
  style="background: {tint}"
  aria-label={done ? doneLabel : label}
  aria-pressed={done}
  onpointerdown={startHold}
  onpointerup={cancelHold}
  onpointerleave={cancelHold}
  onpointercancel={cancelHold}
  oncontextmenu={(e) => e.preventDefault()}
  onkeydown={key}
>
  <span class="craft-hold__idle">
    <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
      <path
        d="M4 4h8M6 4V3h4v1M5 6.5l.5 7h5l.5-7"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </span>
  <span
    class="craft-hold__wipe"
    style="clip-path: inset({(1 - fill) * 100}% 0 0 0); background: {accent}"
  >
    <span class="craft-hold__icon">
      {#if done}
        <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
          <path
            d="M2 6.2 4.7 9 10 3"
            fill="none"
            stroke="#fff"
            stroke-width="1.8"
            stroke-linecap="round"
          />
        </svg>
      {:else}
        <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
          <path
            d="M4 4h8M6 4V3h4v1M5 6.5l.5 7h5l.5-7"
            fill="none"
            stroke="#fff"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      {/if}
    </span>
  </span>
</button>
