<script lang="ts">
  import { binaryStatus, getSettings, pickFolder, saveSettings } from "$lib/api";
  import { store } from "$lib/stores.svelte";
  import type { BinaryStatus, Settings } from "$lib/types";
  import { onMount } from "svelte";

  let form = $state<Settings>({
    defaultRepoPath: "",
    pathToClaude: "",
    pathToGh: "",
    autoRun: false,
    completionMode: "pr_then_approve",
    approveAction: "merge",
    permissionMode: "bypassPermissions",
  });
  let bins = $state<BinaryStatus | null>(null);
  let saved = $state(false);
  let err = $state<string | null>(null);

  onMount(async () => {
    form = await getSettings();
    bins = await binaryStatus();
  });

  async function browse() {
    const path = await pickFolder();
    if (path) form.defaultRepoPath = path;
  }

  async function save() {
    err = null;
    saved = false;
    try {
      form = await saveSettings(form);
      bins = await binaryStatus();
      saved = true;
      store.notice("settings saved");
    } catch (e) {
      err = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<div class="screen">
  <h1>settings</h1>
  <p class="lede">local only. claude code must already be logged in — no api keys here.</p>

  {#if err}
    <p class="err">{err}</p>
  {/if}

  <form
    class="stack"
    onsubmit={(e) => {
      e.preventDefault();
      void save();
    }}
  >
    <label>
      default repo
      <div class="row">
        <input
          bind:value={form.defaultRepoPath}
          placeholder="/Users/you/src/the-repo"
        />
        <button class="ghost" type="button" onclick={() => void browse()}>browse</button>
      </div>
    </label>

    <label>
      path to claude
      <input bind:value={form.pathToClaude} placeholder="claude (on PATH)" />
      <span class="help">
        {#if bins?.claude}found: {bins.claude}{:else}not found on PATH — set an absolute path.{/if}
      </span>
    </label>

    <label>
      path to gh
      <input bind:value={form.pathToGh} placeholder="gh (on PATH)" />
      <span class="help">
        {#if bins?.gh}found: {bins.gh}{:else}not found on PATH — set an absolute path.{/if}
      </span>
    </label>

    <label class="check">
      <input type="checkbox" bind:checked={form.autoRun} />
      auto-run next ready task after approve → done
    </label>

    <label>
      on success
      <select bind:value={form.completionMode}>
        <option value="pr_then_approve">create PR, wait for approve (default)</option>
        <option value="push_and_merge">push and merge (no approve gate)</option>
      </select>
      <span class="help">
        default matches the drain contract: commit, push, open PR, stop. do not merge until
        approve. push-and-merge skips the gate after a successful run.
      </span>
    </label>

    <label>
      approve click
      <select bind:value={form.approveAction}>
        <option value="merge">merge with gh pr merge</option>
        <option value="open_browser">open PR in browser, then mark done</option>
      </select>
    </label>

    <label>
      claude permission mode
      <select bind:value={form.permissionMode}>
        <option value="bypassPermissions">bypassPermissions (unattended queue)</option>
        <option value="acceptEdits">acceptEdits</option>
        <option value="default">default</option>
      </select>
      <span class="help">
        drain invokes <code>claude -p --permission-mode …</code> from rust. bypass is the overnight
        default so the queue does not hang on interactive prompts.
      </span>
    </label>

    <div class="toolbar">
      <button class="primary" type="submit">save</button>
      {#if saved}<span class="help">saved to local sqlite.</span>{/if}
    </div>
  </form>
</div>

<style>
  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 13px;
    font-weight: 500;
    letter-spacing: -0.015em;
    color: var(--ink);
  }
  input,
  select {
    height: 34px;
    border-radius: 10px;
    border: 0;
    background: var(--craft-surface);
    padding: 0 10px;
    font-size: 13px;
    font-weight: 400;
    letter-spacing: -0.15px;
    color: var(--ink);
  }
  .row {
    display: flex;
    gap: 8px;
  }
  .row input {
    flex: 1;
  }
  .help {
    font-size: 12px;
    font-weight: 400;
    letter-spacing: 0;
    text-transform: none;
    color: var(--muted);
  }
  .check {
    flex-direction: row;
    align-items: center;
    gap: 8px;
    text-transform: none;
    letter-spacing: 0;
    font-size: 13px;
    font-weight: 500;
    color: var(--ink);
  }
  .check input {
    width: 14px;
    height: 14px;
  }
  code {
    font-family: var(--mono);
    font-size: 11px;
  }
</style>
