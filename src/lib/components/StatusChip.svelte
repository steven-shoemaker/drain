<script lang="ts">
  import { STATUS_LABEL, type Status } from "$lib/types";

  let { status }: { status: Status } = $props();
</script>

<span class="chip {status}">{STATUS_LABEL[status]}</span>

<style>
  .chip {
    display: inline-flex;
    align-items: center;
    height: 18px;
    padding: 0 7px;
    border-radius: 999px;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.02em;
    line-height: 1;
    white-space: nowrap;
    background: var(--craft-heat-0);
    color: var(--craft-badge-fg);
  }
  .refining,
  .ready {
    background: var(--craft-heat-1);
    color: var(--ink);
  }
  .running {
    background: var(--craft-heat-2);
    color: var(--ink);
  }
  .awaiting_pr,
  .awaiting_approve {
    background: var(--craft-heat-3);
    color: #fff;
  }
  .done {
    background: var(--craft-heat-4);
    color: #fff;
  }
  .failed {
    background: var(--craft-surface);
    color: var(--muted);
  }
</style>
