<script lang="ts">
  import { createEventDispatcher, onMount, afterUpdate } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let path: string;

  const dispatch = createEventDispatcher<{ close: void }>();

  // ─── Constants ───────────────────────────────────────────────────────────────
  const BPR       = 16;          // bytes per row
  const PAGE      = 65536;       // bytes loaded per fetch (4096 rows)
  const VISIBLE   = 38;          // rows shown at once

  // ─── State ───────────────────────────────────────────────────────────────────
  let fileSize  = 0;
  let pageData: number[] = [];
  let pageBase  = 0;   // file offset of pageData[0]
  let viewRow   = 0;   // first visible row (in absolute row coords from file start)
  let cursor    = 0;   // absolute byte offset of cursor
  let loading   = true;
  let error     = '';
  let overlayEl: HTMLElement;

  const filename = path.split('/').pop() ?? path;

  // ─── Derived ─────────────────────────────────────────────────────────────────
  $: totalRows  = Math.ceil(fileSize / BPR);
  $: cursorRow  = Math.floor(cursor / BPR);
  $: cursorCol  = cursor % BPR;

  // Rows to render — slice of absolute row range
  $: firstRow = viewRow;
  $: lastRow  = Math.min(viewRow + VISIBLE - 1, totalRows - 1);

  // ─── Data loading ────────────────────────────────────────────────────────────
  async function loadPage(offset: number) {
    const clamped = Math.max(0, Math.min(offset, fileSize - 1));
    try {
      const chunk = await invoke<{ data: number[]; file_size: number; offset: number }>(
        'read_file_bytes', { path, offset: clamped, count: PAGE }
      );
      pageData  = chunk.data;
      pageBase  = chunk.offset;
      fileSize  = chunk.file_size;
    } catch (e) {
      error = String(e);
    }
  }

  onMount(async () => {
    try {
      const chunk = await invoke<{ data: number[]; file_size: number; offset: number }>(
        'read_file_bytes', { path, offset: 0, count: PAGE }
      );
      pageData = chunk.data;
      pageBase = 0;
      fileSize = chunk.file_size;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
      overlayEl?.focus();
    }
  });

  // ─── Cursor + view management ─────────────────────────────────────────────────
  async function moveCursor(delta: number) {
    const next = Math.max(0, Math.min(cursor + delta, fileSize - 1));
    cursor = next;

    // Keep view window tracking cursor
    const row = Math.floor(cursor / BPR);
    if (row < viewRow) viewRow = row;
    if (row > viewRow + VISIBLE - 1) viewRow = row - VISIBLE + 1;

    // Load new page if cursor moved outside current buffer
    if (cursor < pageBase || cursor >= pageBase + pageData.length) {
      const newBase = Math.floor(cursor / PAGE) * PAGE;
      await loadPage(newBase);
    }
  }

  async function pageDown() {
    const rows = VISIBLE;
    viewRow = Math.min(totalRows - VISIBLE, viewRow + rows);
    cursor  = Math.min(fileSize - 1, cursor + rows * BPR);
    if (cursor < pageBase || cursor >= pageBase + pageData.length) {
      await loadPage(Math.floor(cursor / PAGE) * PAGE);
    }
  }

  async function pageUp() {
    const rows = VISIBLE;
    viewRow = Math.max(0, viewRow - rows);
    cursor  = Math.max(0, cursor - rows * BPR);
    if (cursor < pageBase || cursor >= pageBase + pageData.length) {
      await loadPage(Math.floor(cursor / PAGE) * PAGE);
    }
  }

  function gotoStart() { cursor = 0; viewRow = 0; }
  async function gotoEnd() {
    cursor  = Math.max(0, fileSize - 1);
    viewRow = Math.max(0, totalRows - VISIBLE);
    if (cursor < pageBase || cursor >= pageBase + pageData.length) {
      await loadPage(Math.floor(cursor / PAGE) * PAGE);
    }
  }

  async function handleKey(e: KeyboardEvent) {
    e.stopPropagation();
    switch (e.key) {
      case 'Escape':   dispatch('close'); return;
      case 'ArrowRight': e.preventDefault(); await moveCursor(1);   return;
      case 'ArrowLeft':  e.preventDefault(); await moveCursor(-1);  return;
      case 'ArrowDown':  e.preventDefault(); await moveCursor(BPR); return;
      case 'ArrowUp':    e.preventDefault(); await moveCursor(-BPR);return;
      case 'PageDown':   e.preventDefault(); await pageDown();      return;
      case 'PageUp':     e.preventDefault(); await pageUp();        return;
      case 'Home':       e.preventDefault(); gotoStart();           return;
      case 'End':        e.preventDefault(); await gotoEnd();       return;
    }
  }

  // ─── Rendering helpers ────────────────────────────────────────────────────────
  function getByte(absOffset: number): number | null {
    const idx = absOffset - pageBase;
    if (idx < 0 || idx >= pageData.length) return null;
    return pageData[idx];
  }

  function hex2(n: number): string { return n.toString(16).toUpperCase().padStart(2, '0'); }
  function hex8(n: number): string { return n.toString(16).toUpperCase().padStart(8, '0'); }

  function byteClass(b: number | null): string {
    if (b === null)           return 'b-missing';
    if (b === 0)              return 'b-null';
    if (b < 0x20 || b === 0x7f) return 'b-ctrl';
    if (b >= 0x80)            return 'b-high';
    return 'b-ascii';
  }

  function toAscii(b: number | null): string {
    if (b === null) return ' ';
    if (b >= 0x20 && b < 0x7f) return String.fromCharCode(b);
    return '·';
  }

  // Build row data for a given absolute row index
  function rowBytes(row: number): Array<{ byte: number | null; offset: number }> {
    return Array.from({ length: BPR }, (_, col) => {
      const offset = row * BPR + col;
      return { byte: offset < fileSize ? getByte(offset) : null, offset };
    });
  }

  // ─── Status info for cursor byte ─────────────────────────────────────────────
  $: cursorByte = cursor < fileSize ? getByte(cursor) : null;
  $: cursorInfo = (() => {
    const b = cursorByte;
    if (b === null) return null;
    const ascii = b >= 0x20 && b < 0x7f ? `'${String.fromCharCode(b)}'` : (b === 0 ? 'NUL' : b === 10 ? 'LF' : b === 13 ? 'CR' : b === 9 ? 'TAB' : '·');
    return { hex: hex2(b), dec: b, oct: b.toString(8).padStart(3, '0'), bin: b.toString(2).padStart(8, '0'), ascii };
  })();

  function fmtSize(n: number): string {
    if (n >= 1024*1024) return (n/1024/1024).toFixed(2) + ' MB';
    if (n >= 1024)      return (n/1024).toFixed(1) + ' KB';
    return n + ' B';
  }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="overlay" bind:this={overlayEl} tabindex="-1" on:keydown={handleKey}>
  <div class="viewer" role="dialog" aria-label="Hex Viewer">

    <!-- Title bar -->
    <div class="title-bar">
      <span class="badge">HEX</span>
      <span class="fname">{filename}</span>
      {#if fileSize > 0}
        <span class="size-info">{fmtSize(fileSize)} · {fileSize.toLocaleString()} bytes</span>
      {/if}
      <span class="hint-bar">↑↓←→ navigate · PgUp/PgDn page · Home/End · Esc close</span>
      <button class="close-btn" on:click={() => dispatch('close')}>✕</button>
    </div>

    {#if loading}
      <div class="loading">Loading…</div>
    {:else if error}
      <div class="loading error">{error}</div>
    {:else}
      <!-- Column header -->
      <div class="col-header">
        <span class="offset-col header-label">Offset</span>
        <span class="hex-cols">
          {#each Array.from({length: BPR}, (_, i) => i) as col}
            <span class="hcol" class:half-gap={col === 8} class:hdr-cursor={col === cursorCol}>{hex2(col)}</span>
          {/each}
        </span>
        <span class="ascii-col header-label">ASCII</span>
      </div>

      <!-- Hex rows -->
      <div class="rows-area">
        {#each Array.from({length: lastRow - firstRow + 1}, (_, i) => firstRow + i) as rowIdx}
          {@const rb = rowBytes(rowIdx)}
          {@const rowOffset = rowIdx * BPR}
          {@const isCursorRow = rowIdx === cursorRow}
          <div class="hex-row" class:cursor-row={isCursorRow}>
            <!-- Offset -->
            <span class="offset-col" class:active-offset={isCursorRow}>{hex8(rowOffset)}</span>

            <!-- Hex bytes -->
            <span class="hex-cols">
              {#each rb as cell, col}
                {@const isSelected = cell.offset === cursor}
                <span
                  class="hbyte {byteClass(cell.byte)}"
                  class:half-gap={col === 8}
                  class:selected={isSelected}
                  on:click={() => { cursor = cell.offset; }}
                  on:keydown={() => {}}
                  role="button"
                  tabindex="-1"
                >
                  {cell.byte !== null ? hex2(cell.byte) : '  '}
                </span>
              {/each}
            </span>

            <!-- ASCII -->
            <span class="ascii-col">
              <span class="ascii-bar">|</span>
              {#each rb as cell}
                {@const isSelected = cell.offset === cursor}
                <span
                  class="achar {byteClass(cell.byte)}"
                  class:selected={isSelected}
                  on:click={() => { cursor = cell.offset; }}
                  on:keydown={() => {}}
                  role="button"
                  tabindex="-1"
                >{toAscii(cell.byte)}</span>
              {/each}
              <span class="ascii-bar">|</span>
            </span>
          </div>
        {/each}
      </div>

      <!-- Status bar -->
      <div class="status-bar">
        {#if cursorInfo}
          <span class="stat-label">Offset</span>
          <span class="stat-val accent">0x{hex8(cursor)}</span>
          <span class="stat-sep">·</span>
          <span class="stat-label">Hex</span>
          <span class="stat-val">0x{cursorInfo.hex}</span>
          <span class="stat-sep">·</span>
          <span class="stat-label">Dec</span>
          <span class="stat-val">{cursorInfo.dec}</span>
          <span class="stat-sep">·</span>
          <span class="stat-label">Oct</span>
          <span class="stat-val">{cursorInfo.oct}</span>
          <span class="stat-sep">·</span>
          <span class="stat-label">Bin</span>
          <span class="stat-val mono">{cursorInfo.bin}</span>
          <span class="stat-sep">·</span>
          <span class="stat-label">ASCII</span>
          <span class="stat-val">{cursorInfo.ascii}</span>
        {:else}
          <span class="stat-val dim">—</span>
        {/if}
        <span class="stat-right">
          Row {cursorRow + 1} / {totalRows}
          {#if fileSize > PAGE}
            · Page {Math.floor(pageBase / PAGE) + 1}/{Math.ceil(fileSize / PAGE)}
          {/if}
        </span>
      </div>
    {/if}

  </div>
</div>

<style>
  .overlay {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.90);
    display: flex; align-items: center; justify-content: center;
    z-index: 100;
  }
  .overlay:focus { outline: none; }

  .viewer {
    display: flex; flex-direction: column;
    width: min(1040px, 96vw); height: 85vh;
    background: var(--bg);
    border: 1px solid var(--border-panel);
    font-family: 'Courier New', monospace;
    font-size: 12px;
  }

  /* Title bar */
  .title-bar {
    display: flex; align-items: center; gap: 8px;
    padding: 0 10px; height: 26px; flex-shrink: 0;
    background: var(--bg-header);
    border-bottom: 1px solid var(--border-panel);
    font-size: 11px; overflow: hidden;
  }
  .badge {
    background: var(--border-panel); color: var(--bg);
    font-size: 9px; font-weight: bold; padding: 1px 5px;
    letter-spacing: .1em; flex-shrink: 0;
  }
  .fname  { color: var(--border-panel); font-weight: bold; white-space: nowrap; flex-shrink: 0; }
  .size-info { color: var(--text-dim); font-size: 10px; white-space: nowrap; flex-shrink: 0; }
  .hint-bar  { color: var(--text-dim); font-size: 9px; flex: 1; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
  .close-btn {
    background: none; border: none; color: var(--text-dim);
    cursor: pointer; font-size: 13px; flex-shrink: 0;
  }
  .close-btn:hover { color: var(--text-danger); }

  .loading {
    flex: 1; display: flex; align-items: center; justify-content: center;
    color: var(--text-dim); font-size: 13px;
  }
  .loading.error { color: var(--text-danger); }

  /* Column header */
  .col-header {
    display: flex; align-items: center; gap: 0;
    padding: 2px 0;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border-dim);
    flex-shrink: 0;
    font-size: 10px; color: var(--text-dim);
    user-select: none;
  }

  /* Hex rows */
  .rows-area {
    flex: 1; overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--bg);
  }

  .hex-row {
    display: flex; align-items: center;
    border-bottom: 1px solid transparent;
  }
  .hex-row:hover { background: var(--bg-row-hover); }
  .hex-row.cursor-row { background: var(--bg-row-cursor); }

  /* Shared column styles */
  .offset-col {
    width: 84px; flex-shrink: 0;
    padding: 1px 10px 1px 8px;
    color: var(--text-dim); font-size: 11px;
    text-align: right;
    border-right: 1px solid var(--border-dim);
    background: var(--bg-panel);
    user-select: none;
  }
  .offset-col.active-offset { color: var(--border-panel); }
  .offset-col.header-label  { color: var(--text-dim); font-size: 10px; text-align: center; }

  .hex-cols {
    display: flex; align-items: center;
    padding: 1px 8px;
    flex-shrink: 0;
  }

  .hcol {
    width: 26px; text-align: center; font-size: 10px;
    color: var(--text-dim); user-select: none;
  }
  .hcol.hdr-cursor { color: var(--border-panel); }

  .hbyte {
    width: 26px; text-align: center;
    padding: 0 1px;
    cursor: pointer;
    border-radius: 1px;
    line-height: 20px;
    background: none; border: 1px solid transparent;
  }
  .hbyte:hover { background: var(--bg-row-hover); }
  .hbyte.selected {
    background: var(--border-panel); color: var(--bg) !important;
    border-color: var(--border-panel);
  }
  .half-gap { margin-left: 8px; }

  .ascii-col {
    display: flex; align-items: center;
    padding: 1px 8px;
    border-left: 1px solid var(--border-dim);
    flex-shrink: 0;
  }
  .ascii-col.header-label {
    color: var(--text-dim); font-size: 10px;
    justify-content: center; width: 160px;
  }
  .ascii-bar { color: var(--text-dim); user-select: none; }

  .achar {
    width: 10px; text-align: center;
    cursor: pointer; line-height: 20px;
    background: none; border: 1px solid transparent;
    border-radius: 1px;
  }
  .achar:hover { background: var(--bg-row-hover); }
  .achar.selected {
    background: var(--border-panel); color: var(--bg) !important;
    border-color: var(--border-panel);
  }

  /* Byte color classes */
  .b-null  { color: var(--text-dim); opacity: .35; }
  .b-ctrl  { color: #7a6ab0; }
  .b-ascii { color: var(--text); }
  .b-high  { color: #c08840; }
  .b-missing { color: transparent; }

  /* Status bar */
  .status-bar {
    display: flex; align-items: center; gap: 6px;
    padding: 0 10px; height: 22px; flex-shrink: 0;
    background: var(--bg-panel);
    border-top: 1px solid var(--border-dim);
    font-size: 10px;
    overflow: hidden;
  }
  .stat-label { color: var(--text-dim); text-transform: uppercase; font-size: 9px; letter-spacing: .06em; }
  .stat-val   { color: var(--text); }
  .stat-val.accent { color: var(--border-panel); }
  .stat-val.mono   { font-family: 'Courier New', monospace; letter-spacing: .05em; }
  .stat-val.dim    { color: var(--text-dim); }
  .stat-sep  { color: var(--border-dim); }
  .stat-right { margin-left: auto; color: var(--text-dim); white-space: nowrap; }
</style>
