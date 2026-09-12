<script lang="ts">
  import { page } from "$app/state";
  import { store } from "$lib/stores.svelte";
  import { isMock } from "$lib/api";

  const items = [
    { href: "/", label: "inbox", key: "inbox" },
    { href: "/queue", label: "queue", key: "queue" },
    { href: "/run", label: "run", key: "run" },
    { href: "/settings", label: "settings", key: "settings" },
  ];

  function active(href: string) {
    if (href === "/") return page.url.pathname === "/";
    return page.url.pathname.startsWith(href);
  }
</script>

<aside class="side">
  <div class="brand">
    <span class="mark" aria-hidden="true"></span>
    <div>
      <div class="name">drain</div>
      <div class="tag">one at a time</div>
    </div>
  </div>

  <nav>
    {#each items as item}
      <a href={item.href} class:on={active(item.href)}>
        <span>{item.label}</span>
        {#if item.key === "queue" && store.readyCount > 0}
          <em>{store.readyCount}</em>
        {/if}
        {#if item.key === "run" && store.running}
          <i class="pulse"></i>
        {/if}
      </a>
    {/each}
  </nav>

  <div class="foot">
    {#if store.running}
      <div class="now">running · {store.running.title}</div>
    {:else if store.awaiting}
      <div class="now wait">waiting · approve</div>
    {:else if store.paused}
      <div class="now idle">paused</div>
    {:else}
      <div class="now idle">idle</div>
    {/if}
    {#if isMock()}
      <div class="mock">browser preview — spawn is mocked</div>
    {/if}
  </div>
</aside>

<style>
  .side {
    display: flex;
    flex-direction: column;
    width: 212px;
    flex-shrink: 0;
    padding: 4px 12px 16px;
    background: var(--craft-surface);
    -webkit-app-region: no-drag;
    app-region: no-drag;
  }
  .brand {
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 0 8px 22px;
  }
  .mark {
    width: 18px;
    height: 18px;
    border-radius: 6px;
    background: var(--craft-heat-1);
    box-shadow: var(--shadow-border);
  }
  .name {
    font-family: var(--font-sf-rounded);
    font-size: 1.05rem;
    font-weight: 700;
    letter-spacing: -0.03em;
    text-transform: lowercase;
  }
  .tag {
    font-size: 11px;
    color: var(--muted);
    letter-spacing: -0.01em;
    margin-top: 1px;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
  }
  a {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 32px;
    padding: 0 10px;
    border-radius: 8px;
    color: var(--ink);
    font-size: 13px;
    font-weight: 500;
    letter-spacing: -0.015em;
    transition: background-color 140ms ease;
  }
  @media (hover: hover) {
    a:hover {
      background: color-mix(in srgb, var(--ink) 4%, transparent);
    }
  }
  a.on {
    background: var(--bg);
    box-shadow: var(--shadow-border);
  }
  em {
    font-style: normal;
    font-family: var(--font-sf-mono);
    font-size: 10px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    min-width: 16px;
    height: 16px;
    padding: 0 5px;
    border-radius: 999px;
    background: var(--ink);
    color: #fff;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
  .pulse {
    width: 7px;
    height: 7px;
    border-radius: 99px;
    background: var(--craft-heat-3);
  }
  .foot {
    padding: 10px 8px 0;
    border-top: 1px solid var(--line);
  }
  .now {
    font-size: 11px;
    color: var(--ink);
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .now.idle,
  .now.wait {
    color: var(--muted);
  }
  .mock {
    margin-top: 8px;
    font-size: 10px;
    color: var(--muted);
    line-height: 1.35;
  }
</style>
