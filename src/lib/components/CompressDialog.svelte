<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let paths: string[];
  export let dstDir: string;

  const dispatch = createEventDispatcher<{ done: void; cancel: void }>();

  let archiveName = 'archive.zip';
  let destination = dstDir;
  let running = false;
  let error = '';
  let inputEl: HTMLInputElement;

  onMount(() => {
    inputEl?.focus();
    inputEl?.select();
  });

  async function confirm() {
    if (!archiveName.trim()) return;
    running = true;
    error = '';
    try {
      const dstPath = destination.replace(/\/+$/, '') + '/' + archiveName.trim();
      await invoke('create_zip', { srcs: paths, dstPath });
      dispatch('done');
    } catch (e) {
      error = String(e);
      running = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') { e.stopPropagation(); dispatch('cancel'); }
    if (e.key === 'Enter' && !running) { e.stopPropagation(); confirm(); }
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="overlay" on:click|self={() => dispatch('cancel')}>
  <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
  <div class="dialog" role="dialog" tabindex="-1" on:keydown={handleKeydown}>
    <div class="dialog-title">COMPRESS TO ZIP</div>
    <div class="dialog-body">
      <div class="item-count">{paths.length} item{paths.length !== 1 ? 's' : ''} selected</div>
      <label for="archive-name">Archive name:</label>
      <input id="archive-name" bind:this={inputEl} bind:value={archiveName} disabled={running} />
      <label for="dst-dir" style="margin-top: 8px;">Destination:</label>
      <input id="dst-dir" bind:value={destination} disabled={running} />
      {#if running}
        <div class="spinner">Compressing…</div>
      {/if}
      {#if error}<div class="dialog-error">{error}</div>{/if}
    </div>
    <div class="dialog-footer">
      <button on:click={confirm} disabled={running}>
        {running ? 'Compressing…' : 'Compress'}
      </button>
      <button on:click={() => dispatch('cancel')} disabled={running}>Cancel (Esc)</button>
    </div>
  </div>
</div>

<style>
  .item-count {
    font-size: 11px;
    color: var(--text-dim);
    margin-bottom: 10px;
  }
  .spinner {
    font-size: 11px;
    color: var(--border);
    margin-top: 8px;
    animation: pulse 1s ease-in-out infinite alternate;
  }
  @keyframes pulse {
    from { opacity: 0.5; }
    to   { opacity: 1; }
  }
</style>
