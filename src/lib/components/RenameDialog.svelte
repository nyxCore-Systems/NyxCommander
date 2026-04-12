<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let src: string;
  export let currentName: string;

  const dispatch = createEventDispatcher<{ done: void; cancel: void }>();

  let newName = currentName;
  let running = false;
  let error = '';
  let input: HTMLInputElement;

  onMount(() => {
    input?.focus();
    // Select filename without extension
    const dot = currentName.lastIndexOf('.');
    if (dot > 0) input?.setSelectionRange(0, dot);
    else input?.select();
  });

  async function confirm() {
    if (!newName.trim() || newName === currentName) {
      dispatch('cancel');
      return;
    }
    running = true;
    error = '';
    try {
      await invoke('rename_path', { src, newName: newName.trim() });
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
    <div class="dialog-title">Rename</div>
    <div class="dialog-body">
      <label for="rename-input">New name:</label>
      <input id="rename-input" bind:this={input} bind:value={newName} disabled={running} />
      {#if error}<div class="dialog-error">{error}</div>{/if}
    </div>
    <div class="dialog-footer">
      <button on:click={confirm} disabled={running || !newName.trim()}>
        {running ? 'Renaming...' : 'Rename'}
      </button>
      <button on:click={() => dispatch('cancel')} disabled={running}>Cancel (Esc)</button>
    </div>
  </div>
</div>
