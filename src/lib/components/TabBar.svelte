<script lang="ts">
  import type { PanelStore, PanelState, TabSnapshot } from '$lib/stores/panelStore';
  import { createEventDispatcher } from 'svelte';

  export let store: PanelStore;

  const dispatch = createEventDispatcher<{ newtab: void }>();

  $: state = $store as PanelState;

  function tabLabel(tab: TabSnapshot, idx: number): string {
    if (tab.archiveRoot) {
      const arcName = tab.archiveRoot.split('/').pop() ?? 'archive';
      return arcName + (tab.archiveInner ? '/' + tab.archiveInner.split('/').pop() : '');
    }
    const parts = tab.path.split('/').filter(Boolean);
    return parts.at(-1) ?? '/';
  }

  function handleClose(e: MouseEvent, idx: number) {
    e.stopPropagation();
    store.closeTab(idx);
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="tab-bar">
  {#each state.tabs as tab, i (tab.id)}
    <div
      class="tab"
      class:active={i === state.activeTabIdx}
      on:click={() => store.switchTab(i)}
      title={tab.path}
    >
      <span class="tab-label">{tabLabel(tab, i)}</span>
      {#if state.tabs.length > 1}
        <button class="close-btn" on:click={e => handleClose(e, i)}>×</button>
      {/if}
    </div>
  {/each}
  <button class="new-tab-btn" on:click={() => dispatch('newtab')} title="New tab (Ctrl+T)">+</button>
</div>

<style>
  .tab-bar {
    display: flex;
    align-items: stretch;
    background: var(--bg);
    border-bottom: 1px solid var(--border-dim);
    height: 22px;
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
    flex-shrink: 0;
  }
  .tab-bar::-webkit-scrollbar { display: none; }

  .tab {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 8px 0 10px;
    font-family: 'Courier New', monospace;
    font-size: 10px;
    color: var(--text-dim);
    background: var(--bg);
    border-right: 1px solid var(--border-dim);
    cursor: pointer;
    white-space: nowrap;
    flex-shrink: 0;
    user-select: none;
    max-width: 140px;
  }
  .tab:hover { background: var(--bg-panel); color: var(--text); }
  .tab.active {
    background: var(--bg-panel);
    color: var(--accent);
    border-top: 1px solid var(--accent);
  }

  .tab-label {
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-dim);
    cursor: pointer;
    font-size: 12px;
    padding: 0 2px;
    line-height: 1;
    flex-shrink: 0;
  }
  .close-btn:hover { color: var(--danger); }

  .new-tab-btn {
    background: none;
    border: none;
    color: var(--text-dim);
    cursor: pointer;
    font-size: 14px;
    padding: 0 8px;
    flex-shrink: 0;
    align-self: center;
  }
  .new-tab-btn:hover { color: var(--accent); }
</style>
