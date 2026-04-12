import { writable } from 'svelte/store';

export type ActivePanel = 'left' | 'right';

export type MenuSection =
  | 'favorites' | 'remotes' | 'themes' | 'shortcuts' | 'settings' | 'plugins';

export type DialogState =
  | { kind: 'none' }
  | { kind: 'help'; section?: number }
  | { kind: 'copy';         srcs: string[]; dstDir: string }
  | { kind: 'move';         srcs: string[]; dstDir: string }
  | { kind: 'delete';       paths: string[] }
  | { kind: 'mkdir';        parentPath: string }
  | { kind: 'rename';       src: string; currentName: string }
  | { kind: 'multi-rename'; paths: string[] }
  | { kind: 'processes' }
  | { kind: 'find';         startPath: string }
  | { kind: 'view';         path: string }
  | { kind: 'hex';          path: string }
  | { kind: 'diff';         left: string; right: string }
  | { kind: 'image-diff';  left: string; right: string }
  | { kind: 'sync';         left: string; right: string }
  | { kind: 'color-rules' }
  | { kind: 'transfers' }
  | { kind: 'plugin-viewer'; path: string; mimeType: string; pluginId: string }
  | { kind: 'plugin-manager' }
  | { kind: 'menu';         section: MenuSection }
  | { kind: 'goto';         startPath?: string };

interface UiState {
  activePanel: ActivePanel;
  dialog: DialogState;
  cmdBarVisible: boolean;
  quickViewVisible: boolean;
  diffPick: string | null;   // path of first file picked for diff
}

function createUiStore() {
  const { subscribe, update } = writable<UiState>({
    activePanel: 'left',
    dialog: { kind: 'none' },
    cmdBarVisible: false,
    quickViewVisible: false,
    diffPick: null,
  });

  function switchPanel() {
    update(s => ({
      ...s,
      activePanel: s.activePanel === 'left' ? 'right' : 'left',
    }));
  }

  function setActivePanel(panel: ActivePanel) {
    update(s => ({ ...s, activePanel: panel }));
  }

  function setDialog(dialog: DialogState) {
    update(s => ({ ...s, dialog, diffPick: null }));
  }

  function closeDialog() {
    update(s => ({ ...s, dialog: { kind: 'none' } }));
  }

  function toggleCmdBar() {
    update(s => ({ ...s, cmdBarVisible: !s.cmdBarVisible }));
  }

  function toggleQuickView() {
    update(s => ({ ...s, quickViewVisible: !s.quickViewVisible }));
  }

  function setDiffPick(path: string | null) {
    update(s => ({ ...s, diffPick: path }));
  }

  return {
    subscribe,
    switchPanel,
    setActivePanel,
    setDialog,
    closeDialog,
    toggleCmdBar,
    toggleQuickView,
    setDiffPick,
  };
}

export const uiStore = createUiStore();
