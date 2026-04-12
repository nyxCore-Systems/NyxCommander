<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let paths: string[];

  const dispatch = createEventDispatcher<{ close: void }>();

  interface Result {
    path: string;
    name: string;
    hash: string | null;
    error: string | null;
    loading: boolean;
    copied: boolean;
  }

  let results: Result[] = paths.map(p => ({
    path: p,
    name: p.split('/').at(-1) ?? p,
    hash: null,
    error: null,
    loading: true,
    copied: false,
  }));

  let dialogEl: HTMLElement;

  onMount(async () => {
    dialogEl?.focus();
    await Promise.all(results.map((r, i) => computeOne(i)));
  });

  async function computeOne(i: number) {
    try {
      const hash = await invoke<string>('compute_checksum', { path: results[i].path });
      results[i] = { ...results[i], hash, loading: false };
    } catch (e) {
      results[i] = { ...results[i], error: String(e), loading: false };
    }
    results = results; // trigger reactivity
  }

  async function copyHash(i: number) {
    if (!results[i].hash) return;
    await navigator.clipboard.writeText(results[i].hash!);
    results[i] = { ...results[i], copied: true };
    results = results;
    setTimeout(() => {
      results[i] = { ...results[i], copied: false };
      results = results;
    }, 1500);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') { e.stopPropagation(); dispatch('close'); }
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="overlay" on:click|self={() => dispatch('close')}>
  <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
  <div class="dialog wide" role="dialog" tabindex="-1" bind:this={dialogEl} on:keydown={handleKeydown}>
    <div class="dialog-title">SHA-256 CHECKSUMS</div>
    <div class="dialog-body">
      <div class="results">
        {#each results as r, i}
          <div class="result-row">
            <div class="result-name">{r.name}</div>
            <div class="result-hash">
              {#if r.loading}
                <span class="spinner">computing…</span>
              {:else if r.error}
                <span class="err">{r.error}</span>
              {:else}
                <span class="hash">{r.hash}</span>
                <button class="copy-btn" on:click={() => copyHash(i)}>
                  {r.copied ? '✓' : 'copy'}
                </button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>
    <div class="dialog-footer">
      <button on:click={() => dispatch('close')}>Close (Esc)</button>
    </div>
  </div>
</div>

<style>
  :global(.dialog.wide) {
    min-width: 540px;
    max-width: 800px;
  }
  .results {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 300px;
    overflow-y: auto;
  }
  .result-row {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px 0;
    border-bottom: 1px solid var(--border-dim);
  }
  .result-row:last-child {
    border-bottom: none;
  }
  .result-name {
    font-size: 11px;
    color: var(--text-dim);
    letter-spacing: 0.03em;
  }
  .result-hash {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .hash {
    font-size: 11px;
    color: var(--text);
    word-break: break-all;
    flex: 1;
  }
  .spinner {
    font-size: 11px;
    color: var(--border);
    animation: pulse 1s ease-in-out infinite alternate;
  }
  @keyframes pulse {
    from { opacity: 0.4; }
    to   { opacity: 1; }
  }
  .err {
    font-size: 11px;
    color: var(--text-danger);
  }
  .copy-btn {
    background: var(--bg-fnkey);
    color: var(--text-dim);
    border: 1px solid var(--border-dim);
    padding: 2px 8px;
    cursor: pointer;
    font-family: 'Courier New', monospace;
    font-size: 10px;
    flex-shrink: 0;
  }
  .copy-btn:hover {
    border-color: var(--border);
    color: var(--border);
  }
</style>
