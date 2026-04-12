<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import type { PluginInfo } from '$lib/stores/pluginStore';

  export let plugin: PluginInfo;
  export let filePath: string;
  export let mimeType: string = 'application/octet-stream';

  const dispatch = createEventDispatcher<{ close: void }>();

  let iframeEl: HTMLIFrameElement;
  let statusText = '';
  let errorText = '';

  $: viewerSrc = convertFileSrc(plugin.entrypointPath);
  $: fileName = filePath.split('/').pop() ?? filePath;

  function sendLoad() {
    if (!iframeEl?.contentWindow) return;
    const styles = getComputedStyle(document.documentElement);
    iframeEl.contentWindow.postMessage(
      {
        type: 'load',
        filePath,
        fileUrl: convertFileSrc(filePath),
        mimeType,
        theme: {
          bg: styles.getPropertyValue('--bg').trim(),
          fg: styles.getPropertyValue('--text').trim(),
          accent: styles.getPropertyValue('--border-panel').trim(),
        },
      },
      '*',
    );
  }

  function handleMessage(e: MessageEvent) {
    if (e.source !== iframeEl?.contentWindow) return;
    const msg = e.data as { type: string; text?: string; message?: string };
    if (msg?.type === 'status') {
      statusText = msg.text ?? '';
    } else if (msg?.type === 'error') {
      errorText = msg.message ?? 'Unknown error';
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') dispatch('close');
  }

  onMount(() => window.addEventListener('message', handleMessage));
  onDestroy(() => window.removeEventListener('message', handleMessage));
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="overlay">
  <div class="viewer-shell">
    <div class="title-bar">
      <span class="plugin-badge">{plugin.name}</span>
      <span class="filename">{fileName}</span>
      {#if statusText}<span class="status-text">{statusText}</span>{/if}
      {#if errorText}<span class="error-text">{errorText}</span>{/if}
      <button class="close-btn" on:click={() => dispatch('close')} title="Close (Esc)">✕</button>
    </div>
    <iframe
      bind:this={iframeEl}
      src={viewerSrc}
      on:load={sendLoad}
      title="Plugin viewer: {fileName}"
      sandbox="allow-scripts"
      class="viewer-frame"
    ></iframe>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.9);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .viewer-shell {
    display: flex;
    flex-direction: column;
    width: 92vw;
    height: 90vh;
    background: var(--bg);
    border: 1px solid var(--border-panel);
    box-shadow: 0 0 40px rgba(0, 0, 0, 0.8);
  }

  .title-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 10px;
    height: 26px;
    flex-shrink: 0;
    background: var(--bg-header);
    border-bottom: 1px solid var(--border-panel);
    font-family: 'Courier New', monospace;
    font-size: 11px;
  }

  .plugin-badge {
    color: var(--border-panel);
    font-weight: bold;
    flex-shrink: 0;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .filename {
    color: var(--text);
    flex-shrink: 0;
  }

  .status-text {
    color: var(--text-dim);
    font-size: 10px;
    margin-left: auto;
    flex-shrink: 0;
  }

  .error-text {
    color: var(--text-danger);
    font-size: 10px;
    margin-left: auto;
    flex-shrink: 0;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-dim);
    cursor: pointer;
    font-size: 13px;
    padding: 0 2px;
    margin-left: auto;
    line-height: 1;
    flex-shrink: 0;
  }
  .close-btn:hover { color: var(--text-danger); }

  .viewer-frame {
    flex: 1;
    border: none;
    width: 100%;
    background: var(--bg);
  }
</style>
