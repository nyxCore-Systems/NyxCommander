<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let paths: string[];

  const dispatch = createEventDispatcher<{ done: void; cancel: void }>();

  let running = false;
  let error = '';
  let dialogEl: HTMLElement;

  onMount(() => dialogEl?.focus());

  const names = paths.map(p => p.split('/').at(-1) ?? p);

  async function confirm() {
    running = true;
    error = '';
    try {
      await invoke('delete_items', { paths });
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
  <div class="dialog" role="dialog" tabindex="-1" bind:this={dialogEl} on:keydown={handleKeydown}>
    <div class="dialog-title danger">Delete {paths.length} item{paths.length !== 1 ? 's' : ''}?</div>
    <div class="dialog-body">
      <div class="file-list">
        {#each names.slice(0, 8) as name}
          <div class="file-name">{name}</div>
        {/each}
        {#if names.length > 8}
          <div class="more">… and {names.length - 8} more</div>
        {/if}
      </div>
      {#if error}<div class="dialog-error">{error}</div>{/if}
    </div>
    <div class="dialog-footer">
      <button class="danger" on:click={confirm} disabled={running}>
        {running ? 'Deleting...' : 'Delete (F8)'}
      </button>
      <button on:click={() => dispatch('cancel')} disabled={running}>Cancel (Esc)</button>
    </div>
  </div>
</div>

<style>
  .file-list { margin: 4px 0; max-height: 120px; overflow-y: auto; }
  .file-name { font-size: 11px; padding: 1px 0; color: #e0e0f0; }
  .more { font-size: 10px; color: #4a4a6e; }
</style>
