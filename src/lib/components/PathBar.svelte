<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let path: string;
  export let active: boolean = false;

  const dispatch = createEventDispatcher<{ navigate: string; editpath: void }>();

  $: segments = (() => {
    const parts = path.split('/').filter(Boolean);
    return parts.map((part, i) => ({
      label: part,
      path: '/' + parts.slice(0, i + 1).join('/'),
    }));
  })();
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="path-bar" class:active title={path} on:click={() => dispatch('editpath')}>
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
  <span class="sep root" on:click|stopPropagation={() => dispatch('navigate', '/')}>/</span>
  {#each segments as seg, i}
    {#if i > 0}<span class="sep">/</span>{/if}
    <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
    <span class="seg" on:click|stopPropagation={() => dispatch('navigate', seg.path)}>{seg.label}</span>
  {/each}
  <span class="edit-hint">✎</span>
</div>

<style>
  .path-bar {
    display: flex;
    align-items: center;
    flex-wrap: nowrap;
    overflow: hidden;
    padding: 3px 8px;
    background: var(--bg-header);
    color: var(--text-dim);
    font-family: 'Courier New', monospace;
    font-size: 11px;
    border-bottom: 1px solid var(--border-dim);
    white-space: nowrap;
    gap: 0;
    min-height: 20px;
    flex-shrink: 0;
    cursor: pointer;
  }
  .path-bar:hover .edit-hint { opacity: 0.5; }
  .path-bar.active {
    color: var(--border);
    border-bottom-color: var(--border);
  }
  .seg {
    cursor: pointer;
    color: inherit;
  }
  .seg:hover { color: var(--text); text-decoration: underline; }
  .sep { color: var(--border-dim); margin: 0 1px; }
  .root { cursor: pointer; }
  .root:hover { color: var(--text); }
  .edit-hint {
    margin-left: auto;
    padding-left: 6px;
    flex-shrink: 0;
    opacity: 0;
    font-size: 10px;
    color: var(--accent);
    transition: opacity 0.1s;
    pointer-events: none;
  }
</style>
