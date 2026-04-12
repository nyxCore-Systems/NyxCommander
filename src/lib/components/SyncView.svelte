<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let left: string;
  export let right: string;

  const dispatch = createEventDispatcher<{ close: void; navigate: { path: string } }>();

  interface SyncEntry {
    name: string;
    left_path: string | null;
    right_path: string | null;
    left_size: number | null;
    right_size: number | null;
    status: 'left-only' | 'right-only' | 'different' | 'same';
  }

  let entries: SyncEntry[] = [];
  let loading = true;
  let error = '';
  let cursor = 0;
  let showSame = false;
  let overlayEl: HTMLElement;

  onMount(async () => {
    overlayEl?.focus();
    try {
      entries = await invoke<SyncEntry[]>('sync_dirs_list', { left, right });
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  $: filtered = showSame ? entries : entries.filter(e => e.status !== 'same');

  function handleKey(e: KeyboardEvent) {
    e.stopPropagation();
    if (e.key === 'Escape') { dispatch('close'); return; }
    if (e.key === 'ArrowUp') { e.preventDefault(); cursor = Math.max(0, cursor - 1); }
    if (e.key === 'ArrowDown') { e.preventDefault(); cursor = Math.min(filtered.length - 1, cursor + 1); }
  }

  async function copyToRight(entry: SyncEntry) {
    if (!entry.left_path) return;
    try {
      await invoke('copy_items', { srcs: [entry.left_path], dstDir: right });
      refresh();
    } catch (e) { error = String(e); }
  }

  async function copyToLeft(entry: SyncEntry) {
    if (!entry.right_path) return;
    try {
      await invoke('copy_items', { srcs: [entry.right_path], dstDir: left });
      refresh();
    } catch (e) { error = String(e); }
  }

  async function refresh() {
    loading = true;
    error = '';
    try {
      entries = await invoke<SyncEntry[]>('sync_dirs_list', { left, right });
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function fmtSize(n: number | null): string {
    if (n === null) return '—';
    if (n < 1024) return `${n}B`;
    if (n < 1048576) return `${(n / 1024).toFixed(0)}K`;
    return `${(n / 1048576).toFixed(1)}M`;
  }

  function statusColor(s: string): string {
    if (s === 'left-only') return 'var(--accent)';
    if (s === 'right-only') return 'var(--text-selected)';
    if (s === 'different') return '#ffcc00';
    return 'var(--text-dim)';
  }
</script>

<!-- svelte-ignore a11y-no-noninteractive-element-interactions a11y-no-noninteractive-tabindex -->
<div class="overlay" role="dialog" tabindex="-1" bind:this={overlayEl} on:keydown={handleKey}>
  <div class="dialog">
    <div class="titlebar">
      <span class="title">Directory Sync</span>
      <button class="close-btn" on:click={() => dispatch('close')}>✕</button>
    </div>

    <div class="dir-labels">
      <span class="dir-label left">{left}</span>
      <span class="vs">↔</span>
      <span class="dir-label right">{right}</span>
    </div>

    <div class="toolbar">
      <label>
        <input type="checkbox" bind:checked={showSame} />
        Show identical
      </label>
      <button on:click={refresh} disabled={loading}>↻ Refresh</button>
    </div>

    {#if loading}
      <div class="msg">Comparing…</div>
    {:else if error}
      <div class="msg err">{error}</div>
    {:else if filtered.length === 0}
      <div class="msg">All files are identical.</div>
    {:else}
      <div class="list">
        <div class="list-header">
          <span class="col-name">Name</span>
          <span class="col-size">Left</span>
          <span class="col-status">Status</span>
          <span class="col-size">Right</span>
          <span class="col-actions">Actions</span>
        </div>
        {#each filtered as entry, i}
          <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
          <div
            class="list-row"
            class:cursor-row={i === cursor}
            on:click={() => cursor = i}
          >
            <span class="col-name" style="color: {statusColor(entry.status)}">{entry.name}</span>
            <span class="col-size">{fmtSize(entry.left_size)}</span>
            <span class="col-status" style="color: {statusColor(entry.status)}">
              {entry.status.replace('-', ' ')}
            </span>
            <span class="col-size">{fmtSize(entry.right_size)}</span>
            <span class="col-actions">
              {#if entry.left_path}
                <button class="act" on:click={() => copyToRight(entry)} title="Copy to right →">→</button>
              {/if}
              {#if entry.right_path}
                <button class="act" on:click={() => copyToLeft(entry)} title="← Copy to left">←</button>
              {/if}
            </span>
          </div>
        {/each}
      </div>
    {/if}

    <div class="footer">
      {filtered.length} item{filtered.length !== 1 ? 's' : ''}
      · {entries.filter(e => e.status !== 'same').length} diff{entries.filter(e => e.status !== 'same').length !== 1 ? 's' : ''}
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.75);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    outline: none;
  }
  .dialog {
    background: var(--bg-panel);
    border: 1px solid var(--border-panel);
    width: min(860px, 96vw);
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    font-family: 'Courier New', monospace;
    font-size: 12px;
  }
  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    background: var(--bg-header);
    border-bottom: 1px solid var(--border-dim);
  }
  .title { color: var(--accent); font-size: 11px; text-transform: uppercase; letter-spacing: 0.06em; }
  .close-btn {
    background: none; border: none; color: var(--text-dim); cursor: pointer; font-size: 14px; padding: 0;
  }
  .close-btn:hover { color: var(--danger); }

  .dir-labels {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    background: var(--bg-header);
    border-bottom: 1px solid var(--border-dim);
    font-size: 10px;
  }
  .dir-label { color: var(--text-dim); flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .dir-label.right { text-align: right; }
  .vs { color: var(--border-dim); flex-shrink: 0; }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 5px 10px;
    border-bottom: 1px solid var(--border-dim);
    font-size: 11px;
    color: var(--text-dim);
  }
  .toolbar label { display: flex; align-items: center; gap: 5px; cursor: pointer; }
  .toolbar button {
    background: none; border: 1px solid var(--border-dim); color: var(--text-dim);
    font-family: inherit; font-size: 10px; padding: 2px 8px; cursor: pointer;
  }
  .toolbar button:hover { border-color: var(--accent); color: var(--accent); }
  .toolbar button:disabled { opacity: 0.4; }

  .msg {
    padding: 20px;
    text-align: center;
    color: var(--text-dim);
    flex: 1;
  }
  .msg.err { color: var(--danger); }

  .list {
    flex: 1;
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--bg);
  }
  .list-header, .list-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 2px 10px;
    font-size: 11px;
  }
  .list-header {
    color: var(--text-dim);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    border-bottom: 1px solid var(--border-dim);
    position: sticky;
    top: 0;
    background: var(--bg-header);
  }
  .list-row { cursor: pointer; }
  .list-row:hover { background: var(--bg-header); }
  .list-row.cursor-row { outline: 1px solid var(--accent); }

  .col-name { flex: 2; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .col-size { width: 56px; text-align: right; flex-shrink: 0; color: var(--text-dim); }
  .col-status { width: 90px; flex-shrink: 0; font-size: 10px; }
  .col-actions { width: 60px; flex-shrink: 0; display: flex; gap: 4px; justify-content: flex-end; }
  .act {
    background: none; border: 1px solid var(--border-dim); color: var(--text-dim);
    font-size: 11px; padding: 1px 5px; cursor: pointer;
  }
  .act:hover { border-color: var(--accent); color: var(--accent); }

  .footer {
    padding: 5px 10px;
    font-size: 10px;
    color: var(--text-dim);
    border-top: 1px solid var(--border-dim);
    background: var(--bg-header);
  }
</style>
