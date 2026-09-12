<script lang="ts">
  import { onMount } from "svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import { listenLog, listenStatus } from "$lib/api";
  import { store } from "$lib/stores.svelte";
  import "../app.css";

  let { children } = $props();

  onMount(() => {
    void store.refresh();
    let unlog: (() => void) | undefined;
    let unstat: (() => void) | undefined;
    void listenLog((ev) => {
      store.pushLog(ev);
    }).then((u) => (unlog = u));
    void listenStatus((ev) => {
      if (ev.status === "idea_refining" && ev.message) {
        store.startRefine(Number(ev.message));
      }
      if (ev.status === "idea_trace" && ev.message) {
        const [, idx] = ev.message.split(":");
        store.advanceTrace(Number(idx));
      }
      if (ev.status === "idea_refined" || ev.status === "idea_refused" || ev.status === "idea_failed") {
        store.endRefine();
      }
      void store.refresh();
      if (ev.status === "idea_failed") {
        store.notice(ev.message ?? "refine failed");
      }
      if (ev.status === "failed") {
        store.notice(ev.message ?? "run failed");
      }
      if (ev.taskId && (ev.status === "running" || ev.status === "awaiting_approve")) {
        store.activeTaskId = ev.taskId;
      }
    }).then((u) => (unstat = u));
    return () => {
      unlog?.();
      unstat?.();
    };
  });
</script>

<div class="shell">
  <div class="drag" data-tauri-drag-region></div>
  <div class="body">
    <Sidebar />
    <main>
      {@render children()}
    </main>
  </div>
</div>
{#if store.flash}
  <div class="flash">{store.flash}</div>
{/if}

<style>
  .shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100%;
    background: var(--bg);
  }
  .drag {
    height: 36px;
    flex-shrink: 0;
    -webkit-app-region: drag;
    app-region: drag;
  }
  .body {
    display: flex;
    flex: 1;
    min-height: 0;
  }
  main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    -webkit-app-region: no-drag;
    app-region: no-drag;
  }
</style>
