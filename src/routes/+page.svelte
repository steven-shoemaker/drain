<script lang="ts">
  import DraftCard from "$lib/components/DraftCard.svelte";
  import PromptBar from "$lib/components/PromptBar.svelte";
  import RefusalCard from "$lib/components/RefusalCard.svelte";
  import ThinkingTrace from "$lib/components/ThinkingTrace.svelte";
  import Fan from "$lib/craft/Fan.svelte";
  import HeatMap from "$lib/craft/HeatMap.svelte";
  import Recommend from "$lib/craft/Recommend.svelte";
  import {
    createIdea,
    enqueueIdea,
    refineIdea,
    updateIdeaBody,
    updateIdeaDrafts,
  } from "$lib/api";
  import { store } from "$lib/stores.svelte";
  import type { Idea, RefinedTask } from "$lib/types";

  let body = $state("");
  let busyId = $state<number | null>(null);
  let enqueueing = $state<number | null>(null);
  let localError = $state<string | null>(null);
  let answers: Record<number, string> = $state({});

  async function capture() {
    localError = null;
    const text = body.trim();
    if (!text) return;
    try {
      await createIdea(text);
      body = "";
      await store.refresh();
    } catch (e) {
      localError = e instanceof Error ? e.message : String(e);
    }
  }

  async function captureAndRefine() {
    localError = null;
    const text = body.trim();
    if (!text) return;
    try {
      const idea = await createIdea(text);
      body = "";
      await store.refresh();
      await refine(idea);
    } catch (e) {
      localError = e instanceof Error ? e.message : String(e);
    }
  }

  async function refine(idea: Idea) {
    localError = null;
    busyId = idea.id;
    store.startRefine(idea.id);
    try {
      await refineIdea(idea.id);
      await store.refresh();
    } catch (e) {
      localError = e instanceof Error ? e.message : String(e);
      store.endRefine();
    } finally {
      busyId = null;
    }
  }

  async function persistDrafts(idea: Idea, drafts: RefinedTask[]) {
    await updateIdeaDrafts(idea.id, drafts);
    await store.refresh();
  }

  async function enqueue(idea: Idea) {
    localError = null;
    if (idea.drafts.some((d) => d.acceptanceCriteria.length === 0)) {
      localError = "every task needs at least one checkable acceptance criterion.";
      return;
    }
    enqueueing = idea.id;
    try {
      await persistDrafts(idea, idea.drafts);
      await enqueueIdea(idea.id, idea.drafts);
      store.notice(`enqueued ${idea.drafts.length}`);
      await store.refresh();
    } catch (e) {
      localError = e instanceof Error ? e.message : String(e);
    } finally {
      enqueueing = null;
    }
  }

  async function reRefine(idea: Idea) {
    const extra = (answers[idea.id] ?? "").trim();
    if (!extra) return;
    const merged = `${idea.body.trim()}\n\nClarification: ${extra}`;
    await updateIdeaBody(idea.id, merged);
    answers[idea.id] = "";
    await store.refresh();
    const fresh = store.ideas.find((i) => i.id === idea.id);
    if (fresh) await refine(fresh);
  }

  function isRefusal(idea: Idea) {
    return Boolean(idea.error?.includes("?")) && idea.drafts.length === 0;
  }
</script>

<div class="screen">
  <h1>inbox</h1>
  <p class="lede">messy in. the refiner splits it into tasks a stranger could check off.</p>

  {#if localError}
    <p class="err">{localError}</p>
  {/if}

  <div class="stack">
    <PromptBar
      bind:value={body}
      disabled={busyId !== null}
      oncapture={() => void capture()}
      onrefine={() => void captureAndRefine()}
    />

    {#if store.ideas.length === 0}
      <p class="craft-empty">nothing captured yet. try two jobs in one thought — refine will split them.</p>
      <Recommend
        prompt="how should this start?"
        options={[
          {
            key: "refine",
            body: "dump a messy thought, then refine splits it into checkable agent tasks.",
            short: "refine now",
            signal: 3,
            label: "strong default",
            cta: "start in the box",
          },
          {
            key: "capture",
            body: "capture only. refine later when you know which pile it belongs in.",
            short: "capture only",
            signal: 1,
            label: "park it",
            cta: "just capture",
          },
        ]}
      />
    {/if}

    {#each store.ideas as idea (idea.id)}
      <section class="idea">
        <p class="body">{idea.body}</p>

        {#if store.refiningId === idea.id || idea.status === "refining" || busyId === idea.id}
          <div class="working">
            <HeatMap weeks={12} />
            <ThinkingTrace steps={store.refineSteps} />
          </div>
        {:else if isRefusal(idea) && idea.error}
          <RefusalCard
            question={idea.error}
            answer={answers[idea.id] ?? ""}
            onanswer={(v: string) => (answers[idea.id] = v)}
            onretry={() => void reRefine(idea)}
          />
        {:else if idea.error}
          <p class="err">{idea.error}</p>
        {/if}

        <div class="toolbar">
          <button
            class="craft-btn craft-btn--ghost"
            type="button"
            disabled={busyId === idea.id || idea.status === "refining"}
            onclick={() => void refine(idea)}
          >
            {idea.drafts.length ? "re-refine" : "refine this"}
          </button>
          {#if idea.drafts.length > 0}
            <button
              class="craft-btn craft-btn--primary"
              type="button"
              disabled={enqueueing === idea.id}
              onclick={() => void enqueue(idea)}
            >
              {enqueueing === idea.id ? "enqueueing…" : `enqueue ${idea.drafts.length}`}
            </button>
          {/if}
        </div>

        {#if idea.drafts.length > 0}
          <Fan items={idea.drafts.map((d) => d.title)} />
          <div class="drafts">
            {#each idea.drafts as draft, i}
              <DraftCard
                bind:draft={idea.drafts[i]}
                onremove={() => {
                  idea.drafts = idea.drafts.filter((_, j) => j !== i);
                  void persistDrafts(idea, idea.drafts);
                }}
              />
            {/each}
          </div>
        {/if}
      </section>
    {/each}
  </div>
</div>

<style>
  .idea {
    padding: 22px 0 8px;
    border-top: 1px solid var(--craft-border);
  }
  .body {
    margin: 0 0 12px;
    white-space: pre-wrap;
    font-size: 1.02rem;
    line-height: 1.55;
    color: color-mix(in srgb, var(--ink) 78%, transparent);
    text-wrap: pretty;
  }
  .working {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-bottom: 12px;
  }
  .drafts {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: 8px;
  }
</style>
