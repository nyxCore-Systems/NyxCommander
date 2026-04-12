<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { favoritesStore, type Favorite } from '$lib/stores/favoritesStore';
  import { remotesStore, type RemoteConn, type RemoteType, DEFAULT_PORTS } from '$lib/stores/remotesStore';
  import {
    THEMES, THEME_VAR_GROUPS,
    customThemes, saveCustomTheme, deleteCustomTheme,
    themeStore, type Theme,
  } from '$lib/stores/themeStore';
  import { invoke } from '@tauri-apps/api/core';
  import type { MenuSection } from '$lib/stores/uiStore';
  import PluginManager from './PluginManager.svelte';

  export let section: MenuSection = 'favorites';
  export let activePath: string = '/';

  const dispatch = createEventDispatcher<{
    close: void;
    navigate: { path: string };
    'set-section': { section: MenuSection };
  }>();

  $: favs        = $favoritesStore;
  $: remotes     = $remotesStore;
  $: builtinThemes = THEMES;
  $: custom      = $customThemes;
  $: currentTheme = $themeStore;

  // ─── Navigation state ────────────────────────────────────────────────────────
  let localSection: MenuSection = section;
  // 'sidebar' = arrow keys move between sections; 'content' = arrow keys move within section
  let menuFocus: 'sidebar' | 'content' = 'sidebar';
  let contentCursor = 0;
  let overlayEl: HTMLElement;

  const SECTIONS: Array<{ id: MenuSection; label: string }> = [
    { id: 'favorites', label: 'Favorites' },
    { id: 'remotes',   label: 'Remotes'   },
    { id: 'themes',    label: 'Themes'    },
    { id: 'shortcuts', label: 'Shortcuts' },
    { id: 'plugins',   label: 'Plugins'   },
    { id: 'settings',  label: 'Settings'  },
  ];

  $: sidebarIdx = SECTIONS.findIndex(s => s.id === localSection);

  onMount(() => overlayEl?.focus());

  function setSection(s: MenuSection) {
    localSection = s;
    contentCursor = 0;
    menuFocus = 'sidebar';
    dispatch('set-section', { section: s });
  }

  function contentLength(): number {
    switch (localSection) {
      case 'favorites': return favs.length;
      case 'remotes':   return remotes.length;
      case 'themes':    return builtinThemes.length + custom.length;
      default:          return 0;
    }
  }

  function activateContent() {
    switch (localSection) {
      case 'favorites': {
        const f = favs[contentCursor];
        if (f) navToFav(f);
        break;
      }
      case 'remotes': {
        const r = remotes[contentCursor];
        if (r) connectRemote(r);
        break;
      }
      case 'themes': {
        const all = [...builtinThemes, ...custom];
        const t = all[contentCursor];
        if (t) applyBuiltin(t);
        break;
      }
    }
  }

  // ─── Favorites ───────────────────────────────────────────────────────────────
  let editingFavId: string | null = null;
  let editingFavName = '';

  function startEditFav(f: Favorite) {
    editingFavId = f.id;
    editingFavName = f.name;
  }

  function saveFav() {
    if (editingFavId && editingFavName.trim()) {
      favoritesStore.rename(editingFavId, editingFavName.trim());
    }
    editingFavId = null;
  }

  function navToFav(f: Favorite) {
    dispatch('navigate', { path: f.path });
  }

  // ─── Remotes ─────────────────────────────────────────────────────────────────
  type RemoteForm = Omit<RemoteConn, 'id'>;
  const emptyForm = (): RemoteForm => ({
    name: '', type: 'sftp', host: '', port: 22, user: '',
    password: '', keyPath: '', remotePath: '/',
  });

  let showRemoteForm = false;
  let editingRemoteId: string | null = null;
  let remoteForm: RemoteForm = emptyForm();
  let remoteError = '';

  function newRemote() {
    editingRemoteId = null;
    remoteForm = emptyForm();
    remoteError = '';
    showRemoteForm = true;
  }

  function editRemote(r: RemoteConn) {
    editingRemoteId = r.id;
    remoteForm = { name: r.name, type: r.type, host: r.host, port: r.port,
                   user: r.user, password: r.password ?? '', keyPath: r.keyPath ?? '',
                   remotePath: r.remotePath };
    remoteError = '';
    showRemoteForm = true;
  }

  function saveRemote() {
    if (!remoteForm.host.trim()) { remoteError = 'Host is required'; return; }
    if (!remoteForm.name.trim()) remoteForm.name = remoteForm.host;
    if (editingRemoteId) {
      remotesStore.edit(editingRemoteId, remoteForm);
    } else {
      remotesStore.add(remoteForm);
    }
    showRemoteForm = false;
    editingRemoteId = null;
  }

  function cancelRemote() { showRemoteForm = false; editingRemoteId = null; }

  function onTypeChange() {
    remoteForm.port = DEFAULT_PORTS[remoteForm.type];
  }

  async function connectRemote(r: RemoteConn) {
    // Use system-level open for mounting / opening terminal
    let url = '';
    if (r.type === 'ssh') {
      url = `ssh://${r.user ? r.user + '@' : ''}${r.host}${r.port !== 22 ? ':' + r.port : ''}`;
    } else if (r.type === 'sftp') {
      url = `sftp://${r.user ? r.user + '@' : ''}${r.host}${r.port !== 22 ? ':' + r.port : ''}${r.remotePath}`;
    } else if (r.type === 'ftp') {
      url = `ftp://${r.user ? r.user + '@' : ''}${r.host}${r.port !== 21 ? ':' + r.port : ''}${r.remotePath}`;
    } else if (r.type === 'cifs') {
      url = `smb://${r.user ? r.user + '@' : ''}${r.host}${r.remotePath}`;
    }
    if (url) {
      await invoke('open_file', { path: url }).catch(console.error);
    }
  }

  // ─── Themes ──────────────────────────────────────────────────────────────────
  let editingTheme: Theme | null = null;
  let themeEditorVars: Record<string, string> = {};
  let themeEditorLabel = '';
  let isNewTheme = false;

  function applyBuiltin(t: Theme) {
    themeStore.setTheme(t.name);
  }

  function startNewTheme() {
    // Clone current theme as base
    isNewTheme = true;
    themeEditorLabel = 'My Theme';
    themeEditorVars = { ...currentTheme.vars };
    editingTheme = { name: 'custom:' + Date.now(), label: themeEditorLabel, vars: themeEditorVars };
  }

  function startEditTheme(t: Theme) {
    isNewTheme = false;
    editingTheme = t;
    themeEditorLabel = t.label;
    themeEditorVars = { ...t.vars };
  }

  function cancelThemeEdit() {
    editingTheme = null;
    themeEditorVars = {};
  }

  function saveTheme() {
    if (!editingTheme) return;
    const saved: Theme = {
      name: editingTheme.name,
      label: themeEditorLabel.trim() || editingTheme.name,
      vars: { ...themeEditorVars },
    };
    saveCustomTheme(saved);
    themeStore.setTheme(saved.name);
    editingTheme = null;
  }

  function removeCustomTheme(name: string) {
    deleteCustomTheme(name);
    if (currentTheme.name === name) themeStore.setTheme('neon');
  }

  // Live-preview as the user changes vars
  function previewVar(cssVar: string, value: string) {
    document.documentElement.style.setProperty(cssVar, value);
  }

  // ─── Keyboard handler ─────────────────────────────────────────────────────────
  function handleKey(e: KeyboardEvent) {
    // Sub-state Escape: absorb so App.svelte doesn't close the whole menu
    if (e.key === 'Escape') {
      if (editingTheme)  { e.stopPropagation(); cancelThemeEdit(); return; }
      if (showRemoteForm){ e.stopPropagation(); cancelRemote();    return; }
      // No sub-state: let bubble — App.svelte window handler closes the dialog
      return;
    }

    // Sidebar + content navigation (absorb to prevent App.svelte panel movement)
    if (e.key === 'ArrowUp') {
      e.preventDefault(); e.stopPropagation();
      if (menuFocus === 'content' && contentLength() > 0) {
        contentCursor = Math.max(0, contentCursor - 1);
      } else {
        const next = Math.max(0, sidebarIdx - 1);
        if (next !== sidebarIdx) setSection(SECTIONS[next].id);
      }
      return;
    }

    if (e.key === 'ArrowDown') {
      e.preventDefault(); e.stopPropagation();
      if (menuFocus === 'content' && contentLength() > 0) {
        contentCursor = Math.min(contentLength() - 1, contentCursor + 1);
      } else {
        const next = Math.min(SECTIONS.length - 1, sidebarIdx + 1);
        if (next !== sidebarIdx) setSection(SECTIONS[next].id);
      }
      return;
    }

    if (e.key === 'ArrowRight') {
      e.preventDefault(); e.stopPropagation();
      if (menuFocus === 'sidebar' && contentLength() > 0) {
        menuFocus = 'content';
        contentCursor = 0;
      }
      return;
    }

    if (e.key === 'ArrowLeft') {
      e.preventDefault(); e.stopPropagation();
      menuFocus = 'sidebar';
      return;
    }

    if (e.key === 'Enter') {
      e.preventDefault(); e.stopPropagation();
      if (menuFocus === 'sidebar') {
        if (contentLength() > 0) { menuFocus = 'content'; contentCursor = 0; }
      } else {
        activateContent();
      }
      return;
    }

    if ((e.key === 'Delete' || e.key === 'Backspace') && menuFocus === 'content') {
      e.preventDefault(); e.stopPropagation();
      if (localSection === 'favorites' && favs[contentCursor]) {
        favoritesStore.remove(favs[contentCursor].id);
        contentCursor = Math.min(contentCursor, Math.max(0, favs.length - 2));
      } else if (localSection === 'remotes' && remotes[contentCursor]) {
        remotesStore.remove(remotes[contentCursor].id);
        contentCursor = Math.min(contentCursor, Math.max(0, remotes.length - 2));
      }
      return;
    }

    // Absorb everything else so it doesn't reach App.svelte
    e.stopPropagation();
  }

  const SHORTCUTS = [
    { key: '↑ / ↓',      desc: 'Move cursor' },
    { key: 'Tab',         desc: 'Switch panel' },
    { key: 'Enter',       desc: 'Enter dir · view text file in-app · open binary in system app' },
    { key: 'Cmd+Enter',   desc: 'Always open in system app' },
    { key: 'Backspace',   desc: 'Go to parent dir' },
    { key: 'Space',       desc: 'Toggle selection + move' },
    { key: 'Cmd+A',       desc: 'Select all' },
    { key: 'Cmd+F',       desc: 'Find in current dir' },
    { key: 'Cmd+G',       desc: 'Go to path (with Tab complete)' },
    { key: 'Esc',         desc: 'Open this menu' },
    { key: 'F2',          desc: 'Rename' },
    { key: 'F3',          desc: 'Process explorer' },
    { key: 'F4',          desc: 'Open in system editor' },
    { key: 'd',           desc: 'Diff: cursor in left vs cursor in right' },
    { key: 'F5',          desc: 'Copy to other panel' },
    { key: 'F6',          desc: 'Move to other panel' },
    { key: 'F7',          desc: 'Make directory' },
    { key: 'F8',          desc: 'Delete' },
    { key: 'F9',          desc: 'Cycle theme' },
    { key: 'F10',         desc: 'Quit' },
  ];

  // ─── Settings ────────────────────────────────────────────────────────────────
  let showHidden = false; // TODO: wire to panel store
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="overlay" bind:this={overlayEl} tabindex="-1" on:keydown={handleKey}>
  <div class="menu-shell" role="dialog" aria-label="Main Menu">

    <!-- Title bar -->
    <div class="title-bar">
      <span class="title-logo">NYX.COMMANDER</span>
      <span class="title-hint">↑↓ sections · → enter · ← back · Enter activate · Del remove · Esc close</span>
      <button class="close-btn" on:click={() => dispatch('close')}>✕</button>
    </div>

    <div class="body">
      <!-- Sidebar -->
      <nav class="sidebar">
        {#each SECTIONS as s, i}
          <button
            class="nav-item"
            class:active={localSection === s.id}
            class:kbd-cursor={menuFocus === 'sidebar' && i === sidebarIdx}
            on:click={() => setSection(s.id)}
          >
            <span class="nav-arrow">{localSection === s.id ? '▸' : ' '}</span>
            {s.label}
          </button>
        {/each}

        <div class="sidebar-sep"></div>
        <div class="sidebar-path">
          <div class="sidebar-label">Current dir</div>
          <div class="sidebar-val">{activePath}</div>
          <button class="add-fav-btn" on:click={() => { favoritesStore.add(activePath); dispatch('set-section', { section: 'favorites' }); }}>
            + Add to Favorites
          </button>
        </div>
      </nav>

      <!-- Content -->
      <main class="content">

        <!-- ── Favorites ──────────────────────────────────────────── -->
        {#if section === 'favorites'}
          <div class="section-title">Favorites</div>

          {#if favs.length === 0}
            <div class="empty-state">
              No favorites yet.<br>
              Navigate to a directory and click <em>+ Add to Favorites</em> in the sidebar.
            </div>
          {:else}
            <div class="list">
              {#each favs as fav, fi (fav.id)}
                <div class="list-row" class:kbd-row={menuFocus === 'content' && fi === contentCursor}>
                  {#if editingFavId === fav.id}
                    <!-- svelte-ignore a11y-autofocus -->
                    <input
                      class="inline-input"
                      bind:value={editingFavName}
                      on:keydown={e => { if (e.key === 'Enter') saveFav(); if (e.key === 'Escape') editingFavId = null; }}
                      on:blur={saveFav}
                      autofocus
                    />
                  {:else}
                    <button class="list-main" on:click={() => navToFav(fav)}>
                      <span class="list-name">{fav.name}</span>
                      <span class="list-sub">{fav.path}</span>
                    </button>
                    <button class="icon-btn" on:click={() => startEditFav(fav)} title="Rename">✎</button>
                    <button class="icon-btn danger" on:click={() => favoritesStore.remove(fav.id)} title="Remove">✕</button>
                  {/if}
                </div>
              {/each}
            </div>
          {/if}

        <!-- ── Remotes ─────────────────────────────────────────────── -->
        {:else if section === 'remotes'}
          <div class="section-title">Remote Connections</div>

          {#if !showRemoteForm}
            <button class="action-btn" on:click={newRemote}>+ New Connection</button>

            {#if remotes.length === 0}
              <div class="empty-state">No connections configured.</div>
            {:else}
              <div class="list">
                {#each remotes as r, ri (r.id)}
                  <div class="list-row" class:kbd-row={menuFocus === 'content' && ri === contentCursor}>
                    <div class="list-main non-btn">
                      <span class="type-badge type-{r.type}">{r.type.toUpperCase()}</span>
                      <span class="list-name">{r.name}</span>
                      <span class="list-sub">{r.user ? r.user + '@' : ''}{r.host}:{r.port}{r.remotePath}</span>
                    </div>
                    <button class="icon-btn accent" on:click={() => connectRemote(r)} title="Connect">⇌</button>
                    <button class="icon-btn" on:click={() => editRemote(r)} title="Edit">✎</button>
                    <button class="icon-btn danger" on:click={() => remotesStore.remove(r.id)} title="Remove">✕</button>
                  </div>
                {/each}
              </div>
            {/if}

          {:else}
            <!-- Remote connection form -->
            <div class="form-title">{editingRemoteId ? 'Edit Connection' : 'New Connection'}</div>
            <div class="form">
              <div class="field-row">
                <label for="rf-type">Type</label>
                <select id="rf-type" bind:value={remoteForm.type} on:change={onTypeChange}>
                  <option value="sftp">SFTP</option>
                  <option value="ssh">SSH</option>
                  <option value="ftp">FTP</option>
                  <option value="cifs">CIFS / SMB</option>
                </select>
              </div>
              <div class="field-row">
                <label for="rf-name">Name</label>
                <input id="rf-name" bind:value={remoteForm.name} placeholder="My Server" />
              </div>
              <div class="field-row">
                <label for="rf-host">Host</label>
                <input id="rf-host" bind:value={remoteForm.host} placeholder="192.168.1.1 or host.example.com" />
              </div>
              <div class="field-row">
                <label for="rf-port">Port</label>
                <input id="rf-port" type="number" bind:value={remoteForm.port} style="width:80px" />
              </div>
              <div class="field-row">
                <label for="rf-user">User</label>
                <input id="rf-user" bind:value={remoteForm.user} placeholder="username" />
              </div>
              <div class="field-row">
                <label for="rf-pass">Password</label>
                <input id="rf-pass" type="password" bind:value={remoteForm.password} placeholder="(leave blank to use key)" />
              </div>
              {#if remoteForm.type === 'ssh' || remoteForm.type === 'sftp'}
                <div class="field-row">
                  <label for="rf-key">Key file</label>
                  <input id="rf-key" bind:value={remoteForm.keyPath} placeholder="~/.ssh/id_rsa" />
                </div>
              {/if}
              <div class="field-row">
                <label for="rf-rpath">Remote path</label>
                <input id="rf-rpath" bind:value={remoteForm.remotePath} placeholder="/" />
              </div>

              {#if remoteError}
                <div class="form-error">{remoteError}</div>
              {/if}

              <div class="form-footer">
                <button class="btn-cancel" on:click={cancelRemote}>Cancel</button>
                <button class="btn-ok" on:click={saveRemote}>Save</button>
              </div>
            </div>
          {/if}

        <!-- ── Themes ──────────────────────────────────────────────── -->
        {:else if section === 'themes'}
          {#if !editingTheme}
            <div class="section-title">Themes</div>

            <div class="theme-grid">
              {#each [...builtinThemes, ...custom] as t, ti (t.name)}
                <div
                  class="theme-card"
                  class:active-theme={currentTheme.name === t.name}
                  class:kbd-card={menuFocus === 'content' && ti === contentCursor}
                  style="background:{t.vars['--bg-panel']};border-color:{t.vars['--border-panel']}"
                >
                  <div class="theme-preview" style="color:{t.vars['--text']}">
                    <span style="color:{t.vars['--border-panel']}">{t.label}</span>
                    <span style="color:{t.vars['--text-dir']}">▸ folder</span>
                    <span style="color:{t.vars['--text-sel']}">selected</span>
                    <span style="color:{t.vars['--text-dim']}">dim text</span>
                  </div>
                  <div class="theme-actions">
                    <button class="theme-apply" on:click={() => applyBuiltin(t)}
                      style="border-color:{t.vars['--border-panel']};color:{t.vars['--border-panel']}">
                      {currentTheme.name === t.name ? '✓ Active' : 'Apply'}
                    </button>
                    {#if custom.some(c => c.name === t.name)}
                      <button class="theme-edit-btn" on:click={() => startEditTheme(t)}>Edit</button>
                      <button class="theme-del-btn" on:click={() => removeCustomTheme(t.name)}>✕</button>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>

            <button class="action-btn" on:click={startNewTheme} style="margin-top:12px">
              + Create Custom Theme
            </button>

          {:else}
            <!-- Theme editor -->
            <div class="section-title">
              {isNewTheme ? 'New Theme' : 'Edit: ' + editingTheme.label}
            </div>

            <div class="field-row" style="margin-bottom:12px">
              <label for="te-name">Name</label>
              <input id="te-name" bind:value={themeEditorLabel} placeholder="My Theme" style="width:200px" />
            </div>

            <div class="vars-scroll">
              {#each THEME_VAR_GROUPS as group}
                <div class="var-group-label">{group.label}</div>
                <div class="var-group">
                  {#each group.vars as cssVar}
                    <div class="var-row">
                      <span class="var-name">{cssVar}</span>
                      <input
                        type="color"
                        value={themeEditorVars[cssVar] ?? '#000000'}
                        on:input={e => {
                          themeEditorVars[cssVar] = (e.currentTarget as HTMLInputElement).value;
                          previewVar(cssVar, themeEditorVars[cssVar]);
                        }}
                      />
                      <input
                        class="hex-input"
                        value={themeEditorVars[cssVar] ?? '#000000'}
                        on:change={e => {
                          const v = (e.currentTarget as HTMLInputElement).value;
                          if (/^#[0-9a-fA-F]{6}$/.test(v)) {
                            themeEditorVars[cssVar] = v;
                            previewVar(cssVar, v);
                          }
                        }}
                      />
                    </div>
                  {/each}
                </div>
              {/each}
            </div>

            <div class="form-footer">
              <button class="btn-cancel" on:click={cancelThemeEdit}>Cancel</button>
              <button class="btn-ok" on:click={saveTheme}>Save & Apply</button>
            </div>
          {/if}

        <!-- ── Shortcuts ───────────────────────────────────────────── -->
        {:else if section === 'shortcuts'}
          <div class="section-title">Keyboard Shortcuts</div>
          <div class="shortcuts-table">
            {#each SHORTCUTS as s}
              <div class="shortcut-row">
                <span class="shortcut-key">{s.key}</span>
                <span class="shortcut-desc">{s.desc}</span>
              </div>
            {/each}
          </div>

        <!-- ── Plugins ─────────────────────────────────────────────── -->
        {:else if section === 'plugins'}
          <PluginManager on:close={() => dispatch('close')} />

        <!-- ── Settings ────────────────────────────────────────────── -->
        {:else if section === 'settings'}
          <div class="section-title">Settings</div>
          <div class="settings-list">
            <div class="setting-row">
              <div class="setting-info">
                <span class="setting-name">Show hidden files</span>
                <span class="setting-desc">Display files beginning with a dot</span>
              </div>
              <label class="toggle-switch">
                <input type="checkbox" bind:checked={showHidden} />
                <span class="toggle-track"></span>
              </label>
            </div>
          </div>

          <div class="settings-footer">
            Settings are saved automatically.
          </div>
        {/if}

      </main>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.88);
    display: flex; align-items: center; justify-content: center;
    z-index: 100;
  }

  .menu-shell {
    display: flex; flex-direction: column;
    width: 860px; max-width: 96vw; height: 80vh; max-height: 700px;
    background: var(--bg-panel);
    border: 1px solid var(--border-panel);
    font-family: 'Courier New', monospace;
    font-size: 12px;
  }

  .title-bar {
    display: flex; align-items: center; justify-content: space-between;
    padding: 0 12px; height: 26px;
    background: var(--bg-header);
    border-bottom: 1px solid var(--border-panel);
    flex-shrink: 0;
  }
  .title-logo {
    color: var(--border-panel); font-weight: bold;
    font-size: 12px; letter-spacing: .15em;
  }
  .title-hint {
    flex: 1; color: var(--text-dim); font-size: 9px; letter-spacing: .02em;
    overflow: hidden; white-space: nowrap; text-overflow: ellipsis;
  }
  .close-btn {
    background: none; border: 1px solid var(--border-dim);
    color: var(--text-dim); font-family: 'Courier New', monospace;
    font-size: 10px; padding: 2px 8px; cursor: pointer;
  }
  .close-btn:hover { border-color: var(--text-danger); color: var(--text-danger); }

  .body {
    display: flex; flex: 1; min-height: 0;
  }

  /* ── Sidebar ── */
  .sidebar {
    width: 160px; flex-shrink: 0;
    background: var(--bg);
    border-right: 1px solid var(--border-dim);
    display: flex; flex-direction: column;
    padding: 8px 0;
    overflow-y: auto;
  }

  .nav-item {
    background: none; border: none;
    text-align: left; padding: 8px 12px;
    color: var(--text-dim); font-family: 'Courier New', monospace;
    font-size: 11px; cursor: pointer; white-space: nowrap;
  }
  .nav-item:hover { color: var(--text); background: var(--bg-header); }
  .nav-item.active { color: var(--border-panel); background: var(--bg-panel); }
  /* Keyboard cursor on sidebar item */
  .nav-item.kbd-cursor { outline: 1px solid var(--border-panel); outline-offset: -2px; }
  .nav-arrow { display: inline-block; width: 1ch; }

  .sidebar-sep {
    flex: 1;
    border-top: 1px solid var(--border-dim); margin-top: 8px;
  }

  .sidebar-path {
    padding: 10px 10px 8px;
    font-size: 10px;
  }
  .sidebar-label { color: var(--text-dim); text-transform: uppercase; letter-spacing: .06em; margin-bottom: 3px; }
  .sidebar-val {
    color: var(--text); word-break: break-all; margin-bottom: 6px;
    font-size: 10px;
  }
  .add-fav-btn {
    background: none; border: 1px solid var(--border-dim);
    color: var(--text-dim); font-family: 'Courier New', monospace;
    font-size: 10px; padding: 2px 6px; cursor: pointer; width: 100%;
    text-align: left;
  }
  .add-fav-btn:hover { border-color: var(--border-panel); color: var(--border-panel); }

  /* ── Content ── */
  .content {
    flex: 1; padding: 16px; overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--bg);
  }

  .section-title {
    color: var(--border-panel); font-size: 13px; font-weight: bold;
    letter-spacing: .08em; margin-bottom: 14px;
    border-bottom: 1px solid var(--border-dim); padding-bottom: 6px;
  }

  .empty-state {
    color: var(--text-dim); font-size: 11px; padding: 12px 0;
    line-height: 1.6;
  }

  /* ── Generic list ── */
  .list { display: flex; flex-direction: column; gap: 2px; }

  .list-row {
    display: flex; align-items: center; gap: 4px;
    border: 1px solid var(--border-dim);
    padding: 4px 6px;
    background: var(--bg);
  }
  .list-row:hover { border-color: var(--border-dim); background: var(--bg-header); }
  /* Keyboard cursor on list row */
  .list-row.kbd-row { border-color: var(--border-panel); background: var(--bg-row-cursor); }

  .list-main {
    flex: 1; display: flex; flex-direction: column; gap: 1px;
    background: none; border: none; text-align: left;
    cursor: pointer; padding: 0; min-width: 0;
  }
  .list-main.non-btn { cursor: default; }
  .list-name {
    color: var(--text); font-size: 12px;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .list-main:hover .list-name { color: var(--border-panel); }
  .list-sub {
    color: var(--text-dim); font-size: 10px;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }

  .inline-input {
    flex: 1; background: var(--bg); color: var(--text);
    border: 1px solid var(--border-panel);
    padding: 2px 6px; font-family: 'Courier New', monospace; font-size: 11px;
    outline: none;
  }

  .icon-btn {
    background: none; border: 1px solid transparent;
    color: var(--text-dim); font-size: 13px;
    padding: 2px 5px; cursor: pointer; flex-shrink: 0;
  }
  .icon-btn:hover { border-color: var(--border-dim); color: var(--text); }
  .icon-btn.danger:hover { color: var(--text-danger); border-color: var(--text-danger); }
  .icon-btn.accent:hover { color: var(--border-panel); border-color: var(--border-panel); }

  /* ── Type badges ── */
  .type-badge {
    font-size: 9px; padding: 0 4px; margin-right: 6px; flex-shrink: 0;
    border: 1px solid; font-weight: bold; letter-spacing: .05em;
  }
  .type-ssh  { color: #00d4ff; border-color: #00d4ff; }
  .type-sftp { color: #00ff88; border-color: #00ff88; }
  .type-ftp  { color: #ff9900; border-color: #ff9900; }
  .type-cifs { color: #cc88ff; border-color: #cc88ff; }

  /* ── Form ── */
  .form-title {
    color: var(--text-dim); font-size: 11px; text-transform: uppercase;
    letter-spacing: .06em; margin-bottom: 10px;
  }
  .form { display: flex; flex-direction: column; gap: 6px; max-width: 480px; }

  .field-row {
    display: flex; align-items: center; gap: 10px;
  }
  .field-row label {
    color: var(--text-dim); font-size: 10px; text-transform: uppercase;
    letter-spacing: .06em; width: 90px; flex-shrink: 0; text-align: right;
  }
  .field-row input, .field-row select {
    flex: 1; background: var(--bg); color: var(--text);
    border: 1px solid var(--border-dim);
    padding: 4px 8px; font-family: 'Courier New', monospace; font-size: 11px;
    outline: none;
  }
  .field-row input:focus, .field-row select:focus { border-color: var(--border-panel); }
  .field-row select option { background: var(--bg); }

  .form-error {
    color: var(--text-danger); font-size: 10px; margin-top: 4px;
  }

  .form-footer {
    display: flex; gap: 8px; margin-top: 14px;
  }
  .btn-cancel, .btn-ok {
    background: transparent; border: 1px solid;
    font-family: 'Courier New', monospace; font-size: 11px;
    padding: 4px 20px; cursor: pointer;
  }
  .btn-cancel {
    border-color: var(--border-dim); color: var(--text-dim);
  }
  .btn-cancel:hover { border-color: var(--text); color: var(--text); }
  .btn-ok {
    border-color: var(--border-panel); color: var(--border-panel);
    background: rgba(0,0,0,.2);
  }
  .btn-ok:hover { background: var(--border-panel); color: var(--bg); }

  /* ── Theme grid ── */
  .theme-grid {
    display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 10px;
  }

  .theme-card {
    border: 1px solid; padding: 10px;
    display: flex; flex-direction: column; gap: 6px;
  }
  .theme-card.active-theme { outline: 2px solid var(--border-panel); outline-offset: 1px; }
  .theme-card.kbd-card { outline: 2px dashed var(--border-panel); outline-offset: 2px; }

  .theme-preview {
    display: flex; flex-direction: column; gap: 2px;
    font-size: 10px;
  }

  .theme-actions {
    display: flex; gap: 4px; align-items: center;
  }
  .theme-apply {
    flex: 1; background: transparent; border: 1px solid;
    font-family: 'Courier New', monospace; font-size: 10px;
    padding: 2px 6px; cursor: pointer;
  }
  .theme-apply:hover { opacity: .7; }
  .theme-edit-btn, .theme-del-btn {
    background: none; border: 1px solid var(--border-dim);
    color: var(--text-dim); font-family: 'Courier New', monospace;
    font-size: 10px; padding: 2px 6px; cursor: pointer;
  }
  .theme-edit-btn:hover { color: var(--text); border-color: var(--text); }
  .theme-del-btn:hover { color: var(--text-danger); border-color: var(--text-danger); }

  /* ── Theme editor ── */
  .vars-scroll {
    max-height: 420px; overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--bg);
    margin-bottom: 14px;
  }

  .var-group-label {
    color: var(--text-dim); font-size: 10px; text-transform: uppercase;
    letter-spacing: .08em; margin: 12px 0 4px;
  }
  .var-group {
    display: flex; flex-direction: column; gap: 3px;
  }
  .var-row {
    display: flex; align-items: center; gap: 8px;
  }
  .var-name {
    color: var(--text-dim); font-size: 10px; width: 180px; flex-shrink: 0;
  }
  input[type="color"] {
    width: 32px; height: 22px; border: 1px solid var(--border-dim);
    padding: 1px; background: none; cursor: pointer; flex-shrink: 0;
  }
  .hex-input {
    width: 82px; flex-shrink: 0;
    background: var(--bg); color: var(--text);
    border: 1px solid var(--border-dim);
    padding: 2px 6px; font-family: 'Courier New', monospace; font-size: 11px;
    outline: none;
  }
  .hex-input:focus { border-color: var(--border-panel); }

  /* ── Shortcuts ── */
  .shortcuts-table { display: flex; flex-direction: column; gap: 1px; }
  .shortcut-row {
    display: flex; gap: 16px; padding: 5px 0;
    border-bottom: 1px solid var(--border-dim);
  }
  .shortcut-key {
    width: 110px; flex-shrink: 0;
    color: var(--border-panel); font-size: 11px;
    font-weight: bold;
  }
  .shortcut-desc { color: var(--text); font-size: 11px; }

  /* ── Settings ── */
  .settings-list { display: flex; flex-direction: column; gap: 1px; }
  .setting-row {
    display: flex; align-items: center; justify-content: space-between;
    padding: 10px 0; border-bottom: 1px solid var(--border-dim);
  }
  .setting-info { display: flex; flex-direction: column; gap: 2px; }
  .setting-name { color: var(--text); font-size: 12px; }
  .setting-desc { color: var(--text-dim); font-size: 10px; }

  .toggle-switch { position: relative; display: inline-block; }
  .toggle-switch input { opacity: 0; width: 0; height: 0; position: absolute; }
  .toggle-track {
    display: block; width: 36px; height: 18px;
    background: var(--border-dim); border: 1px solid var(--border-dim);
    cursor: pointer; transition: background .15s;
    position: relative;
  }
  .toggle-track::after {
    content: ''; position: absolute; top: 2px; left: 2px;
    width: 12px; height: 12px; background: var(--text-dim);
    transition: left .15s;
  }
  .toggle-switch input:checked + .toggle-track { background: var(--bg-row-sel); border-color: var(--border-panel); }
  .toggle-switch input:checked + .toggle-track::after { left: 20px; background: var(--border-panel); }

  .settings-footer {
    margin-top: 16px; color: var(--text-dim); font-size: 10px;
  }

  /* ── Generic ── */
  .action-btn {
    background: transparent; border: 1px solid var(--border-dim);
    color: var(--text-dim); font-family: 'Courier New', monospace;
    font-size: 11px; padding: 5px 14px; cursor: pointer;
  }
  .action-btn:hover { border-color: var(--border-panel); color: var(--border-panel); }
</style>
