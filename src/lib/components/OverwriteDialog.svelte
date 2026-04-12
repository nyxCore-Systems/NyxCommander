<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let conflicts: string[];
  export let srcs: string[];
  export let dstDir: string;
  export let move: boolean = false;

  const dispatch = createEventDispatcher<{ done: void; cancel: void }>();

  let running = false;
  let error = '';
  let dialogEl: HTMLElement;

  onMount(() => dialogEl?.focus());

  const shown = conflicts.slice(0, 5);
  const extra = conflicts.length - shown.length;

  async function act(mode: 'overwrite' | 'skip' | 'rename') {
    running = true;
    error = '';
    try {
      await invoke('copy_items_mode', { srcs, dstDir, mode });
      if (move) {
        await invoke('delete_items', { paths: srcs });
      }
      dispatch('done');
    } catch (e) {
      error = String(e);
      running = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') { e.stopPropagation(); dispatch('cancel'); }
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="overlay" on:click|self={() => dispatch('cancel')}>
  <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
  <div class="dialog" role="dialog" tabindex="-1" bind:this={dialogEl} on:keydown={handleKeydown}>
    <div class="dialog-title danger">FILES ALREADY EXIST</div>
    <div class="dialog-body">
      <div class="subtitle">The following files already exist in the destination:</div>
      <div class="file-list">
        {#each shown as name}
          <div class="file-name">{name.split('/').at(-1) ?? name}</div>
        {/each}
        {#if extra > 0}
          <div class="more">+ {extra} more</div>
        {/if}
      </div>
      {#if error}<div class="dialog-error">{error}</div>{/if}
    </div>
    <div class="dialog-footer">
      <button on:click={() => act('overwrite')} disabled={running}>Overwrite all</button>
      <button on:click={() => act('skip')}      disabled={running}>Skip existing</button>
      <button on:click={() => act('rename')}    disabled={running}>Auto-rename</button>
      <button on:click={() => dispatch('cancel')} disabled={running}>Cancel (Esc)</button>
    </div>
  </div>
</div>

<style>
  .subtitle {
    font-size: 11px;
    color: var(--text-dim);
    margin-bottom: 8px;
  }
  .file-list {
    margin: 4px 0;
    max-height: 120px;
    overflow-y: auto;
  }
  .file-name {
    font-size: 11px;
    padding: 1px 0;
    color: var(--text);
  }
  .more {
    font-size: 10px;
    color: var(--text-dim);
    margin-top: 2px;
  }
</style>
