<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { themeStore } from '$lib/stores/themeStore';
  import { transferQueue } from '$lib/stores/transferQueueStore';

  export let activeDialog: boolean = false;
  export let cmdBarVisible: boolean = false;
  export let quickViewVisible: boolean = false;

  const dispatch = createEventDispatcher<{ action: string }>();

  $: currentTheme = $themeStore;
  $: pendingTransfers = $transferQueue.filter(j => j.status === 'pending' || j.status === 'running').length;

  const keys: Array<{ f: number; label: string; action: string; danger?: boolean; always?: boolean }> = [
    { f: 1,  label: 'Help',   action: 'f1', always: true },
    { f: 2,  label: 'Rename', action: 'f2' },
    { f: 3,  label: 'Procs',  action: 'f3', always: true },
    { f: 4,  label: 'Open',   action: 'f4' },
    { f: 5,  label: 'Copy',   action: 'f5' },
    { f: 6,  label: 'Move',   action: 'f6' },
    { f: 7,  label: 'Mkdir',  action: 'f7' },
    { f: 8,  label: 'Delete', action: 'f8', danger: true },
    { f: 9,  label: 'Theme',  action: 'f9', always: true },
    { f: 10, label: 'Quit',   action: 'f10' },
  ];
</script>

<div class="fn-bar">
  {#each keys as key}
    <button
      class="fn-key"
      class:danger={key.danger}
      class:theme-key={key.action === 'f9'}
      disabled={activeDialog && !key.always}
      on:click={() => dispatch('action', key.action)}
      title={key.action === 'f9' ? `Theme: ${currentTheme.label}` : undefined}
    >
      <span class="fn-num">{key.f}</span><span class="fn-label">{key.label}</span>
    </button>
  {/each}
  <button
    class="fn-key extra"
    class:active-btn={cmdBarVisible}
    title="Command bar (Ctrl+`)"
    on:click={() => dispatch('action', 'cmd')}
  >
    <span class="fn-label">$_</span>
  </button>
  <button
    class="fn-key extra"
    class:active-btn={quickViewVisible}
    title="Quick view (Ctrl+Q)"
    on:click={() => dispatch('action', 'quickview')}
  >
    <span class="fn-label">QV</span>
  </button>
  <button
    class="fn-key extra"
    title="Open terminal here (Ctrl+Shift+T)"
    on:click={() => dispatch('action', 'terminal')}
  >
    <span class="fn-label">&gt;_</span>
  </button>
  {#if pendingTransfers > 0}
    <button
      class="fn-key extra transfers"
      title="Transfer queue"
      on:click={() => dispatch('action', 'transfers')}
    >
      <span class="fn-label">▶{pendingTransfers}</span>
    </button>
  {/if}
</div>

<style>
  .fn-bar {
    display: flex;
    background: var(--bg-fnbar);
    border-top: 1px solid var(--border-dim);
    flex-shrink: 0;
    height: 22px;
  }
  .fn-key {
    flex: 1;
    background: var(--bg-fnkey);
    border: none;
    border-right: 1px solid var(--border-dim);
    padding: 0 2px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 1px;
    color: var(--text-fn-label);
    font-family: 'Courier New', monospace;
    font-size: 10px;
    transition: filter 0.05s;
  }
  .fn-key.extra { flex: 0; padding: 0 6px; }
  .fn-key:last-child { border-right: none; }
  .fn-key:hover  { filter: brightness(1.4); }
  .fn-key:active { filter: brightness(0.8); }
  .fn-key:disabled { opacity: 0.35; cursor: default; }
  .fn-key.danger .fn-label { color: var(--text-danger); }
  .fn-key.theme-key { opacity: 1; }
  .fn-key.active-btn { color: var(--accent); }
  .fn-key.transfers { color: var(--accent); animation: pulse 1.5s ease-in-out infinite; }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .fn-num {
    color: var(--text-fn-num);
    font-weight: bold;
    font-size: 9px;
    margin-right: 1px;
  }
  .fn-label {
    color: inherit;
  }
</style>
