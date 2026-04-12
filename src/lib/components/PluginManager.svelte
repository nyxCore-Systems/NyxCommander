<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { pluginStore, type PluginInfo } from '$lib/stores/pluginStore';
  import { invoke } from '@tauri-apps/api/core';

  const dispatch = createEventDispatcher<{ close: void }>();

  $: plugins = $pluginStore.plugins;

  const CATEGORY_ORDER: PluginInfo['category'][] = ['viewer', 'column', 'action', 'panel'];
  const CATEGORY_LABELS: Record<string, string> = {
    viewer: 'Viewer',
    column: 'Column',
    action: 'Action',
    panel: 'Panel',
  };

  $: byCategory = CATEGORY_ORDER
    .map(cat => ({
      cat,
      label: CATEGORY_LABELS[cat],
      items: plugins.filter(p => p.category === cat),
    }))
    .filter(g => g.items.length > 0);

  let installError = '';
  let installSuccess = '';
  let installing = false;

  async function togglePlugin(p: PluginInfo) {
    try {
      await pluginStore.setPluginEnabled(p.id, !p.enabled);
    } catch (e) {
      console.error('[plugins] toggle failed:', e);
    }
  }

  async function handleInstallDrop(e: DragEvent) {
    e.preventDefault();
    installError = '';
    installSuccess = '';
    const file = e.dataTransfer?.files[0];
    if (!file) return;
    if (!file.name.endsWith('.nyx-plugin')) {
      installError = 'Only .nyx-plugin files are accepted';
      return;
    }
    // Tauri exposes the real fs path on the File object
    const path = (file as File & { path?: string }).path;
    if (!path) {
      installError = 'Could not read file path — try copying to ~/.nyx/plugins/ manually';
      return;
    }
    installing = true;
    try {
      await pluginStore.installPlugin(path);
      installSuccess = `Installed: ${file.name}`;
    } catch (err) {
      installError = String(err);
    } finally {
      installing = false;
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
  }

  async function openPluginsDir() {
    const home = (await invoke('get_home').catch(() => '')) as string;
    invoke('open_file', { path: `${home}/.nyx/plugins` }).catch(console.error);
  }
</script>

<div class="plugin-manager">
  <div class="section-header">
    <span class="section-title">Installed Plugins</span>
    <button class="open-dir-btn" on:click={openPluginsDir}>Open plugins folder</button>
  </div>

  <div class="plugin-list">
    {#if byCategory.length === 0}
      <div class="empty-state">
        No plugins installed.<br />
        Drop a <code>.nyx-plugin</code> file below, or copy a folder into <code>~/.nyx/plugins/</code>.
      </div>
    {:else}
      {#each byCategory as group}
        <div class="category-group">
          <div class="category-label">{group.label} Plugins</div>
          {#each group.items as plugin (plugin.id)}
            <div class="plugin-row">
              <!-- Enable/disable toggle -->
              <label class="toggle" title="{plugin.enabled ? 'Disable' : 'Enable'} plugin">
                <input
                  type="checkbox"
                  checked={plugin.enabled}
                  on:change={() => togglePlugin(plugin)}
                />
                <span class="toggle-track"></span>
              </label>

              <div class="plugin-info">
                <div class="plugin-name-row">
                  <span class="plugin-name">{plugin.name}</span>
                  <span class="plugin-version">v{plugin.version}</span>
                  {#if plugin.author}<span class="plugin-author">by {plugin.author}</span>{/if}
                  {#if plugin.keybinding}
                    <span class="kbd-badge">{plugin.keybinding}</span>
                  {/if}
                  {#if plugin.columnName}
                    <span class="meta-badge">col: {plugin.columnName}</span>
                  {/if}
                  {#if plugin.protocol}
                    <span class="meta-badge">proto: {plugin.protocol}</span>
                  {/if}
                </div>
                {#if plugin.description}
                  <div class="plugin-desc">{plugin.description}</div>
                {/if}
                {#if plugin.fileExtensions.length > 0}
                  <div class="plugin-exts">
                    {plugin.fileExtensions.map(e => `.${e}`).join('  ')}
                  </div>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/each}
    {/if}
  </div>

  <!-- Drag-and-drop install zone -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
    class="drop-zone"
    class:installing
    on:dragover={handleDragOver}
    on:drop={handleInstallDrop}
  >
    {#if installing}
      Installing…
    {:else}
      Drop a <code>.nyx-plugin</code> file here to install
    {/if}
  </div>

  {#if installError}
    <div class="msg error">{installError}</div>
  {/if}
  {#if installSuccess}
    <div class="msg success">{installSuccess}</div>
  {/if}

  <div class="help-text">
    Plugins live in <code>~/.nyx/plugins/&lt;id&gt;/</code>. Changes are picked up automatically — no restart needed.
  </div>
</div>

<style>
  .plugin-manager {
    display: flex;
    flex-direction: column;
    height: 100%;
    font-family: 'Courier New', monospace;
    font-size: 12px;
    color: var(--text);
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-dim);
    flex-shrink: 0;
  }

  .section-title {
    font-size: 11px;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .open-dir-btn {
    background: transparent;
    border: 1px solid var(--border-dim);
    color: var(--text-dim);
    font-family: inherit;
    font-size: 10px;
    padding: 2px 8px;
    cursor: pointer;
  }
  .open-dir-btn:hover {
    border-color: var(--border-panel);
    color: var(--border-panel);
  }

  .plugin-list {
    flex: 1;
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--bg);
  }

  .category-group {
    border-bottom: 1px solid var(--border-dim);
    padding-bottom: 4px;
  }

  .category-label {
    padding: 6px 12px 3px;
    font-size: 10px;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .plugin-row {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 6px 12px;
  }
  .plugin-row:hover { background: var(--bg-row-hover); }

  .plugin-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .plugin-name-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .plugin-name { color: var(--text); font-size: 12px; }
  .plugin-version { color: var(--text-dim); font-size: 10px; }
  .plugin-author { color: var(--text-dim); font-size: 10px; }

  .plugin-desc {
    color: var(--text-dim);
    font-size: 10px;
    line-height: 1.4;
  }

  .plugin-exts {
    color: var(--border-panel);
    font-size: 10px;
    opacity: 0.8;
  }

  .kbd-badge {
    background: var(--bg-fnkey);
    border: 1px solid var(--border-dim);
    color: var(--text-fn-label, var(--text-dim));
    font-size: 9px;
    padding: 1px 5px;
    flex-shrink: 0;
  }

  .meta-badge {
    color: var(--border-panel);
    font-size: 9px;
    border: 1px solid var(--border-panel);
    padding: 1px 4px;
    opacity: 0.7;
    flex-shrink: 0;
  }

  /* Toggle switch */
  .toggle {
    position: relative;
    display: inline-flex;
    cursor: pointer;
    flex-shrink: 0;
    margin-top: 2px;
  }
  .toggle input { opacity: 0; width: 0; height: 0; position: absolute; }
  .toggle-track {
    width: 28px;
    height: 14px;
    background: var(--border-dim);
    display: block;
    transition: background 0.15s;
    flex-shrink: 0;
  }
  .toggle input:checked + .toggle-track { background: var(--border-panel); }

  .drop-zone {
    margin: 10px 12px;
    padding: 16px;
    text-align: center;
    border: 1px dashed var(--border-dim);
    color: var(--text-dim);
    font-size: 11px;
    cursor: default;
    flex-shrink: 0;
    transition: border-color 0.15s, color 0.15s;
  }
  .drop-zone:hover, .drop-zone.installing {
    border-color: var(--border-panel);
    color: var(--border-panel);
  }
  .drop-zone code {
    background: none;
    color: inherit;
    font-family: inherit;
  }

  .msg {
    padding: 4px 12px;
    font-size: 11px;
    flex-shrink: 0;
  }
  .msg.error { color: var(--text-danger); }
  .msg.success { color: var(--border-panel); }

  .help-text {
    padding: 6px 12px 10px;
    font-size: 10px;
    color: var(--text-dim);
    line-height: 1.5;
    flex-shrink: 0;
    border-top: 1px solid var(--border-dim);
  }
  .help-text code {
    color: var(--text);
    background: none;
    font-family: inherit;
  }

  .empty-state {
    padding: 24px 16px;
    text-align: center;
    color: var(--text-dim);
    font-size: 11px;
    line-height: 1.7;
  }
  .empty-state code {
    color: var(--border-panel);
    background: none;
    font-family: inherit;
  }
</style>
