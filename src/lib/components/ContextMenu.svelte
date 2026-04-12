<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import type { FileEntry } from '$lib/stores/panelStore';

  export let x: number;
  export let y: number;
  export let entry: FileEntry | null;
  export let activePath: string;

  const dispatch = createEventDispatcher<{ action: string; close: void }>();

  let menuEl: HTMLElement;
  let clampedX = x;
  let clampedY = y;

  onMount(() => {
    menuEl?.focus();
    // Clamp to viewport so menu never goes off-screen
    const rect = menuEl.getBoundingClientRect();
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    clampedX = Math.min(x, vw - rect.width - 4);
    clampedY = Math.min(y, vh - rect.height - 4);
  });

  function act(action: string) {
    dispatch('action', action);
    dispatch('close');
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') { e.stopPropagation(); dispatch('close'); }
  }

  // Close on click outside is handled by the invisible backdrop
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="backdrop" on:click={() => dispatch('close')}></div>

<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
<nav
  class="context-menu"
  role="menu"
  tabindex="-1"
  bind:this={menuEl}
  style="left: {clampedX}px; top: {clampedY}px;"
  on:click|stopPropagation
  on:keydown={handleKeydown}
>
  <button role="menuitem" on:click={() => act('open')}>Open</button>
  <button role="menuitem" on:click={() => act('open-system')}>Open with System App</button>
  <div class="sep"></div>
  <button role="menuitem" on:click={() => act('rename')}>Rename</button>
  <button role="menuitem" on:click={() => act('copy')}>Copy</button>
  <button role="menuitem" on:click={() => act('move')}>Move</button>
  <button role="menuitem" on:click={() => act('delete')}>Delete</button>
  <div class="sep"></div>
  <button role="menuitem" on:click={() => act('copy-path')}>Copy Path</button>
  {#if entry !== null}
    <button role="menuitem" on:click={() => act('compress')}>Compress to ZIP</button>
  {/if}
  <div class="sep"></div>
  {#if entry && !entry.is_dir}
    <button role="menuitem" on:click={() => act('checksum')}>Checksum</button>
  {/if}
  <button role="menuitem" on:click={() => act('properties')}>Properties</button>
  <div class="sep"></div>
  <button role="menuitem" on:click={() => act('terminal')}>Open Terminal Here</button>
  <button role="menuitem" on:click={() => act('mkdir')}>New Folder</button>
</nav>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 199;
  }

  .context-menu {
    position: fixed;
    z-index: 200;
    width: 180px;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.7);
    padding: 3px 0;
    font-family: 'Courier New', monospace;
    outline: none;
  }

  .context-menu button {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    color: var(--text);
    font-family: 'Courier New', monospace;
    font-size: 11px;
    padding: 5px 12px;
    cursor: pointer;
    letter-spacing: 0.02em;
    white-space: nowrap;
  }

  .context-menu button:hover {
    background: var(--bg-row-hover);
    color: var(--border);
  }

  .sep {
    height: 1px;
    background: var(--border-dim);
    margin: 3px 0;
  }
</style>
