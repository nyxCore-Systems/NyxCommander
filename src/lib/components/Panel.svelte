<script lang="ts">
  import type { AtlasData } from '$lib/atlas';
  import PathBar from './PathBar.svelte';
  import FileRow from './FileRow.svelte';
  import TabBar from './TabBar.svelte';
  import type { PanelStore, PanelState, FileEntry } from '$lib/stores/panelStore';
  import { colorRulesStore, applyColorRule } from '$lib/stores/colorRulesStore';
  import { pluginStore, type PluginInfo, type ColumnCache } from '$lib/stores/pluginStore';
  import { uiStore } from '$lib/stores/uiStore';
  import { createEventDispatcher, afterUpdate, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  const ARCHIVE_EXTS = new Set([
    'zip','jar','war','ear','apk','ipa',
    'tar','tgz','tbz2','txz',
    'docx','xlsx','pptx','odt','ods',
  ]);

  function isArchiveFile(entry: FileEntry): boolean {
    if (entry.is_dir) return false;
    const ext = entry.extension.toLowerCase();
    if (ARCHIVE_EXTS.has(ext)) return true;
    return entry.name.endsWith('.tar.gz') || entry.name.endsWith('.tar.bz2') || entry.name.endsWith('.tar.xz');
  }

  export let store: PanelStore;
  export let atlas: AtlasData;
  export let active: boolean = false;
  export let diffPick: string | null = null;

  const dispatch = createEventDispatcher<{ activate: void }>();

  $: state = $store as PanelState;
  $: colorRules = $colorRulesStore;
  $: columnPlugins = $pluginStore.plugins.filter(
    (p: PluginInfo) => p.category === 'column',
  );
  $: columnCache = $pluginStore.columnCache as ColumnCache;

  // Track path changes to invalidate cache and trigger column fetches
  let prevPath = '';
  $: {
    if (active && state.path && state.path !== prevPath) {
      prevPath = state.path;
      pluginStore.invalidateColumnCache();
    }
  }
  $: {
    if (
      active &&
      !state.loading &&
      state.entries.length > 0 &&
      columnPlugins.length > 0
    ) {
      for (const plugin of columnPlugins) {
        pluginStore.fetchColumnValues(plugin.id, state.entries).catch(console.error);
      }
    }
  }

  // Quick filter: computed entries
  $: displayedEntries = state.filter
    ? state.entries.filter(e => e.name === '..' || e.name.toLowerCase().includes(state.filter.toLowerCase()))
    : state.entries;

  let listEl: HTMLElement;
  let filterInputEl: HTMLInputElement;
  let filterMode = false;

  afterUpdate(() => {
    if (!listEl) return;
    const rows = listEl.querySelectorAll('.file-row');
    const row = rows[state.cursor] as HTMLElement | undefined;
    row?.scrollIntoView({ block: 'nearest', behavior: 'auto' });
  });

  function handleDblClick(entry: FileEntry) {
    if (entry.is_dir) {
      if (entry.name === '..') {
        handleDirUp();
      } else if (state.archiveRoot !== null) {
        store.navigateArchive(state.archiveRoot!, entry.path);
      } else {
        store.navigate(entry.path);
      }
    } else if (state.archiveRoot !== null) {
      // Files inside archives have internal paths — can't open directly.
      // Cmd+Enter (handled by App.svelte) extracts; double-click does nothing.
    } else if (isArchiveFile(entry)) {
      store.navigateArchive(entry.path, '');
    } else {
      invoke('open_file', { path: entry.path }).catch(console.error);
    }
  }

  function handleDirUp() {
    if (state.archiveRoot !== null) {
      if (state.archiveInner === '') {
        const parts = (state.archiveRoot ?? '').split('/').filter(Boolean);
        const parent = parts.length > 0 ? '/' + parts.slice(0, -1).join('/') || '/' : '/';
        const archiveName = parts[parts.length - 1] ?? '';
        store.navigate(parent, archiveName);
      } else {
        const idx = state.archiveInner.lastIndexOf('/');
        const parent = idx > 0 ? state.archiveInner.slice(0, idx) : '';
        store.navigateArchive(state.archiveRoot!, parent);
      }
    } else {
      const parts = state.path.split('/').filter(Boolean);
      const parent = parts.length > 0 ? '/' + parts.slice(0, -1).join('/') || '/' : '/';
      const childName = parts[parts.length - 1] ?? '';
      store.navigate(parent, childName);
    }
  }

  function handleClick(entry: FileEntry, index: number) {
    store.setCursor(index);
  }

  // Panel-level keydown for quick filter
  function handlePanelKey(e: KeyboardEvent) {
    if (!active) return;
    // Don't intercept if in filter mode (input handles it)
    if (filterMode) return;
    // Typing a printable char starts filter
    if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) {
      e.stopPropagation();
      filterMode = true;
      store.setFilter(e.key);
      tick().then(() => filterInputEl?.focus());
    }
  }

  function exitFilterMode() {
    filterMode = false;
    store.setFilter('');
  }

  function handleFilterKey(e: KeyboardEvent) {
    e.stopPropagation();
    if (e.key === 'Escape') { exitFilterMode(); return; }
    if (e.key === 'Enter') {
      // navigate to first result
      if (displayedEntries.length > 0) {
        const first = displayedEntries[0];
        if (first.name !== '..') store.setCursor(state.entries.indexOf(first));
      }
      exitFilterMode();
    }
  }

  function formatStatus(s: PanelState): string {
    const files = s.entries.filter(e => !e.is_dir && e.name !== '..').length;
    const dirs  = s.entries.filter(e => e.is_dir && e.name !== '..').length;
    const sel   = s.selected.size;
    let str = `${files} file${files !== 1 ? 's' : ''}, ${dirs} dir${dirs !== 1 ? 's' : ''}`;
    if (sel > 0) str += `, ${sel} selected`;
    if (s.flatView) str += ' [flat]';
    if (s.archiveRoot) str += ' [archive]';
    return str;
  }

  // Build archive breadcrumb label
  function archiveCrumb(s: PanelState): string {
    const arcName = (s.archiveRoot ?? '').split('/').pop() ?? '';
    if (!s.archiveInner) return arcName;
    return `${arcName}/${s.archiveInner}`;
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="panel" class:active on:click={() => dispatch('activate')} on:keydown={handlePanelKey}>

  <TabBar {store} on:newtab={() => store.newTab()} />

  {#if state.archiveRoot}
    <div class="archive-bar">
      <span class="arc-icon">⟦arc⟧</span>
      <span class="arc-crumb">{archiveCrumb(state)}</span>
      <button class="arc-exit" on:click|stopPropagation={handleDirUp}>✕ exit archive</button>
    </div>
  {:else}
    <PathBar
      path={state.path}
      {active}
      on:navigate={e => store.navigate(e.detail)}
      on:editpath={() => uiStore.setDialog({ kind: 'goto', startPath: state.path })}
    />
  {/if}

  <div class="col-headers">
    <span class="col-icon"></span>
    <span class="col-name">Name</span>
    {#each columnPlugins as col}
      <span class="col-plugin" style="width: {col.columnWidth ?? 72}px">
        {col.columnName ?? col.name}
      </span>
    {/each}
    <span class="col-size">Size</span>
    <span class="col-date">Date</span>
    <div class="header-controls" on:click|stopPropagation>
      <button
        class="hdr-btn"
        class:active-btn={state.flatView}
        title="Flat view (show all files recursively)"
        on:click={() => store.toggleFlatView()}
      >≡</button>
      <button
        class="hdr-btn"
        class:active-btn={state.showHidden}
        title="Show hidden files"
        on:click={() => store.toggleShowHidden()}
      >.</button>
    </div>
  </div>

  <div class="file-list" bind:this={listEl}>
    {#if state.loading}
      <div class="message">Loading…</div>
    {:else if state.error}
      <div class="message error">{state.error}</div>
    {:else if displayedEntries.length === 0 && state.filter}
      <div class="message">No match for "{state.filter}"</div>
    {:else if state.entries.length === 0}
      <div class="message">Empty directory</div>
    {:else}
      {#each displayedEntries as entry, i (entry.path + '|' + i)}
        <FileRow
          {entry}
          {atlas}
          colorStyle={applyColorRule(colorRules, entry)}
          isCursor={active && state.entries.indexOf(entry) === state.cursor}
          isSelected={state.selected.has(entry.path)}
          isDiffPick={entry.path === diffPick}
          {columnPlugins}
          {columnCache}
          on:click={() => handleClick(entry, state.entries.indexOf(entry))}
          on:dblclick={() => handleDblClick(entry)}
        />
      {/each}
    {/if}
  </div>

  {#if filterMode}
    <div class="filter-bar" on:click|stopPropagation>
      <span class="filter-label">Filter:</span>
      <input
        bind:this={filterInputEl}
        value={state.filter}
        on:input={e => store.setFilter((e.target as HTMLInputElement).value)}
        on:keydown={handleFilterKey}
        class="filter-input"
        spellcheck="false"
        placeholder="type to filter…"
      />
      <span class="filter-count">{displayedEntries.length}/{state.entries.length}</span>
      <button class="filter-clear" on:click={exitFilterMode}>✕</button>
    </div>
  {/if}

  <div class="status-bar">
    <button class="hist-btn" disabled={state.historyIdx <= 0} on:click|stopPropagation={() => store.back()} title="Back (Alt+←)">‹</button>
    <button class="hist-btn" disabled={state.historyIdx >= state.history.length - 1} on:click|stopPropagation={() => store.forward()} title="Forward (Alt+→)">›</button>
    <span class="status-text">{formatStatus(state)}</span>
  </div>
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    flex: 1;
    background: var(--bg-panel);
    border: 1px solid var(--border-dim);
    overflow: hidden;
    min-width: 0;
    transition: border-color 0.1s;
  }
  .panel.active { border-color: var(--border-panel); }

  .archive-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 2px 8px;
    background: rgba(255,204,0,0.08);
    border-bottom: 1px solid #ffcc0040;
    font-family: 'Courier New', monospace;
    font-size: 10px;
    color: #ffcc00;
    flex-shrink: 0;
    height: 20px;
  }
  .arc-icon { opacity: 0.6; flex-shrink: 0; }
  .arc-crumb { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .arc-exit {
    background: none; border: 1px solid #ffcc0040; color: #ffcc00;
    font-family: inherit; font-size: 9px; padding: 1px 5px; cursor: pointer;
    flex-shrink: 0;
  }
  .arc-exit:hover { background: rgba(255,204,0,0.15); }

  .col-headers {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 6px 0 2px;
    background: var(--bg-header);
    font-family: 'Courier New', monospace;
    font-size: 10px;
    color: var(--text-dim);
    border-bottom: 1px solid var(--border-dim);
    flex-shrink: 0;
    height: 20px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .col-icon { width: 38px; flex-shrink: 0; }
  .col-name { flex: 1; }
  .col-plugin {
    flex-shrink: 0;
    text-align: right;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .col-size { width: 64px; text-align: right; flex-shrink: 0; }
  .col-date { width: 118px; text-align: right; flex-shrink: 0; }
  .panel.active .col-headers { color: var(--text); }

  .header-controls { display: flex; gap: 2px; margin-left: auto; }
  .hdr-btn {
    background: none; border: none; color: var(--text-dim); cursor: pointer;
    font-size: 11px; padding: 0 3px; line-height: 1;
  }
  .hdr-btn:hover { color: var(--accent); }
  .hdr-btn.active-btn { color: var(--accent); }

  .file-list {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--bg);
  }

  .filter-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    background: var(--bg-header);
    border-top: 1px solid var(--border-dim);
    flex-shrink: 0;
    font-family: 'Courier New', monospace;
    font-size: 11px;
  }
  .filter-label { color: var(--accent); flex-shrink: 0; }
  .filter-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text);
    font-family: inherit;
    font-size: 11px;
    caret-color: var(--accent);
  }
  .filter-count { color: var(--text-dim); font-size: 10px; flex-shrink: 0; }
  .filter-clear {
    background: none; border: none; color: var(--text-dim); cursor: pointer; font-size: 11px;
  }
  .filter-clear:hover { color: var(--danger); }

  .status-bar {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 6px;
    background: var(--bg-header);
    font-family: 'Courier New', monospace;
    font-size: 10px;
    color: var(--text-dim);
    border-top: 1px solid var(--border-dim);
    height: 18px;
  }
  .panel.active .status-bar { color: var(--text); }

  .hist-btn {
    background: none;
    border: none;
    color: var(--text-dim);
    cursor: pointer;
    font-size: 14px;
    padding: 0 2px;
    line-height: 1;
    flex-shrink: 0;
  }
  .hist-btn:hover:not(:disabled) { color: var(--accent); }
  .hist-btn:disabled { opacity: 0.3; cursor: default; }
  .status-text { flex: 1; }

  .message {
    padding: 12px 8px;
    color: var(--text-dim);
    font-family: 'Courier New', monospace;
    font-size: 11px;
  }
  .message.error { color: var(--danger); }
</style>
