<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { createEventDispatcher, onMount } from 'svelte';

  export let left: string;
  export let right: string;

  const dispatch = createEventDispatcher<{ close: void }>();

  const MIME: Record<string, string> = {
    png: 'image/png', jpg: 'image/jpeg', jpeg: 'image/jpeg',
    gif: 'image/gif', webp: 'image/webp', svg: 'image/svg+xml',
    bmp: 'image/bmp', ico: 'image/x-icon', tiff: 'image/tiff',
    tif: 'image/tiff', avif: 'image/avif',
  };

  function ext(path: string): string {
    return path.split('.').pop()?.toLowerCase() ?? '';
  }
  function basename(path: string): string {
    return path.split('/').pop() ?? path;
  }

  let leftSrc = '';
  let rightSrc = '';
  let leftErr = '';
  let rightErr = '';
  let loading = true;

  // split view: 0–100, percentage of left panel width
  let split = 50;
  let dragging = false;
  let containerEl: HTMLElement;

  async function loadImage(path: string): Promise<string> {
    const e = ext(path);
    const size = (await invoke<{ data: number[]; file_size: number; offset: number }>(
      'read_file_bytes', { path, offset: 0, count: 0 }
    )).file_size;
    const chunk = await invoke<{ data: number[]; file_size: number; offset: number }>(
      'read_file_bytes', { path, offset: 0, count: size }
    );
    const bytes = new Uint8Array(chunk.data);
    let binary = '';
    for (let i = 0; i < bytes.length; i++) binary += String.fromCharCode(bytes[i]);
    return `data:${MIME[e] ?? 'image/png'};base64,${btoa(binary)}`;
  }

  onMount(async () => {
    loading = true;
    const results = await Promise.allSettled([loadImage(left), loadImage(right)]);
    if (results[0].status === 'fulfilled') leftSrc = results[0].value;
    else leftErr = String((results[0] as PromiseRejectedResult).reason);
    if (results[1].status === 'fulfilled') rightSrc = results[1].value;
    else rightErr = String((results[1] as PromiseRejectedResult).reason);
    loading = false;
  });

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') { e.stopPropagation(); dispatch('close'); }
  }

  function startDrag(e: MouseEvent) {
    dragging = true;
    moveDrag(e);
  }
  function moveDrag(e: MouseEvent) {
    if (!dragging || !containerEl) return;
    const rect = containerEl.getBoundingClientRect();
    split = Math.max(5, Math.min(95, ((e.clientX - rect.left) / rect.width) * 100));
  }
  function endDrag() { dragging = false; }
</script>

<svelte:window on:keydown={onKeydown} on:mousemove={moveDrag} on:mouseup={endDrag} />

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="overlay" on:click|self={() => dispatch('close')}>
  <div class="viewer">
    <div class="titlebar">
      <span class="title">Image Compare</span>
      <button class="close-btn" on:click={() => dispatch('close')}>✕</button>
    </div>

    {#if loading}
      <div class="status">Loading…</div>
    {:else}
      <div
        class="compare-area"
        bind:this={containerEl}
        class:dragging
      >
        <!-- Left panel -->
        <div class="img-pane left-pane" style="width: {split}%">
          <div class="pane-header">
            <span class="pane-name">{basename(left)}</span>
          </div>
          <div class="pane-body">
            {#if leftErr}
              <div class="err">{leftErr}</div>
            {:else if leftSrc}
              <img src={leftSrc} alt={basename(left)} />
            {/if}
          </div>
        </div>

        <!-- Draggable divider -->
        <!-- svelte-ignore a11y-no-static-element-interactions -->
        <div
          class="divider"
          style="left: {split}%"
          on:mousedown|preventDefault={startDrag}
        >
          <div class="divider-handle">⟺</div>
        </div>

        <!-- Right panel -->
        <div class="img-pane right-pane" style="left: {split}%; width: {100 - split}%">
          <div class="pane-header">
            <span class="pane-name">{basename(right)}</span>
          </div>
          <div class="pane-body">
            {#if rightErr}
              <div class="err">{rightErr}</div>
            {:else if rightSrc}
              <img src={rightSrc} alt={basename(right)} />
            {/if}
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }

  .viewer {
    width: 92vw;
    height: 88vh;
    background: var(--bg-panel, #1a1a2e);
    border: 1px solid var(--border-panel, #00d4ff);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    font-family: 'Courier New', monospace;
  }

  .titlebar {
    display: flex;
    align-items: center;
    padding: 0 10px;
    height: 28px;
    background: var(--bg-header, #0f1020);
    border-bottom: 1px solid var(--border-dim, #2a2a4e);
    flex-shrink: 0;
  }
  .title {
    flex: 1;
    font-size: 11px;
    color: var(--accent, #00d4ff);
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .close-btn {
    background: none;
    border: none;
    color: var(--text-dim, #4a4a6e);
    cursor: pointer;
    font-size: 13px;
    padding: 0 4px;
  }
  .close-btn:hover { color: var(--danger, #ff2244); }

  .status {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-dim);
    font-size: 12px;
  }

  .compare-area {
    flex: 1;
    position: relative;
    overflow: hidden;
    user-select: none;
  }
  .compare-area.dragging { cursor: col-resize; }

  .img-pane {
    position: absolute;
    top: 0;
    bottom: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .left-pane { left: 0; }
  .right-pane { border-left: none; }

  .pane-header {
    height: 22px;
    display: flex;
    align-items: center;
    padding: 0 8px;
    background: var(--bg-header, #0f1020);
    border-bottom: 1px solid var(--border-dim, #2a2a4e);
    flex-shrink: 0;
  }
  .pane-name {
    font-size: 10px;
    color: var(--text-dim);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .pane-body {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    padding: 8px;
    background: #0a0a14;
  }
  .pane-body img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    image-rendering: auto;
  }
  .err {
    color: var(--danger, #ff2244);
    font-size: 11px;
    text-align: center;
    padding: 8px;
  }

  .divider {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 6px;
    transform: translateX(-50%);
    background: var(--border-panel, #00d4ff);
    cursor: col-resize;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10;
    opacity: 0.7;
    transition: opacity 0.1s;
  }
  .divider:hover { opacity: 1; }
  .divider-handle {
    background: var(--bg-panel, #1a1a2e);
    color: var(--accent, #00d4ff);
    font-size: 10px;
    padding: 4px 2px;
    border: 1px solid var(--border-panel, #00d4ff);
    line-height: 1;
    pointer-events: none;
  }
</style>
