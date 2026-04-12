import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  is_symlink: boolean;
  size: number;
  modified: number;
  extension: string;
  is_hidden: boolean;
}

// ─── Tab / History ─────────────────────────────────────────────────────────

export interface HistoryItem {
  path: string;
  archiveRoot: string | null;
  archiveInner: string;
}

export interface TabSnapshot {
  id: string;
  path: string;
  archiveRoot: string | null;
  archiveInner: string;
  cursor: number;
  selected: string[];
  history: HistoryItem[];
  historyIdx: number;
  sortBy: 'name' | 'size' | 'date' | 'ext';
  sortDir: 'asc' | 'desc';
}

// ─── Panel State ────────────────────────────────────────────────────────────

export interface PanelState {
  // Navigation
  path: string;
  archiveRoot: string | null;   // null = real filesystem
  archiveInner: string;         // internal path within archive ("" = root)
  // Entries
  entries: FileEntry[];
  cursor: number;
  selected: Set<string>;
  // Display
  filter: string;
  flatView: boolean;
  showHidden: boolean;
  // Status
  loading: boolean;
  error: string | null;
  // Tabs
  tabs: TabSnapshot[];
  activeTabIdx: number;
  // History (mirrored from active tab)
  history: HistoryItem[];
  historyIdx: number;
  // Sorting
  sortBy: 'name' | 'size' | 'date' | 'ext';
  sortDir: 'asc' | 'desc';
}

// ─── Store Factory ───────────────────────────────────────────────────────────

function uuid(): string {
  return Math.random().toString(36).slice(2, 10);
}

function makeTab(path: string): TabSnapshot {
  return {
    id: uuid(),
    path,
    archiveRoot: null,
    archiveInner: '',
    cursor: 0,
    selected: [],
    history: [{ path, archiveRoot: null, archiveInner: '' }],
    historyIdx: 0,
    sortBy: 'name',
    sortDir: 'asc',
  };
}

function createPanelStore(initialPath: string) {
  const initialTab = makeTab(initialPath);

  const initial: PanelState = {
    path: initialPath,
    archiveRoot: null,
    archiveInner: '',
    entries: [],
    cursor: 0,
    selected: new Set(),
    filter: '',
    flatView: false,
    showHidden: false,
    loading: false,
    error: null,
    tabs: [initialTab],
    activeTabIdx: 0,
    history: initialTab.history,
    historyIdx: 0,
    sortBy: 'name',
    sortDir: 'asc',
  };

  const { subscribe, update, set } = writable<PanelState>(initial);

  // ── Internal helpers ──────────────────────────────────────────────────────

  function snapshotActiveTab(s: PanelState): TabSnapshot {
    return {
      ...s.tabs[s.activeTabIdx],
      path: s.path,
      archiveRoot: s.archiveRoot,
      archiveInner: s.archiveInner,
      cursor: s.cursor,
      selected: [...s.selected],
      history: s.history,
      historyIdx: s.historyIdx,
      sortBy: s.sortBy,
      sortDir: s.sortDir,
    };
  }

  function sortEntries(entries: FileEntry[], by: PanelState['sortBy'], dir: PanelState['sortDir']): FileEntry[] {
    const up = entries.find(e => e.name === '..');
    const rest = entries.filter(e => e.name !== '..');
    const dirs = rest.filter(e => e.is_dir);
    const files = rest.filter(e => !e.is_dir);

    function cmp(a: FileEntry, b: FileEntry): number {
      let v = 0;
      switch (by) {
        case 'size': v = a.size - b.size; break;
        case 'date': v = a.modified - b.modified; break;
        case 'ext':  v = a.extension.localeCompare(b.extension); break;
        default:     v = a.name.localeCompare(b.name, undefined, { sensitivity: 'base' });
      }
      return dir === 'asc' ? v : -v;
    }

    dirs.sort(cmp);
    files.sort(cmp);
    return up ? [up, ...dirs, ...files] : [...dirs, ...files];
  }

  async function loadEntries(s: PanelState, cursorHint?: string): Promise<Partial<PanelState>> {
    try {
      let entries: FileEntry[];
      if (s.archiveRoot !== null) {
        entries = await invoke<FileEntry[]>('list_archive_dir', {
          archivePath: s.archiveRoot,
          innerPath: s.archiveInner,
        });
      } else if (s.flatView) {
        entries = await invoke<FileEntry[]>('list_dir_flat', {
          root: s.path,
          maxResults: 3000,
        });
      } else {
        entries = await invoke<FileEntry[]>('list_dir', { path: s.path });
      }

      // Apply showHidden filter
      if (!s.showHidden) {
        entries = entries.filter(e => e.name === '..' || !e.is_hidden);
      }

      entries = sortEntries(entries, s.sortBy, s.sortDir);

      // Restore cursor to a named entry if requested (e.g. after Backspace)
      let cursor = Math.min(s.cursor, Math.max(0, entries.length - 1));
      if (cursorHint) {
        const idx = entries.findIndex(e => e.name === cursorHint);
        if (idx >= 0) cursor = idx;
      }

      return {
        entries,
        cursor,
        selected: new Set(),
        loading: false,
        error: null,
      };
    } catch (e) {
      return { entries: [], loading: false, error: String(e) };
    }
  }

  // ── Navigation ────────────────────────────────────────────────────────────

  async function navigate(path: string, cursorHint?: string) {
    let newState: PanelState;
    update(s => {
      const hist = s.history.slice(0, s.historyIdx + 1);
      hist.push({ path, archiveRoot: null, archiveInner: '' });
      newState = {
        ...s,
        path,
        archiveRoot: null,
        archiveInner: '',
        cursor: 0,
        selected: new Set(),
        filter: '',
        flatView: false,
        loading: true,
        history: hist,
        historyIdx: hist.length - 1,
      };
      // snapshot into tab
      const tabs = [...s.tabs];
      tabs[s.activeTabIdx] = snapshotActiveTab({ ...newState, history: hist, historyIdx: hist.length - 1 });
      return { ...newState, tabs };
    });
    const patch = await loadEntries(newState!, cursorHint);
    update(s => {
      const tabs = [...s.tabs];
      tabs[s.activeTabIdx] = { ...tabs[s.activeTabIdx], ...patch, selected: [...(patch.selected ?? new Set())] };
      return { ...s, ...patch, tabs };
    });
  }

  async function navigateArchive(archiveRoot: string, archiveInner: string) {
    let newState: PanelState;
    update(s => {
      const hist = s.history.slice(0, s.historyIdx + 1);
      hist.push({ path: s.path, archiveRoot, archiveInner });
      newState = {
        ...s,
        archiveRoot,
        archiveInner,
        cursor: 0,
        selected: new Set(),
        filter: '',
        loading: true,
        history: hist,
        historyIdx: hist.length - 1,
      };
      const tabs = [...s.tabs];
      tabs[s.activeTabIdx] = snapshotActiveTab(newState);
      return { ...newState, tabs };
    });
    const patch = await loadEntries(newState!);
    update(s => {
      const tabs = [...s.tabs];
      const { selected: _sel, ...tabPatch } = patch;
      tabs[s.activeTabIdx] = { ...tabs[s.activeTabIdx], ...tabPatch };
      return { ...s, ...patch, tabs };
    });
  }

  async function back() {
    let newState: PanelState | null = null;
    update(s => {
      if (s.historyIdx <= 0) return s;
      const idx = s.historyIdx - 1;
      const item = s.history[idx];
      newState = {
        ...s,
        path: item.path,
        archiveRoot: item.archiveRoot,
        archiveInner: item.archiveInner,
        historyIdx: idx,
        cursor: 0,
        selected: new Set(),
        filter: '',
        loading: true,
      };
      const tabs = [...s.tabs];
      tabs[s.activeTabIdx] = snapshotActiveTab(newState);
      return { ...newState, tabs };
    });
    if (newState) {
      const patch = await loadEntries(newState);
      update(s => {
        const tabs = [...s.tabs];
        tabs[s.activeTabIdx] = { ...tabs[s.activeTabIdx], ...patch, selected: [...(patch.selected ?? new Set())] };
        return { ...s, ...patch, tabs };
      });
    }
  }

  async function forward() {
    let newState: PanelState | null = null;
    update(s => {
      if (s.historyIdx >= s.history.length - 1) return s;
      const idx = s.historyIdx + 1;
      const item = s.history[idx];
      newState = {
        ...s,
        path: item.path,
        archiveRoot: item.archiveRoot,
        archiveInner: item.archiveInner,
        historyIdx: idx,
        cursor: 0,
        selected: new Set(),
        filter: '',
        loading: true,
      };
      const tabs = [...s.tabs];
      tabs[s.activeTabIdx] = snapshotActiveTab(newState);
      return { ...newState, tabs };
    });
    if (newState) {
      const patch = await loadEntries(newState);
      update(s => {
        const tabs = [...s.tabs];
        tabs[s.activeTabIdx] = { ...tabs[s.activeTabIdx], ...patch, selected: [...(patch.selected ?? new Set())] };
        return { ...s, ...patch, tabs };
      });
    }
  }

  async function reload() {
    let snap: PanelState;
    update(s => { snap = { ...s, loading: true }; return snap; });
    const patch = await loadEntries(snap!);
    update(s => ({ ...s, ...patch }));
  }

  // ── Tabs ──────────────────────────────────────────────────────────────────

  async function newTab(path?: string) {
    let targetPath: string;
    let newState: PanelState;
    update(s => {
      const snapped = snapshotActiveTab(s);
      const tabs = [...s.tabs];
      tabs[s.activeTabIdx] = snapped;
      targetPath = path ?? s.path;
      const tab = makeTab(targetPath);
      tabs.splice(s.activeTabIdx + 1, 0, tab);
      const newIdx = s.activeTabIdx + 1;
      newState = {
        ...s,
        tabs,
        activeTabIdx: newIdx,
        path: targetPath,
        archiveRoot: null,
        archiveInner: '',
        cursor: 0,
        selected: new Set(),
        filter: '',
        flatView: false,
        loading: true,
        history: tab.history,
        historyIdx: 0,
      };
      return newState;
    });
    const patch = await loadEntries(newState!);
    update(s => {
      const tabs = [...s.tabs];
      const { selected: _sel, ...tabPatch } = patch;
      tabs[s.activeTabIdx] = { ...tabs[s.activeTabIdx], ...tabPatch };
      return { ...s, ...patch, tabs };
    });
  }

  async function closeTab(idx?: number) {
    let newState: PanelState;
    update(s => {
      if (s.tabs.length <= 1) return s;
      const closeIdx = idx ?? s.activeTabIdx;
      const tabs = s.tabs.filter((_, i) => i !== closeIdx);
      const newIdx = Math.min(closeIdx, tabs.length - 1);
      const tab = tabs[newIdx];
      newState = {
        ...s,
        tabs,
        activeTabIdx: newIdx,
        path: tab.path,
        archiveRoot: tab.archiveRoot,
        archiveInner: tab.archiveInner,
        cursor: tab.cursor,
        selected: new Set(tab.selected),
        history: tab.history,
        historyIdx: tab.historyIdx,
        sortBy: tab.sortBy ?? 'name',
        sortDir: tab.sortDir ?? 'asc',
        filter: '',
        loading: true,
      };
      return newState;
    });
    const patch = await loadEntries(newState!);
    update(s => ({ ...s, ...patch }));
  }

  async function switchTab(idx: number) {
    let newState: PanelState;
    update(s => {
      if (idx === s.activeTabIdx || idx < 0 || idx >= s.tabs.length) return s;
      const tabs = [...s.tabs];
      tabs[s.activeTabIdx] = snapshotActiveTab(s);
      const tab = tabs[idx];
      newState = {
        ...s,
        tabs,
        activeTabIdx: idx,
        path: tab.path,
        archiveRoot: tab.archiveRoot,
        archiveInner: tab.archiveInner,
        cursor: tab.cursor,
        selected: new Set(tab.selected),
        history: tab.history,
        historyIdx: tab.historyIdx,
        sortBy: tab.sortBy ?? 'name',
        sortDir: tab.sortDir ?? 'asc',
        filter: '',
        loading: true,
      };
      return newState;
    });
    const patch = await loadEntries(newState!);
    update(s => ({ ...s, ...patch }));
  }

  // ── Cursor / Selection ────────────────────────────────────────────────────

  function moveCursor(delta: number) {
    update(s => {
      const next = Math.max(0, Math.min(s.entries.length - 1, s.cursor + delta));
      return { ...s, cursor: next };
    });
  }

  function moveCursorPage(delta: number, pageSize = 20) {
    update(s => {
      const next = Math.max(0, Math.min(s.entries.length - 1, s.cursor + delta * pageSize));
      return { ...s, cursor: next };
    });
  }

  function setCursor(index: number) {
    update(s => ({
      ...s,
      cursor: Math.max(0, Math.min(s.entries.length - 1, index)),
    }));
  }

  function setCursorToName(name: string) {
    update(s => {
      const idx = s.entries.findIndex(e => e.name === name);
      return idx >= 0 ? { ...s, cursor: idx } : s;
    });
  }

  function toggleSelection(path: string) {
    update(s => {
      const next = new Set(s.selected);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return { ...s, selected: next };
    });
  }

  function selectAll() {
    update(s => {
      const next = new Set(s.entries.filter(e => e.name !== '..').map(e => e.path));
      return { ...s, selected: next };
    });
  }

  function clearSelection() {
    update(s => ({ ...s, selected: new Set() }));
  }

  // ── Display ───────────────────────────────────────────────────────────────

  function setFilter(text: string) {
    update(s => ({ ...s, filter: text }));
  }

  async function toggleFlatView() {
    let newState: PanelState;
    update(s => {
      newState = { ...s, flatView: !s.flatView, loading: true, filter: '' };
      return newState;
    });
    const patch = await loadEntries(newState!);
    update(s => ({ ...s, ...patch }));
  }

  async function toggleShowHidden() {
    let newState: PanelState;
    update(s => {
      newState = { ...s, showHidden: !s.showHidden, loading: true };
      return newState;
    });
    const patch = await loadEntries(newState!);
    update(s => ({ ...s, ...patch }));
  }

  async function setSort(by: PanelState['sortBy'], dir: PanelState['sortDir']) {
    let newState: PanelState;
    update(s => {
      newState = { ...s, sortBy: by, sortDir: dir, loading: true };
      const tabs = [...s.tabs];
      tabs[s.activeTabIdx] = { ...tabs[s.activeTabIdx], sortBy: by, sortDir: dir };
      return { ...newState, tabs };
    });
    const patch = await loadEntries(newState!);
    update(s => ({ ...s, ...patch }));
  }

  function selectByPattern(pattern: string) {
    update(s => {
      const re = new RegExp(
        '^' + pattern.replace(/[.+^${}()|[\]\\]/g, '\\$&').replace(/\*/g, '.*').replace(/\?/g, '.') + '$',
        'i'
      );
      const next = new Set(s.entries.filter(e => e.name !== '..' && re.test(e.name)).map(e => e.path));
      return { ...s, selected: next };
    });
  }

  function invertSelection() {
    update(s => {
      const all = new Set(s.entries.filter(e => e.name !== '..').map(e => e.path));
      const next = new Set([...all].filter(p => !s.selected.has(p)));
      return { ...s, selected: next };
    });
  }

  return {
    subscribe,
    navigate,
    navigateArchive,
    back,
    forward,
    reload,
    newTab,
    closeTab,
    switchTab,
    moveCursor,
    moveCursorPage,
    setCursor,
    setCursorToName,
    toggleSelection,
    selectAll,
    clearSelection,
    setFilter,
    toggleFlatView,
    toggleShowHidden,
    setSort,
    selectByPattern,
    invertSelection,
  };
}

export const leftPanel  = createPanelStore('/');
export const rightPanel = createPanelStore('/');

export type PanelStore = ReturnType<typeof createPanelStore>;
