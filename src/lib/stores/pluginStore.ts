import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { FileEntry } from './panelStore';

export interface PluginInfo {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  category: 'viewer' | 'column' | 'action' | 'panel';
  fileExtensions: string[];
  entrypointPath: string;
  columnName: string | null;
  columnWidth: number | null;
  keybinding: string | null;
  menuLabel: string | null;
  protocol: string | null;
  enabled: boolean;
}

export interface ColumnValue {
  path: string;
  value: string;
}

// pluginId → (filePath → display value)
export type ColumnCache = Map<string, Map<string, string>>;

interface PluginStoreState {
  plugins: PluginInfo[];
  columnCache: ColumnCache;
  columnStatus: Map<string, 'loading' | 'error' | 'ready'>;
  lastActionResult: string | null;
}

function createPluginStore() {
  const { subscribe, update } = writable<PluginStoreState>({
    plugins: [],
    columnCache: new Map(),
    columnStatus: new Map(),
    lastActionResult: null,
  });

  async function loadPlugins() {
    try {
      const plugins = await invoke<PluginInfo[]>('list_plugins');
      update(s => ({ ...s, plugins }));
    } catch (e) {
      console.error('[plugins] loadPlugins failed:', e);
    }
  }

  function viewerForExtension(ext: string): PluginInfo | null {
    const { plugins } = get({ subscribe });
    return (
      plugins.find(
        p =>
          p.category === 'viewer' &&
          p.fileExtensions.includes(ext.toLowerCase()),
      ) ?? null
    );
  }

  function columnPlugins(): PluginInfo[] {
    return get({ subscribe }).plugins.filter(p => p.category === 'column');
  }

  function actionPlugins(): PluginInfo[] {
    return get({ subscribe }).plugins.filter(p => p.category === 'action');
  }

  function panelPlugins(): PluginInfo[] {
    return get({ subscribe }).plugins.filter(p => p.category === 'panel');
  }

  async function fetchColumnValues(pluginId: string, files: FileEntry[]) {
    update(s => {
      const status = new Map(s.columnStatus);
      status.set(pluginId, 'loading');
      return { ...s, columnStatus: status };
    });

    try {
      const results = await invoke<ColumnValue[]>('get_column_values', {
        pluginId,
        files: files.map(f => ({
          path: f.path,
          name: f.name,
          is_dir: f.is_dir,
          size: f.size,
          modified: f.modified,
          extension: f.extension,
        })),
      });

      update(s => {
        const cache = new Map(s.columnCache);
        const pluginMap = new Map(cache.get(pluginId) ?? []);
        for (const r of results) pluginMap.set(r.path, r.value);
        cache.set(pluginId, pluginMap);
        const status = new Map(s.columnStatus);
        status.set(pluginId, 'ready');
        return { ...s, columnCache: cache, columnStatus: status };
      });
    } catch (e) {
      console.error(`[plugins] fetchColumnValues(${pluginId}) failed:`, e);
      update(s => {
        const status = new Map(s.columnStatus);
        status.set(pluginId, 'error');
        return { ...s, columnStatus: status };
      });
    }
  }

  function invalidateColumnCache() {
    update(s => ({ ...s, columnCache: new Map(), columnStatus: new Map() }));
  }

  function getColumnValue(pluginId: string, filePath: string): string {
    return get({ subscribe }).columnCache.get(pluginId)?.get(filePath) ?? '';
  }

  async function runAction(
    pluginId: string,
    files: string[],
    targetDir: string,
  ): Promise<string> {
    const msg = await invoke<string>('run_action_plugin', {
      pluginId,
      files,
      targetDir,
    });
    update(s => ({ ...s, lastActionResult: msg }));
    return msg;
  }

  async function setPluginEnabled(pluginId: string, enabled: boolean) {
    await invoke('set_plugin_enabled', { pluginId, enabled });
    await loadPlugins();
  }

  async function installPlugin(zipPath: string) {
    await invoke('install_plugin', { zipPath });
    await loadPlugins();
  }

  async function initEventListener() {
    await listen('plugins-changed', async () => {
      await loadPlugins();
    });
  }

  return {
    subscribe,
    loadPlugins,
    viewerForExtension,
    columnPlugins,
    actionPlugins,
    panelPlugins,
    fetchColumnValues,
    invalidateColumnCache,
    getColumnValue,
    runAction,
    setPluginEnabled,
    installPlugin,
    initEventListener,
  };
}

export const pluginStore = createPluginStore();
