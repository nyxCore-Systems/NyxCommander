<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let left: string;
  export let right: string;

  const dispatch = createEventDispatcher<{ close: void }>();

  interface DiffLine {
    kind: 'header' | 'hunk' | 'add' | 'del' | 'ctx' | 'binary' | 'msg';
    text: string;
    leftNo: number | null;
    rightNo: number | null;
  }

  let rawDiff = '';
  let parsed: DiffLine[] = [];
  let loading = true;
  let error = '';
  let identical = false;
  let isBinary = false;
  let overlayEl: HTMLElement;
  let diffAreaEl: HTMLElement;

  const leftName  = left.split('/').pop()  ?? left;
  const rightName = right.split('/').pop() ?? right;

  onMount(async () => {
    overlayEl.focus();
    try {
      rawDiff = await invoke<string>('diff_files', { left, right });
      if (rawDiff.trim() === '') {
        identical = true;
      } else {
        parsed = parseDiff(rawDiff);
      }
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  function parseDiff(raw: string): DiffLine[] {
    const result: DiffLine[] = [];
    let leftNo = 0;
    let rightNo = 0;

    for (const line of raw.split('\n')) {
      if (line === '') continue;

      if (line.startsWith('Binary files')) {
        isBinary = true;
        result.push({ kind: 'binary', text: line, leftNo: null, rightNo: null });
        continue;
      }

      if (line.startsWith('--- ') || line.startsWith('+++ ')) {
        result.push({ kind: 'header', text: line, leftNo: null, rightNo: null });
        continue;
      }

      if (line.startsWith('@@ ')) {
        // Parse @@ -l,s +l,s @@ to get starting line numbers
        const m = line.match(/@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/);
        if (m) {
          leftNo  = parseInt(m[1], 10) - 1;
          rightNo = parseInt(m[2], 10) - 1;
        }
        result.push({ kind: 'hunk', text: line, leftNo: null, rightNo: null });
        continue;
      }

      if (line.startsWith('+')) {
        rightNo++;
        result.push({ kind: 'add', text: line.slice(1), leftNo: null, rightNo });
        continue;
      }

      if (line.startsWith('-')) {
        leftNo++;
        result.push({ kind: 'del', text: line.slice(1), leftNo, rightNo: null });
        continue;
      }

      // context (space prefix) or no prefix
      const text = line.startsWith(' ') ? line.slice(1) : line;
      leftNo++;
      rightNo++;
      result.push({ kind: 'ctx', text, leftNo, rightNo });
    }

    return result;
  }

  $: stats = (() => {
    const adds = parsed.filter(l => l.kind === 'add').length;
    const dels = parsed.filter(l => l.kind === 'del').length;
    return { adds, dels };
  })();

  const LINE_PX = 18;

  function handleKey(e: KeyboardEvent) {
    e.stopPropagation();
    switch (e.key) {
      case 'Escape':    dispatch('close'); break;
      case 'ArrowDown': scroll(LINE_PX);   e.preventDefault(); break;
      case 'ArrowUp':   scroll(-LINE_PX);  e.preventDefault(); break;
      case 'PageDown':  scroll(pageSize()); e.preventDefault(); break;
      case 'PageUp':    scroll(-pageSize()); e.preventDefault(); break;
      case 'Home':      if (e.ctrlKey || e.metaKey) scrollTo(0); e.preventDefault(); break;
      case 'End':       if (e.ctrlKey || e.metaKey) scrollTo(Infinity); e.preventDefault(); break;
    }
  }

  function scroll(delta: number) {
    if (diffAreaEl) diffAreaEl.scrollTop += delta;
  }

  function scrollTo(pos: number) {
    if (diffAreaEl) diffAreaEl.scrollTop = pos;
  }

  function pageSize() {
    return diffAreaEl ? diffAreaEl.clientHeight * 0.9 : 400;
  }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="overlay" bind:this={overlayEl} on:keydown={handleKey} tabindex="-1">
  <div class="viewer" role="dialog" tabindex="-1">

    <div class="title-bar">
      <span class="title-label">DIFF</span>
      <span class="file-left">{leftName}</span>
      <span class="vs">↔</span>
      <span class="file-right">{rightName}</span>
      <button class="close-btn" on:click={() => dispatch('close')}>✕</button>
    </div>

    <div class="path-bar">
      <span class="path-del">{left}</span>
      <span class="path-sep">→</span>
      <span class="path-add">{right}</span>
    </div>

    {#if loading}
      <div class="msg-row">Computing diff…</div>

    {:else if error}
      <div class="msg-row error">{error}</div>

    {:else if identical}
      <div class="msg-row identical">
        ✓ Files are identical
      </div>

    {:else if isBinary}
      <div class="msg-row binary">
        Binary files differ — no text diff available
      </div>

    {:else}
      <div class="stats-bar">
        <span class="stat-add">+{stats.adds} added</span>
        <span class="stat-del">−{stats.dels} removed</span>
        <span class="hint">↑↓ scroll · PgUp/Dn page · Esc close</span>
      </div>

      <div class="diff-area" bind:this={diffAreaEl}>
        <table class="diff-table">
          <tbody>
            {#each parsed as dl, i (i)}
              {#if dl.kind === 'header'}
                <tr class="row-header">
                  <td class="lno"></td>
                  <td class="lno"></td>
                  <td class="diff-text header-text">{dl.text}</td>
                </tr>
              {:else if dl.kind === 'hunk'}
                <tr class="row-hunk">
                  <td class="lno"></td>
                  <td class="lno"></td>
                  <td class="diff-text hunk-text">{dl.text}</td>
                </tr>
              {:else if dl.kind === 'add'}
                <tr class="row-add">
                  <td class="lno lno-left"></td>
                  <td class="lno lno-right">{dl.rightNo}</td>
                  <td class="diff-text add-text">+{dl.text}</td>
                </tr>
              {:else if dl.kind === 'del'}
                <tr class="row-del">
                  <td class="lno lno-left">{dl.leftNo}</td>
                  <td class="lno lno-right"></td>
                  <td class="diff-text del-text">−{dl.text}</td>
                </tr>
              {:else if dl.kind === 'ctx'}
                <tr class="row-ctx">
                  <td class="lno lno-left">{dl.leftNo}</td>
                  <td class="lno lno-right">{dl.rightNo}</td>
                  <td class="diff-text ctx-text"> {dl.text}</td>
                </tr>
              {:else if dl.kind === 'binary'}
                <tr class="row-binary">
                  <td class="lno"></td>
                  <td class="lno"></td>
                  <td class="diff-text binary-text">{dl.text}</td>
                </tr>
              {/if}
            {/each}
          </tbody>
        </table>
      </div>
    {/if}

  </div>
</div>

<style>
  .overlay {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.88);
    display: flex; align-items: center; justify-content: center;
    z-index: 100;
  }

  .viewer {
    display: flex; flex-direction: column;
    width: 960px; max-width: 96vw; height: 82vh;
    background: var(--bg);
    border: 1px solid var(--border-panel);
    font-family: 'Courier New', monospace;
  }

  .title-bar {
    display: flex; align-items: center; gap: 8px;
    padding: 0 10px; height: 26px; flex-shrink: 0;
    background: var(--bg-header);
    border-bottom: 1px solid var(--border-panel);
    font-size: 11px; overflow: hidden;
  }
  .title-label {
    color: var(--border-panel); font-weight: bold;
    letter-spacing: .1em; flex-shrink: 0;
  }
  .file-left  { color: var(--text-danger); flex-shrink: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .vs         { color: var(--text-dim); flex-shrink: 0; }
  .file-right { color: var(--text-sel); flex-shrink: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; }
  .close-btn  {
    background: none; border: none; color: var(--text-dim);
    cursor: pointer; font-size: 13px; flex-shrink: 0; margin-left: auto;
  }
  .close-btn:hover { color: var(--text-danger); }

  .path-bar {
    display: flex; gap: 8px; align-items: center;
    padding: 0 10px; height: 18px; flex-shrink: 0;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border-dim);
    font-size: 10px; overflow: hidden;
  }
  .path-del { color: var(--text-danger); opacity: .7; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; }
  .path-sep { color: var(--text-dim); flex-shrink: 0; }
  .path-add { color: var(--text-sel); opacity: .7; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; }

  .stats-bar {
    display: flex; gap: 14px; align-items: center;
    padding: 0 10px; height: 20px; flex-shrink: 0;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border-dim);
    font-size: 10px;
  }
  .stat-add { color: var(--text-sel); }
  .stat-del { color: var(--text-danger); }
  .hint { margin-left: auto; color: var(--text-dim); }

  .msg-row {
    flex: 1; display: flex; align-items: center; justify-content: center;
    color: var(--text-dim); font-size: 13px; font-family: 'Courier New', monospace;
  }
  .msg-row.error    { color: var(--text-danger); }
  .msg-row.identical { color: var(--text-sel); }
  .msg-row.binary   { color: var(--text-dir); }

  /* Diff table */
  .diff-area {
    flex: 1; overflow: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--bg);
  }

  .diff-table {
    width: 100%; border-collapse: collapse;
    font-size: 12px; font-family: 'Courier New', monospace;
  }

  .lno {
    width: 48px; min-width: 48px; padding: 0 6px;
    text-align: right; color: var(--text-dim);
    font-size: 10px; user-select: none;
    vertical-align: top; line-height: 1.5;
    border-right: 1px solid var(--border-dim);
    background: var(--bg-panel);
    white-space: nowrap;
  }

  .diff-text {
    padding: 0 10px; white-space: pre; tab-size: 4;
    line-height: 1.5; vertical-align: top;
    width: 100%;
  }

  /* Row colors */
  .row-add     { background: rgba(0, 255, 136, 0.07); }
  .row-del     { background: rgba(255, 50,  80,  0.08); }
  .row-ctx     { background: transparent; }
  .row-hunk    { background: rgba(0, 180, 220, 0.08); }
  .row-header  { background: transparent; }
  .row-binary  { background: rgba(255, 150, 0, 0.08); }

  .add-text    { color: var(--text-sel); }
  .del-text    { color: var(--text-danger); }
  .ctx-text    { color: var(--text-dim); }
  .hunk-text   { color: var(--border-panel); font-style: italic; }
  .header-text { color: var(--text-dim); font-size: 10px; }
  .binary-text { color: #ff9900; }

  /* Hover highlight on context rows for readability */
  .row-ctx:hover .ctx-text { color: var(--text); }
</style>
