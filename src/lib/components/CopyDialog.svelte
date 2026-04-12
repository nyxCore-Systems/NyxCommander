<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let srcs: string[];
  export let dstDir: string;
  export let move: boolean = false;

  const dispatch = createEventDispatcher<{ done: void; cancel: void }>();

  let destination = dstDir;
  let running = false;
  let error = '';

  async function confirm() {
    running = true;
    error = '';
    try {
      if (move) {
        await invoke('move_items', { srcs, dstDir: destination });
      } else {
        await invoke('copy_items', { srcs, dstDir: destination });
      }
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
    <div class="dialog-title">{move ? 'Move' : 'Copy'} {srcs.length} item{srcs.length !== 1 ? 's' : ''}</div>
    <div class="dialog-body">
      <label for="dst-input">Destination:</label>
      <input id="dst-input" bind:value={destination} disabled={running} />
      {#if error}<div class="dialog-error">{error}</div>{/if}
    </div>
    <div class="dialog-footer">
      <button on:click={confirm} disabled={running}>
        {running ? (move ? 'Moving...' : 'Copying...') : (move ? 'Move (F6)' : 'Copy (F5)')}
      </button>
      <button on:click={() => dispatch('cancel')} disabled={running}>Cancel (Esc)</button>
    </div>
  </div>
</div>
