<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let paths: string[];

  const dispatch = createEventDispatcher<{ done: void; cancel: void }>();

  // Pattern uses: $N = counter, $1 = original name (no ext), $E = extension, $F = full name
  let searchPat = '';
  let replacePat = '';
  let useRegex = false;
  let counterStart = 1;
  let counterStep = 1;
  let counterPad = 2;
  let error = '';
  let running = false;
  let searchInputEl: HTMLInputElement;

  onMount(() => searchInputEl?.focus());

  interface Preview { original: string; newName: string; error: string }
  let previews: Preview[] = [];

  $: previews = computePreviews(paths, searchPat, replacePat, useRegex, counterStart, counterStep, counterPad);

  function computePreviews(
    ps: string[],
    search: string,
    replace: string,
    regex: boolean,
    start: number,
    step: number,
    pad: number,
  ): Preview[] {
    return ps.map((p, i) => {
      const full = p.split('/').pop() ?? '';
      const dotIdx = full.lastIndexOf('.');
      const base = dotIdx > 0 ? full.slice(0, dotIdx) : full;
      const ext = dotIdx > 0 ? full.slice(dotIdx + 1) : '';
      const n = start + i * step;
      const counter = String(n).padStart(pad, '0');

      let result = replace
        .replace(/\$N/g, counter)
        .replace(/\$1/g, base)
        .replace(/\$E/g, ext)
        .replace(/\$F/g, full);

      if (search) {
        try {
          if (regex) {
            const re = new RegExp(search, 'g');
            result = full.replace(re, result);
          } else {
            result = full.replaceAll(search, result);
          }
        } catch (e) {
          return { original: full, newName: '', error: String(e) };
        }
      }

      if (!result) return { original: full, newName: '', error: 'Empty name' };
      return { original: full, newName: result, error: '' };
    });
  }

  async function apply() {
    if (previews.some(p => p.error)) return;
    running = true;
    error = '';
    const renames: [string, string][] = paths.map((p, i) => [p, previews[i].newName]);
    try {
      await invoke('batch_rename', { renames });
      dispatch('done');
    } catch (e) {
      error = String(e);
    } finally {
      running = false;
    }
  }

  function handleKey(e: KeyboardEvent) {
    e.stopPropagation();
    if (e.key === 'Escape') { dispatch('cancel'); }
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) { apply(); }
  }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="overlay" on:keydown={handleKey}>
  <div class="dialog">
    <div class="title">Multi-Rename — {paths.length} file{paths.length !== 1 ? 's' : ''}</div>

    <div class="controls">
      <div class="row">
        <label>
          <span>Search</span>
          <input bind:this={searchInputEl} bind:value={searchPat} placeholder="text or regex…" spellcheck="false" />
        </label>
        <label class="check">
          <input type="checkbox" bind:checked={useRegex} />
          Regex
        </label>
      </div>
      <div class="row">
        <label>
          <span>Replace / Pattern</span>
          <input bind:value={replacePat} placeholder="$1 $N.$E — $1=name $N=counter $E=ext $F=full" spellcheck="false" />
        </label>
      </div>
      <div class="row">
        <label>
          <span>Counter start</span>
          <input type="number" bind:value={counterStart} min="0" />
        </label>
        <label>
          <span>Step</span>
          <input type="number" bind:value={counterStep} min="1" />
        </label>
        <label>
          <span>Pad width</span>
          <input type="number" bind:value={counterPad} min="1" max="6" />
        </label>
      </div>
    </div>

    <div class="preview-list">
      {#each previews as p}
        <div class="preview-row" class:err={!!p.error}>
          <span class="old">{p.original}</span>
          <span class="arrow">→</span>
          <span class="new">{p.error ? `⚠ ${p.error}` : p.newName}</span>
        </div>
      {/each}
    </div>

    {#if error}<div class="error">{error}</div>{/if}

    <div class="actions">
      <button on:click={() => dispatch('cancel')}>Cancel</button>
      <button
        class="primary"
        on:click={apply}
        disabled={running || previews.some(p => !!p.error) || previews.some(p => !p.newName)}
      >
        {running ? 'Renaming…' : 'Rename (⌘↵)'}
      </button>
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
  }
  .dialog {
    background: var(--bg-panel);
    border: 1px solid var(--border-panel);
    width: min(700px, 95vw);
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    font-family: 'Courier New', monospace;
    font-size: 12px;
  }
  .title {
    padding: 8px 12px;
    background: var(--bg-header);
    border-bottom: 1px solid var(--border-dim);
    color: var(--accent);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .controls {
    padding: 10px 12px;
    border-bottom: 1px solid var(--border-dim);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }
  label {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-dim);
    flex: 1;
    min-width: 120px;
  }
  label span { white-space: nowrap; font-size: 10px; }
  label.check { flex: 0; }
  input:not([type="checkbox"]):not([type="number"]):not([type="color"]) {
    flex: 1;
    background: var(--bg);
    border: 1px solid var(--border-dim);
    color: var(--text);
    font-family: inherit;
    font-size: 12px;
    padding: 3px 6px;
    outline: none;
  }
  input[type="number"] {
    width: 60px;
    background: var(--bg);
    border: 1px solid var(--border-dim);
    color: var(--text);
    font-family: inherit;
    font-size: 12px;
    padding: 3px 6px;
    outline: none;
  }
  input:focus { border-color: var(--accent); }
  .preview-list {
    overflow-y: auto;
    flex: 1;
    padding: 4px 0;
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--bg);
  }
  .preview-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 2px 12px;
    font-size: 11px;
    border-bottom: 1px solid transparent;
  }
  .preview-row:hover { background: var(--bg-header); }
  .preview-row.err .new { color: var(--danger); }
  .old { color: var(--text-dim); flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .arrow { color: var(--text-dim); flex-shrink: 0; }
  .new { color: var(--accent); flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .error { padding: 6px 12px; color: var(--danger); font-size: 11px; }
  .actions {
    display: flex;
    gap: 8px;
    padding: 8px 12px;
    justify-content: flex-end;
    border-top: 1px solid var(--border-dim);
  }
  button {
    background: var(--bg-header);
    border: 1px solid var(--border-dim);
    color: var(--text);
    font-family: inherit;
    font-size: 11px;
    padding: 4px 12px;
    cursor: pointer;
  }
  button:hover { border-color: var(--accent); color: var(--accent); }
  button.primary { border-color: var(--accent); color: var(--accent); }
  button:disabled { opacity: 0.4; cursor: default; }
</style>
