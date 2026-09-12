<script lang="ts">
  export type RecommendOption = {
    key: string;
    body: string;
    short: string;
    signal: 0 | 1 | 2 | 3;
    label: string;
    cta: string;
    onAccept?: () => void;
  };

  let { prompt, options }: { prompt: string; options: RecommendOption[] } = $props();

  let selected = $state(0);
  let open = $state(false);
  let accepted = $state(false);

  const active = $derived(options[selected]);

  function tone(signal: number) {
    if (signal >= 3) return "var(--craft-heat-3)";
    if (signal >= 2) return "var(--craft-heat-2)";
    if (signal >= 1) return "var(--craft-heat-1)";
    return "var(--craft-border)";
  }
</script>

{#if active}
  <div class="craft-recommend">
    <div class="craft-recommend__body">
      <span class="craft-recommend__prompt">{prompt}</span>
      <p class="craft-recommend__copy">{active.body}</p>
    </div>

    <div class="craft-recommend__drawer" data-open={open}>
      <div class="craft-recommend__drawer-clip">
        <div class="craft-recommend__drawer-inner">
          <p class="craft-recommend__drawer-label">other options</p>
          {#each options as option, i}
            {#if i !== selected}
              <button
                type="button"
                class="craft-recommend__choice"
                onclick={() => {
                  selected = i;
                  accepted = false;
                  open = false;
                }}
              >
                <span class="craft-meter" aria-hidden="true">
                  {#each [0, 1, 2] as bar}
                    <span
                      class="craft-meter__bar"
                      style="background: {bar < option.signal ? tone(option.signal) : 'var(--craft-border)'}"
                    ></span>
                  {/each}
                </span>
                <span class="craft-recommend__choice-short">{option.short}</span>
                <span class="craft-recommend__choice-meta">{option.label}</span>
              </button>
            {/if}
          {/each}
        </div>
      </div>
    </div>

    <div class="craft-recommend__footer">
      <span class="craft-recommend__status">
        <span class="craft-meter" aria-hidden="true">
          {#each [0, 1, 2] as bar}
            <span
              class="craft-meter__bar"
              style="background: {bar < active.signal ? tone(active.signal) : 'var(--craft-border)'}"
            ></span>
          {/each}
        </span>
        <span>{active.label}</span>
      </span>
      <span class="craft-recommend__actions">
        <button
          type="button"
          class="craft-btn craft-btn--ghost"
          aria-expanded={open}
          onclick={() => (open = !open)}
        >
          alternatives
        </button>
        <button
          type="button"
          class="craft-btn {accepted ? 'craft-btn--done' : 'craft-btn--primary'}"
          onclick={() => {
            accepted = true;
            active.onAccept?.();
          }}
        >
          {accepted ? "accepted" : active.cta}
        </button>
      </span>
    </div>
  </div>
{/if}
