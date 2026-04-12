<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { FileEntry } from '$lib/stores/panelStore';

  export let entry: FileEntry | null;

  const IMAGE_EXTS = new Set(['png','jpg','jpeg','gif','svg','webp','bmp','ico','tiff','heic']);
  const TEXT_EXTS = new Set([
    'txt','md','mdx','rst','html','htm','css','scss','js','jsx','ts','tsx',
    'json','yaml','yml','toml','ini','cfg','xml','sh','bash','py','rb','rs',
    'go','java','kt','swift','sql','csv','log','diff','patch','env',
    'gitignore','gitattributes','editorconfig','makefile','dockerfile',
  ]);
  const TEXT_NAMES = new Set([
    'Makefile','makefile','Dockerfile','dockerfile','Procfile',
    '.env','.gitignore','.gitattributes','.editorconfig',
    'README','CHANGELOG','LICENSE','AUTHORS',
  ]);

  const IMAGE_MAX = 10 * 1024 * 1024; // 10 MB cap

  const MIME: Record<string, string> = {
    png: 'image/png', jpg: 'image/jpeg', jpeg: 'image/jpeg',
    gif: 'image/gif', webp: 'image/webp', svg: 'image/svg+xml',
    bmp: 'image/bmp', ico: 'image/x-icon', tiff: 'image/tiff',
    tif: 'image/tiff', heic: 'image/heic', avif: 'image/avif',
  };

  type PreviewKind = 'empty' | 'dir' | 'image' | 'text' | 'binary';

  let prevEntry: FileEntry | null = null;
  let imageSrc = '';
  let content = '';
  let kind: PreviewKind = 'empty';
  let loading = false;
  let error = '';

  $: if (entry?.path !== prevEntry?.path) {
    prevEntry = entry;
    loadPreview(entry);
  }

  async function loadPreview(e: FileEntry | null) {
    imageSrc = '';
    content = '';
    error = '';
    if (!e) { kind = 'empty'; return; }
    if (e.is_dir) { kind = 'dir'; return; }

    const ext = e.extension.toLowerCase();

    if (IMAGE_EXTS.has(ext)) {
      if (e.size > IMAGE_MAX) {
        kind = 'binary'; // too large to preview
        return;
      }
      loading = true;
      kind = 'image';
      try {
        const chunk = await invoke<{ data: number[]; file_size: number; offset: number }>(
          'read_file_bytes', { path: e.path, offset: 0, count: e.size }
        );
        const bytes = new Uint8Array(chunk.data);
        // Build base64 in chunks to avoid stack overflow on large arrays
        let binary = '';
        for (let i = 0; i < bytes.length; i++) binary += String.fromCharCode(bytes[i]);
        imageSrc = `data:${MIME[ext] ?? 'image/png'};base64,${btoa(binary)}`;
      } catch (err) {
        error = String(err);
        kind = 'binary';
      } finally {
        loading = false;
      }
      return;
    }

    const isText = TEXT_EXTS.has(ext)
      || TEXT_NAMES.has(e.name)
      || (e.extension === '' && e.name.startsWith('.'));

    if (isText && e.size < 256 * 1024) {
      loading = true;
      try {
        content = await invoke<string>('read_text_file', { path: e.path });
        kind = 'text';
      } catch (err) {
        error = String(err);
        kind = 'binary';
      } finally {
        loading = false;
      }
    } else {
      kind = 'binary';
    }
  }

  function fmtSize(n: number): string {
    if (n < 1024) return `${n} B`;
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
    if (n < 1024 * 1024 * 1024) return `${(n / 1048576).toFixed(1)} MB`;
    return `${(n / 1073741824).toFixed(2)} GB`;
  }
</script>

<div class="quick-view">
  {#if !entry}
    <div class="empty">No selection</div>
  {:else if loading}
    <div class="empty">Loading…</div>
  {:else if kind === 'dir'}
    <div class="meta">
      <div class="meta-icon">📁</div>
      <div class="meta-name">{entry.name}</div>
      <div class="meta-kind">Directory</div>
    </div>
  {:else if kind === 'image'}
    <div class="image-view">
      {#if imageSrc}
        <img src={imageSrc} alt={entry.name} />
      {:else}
        <div class="empty">Loading…</div>
      {/if}
      <div class="filename">{entry.name}</div>
    </div>
  {:else if kind === 'text'}
    <pre class="text-content">{content}</pre>
  {:else}
    <div class="meta">
      <div class="meta-name">{entry.name}</div>
      <div class="meta-kind">{entry.extension ? `.${entry.extension}` : 'binary'} · {fmtSize(entry.size)}</div>
      {#if error}<div class="err">{error}</div>{/if}
    </div>
  {/if}
</div>

<style>
  .quick-view {
    width: 240px;
    flex-shrink: 0;
    border-left: 1px solid var(--border-dim);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg);
    font-family: 'Courier New', monospace;
    font-size: 11px;
  }

  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    color: var(--text-dim);
    font-size: 11px;
  }

  .meta {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: 6px;
    padding: 12px;
    text-align: center;
  }
  .meta-icon { font-size: 32px; }
  .meta-name { color: var(--text); word-break: break-all; }
  .meta-kind { color: var(--text-dim); font-size: 10px; }
  .err { color: var(--danger); font-size: 10px; }

  .image-view {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    flex: 1;
    overflow: hidden;
    padding: 8px;
  }
  .image-view img {
    max-width: 100%;
    max-height: calc(100% - 30px);
    object-fit: contain;
    image-rendering: auto;
  }
  .filename {
    color: var(--text-dim);
    font-size: 10px;
    text-align: center;
    word-break: break-all;
  }

  .text-content {
    flex: 1;
    overflow: auto;
    padding: 8px;
    color: var(--text);
    font-size: 10px;
    white-space: pre-wrap;
    word-break: break-all;
    margin: 0;
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--bg);
  }
</style>
