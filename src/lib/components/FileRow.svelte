<script lang="ts">
  import type { AtlasData } from '$lib/atlas';
  import PixelSprite from './PixelSprite.svelte';
  import type { FileEntry } from '$lib/stores/panelStore';
  import type { PluginInfo, ColumnCache } from '$lib/stores/pluginStore';
  import { createEventDispatcher } from 'svelte';

  export let entry: FileEntry;
  export let atlas: AtlasData;
  export let isCursor: boolean = false;
  export let isSelected: boolean = false;
  export let colorStyle: { fg: string; bg: string; bold: boolean; italic: boolean } | null = null;
  export let columnPlugins: PluginInfo[] = [];
  export let columnCache: ColumnCache = new Map();
  export let isDiffPick: boolean = false;

  $: ruleStyle = (!isSelected && !entry.is_dir && colorStyle)
    ? `color:${colorStyle.fg || 'inherit'};background:${colorStyle.bg || 'transparent'};font-weight:${colorStyle.bold ? 'bold' : 'normal'};font-style:${colorStyle.italic ? 'italic' : 'normal'};`
    : '';

  const dispatch = createEventDispatcher<{
    click: FileEntry;
    dblclick: FileEntry;
  }>();

  function iconName(e: FileEntry): string {
    if (e.name === '..') return 'icon_folder_up';
    if (e.is_symlink)    return 'icon_symlink';
    if (e.is_dir)        return 'icon_folder';
    switch (e.extension) {
      case 'txt': case 'md': case 'rst': case 'log': case 'csv':
        return 'icon_file_text';
      case 'ts': case 'js': case 'jsx': case 'tsx': case 'rs':
      case 'py': case 'go': case 'c': case 'cpp': case 'h':
      case 'java': case 'rb': case 'swift': case 'kt': case 'sh':
      case 'toml': case 'yaml': case 'yml': case 'json': case 'svelte':
      case 'vue': case 'html': case 'css': case 'scss':
        return 'icon_file_code';
      case 'png': case 'jpg': case 'jpeg': case 'gif': case 'svg':
      case 'webp': case 'bmp': case 'ico': case 'tiff':
        return 'icon_file_image';
      case 'zip': case 'tar': case 'gz': case 'bz2': case '7z':
      case 'rar': case 'xz': case 'zst': case 'dmg': case 'pkg':
        return 'icon_file_archive';
      case 'bin': case 'exe': case 'so': case 'dylib': case 'wasm':
      case 'o': case 'a': case 'out':
        return 'icon_file_binary';
      default:
        return 'icon_file';
    }
  }

  function formatSize(bytes: number, isDir: boolean, name: string): string {
    if (name === '..') return '';
    if (isDir) return '<DIR>';
    if (bytes < 1024)              return `${bytes}B`;
    if (bytes < 1024 * 1024)       return `${(bytes / 1024).toFixed(1)}K`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)}M`;
    return `${(bytes / 1024 / 1024 / 1024).toFixed(1)}G`;
  }

  function formatDate(ts: number): string {
    if (!ts) return '';
    const d = new Date(ts * 1000);
    const date = d.toLocaleDateString('en-CA');
    const time = d.toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' });
    return `${date} ${time}`;
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div
  class="file-row"
  class:cursor={isCursor}
  class:selected={isSelected}
  class:diff-pick={isDiffPick}
  class:is-dir={entry.is_dir && entry.name !== '..'}
  class:is-hidden={entry.is_hidden}
  class:is-up={entry.name === '..'}
  style={ruleStyle}
  on:click={() => dispatch('click', entry)}
  on:dblclick={() => dispatch('dblclick', entry)}
>
  <span class="icon">
    <PixelSprite {atlas} name={iconName(entry)} scale={1} />
  </span>
  <span class="name">{entry.name}</span>
  {#each columnPlugins as col}
    <span class="col-plugin" style="width: {col.columnWidth ?? 72}px">
      {columnCache.get(col.id)?.get(entry.path) ?? ''}
    </span>
  {/each}
  <span class="size">{formatSize(entry.size, entry.is_dir, entry.name)}</span>
  <span class="date">{formatDate(entry.modified)}</span>
</div>

<style>
  .file-row {
    display: flex;
    align-items: center;
    padding: 0 6px 0 2px;
    height: 34px;
    font-family: 'Courier New', monospace;
    font-size: 12px;
    color: var(--text);
    cursor: default;
    user-select: none;
    gap: 6px;
    flex-shrink: 0;
  }
  .file-row:hover { background: var(--bg-row-hover); }

  .file-row.cursor {
    background: var(--bg-row-cursor);
    outline: 1px solid var(--cursor-outline);
    outline-offset: -1px;
  }
  .file-row.selected {
    background: var(--bg-row-sel);
    color: var(--text-sel);
  }
  .file-row.cursor.selected {
    background: var(--bg-row-sel);
    outline-color: var(--sel-outline);
  }
  .file-row.diff-pick {
    outline: 1px dashed #ffaa00;
    outline-offset: -1px;
    background: rgba(255, 170, 0, 0.07);
  }
  .file-row.diff-pick .name::after {
    content: ' ◈';
    color: #ffaa00;
    font-size: 9px;
  }
  .file-row.is-dir  { color: var(--text-dir); }
  .file-row.is-up   { color: var(--text-dim); }
  .file-row.is-hidden { opacity: 0.55; }

  .icon {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .col-plugin {
    flex-shrink: 0;
    text-align: right;
    color: var(--border-panel);
    font-size: 10px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    opacity: 0.85;
  }
  .file-row.selected .col-plugin { color: var(--text-sel); opacity: 0.7; }
  .size {
    width: 64px;
    text-align: right;
    color: var(--text-dim);
    font-size: 10px;
    flex-shrink: 0;
  }
  .date {
    width: 118px;
    text-align: right;
    color: var(--text-dim);
    font-size: 10px;
    flex-shrink: 0;
  }

  .file-row.selected .size,
  .file-row.selected .date { color: var(--text-sel); opacity: 0.7; }
  .file-row.is-dir .size,
  .file-row.is-dir .date   { color: var(--text-dir); opacity: 0.7; }
</style>
