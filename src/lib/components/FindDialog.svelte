<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { FileEntry } from '$lib/stores/panelStore';

  export let startPath: string;

  const dispatch = createEventDispatcher<{
    navigate: { path: string };
    close: void;
  }>();

  let query = '';
  let recursive = false;
  let results: FileEntry[] = [];
  let searching = false;
  let error = '';
  let selectedIdx = -1;
  let inputEl: HTMLInputElement;

  onMount(() => inputEl?.focus());

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Escape') { dispatch('close'); return; }
    if (e.key === 'ArrowDown') { e.preventDefault(); selectedIdx = Math.min(selectedIdx + 1, results.length - 1); }
    if (e.key === 'ArrowUp')   { e.preventDefault(); selectedIdx = Math.max(selectedIdx - 1, 0); }
    if (e.key === 'Enter' && selectedIdx >= 0) openSelected();
  }

  async function doSearch() {
    if (!query.trim()) return;
    searching = true;
    error = '';
    results = [];
    selectedIdx = -1;
    try {
      results = await invoke<FileEntry[]>('search_files', {
        root: startPath,
        pattern: query.trim(),
        recursive,
        maxResults: 300,
      });
    } catch (e) {
      error = String(e);
    } finally {
      searching = false;
    }
  }

  function openSelected() {
    const r = results[selectedIdx];
    if (!r) return;
    const dir = r.is_dir ? r.path : r.path.substring(0, r.path.lastIndexOf('/')) || '/';
    dispatch('navigate', { path: dir });
  }

  function handleResultClick(idx: number) { selectedIdx = idx; }
  function handleResultDblClick(idx: number) { selectedIdx = idx; openSelected(); }

  function dirOf(entry: FileEntry): string {
    if (entry.is_dir) return entry.path;
    return entry.path.substring(0, entry.path.lastIndexOf('/')) || '/';
  }

  function relPath(entry: FileEntry): string {
    const rel = entry.path.startsWith(startPath)
      ? entry.path.slice(startPath.length).replace(/^\//, '')
      : entry.path;
    return rel || entry.name;
  }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="overlay" on:keydown={handleKey}>
  <div class="dialog" role="dialog" tabindex="-1" aria-label="Find">

    <div class="title-bar">
      <span>FIND</span>
      <span class="title-sub">in {startPath}</span>
      <button class="close-btn" on:click={() => dispatch('close')}>✕</button>
    </div>

    <div class="search-row">
      <input
        bind:this={inputEl}
        bind:value={query}
        placeholder="filename pattern…"
        on:keydown={e => e.key === 'Enter' && doSearch()}
      />
      <label class="toggle">
        <input type="checkbox" bind:checked={recursive} />
        Recursive
      </label>
      <button class="search-btn" on:click={doSearch} disabled={searching || !query.trim()}>
        {searching ? 'Searching…' : 'Search'}
      </button>
    </div>

    {#if error}
      <div class="error-bar">{error}</div>
    {/if}

    <div class="col-headers">
      <span class="col-icon-h"></span>
      <span class="col-name-h">Name</span>
      <span class="col-path-h">Path</span>
    </div>

    <div class="results">
      {#if results.length === 0 && !searching}
        <div class="empty">{query ? 'No results.' : 'Enter a pattern and press Search.'}</div>
      {:else}
        {#each results as r, i (r.path)}
          <!-- svelte-ignore a11y-click-events-have-key-events -->
          <div
            class="result-row"
            class:selected={i === selectedIdx}
            class:is-dir={r.is_dir}
            on:click={() => handleResultClick(i)}
            on:dblclick={() => handleResultDblClick(i)}
          >
            <span class="col-icon-h">{r.is_dir ? '▸' : ' '}</span>
            <span class="col-name-h">{r.name}</span>
            <span class="col-path-h">{relPath(r)}</span>
          </div>
        {/each}
      {/if}
    </div>

    <div class="footer">
      {#if results.length > 0}
        <span>{results.length} result{results.length !== 1 ? 's' : ''}</span>
      {/if}
      <span class="hint">↑↓ navigate · Enter / Dbl-click go to dir · Esc close</span>
      {#if selectedIdx >= 0}
        <button class="go-btn" on:click={openSelected}>
          Go → {dirOf(results[selectedIdx])}
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.82);
    display: flex; align-items: center; justify-content: center;
    z-index: 100;
  }

  .dialog {
    display: flex; flex-direction: column;
    width: 740px; max-width: 94vw; height: 60vh;
    background: var(--bg-panel);
    border: 1px solid var(--border-panel);
    font-family: 'Courier New', monospace;
    font-size: 11px;
  }

  .title-bar {
    display: flex; align-items: center; gap: 8px;
    padding: 0 8px; height: 22px;
    background: var(--bg-header);
    border-bottom: 1px solid var(--border-panel);
    color: var(--border-panel);
    font-weight: bold; font-size: 11px; letter-spacing: .1em;
    flex-shrink: 0;
  }
  .title-sub { color: var(--text-dim); font-weight: normal; font-size: 10px; flex: 1; }
  .close-btn {
    background: none; border: none; color: var(--text-dim);
    cursor: pointer; font-size: 12px; margin-left: auto;
  }
  .close-btn:hover { color: var(--text-danger); }

  .search-row {
    display: flex; gap: 8px; align-items: center;
    padding: 8px;
    border-bottom: 1px solid var(--border-dim);
    flex-shrink: 0;
  }

  .search-row input {
    flex: 1;
    background: var(--bg); color: var(--text);
    border: 1px solid var(--text-dim);
    padding: 4px 8px;
    font-family: 'Courier New', monospace; font-size: 12px;
  }
  .search-row input:focus { border-color: var(--border); outline: none; }

  .toggle {
    display: flex; align-items: center; gap: 4px;
    color: var(--text-dim); font-size: 10px; cursor: pointer;
    white-space: nowrap;
  }
  .toggle input[type="checkbox"] { cursor: pointer; }

  .search-btn {
    background: var(--bg-fnkey); color: var(--text);
    border: 1px solid var(--border-dim);
    padding: 4px 14px;
    font-family: 'Courier New', monospace; font-size: 11px;
    cursor: pointer; white-space: nowrap;
  }
  .search-btn:hover:not(:disabled) { border-color: var(--border); color: var(--border); }
  .search-btn:disabled { opacity: .4; cursor: default; }

  .error-bar {
    padding: 4px 8px; color: var(--text-danger); font-size: 10px;
    background: rgba(255,50,80,.08);
    flex-shrink: 0;
  }

  .col-headers {
    display: flex; padding: 0 8px; height: 18px; align-items: center;
    background: var(--bg-header); border-bottom: 1px solid var(--border-dim);
    color: var(--text-dim); font-size: 10px; text-transform: uppercase;
    letter-spacing: .06em; flex-shrink: 0;
  }

  .results {
    flex: 1; overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--bg);
  }

  .result-row {
    display: flex; align-items: center; padding: 0 8px; height: 20px;
    cursor: pointer; color: var(--text);
    border-bottom: 1px solid transparent;
  }
  .result-row:hover { background: var(--bg-header); }
  .result-row.selected { background: var(--bg-row-sel); color: var(--text-sel); }
  .result-row.is-dir .col-name-h { color: var(--text-dir); }
  .result-row.selected.is-dir .col-name-h { color: var(--text-sel); }

  .col-icon-h  { width: 18px; flex-shrink: 0; color: var(--text-dim); }
  .col-name-h  { width: 200px; flex-shrink: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .col-path-h  { flex: 1; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .result-row.selected .col-path-h { color: var(--text-sel); }

  .empty {
    padding: 16px 8px; color: var(--text-dim);
  }

  .footer {
    display: flex; align-items: center; gap: 10px;
    padding: 0 8px; height: 22px;
    background: var(--bg-header); border-top: 1px solid var(--border-dim);
    color: var(--text-dim); font-size: 10px;
    flex-shrink: 0;
  }
  .hint { flex: 1; }

  .go-btn {
    background: transparent; border: 1px solid var(--border-panel);
    color: var(--border-panel);
    font-family: 'Courier New', monospace; font-size: 10px;
    padding: 1px 10px; cursor: pointer;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
    max-width: 300px;
  }
  .go-btn:hover { background: var(--border-panel); color: var(--bg); }
</style>
