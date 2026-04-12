<script lang="ts">
  import { onMount } from 'svelte';
  import { loadAtlas, type AtlasData } from '$lib/atlas';
  import { leftPanel, rightPanel } from '$lib/stores/panelStore';
  import { uiStore } from '$lib/stores/uiStore';
  import { themeStore } from '$lib/stores/themeStore';
  import { transferQueue } from '$lib/stores/transferQueueStore';
  import Panel from './Panel.svelte';
  import FunctionBar from './FunctionBar.svelte';
  import CopyDialog from './CopyDialog.svelte';
  import DeleteDialog from './DeleteDialog.svelte';
  import MkdirDialog from './MkdirDialog.svelte';
  import RenameDialog from './RenameDialog.svelte';
  import MultiRenameDialog from './MultiRenameDialog.svelte';
  import ProcessExplorer from './ProcessExplorer.svelte';
  import FindDialog from './FindDialog.svelte';
  import GotoDialog from './GotoDialog.svelte';
  import TextViewer from './TextViewer.svelte';
  import HexViewer from './HexViewer.svelte';
  import DiffViewer from './DiffViewer.svelte';
  import ImageDiffViewer from './ImageDiffViewer.svelte';
  import SyncView from './SyncView.svelte';
  import ColorRulesDialog from './ColorRulesDialog.svelte';
  import TransferQueueDialog from './TransferQueueDialog.svelte';
  import MainMenu from './MainMenu.svelte';
  import CommandBar from './CommandBar.svelte';
  import QuickViewPane from './QuickViewPane.svelte';
  import HelpViewer from './HelpViewer.svelte';
  import PluginViewer from './PluginViewer.svelte';
  import PluginManager from './PluginManager.svelte';
  import type { MenuSection } from '$lib/stores/uiStore';
  import type { PanelState } from '$lib/stores/panelStore';
  import { pluginStore } from '$lib/stores/pluginStore';
  import { invoke } from '@tauri-apps/api/core';

  let atlas: AtlasData | null = null;
  let loadError = '';

  onMount(async () => {
    themeStore.applyTheme($themeStore);
    try {
      atlas = await loadAtlas();
    } catch (e) {
      loadError = `Failed to load atlas: ${e}`;
      return;
    }
    const home = (await invoke('get_home').catch(() => '/Users')) as string;
    await Promise.all([
      leftPanel.navigate(home),
      rightPanel.navigate(home),
      pluginStore.loadPlugins(),
    ]);
    pluginStore.initEventListener();
  });

  $: ui     = $uiStore;
  $: left   = $leftPanel as PanelState;
  $: right  = $rightPanel as PanelState;
  $: active = ui.activePanel === 'left' ? left : right;
  $: inactive = ui.activePanel === 'left' ? right : left;

  function activeStore() { return ui.activePanel === 'left' ? leftPanel : rightPanel; }
  function inactiveStore() { return ui.activePanel === 'left' ? rightPanel : leftPanel; }

  async function refreshBoth() {
    await Promise.all([leftPanel.reload(), rightPanel.reload()]);
  }

  // ── Archive detection ────────────────────────────────────────────────────────

  const ARCHIVE_EXTS = new Set([
    'zip','jar','war','ear','apk','ipa',
    'tar','tgz','tbz2','txz',
    'docx','xlsx','pptx','odt','ods',
  ]);

  function isArchiveFile(entry: { name: string; extension: string; is_dir: boolean; path: string }): boolean {
    if (entry.is_dir) return false;
    const ext = entry.extension.toLowerCase();
    if (ARCHIVE_EXTS.has(ext)) return true;
    // double extensions
    const n = entry.name;
    return n.endsWith('.tar.gz') || n.endsWith('.tar.bz2') || n.endsWith('.tar.xz');
  }

  // ── Text detection ───────────────────────────────────────────────────────────

  const TEXT_EXTENSIONS = new Set([
    'txt','md','mdx','rst','tex',
    'html','htm','css','scss','sass','less','svg',
    'js','jsx','ts','tsx','mjs','cjs','vue','svelte','astro',
    'py','rb','rs','go','java','kt','cs','fs','c','cpp','cc','h','hpp',
    'php','swift','dart','ex','exs','erl','lua','pl','r','jl','nim','sh',
    'bash','zsh','fish','ps1','bat',
    'json','json5','yaml','yml','toml','ini','cfg','conf','xml','plist',
    'env','properties','sql','csv','tsv','log','diff','patch',
    'tf','hcl','nix','lock','mod','sum','gitignore','gitattributes',
    'makefile','cmake','dockerfile','procfile','editorconfig',
  ]);

  const TEXT_NAMES = new Set([
    'Makefile','makefile','Dockerfile','dockerfile','Procfile',
    '.env','.gitignore','.gitattributes','.editorconfig',
    'README','CHANGELOG','LICENSE','LICENCE','AUTHORS','CONTRIBUTING',
    'Gemfile','Rakefile','Guardfile',
  ]);

  function guessMimeType(ext: string): string {
    const map: Record<string, string> = {
      png: 'image/png', jpg: 'image/jpeg', jpeg: 'image/jpeg',
      gif: 'image/gif', webp: 'image/webp', svg: 'image/svg+xml',
      bmp: 'image/bmp', ico: 'image/x-icon',
      pdf: 'application/pdf',
      mp4: 'video/mp4', webm: 'video/webm', mkv: 'video/x-matroska',
      mp3: 'audio/mpeg', ogg: 'audio/ogg', flac: 'audio/flac', wav: 'audio/wav',
      md: 'text/markdown', markdown: 'text/markdown',
    };
    return map[ext.toLowerCase()] ?? 'application/octet-stream';
  }

  function isTextFile(entry: { name: string; extension: string; is_dir: boolean }): boolean {
    if (entry.is_dir) return false;
    if (TEXT_EXTENSIONS.has(entry.extension.toLowerCase())) return true;
    if (TEXT_NAMES.has(entry.name)) return true;
    if (entry.extension === '' && entry.name.startsWith('.')) return true;
    return false;
  }

  // ── Diff pick ────────────────────────────────────────────────────────────────

  const IMAGE_DIFF_EXTS = new Set([
    'png','jpg','jpeg','gif','webp','bmp','ico','svg','tiff','tif','avif','heic',
  ]);

  function openDiff(a: string, b: string) {
    const extA = a.split('.').pop()?.toLowerCase() ?? '';
    const extB = b.split('.').pop()?.toLowerCase() ?? '';
    if (IMAGE_DIFF_EXTS.has(extA) && IMAGE_DIFF_EXTS.has(extB)) {
      uiStore.setDialog({ kind: 'image-diff', left: a, right: b });
    } else {
      uiStore.setDialog({ kind: 'diff', left: a, right: b });
    }
  }

  // ── Selection helpers ────────────────────────────────────────────────────────

  function getSrcs(): string[] {
    if (active.selected.size > 0) return [...active.selected];
    const entry = active.entries[active.cursor];
    if (entry && entry.name !== '..') return [entry.path];
    return [];
  }

  // ── Keyboard ─────────────────────────────────────────────────────────────────

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      // Cancel any active diff pick first
      if (ui.diffPick) { uiStore.setDiffPick(null); return; }
      if (ui.dialog.kind === 'none') {
        uiStore.setDialog({ kind: 'menu', section: 'favorites' });
      } else if (ui.dialog.kind === 'menu') {
        // MainMenu handles sub-state Escape internally via stopPropagation;
        // if the event reached here the menu itself should close.
        uiStore.closeDialog();
      }
      // Help and all other dialogs handle their own Escape internally.
      return;
    }

    // History navigation: Alt+Left / Alt+Right
    if (e.altKey && !e.metaKey && !e.ctrlKey) {
      if (e.key === 'ArrowLeft') {
        e.preventDefault();
        activeStore().back();
        return;
      }
      if (e.key === 'ArrowRight') {
        e.preventDefault();
        activeStore().forward();
        return;
      }
    }

    // Global shortcuts
    if ((e.metaKey || e.ctrlKey) && !e.shiftKey) {
      if (e.key === 'h') {
        e.preventDefault();
        if (ui.dialog.kind === 'none') {
          const entry = active.entries[active.cursor];
          if (entry && !entry.is_dir) uiStore.setDialog({ kind: 'hex', path: entry.path });
        }
        return;
      }
      if (e.key === 'f') {
        e.preventDefault();
        if (ui.dialog.kind === 'none')
          uiStore.setDialog({ kind: 'find', startPath: active.path });
        return;
      }
      if (e.key === 'g') {
        e.preventDefault();
        if (ui.dialog.kind === 'none')
          uiStore.setDialog({ kind: 'goto' });
        return;
      }
      if (e.key === 't') {
        e.preventDefault();
        if (ui.dialog.kind === 'none')
          activeStore().newTab();
        return;
      }
      if (e.key === 'w') {
        e.preventDefault();
        if (ui.dialog.kind === 'none')
          activeStore().closeTab();
        return;
      }
      if (e.key === 'q') {
        e.preventDefault();
        uiStore.toggleQuickView();
        return;
      }
      if (e.key === '`') {
        e.preventDefault();
        uiStore.toggleCmdBar();
        return;
      }
      if (e.key === '=') {
        e.preventDefault();
        inactiveStore().navigate(active.path);
        return;
      }
      if (e.key === 'm') {
        e.preventDefault();
        if (ui.dialog.kind === 'none') {
          const srcs = getSrcs().filter(p => {
            const en = active.entries.find(x => x.path === p);
            return en && !en.is_dir;
          });
          if (srcs.length > 0) uiStore.setDialog({ kind: 'multi-rename', paths: srcs });
        }
        return;
      }
      // Ctrl+1..9: switch tab
      const num = parseInt(e.key);
      if (!isNaN(num) && num >= 1 && num <= 9) {
        e.preventDefault();
        activeStore().switchTab(num - 1);
        return;
      }
    }

    if (ui.dialog.kind !== 'none') return;

    // Action plugin keybindings (Ctrl+Shift+<key>)
    if (e.ctrlKey && e.shiftKey && !e.metaKey) {
      const chord = `ctrl+shift+${e.key.toLowerCase()}`;
      const actionPlugin = $pluginStore.plugins.find(
        p => p.category === 'action' && p.keybinding === chord,
      );
      if (actionPlugin) {
        e.preventDefault();
        const files = getSrcs();
        if (files.length > 0) {
          pluginStore.runAction(actionPlugin.id, files, inactive.path).catch(console.error);
        }
        return;
      }
    }


    switch (e.key) {
      case 'ArrowUp':
        e.preventDefault();
        activeStore().moveCursor(-1);
        break;
      case 'ArrowDown':
        e.preventDefault();
        activeStore().moveCursor(1);
        break;
      case 'PageUp':
        e.preventDefault();
        activeStore().moveCursorPage(-1);
        break;
      case 'PageDown':
        e.preventDefault();
        activeStore().moveCursorPage(1);
        break;
      case 'Tab':
        e.preventDefault();
        uiStore.switchPanel();
        break;
      case 'Enter': {
        e.preventDefault();
        const entry = active.entries[active.cursor];
        if (!entry) break;

        // Archive navigation
        if (active.archiveRoot !== null) {
          if (entry.is_dir) {
            if (entry.name === '..') {
              if (active.archiveInner === '') {
                // Exit archive
                const parts = (active.archiveRoot).split('/').filter(Boolean);
                const parent = parts.length > 0 ? '/' + parts.slice(0, -1).join('/') || '/' : '/';
                activeStore().navigate(parent);
              } else {
                activeStore().navigateArchive(active.archiveRoot, entry.path);
              }
            } else {
              activeStore().navigateArchive(active.archiveRoot, entry.path);
            }
          }
          // Files inside archive: Cmd+Enter = extract+open
          else if (e.metaKey || e.ctrlKey) {
            uiStore.setDialog({
              kind: 'extract',
              archivePath: active.archiveRoot,
              dstDir: inactive.path,
            });
          }
          break;
        }

        // Normal filesystem
        if (entry.is_dir) {
          // When going up (..) restore cursor to the child we came from
          const cursorHint = entry.name === '..'
            ? active.path.split('/').filter(Boolean).pop()
            : undefined;
          activeStore().navigate(entry.path, cursorHint);
        } else if (e.metaKey || e.ctrlKey) {
          invoke('open_file', { path: entry.path }).catch(console.error);
        } else if (isArchiveFile(entry)) {
          activeStore().navigateArchive(entry.path, '');
        } else if (isTextFile(entry)) {
          uiStore.setDialog({ kind: 'view', path: entry.path });
        } else {
          const viewer = pluginStore.viewerForExtension(entry.extension);
          if (viewer) {
            uiStore.setDialog({
              kind: 'plugin-viewer',
              path: entry.path,
              mimeType: guessMimeType(entry.extension),
              pluginId: viewer.id,
            });
          } else {
            invoke('open_file', { path: entry.path }).catch(console.error);
          }
        }
        break;
      }
      case 'Backspace': {
        e.preventDefault();
        if (active.archiveRoot !== null) {
          if (active.archiveInner === '') {
            const parts = (active.archiveRoot).split('/').filter(Boolean);
            const parent = parts.length > 0 ? '/' + parts.slice(0, -1).join('/') || '/' : '/';
            // Restore cursor to the archive file we came from
            const archiveName = parts[parts.length - 1] ?? '';
            activeStore().navigate(parent, archiveName);
          } else {
            const idx = active.archiveInner.lastIndexOf('/');
            const parent = idx > 0 ? active.archiveInner.slice(0, idx) : '';
            activeStore().navigateArchive(active.archiveRoot, parent);
          }
        } else {
          const parts = active.path.split('/').filter(Boolean);
          const parent = parts.length > 0 ? '/' + parts.slice(0, -1).join('/') || '/' : '/';
          // Restore cursor to the dir we came from
          const childName = parts[parts.length - 1] ?? '';
          activeStore().navigate(parent, childName);
        }
        break;
      }
      case ' ': {
        e.preventDefault();
        const entry = active.entries[active.cursor];
        if (entry && entry.name !== '..') {
          activeStore().toggleSelection(entry.path);
          activeStore().moveCursor(1);
        }
        break;
      }
      case 'a':
        if (e.ctrlKey || e.metaKey) {
          e.preventDefault();
          activeStore().selectAll();
        }
        break;
      case 'F1':
        e.preventDefault();
        uiStore.setDialog({ kind: 'help' });
        break;
      case 'F2': {
        e.preventDefault();
        const entry = active.entries[active.cursor];
        if (entry && entry.name !== '..') {
          uiStore.setDialog({ kind: 'rename', src: entry.path, currentName: entry.name });
        }
        break;
      }
      case 'F3':
        e.preventDefault();
        uiStore.setDialog({ kind: 'processes' });
        break;
      case 'F4': {
        e.preventDefault();
        const entry = active.entries[active.cursor];
        if (entry && !entry.is_dir) {
          invoke('open_file', { path: entry.path }).catch(console.error);
        }
        break;
      }
      case 'd': {
        if (e.metaKey || e.ctrlKey || e.altKey) break;
        e.preventDefault();
        uiStore.setDialog({ kind: 'sync', left: left.path, right: right.path });
        break;
      }
      case 'D': {
        if (e.metaKey || e.ctrlKey || e.altKey) break;
        // Shift+D = diff pick flow
        e.preventDefault();
        const dEntry = active.entries[active.cursor];
        if (!dEntry || dEntry.is_dir) break;
        const pick = ui.diffPick;
        if (!pick) {
          uiStore.setDiffPick(dEntry.path);
        } else if (pick === dEntry.path) {
          uiStore.setDiffPick(null);
        } else {
          uiStore.setDiffPick(null);
          openDiff(pick, dEntry.path);
        }
        break;
      }
      case 'F5': {
        e.preventDefault();
        const srcs = getSrcs();
        if (active.archiveRoot !== null && srcs.length > 0) {
          // Extract from archive — run in background via transfer queue
          invoke('extract_from_archive', {
            archivePath: active.archiveRoot,
            innerPaths: srcs,
            dstDir: inactive.path,
          }).then(() => refreshBoth()).catch(console.error);
        } else if (srcs.length > 0) {
          uiStore.setDialog({ kind: 'copy', srcs, dstDir: inactive.path });
        }
        break;
      }
      case 'F6': {
        e.preventDefault();
        const srcs = getSrcs();
        if (srcs.length > 0 && active.archiveRoot === null)
          uiStore.setDialog({ kind: 'move', srcs, dstDir: inactive.path });
        break;
      }
      case 'F7':
        e.preventDefault();
        uiStore.setDialog({ kind: 'mkdir', parentPath: active.path });
        break;
      case 'F8': {
        e.preventDefault();
        const paths = getSrcs();
        if (paths.length > 0 && active.archiveRoot === null)
          uiStore.setDialog({ kind: 'delete', paths });
        break;
      }
      case 'F9':
        e.preventDefault();
        themeStore.cycle();
        break;
      case 'F10':
        e.preventDefault();
        window.close();
        break;
    }
  }

  function handleFnAction(action: string) {
    if (action === 'f9') { themeStore.cycle(); return; }
    if (action === 'cmd') { uiStore.toggleCmdBar(); return; }
    if (action === 'quickview') { uiStore.toggleQuickView(); return; }
    if (action === 'transfers') { uiStore.setDialog({ kind: 'transfers' }); return; }
    if (action === 'f1') { uiStore.setDialog({ kind: 'help' }); return; }
    const keyMap: Record<string, string> = {
      f2: 'F2', f3: 'F3', f4: 'F4', f5: 'F5',
      f6: 'F6', f7: 'F7', f8: 'F8', f10: 'F10',
    };
    const key = keyMap[action];
    if (key) handleKeydown(new KeyboardEvent('keydown', { key, bubbles: false }));
  }

  async function handleDialogDone() {
    uiStore.closeDialog();
    await refreshBoth();
  }

  // Quick view: entry currently under cursor in active panel
  $: quickViewEntry = ui.quickViewVisible
    ? (active.entries[active.cursor] ?? null)
    : null;
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="app-shell">
  {#if loadError}
    <div class="load-error">{loadError}</div>
  {:else if !atlas}
    <div class="loading">Nyx.Commander — loading assets…</div>
  {:else}
    <div class="panels">
      <Panel
        store={leftPanel}
        {atlas}
        active={ui.activePanel === 'left'}
        diffPick={ui.diffPick}
        on:activate={() => uiStore.setActivePanel('left')}
      />
      <div class="divider"></div>
      <Panel
        store={rightPanel}
        {atlas}
        active={ui.activePanel === 'right'}
        diffPick={ui.diffPick}
        on:activate={() => uiStore.setActivePanel('right')}
      />
      {#if ui.quickViewVisible}
        <QuickViewPane entry={quickViewEntry} />
      {/if}
    </div>

    {#if ui.cmdBarVisible}
      <CommandBar cwd={active.path} on:close={() => uiStore.toggleCmdBar()} />
    {/if}

    <FunctionBar
      activeDialog={ui.dialog.kind !== 'none'}
      cmdBarVisible={ui.cmdBarVisible}
      quickViewVisible={ui.quickViewVisible}
      on:action={e => handleFnAction(e.detail)}
    />

    <!-- Dialogs -->
    {#if ui.dialog.kind === 'copy'}
      <CopyDialog
        srcs={ui.dialog.srcs}
        dstDir={ui.dialog.dstDir}
        move={false}
        on:done={handleDialogDone}
        on:cancel={() => uiStore.closeDialog()}
      />
    {:else if ui.dialog.kind === 'move'}
      <CopyDialog
        srcs={ui.dialog.srcs}
        dstDir={ui.dialog.dstDir}
        move={true}
        on:done={handleDialogDone}
        on:cancel={() => uiStore.closeDialog()}
      />
    {:else if ui.dialog.kind === 'delete'}
      <DeleteDialog
        paths={ui.dialog.paths}
        on:done={handleDialogDone}
        on:cancel={() => uiStore.closeDialog()}
      />
    {:else if ui.dialog.kind === 'mkdir'}
      <MkdirDialog
        parentPath={ui.dialog.parentPath}
        on:done={handleDialogDone}
        on:cancel={() => uiStore.closeDialog()}
      />
    {:else if ui.dialog.kind === 'rename'}
      <RenameDialog
        src={ui.dialog.src}
        currentName={ui.dialog.currentName}
        on:done={handleDialogDone}
        on:cancel={() => uiStore.closeDialog()}
      />
    {:else if ui.dialog.kind === 'multi-rename'}
      <MultiRenameDialog
        paths={ui.dialog.paths}
        on:done={handleDialogDone}
        on:cancel={() => uiStore.closeDialog()}
      />
    {:else if ui.dialog.kind === 'processes'}
      <ProcessExplorer on:close={() => uiStore.closeDialog()} />
    {:else if ui.dialog.kind === 'hex'}
      <HexViewer
        path={ui.dialog.path}
        on:close={() => uiStore.closeDialog()}
      />
    {:else if ui.dialog.kind === 'view'}
      <TextViewer
        path={ui.dialog.path}
        on:close={() => uiStore.closeDialog()}
      />
    {:else if ui.dialog.kind === 'diff'}
      <DiffViewer
        left={ui.dialog.left}
        right={ui.dialog.right}
        on:close={() => uiStore.closeDialog()}
      />
    {:else if ui.dialog.kind === 'image-diff'}
      <ImageDiffViewer
        left={ui.dialog.left}
        right={ui.dialog.right}
        on:close={() => uiStore.closeDialog()}
      />
    {:else if ui.dialog.kind === 'sync'}
      <SyncView
        left={ui.dialog.left}
        right={ui.dialog.right}
        on:close={() => uiStore.closeDialog()}
      />
    {:else if ui.dialog.kind === 'color-rules'}
      <ColorRulesDialog on:close={() => uiStore.closeDialog()} />
    {:else if ui.dialog.kind === 'transfers'}
      <TransferQueueDialog on:close={() => uiStore.closeDialog()} />
    {:else if ui.dialog.kind === 'find'}
      <FindDialog
        startPath={ui.dialog.startPath}
        on:navigate={e => { uiStore.closeDialog(); activeStore().navigate(e.detail.path); }}
        on:close={() => uiStore.closeDialog()}
      />
    {:else if ui.dialog.kind === 'goto'}
      <GotoDialog
        on:navigate={e => { uiStore.closeDialog(); activeStore().navigate(e.detail.path); }}
        on:close={() => uiStore.closeDialog()}
      />
    {:else if ui.dialog.kind === 'help'}
      <HelpViewer on:close={() => uiStore.closeDialog()} />
    {:else if ui.dialog.kind === 'plugin-viewer'}
      {@const pvd = ui.dialog}
      {@const pvPlugin = $pluginStore.plugins.find(p => p.id === pvd.pluginId)}
      {#if pvPlugin}
        <PluginViewer
          plugin={pvPlugin}
          filePath={pvd.path}
          mimeType={pvd.mimeType}
          on:close={() => uiStore.closeDialog()}
        />
      {/if}
    {:else if ui.dialog.kind === 'plugin-manager'}
      <PluginManager on:close={() => uiStore.closeDialog()} />
    {:else if ui.dialog.kind === 'menu'}
      <MainMenu
        section={ui.dialog.section}
        activePath={active.path}
        on:close={() => uiStore.closeDialog()}
        on:navigate={e => { uiStore.closeDialog(); activeStore().navigate(e.detail.path); }}
        on:set-section={e => uiStore.setDialog({ kind: 'menu', section: e.detail.section })}
      />
    {/if}
  {/if}
</div>

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    background: var(--bg);
    overflow: hidden;
  }

  .panels {
    display: flex;
    flex: 1;
    overflow: hidden;
    gap: 0;
    min-height: 0;
  }

  .divider {
    width: 2px;
    background: var(--border-dim);
    flex-shrink: 0;
  }

  .loading,
  .load-error {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    color: var(--border);
    font-family: 'Courier New', monospace;
    font-size: 13px;
    background: var(--bg);
  }
  .load-error { color: var(--danger, #ff2244); }
</style>
