<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let path: string;

  const dispatch = createEventDispatcher<{ close: void }>();

  interface FileModeResult {
    mode: number;
    size: number;
    modified: number;
  }

  let loading = true;
  let error = '';
  let modeRaw: number | null = null;
  let octalInput = '';
  let size: number | null = null;
  let modified: number | null = null;
  let modeError = '';

  let checksum: string | null = null;
  let checksumLoading = false;
  let checksumError = '';

  let dialogEl: HTMLElement;

  const name = path.split('/').at(-1) ?? path;

  onMount(async () => {
    dialogEl?.focus();
    try {
      const result = await invoke<FileModeResult>('get_file_mode', { path });
      modeRaw = result.mode;
      octalInput = (result.mode & 0o777).toString(8).padStart(3, '0');
      size = result.size;
      modified = result.modified;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  function formatMode(mode: number): string {
    const bits = mode & 0o777;
    const chars = 'rwxrwxrwx';
    return chars.split('').map((c, i) => (bits >> (8 - i)) & 1 ? c : '-').join('');
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  function formatDate(ts: number): string {
    return new Date(ts * 1000).toLocaleString();
  }

  async function applyMode() {
    modeError = '';
    const parsed = parseInt(octalInput, 8);
    if (isNaN(parsed) || octalInput.length < 3) {
      modeError = 'Invalid octal value';
      return;
    }
    try {
      await invoke('set_file_mode', { path, mode: parsed });
      modeRaw = parsed;
    } catch (e) {
      modeError = String(e);
    }
  }

  async function computeChecksum() {
    checksumLoading = true;
    checksumError = '';
    checksum = null;
    try {
      checksum = await invoke<string>('compute_checksum', { path });
    } catch (e) {
      checksumError = String(e);
    } finally {
      checksumLoading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') { e.stopPropagation(); dispatch('close'); }
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="overlay" on:click|self={() => dispatch('close')}>
  <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
  <div class="dialog" role="dialog" tabindex="-1" bind:this={dialogEl} on:keydown={handleKeydown}>
    <div class="dialog-title">PROPERTIES</div>
    <div class="dialog-body">
      {#if loading}
        <div class="loading">Loading…</div>
      {:else if error}
        <div class="dialog-error">{error}</div>
      {:else}
        <div class="prop-grid">
          <div class="prop-label">Name</div>
          <div class="prop-value">{name}</div>

          <div class="prop-label">Path</div>
          <div class="prop-value path-val">{path}</div>

          {#if size !== null}
            <div class="prop-label">Size</div>
            <div class="prop-value">{formatSize(size)} <span class="dim">({size.toLocaleString()} bytes)</span></div>
          {/if}

          {#if modified !== null}
            <div class="prop-label">Modified</div>
            <div class="prop-value">{formatDate(modified)}</div>
          {/if}

          {#if modeRaw !== null}
            <div class="prop-label">Permissions</div>
            <div class="prop-value">
              <span class="mono">{formatMode(modeRaw)}</span>
              <span class="dim"> ({(modeRaw & 0o777).toString(8).padStart(3, '0')})</span>
            </div>

            <div class="prop-label">Set mode</div>
            <div class="prop-value mode-row">
              <input
                class="mode-input"
                bind:value={octalInput}
                maxlength="4"
                on:keydown={(e) => { if (e.key === 'Enter') applyMode(); }}
              />
              <button class="inline-btn" on:click={applyMode}>Apply</button>
              {#if modeError}<span class="err">{modeError}</span>{/if}
            </div>
          {/if}
        </div>

        <div class="checksum-section">
          {#if checksum}
            <div class="prop-label">SHA-256</div>
            <div class="checksum-value">{checksum}</div>
          {:else if checksumLoading}
            <div class="loading">Computing checksum…</div>
          {:else if checksumError}
            <div class="dialog-error">{checksumError}</div>
          {/if}
        </div>
      {/if}
    </div>
    <div class="dialog-footer">
      {#if !loading && !error && !checksumLoading && !checksum}
        <button on:click={computeChecksum}>Compute checksum</button>
      {/if}
      <button on:click={() => dispatch('close')}>Close (Esc)</button>
    </div>
  </div>
</div>

<style>
  .loading {
    font-size: 11px;
    color: var(--text-dim);
    animation: pulse 1s ease-in-out infinite alternate;
  }
  @keyframes pulse {
    from { opacity: 0.4; }
    to   { opacity: 1; }
  }
  .prop-grid {
    display: grid;
    grid-template-columns: 100px 1fr;
    gap: 6px 12px;
    align-items: baseline;
  }
  .prop-label {
    font-size: 10px;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    white-space: nowrap;
  }
  .prop-value {
    font-size: 11px;
    color: var(--text);
    word-break: break-all;
  }
  .path-val {
    color: var(--border);
  }
  .dim {
    color: var(--text-dim);
  }
  .mono {
    font-family: 'Courier New', monospace;
  }
  .mode-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .mode-input {
    background: var(--bg);
    color: var(--text);
    border: 1px solid var(--text-dim);
    padding: 3px 6px;
    width: 60px;
    font-family: 'Courier New', monospace;
    font-size: 12px;
  }
  .mode-input:focus {
    border-color: var(--border);
    outline: none;
  }
  .inline-btn {
    background: var(--bg-fnkey);
    color: var(--text-dim);
    border: 1px solid var(--border-dim);
    padding: 2px 10px;
    cursor: pointer;
    font-family: 'Courier New', monospace;
    font-size: 10px;
  }
  .inline-btn:hover {
    border-color: var(--border);
    color: var(--border);
  }
  .err {
    font-size: 10px;
    color: var(--text-danger);
  }
  .checksum-section {
    margin-top: 10px;
  }
  .checksum-value {
    font-size: 10px;
    color: var(--text);
    word-break: break-all;
    margin-top: 4px;
    padding: 4px;
    background: var(--bg);
    border: 1px solid var(--border-dim);
  }
</style>
