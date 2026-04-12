<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let parentPath: string;

  const dispatch = createEventDispatcher<{ done: void; cancel: void }>();

  let name = '';
  let running = false;
  let error = '';
  let input: HTMLInputElement;

  import { onMount } from 'svelte';
  onMount(() => input?.focus());

  async function confirm() {
    if (!name.trim()) return;
    running = true;
    error = '';
    try {
      const path = parentPath.replace(/\/$/, '') + '/' + name.trim();
      await invoke('create_dir', { path });
      dispatch('done');
    } catch (e) {
      error = String(e);
      running = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') { e.stopPropagation(); dispatch('cancel'); }
    if (e.key === 'Enter')  { e.stopPropagation(); confirm(); }
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="overlay" on:click|self={() => dispatch('cancel')}>
  <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
  <div class="dialog" role="dialog" tabindex="-1" on:keydown={handleKeydown}>
    <div class="dialog-title">Create Directory</div>
    <div class="dialog-body">
      <div class="in-label">In: <span class="path">{parentPath}</span></div>
      <label for="mkdir-input">Name:</label>
      <input id="mkdir-input" bind:this={input} bind:value={name} disabled={running} placeholder="new-directory" />
      {#if error}<div class="dialog-error">{error}</div>{/if}
    </div>
    <div class="dialog-footer">
      <button on:click={confirm} disabled={running || !name.trim()}>
        {running ? 'Creating...' : 'Create (F7)'}
      </button>
      <button on:click={() => dispatch('cancel')} disabled={running}>Cancel (Esc)</button>
    </div>
  </div>
</div>

<style>
  .in-label { font-size: 10px; color: #4a4a6e; margin-bottom: 6px; }
  .path { color: #00d4ff; }
</style>
