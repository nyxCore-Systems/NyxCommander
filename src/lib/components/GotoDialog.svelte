<script lang="ts">
  import { createEventDispatcher, onMount, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  const dispatch = createEventDispatcher<{
    navigate: { path: string };
    close: void;
  }>();

  let inputValue = '~/';
  let completions: string[] = [];
  let compIdx = -1;
  let error = '';
  let inputEl: HTMLInputElement;
  let homePath = '';

  onMount(async () => {
    try {
      homePath = await invoke<string>('get_home');
      inputValue = homePath + '/';
    } catch {
      homePath = '';
      inputValue = '/';
    }
    await tick();
    inputEl?.focus();
    inputEl?.setSelectionRange(inputValue.length, inputValue.length);
    await fetchCompletions();
  });

  async function fetchCompletions() {
    try {
      completions = await invoke<string[]>('complete_path', { partial: inputValue });
      compIdx = -1;
    } catch {
      completions = [];
    }
  }

  async function handleKey(e: KeyboardEvent) {
    // Prevent all handled keys from reaching App.svelte's global handler.
    // Without this, Enter would trigger a second navigation after the dialog closes.
    const handled = ['Escape','Tab','ArrowDown','ArrowUp','Enter'];
    if (handled.includes(e.key)) {
      e.stopPropagation();
    }

    if (e.key === 'Escape') {
      e.preventDefault();
      if (completions.length > 0) { completions = []; return; }
      dispatch('close');
      return;
    }

    if (e.key === 'Tab') {
      e.preventDefault();
      if (completions.length === 1) {
        inputValue = completions[0];
        completions = [];
        await fetchCompletions();
      } else if (completions.length > 1) {
        compIdx = (compIdx + 1) % completions.length;
      }
      return;
    }

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (completions.length > 0) compIdx = Math.min(compIdx + 1, completions.length - 1);
      return;
    }
    if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (completions.length > 0) compIdx = Math.max(compIdx - 1, -1);
      return;
    }

    if (e.key === 'Enter') {
      e.preventDefault();
      if (compIdx >= 0 && completions[compIdx]) {
        // Completion selected — fill it in; second Enter will navigate
        inputValue = completions[compIdx];
        completions = [];
        compIdx = -1;
        await fetchCompletions();
      } else if (completions.length === 1) {
        // Only one completion — accept it automatically and navigate
        inputValue = completions[0].replace(/\/$/, '');
        completions = [];
        doNavigate();
      } else {
        doNavigate();
      }
      return;
    }
  }

  async function handleInput() {
    error = '';
    await fetchCompletions();
  }

  function pickCompletion(path: string) {
    inputValue = path;
    completions = [];
    fetchCompletions();
    inputEl?.focus();
  }

  function doNavigate() {
    let path = inputValue.trim();
    // Expand ~ using the actual home path
    if (path === '~') {
      path = homePath || '/';
    } else if (path.startsWith('~/')) {
      path = (homePath || '') + path.slice(1);
    }
    const resolved = path.startsWith('/') ? path : '/' + path;
    dispatch('navigate', { path: resolved.replace(/\/$/, '') || '/' });
  }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="overlay" on:keydown={handleKey}>
  <div class="dialog" role="dialog" tabindex="-1">

    <div class="title-bar">
      <span>GO TO PATH</span>
      <button class="close-btn" on:click={() => dispatch('close')}>✕</button>
    </div>

    <div class="input-area">
      <div class="input-wrap">
        <span class="prompt">&gt;</span>
        <input
          bind:this={inputEl}
          bind:value={inputValue}
          on:input={handleInput}
          spellcheck="false"
          autocomplete="off"
        />
      </div>

      {#if completions.length > 0}
        <div class="completions">
          {#each completions as comp, i}
            <!-- svelte-ignore a11y-click-events-have-key-events -->
            <div
              class="comp-item"
              class:active={i === compIdx}
              on:click={() => pickCompletion(comp)}
            >
              {comp.endsWith('/') ? '▸ ' : '  '}{comp}
            </div>
          {/each}
        </div>
      {/if}
    </div>

    {#if error}
      <div class="error-bar">{error}</div>
    {/if}

    <div class="footer">
      <span class="hint">Tab complete · ↑↓ pick · Enter go · Esc cancel</span>
      <button class="go-btn" on:click={doNavigate}>Navigate</button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.82);
    display: flex; align-items: center; justify-content: center;
    z-index: 100;
  }

  .dialog {
    display: flex; flex-direction: column;
    width: 580px; max-width: 92vw;
    background: var(--bg-panel);
    border: 1px solid var(--border-panel);
    font-family: 'Courier New', monospace;
  }

  .title-bar {
    display: flex; align-items: center; justify-content: space-between;
    padding: 0 8px; height: 22px;
    background: var(--bg-header);
    border-bottom: 1px solid var(--border-panel);
    color: var(--border-panel);
    font-size: 11px; font-weight: bold; letter-spacing: .1em;
    flex-shrink: 0;
  }
  .close-btn {
    background: none; border: none; color: var(--text-dim);
    cursor: pointer; font-size: 12px;
  }
  .close-btn:hover { color: var(--text-danger); }

  .input-area {
    position: relative;
    padding: 10px 8px 0;
    background: var(--bg);
  }

  .input-wrap {
    display: flex; align-items: center; gap: 6px;
  }

  .prompt {
    color: var(--border-panel); font-size: 14px; flex-shrink: 0;
  }

  input {
    flex: 1;
    background: transparent; color: var(--text);
    border: none; border-bottom: 1px solid var(--border-panel);
    padding: 4px 0;
    font-family: 'Courier New', monospace; font-size: 13px;
    outline: none;
  }

  .completions {
    position: absolute; top: 100%; left: 0; right: 0;
    background: var(--bg-panel);
    border: 1px solid var(--border-dim);
    border-top: none;
    max-height: 220px; overflow-y: auto;
    z-index: 10;
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--bg);
  }

  .comp-item {
    padding: 3px 10px;
    color: var(--text-dim); font-size: 11px;
    cursor: pointer;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .comp-item:hover, .comp-item.active {
    background: var(--bg-row-cursor);
    color: var(--border-panel);
  }

  .error-bar {
    padding: 4px 8px; color: var(--text-danger); font-size: 10px;
    background: rgba(255,50,80,.08);
  }

  .footer {
    display: flex; align-items: center; gap: 10px;
    padding: 8px;
    background: var(--bg-header);
    border-top: 1px solid var(--border-dim);
    font-size: 10px;
  }
  .hint { flex: 1; color: var(--text-dim); }

  .go-btn {
    background: var(--bg-fnkey); color: var(--text);
    border: 1px solid var(--border-panel);
    padding: 3px 16px;
    font-family: 'Courier New', monospace; font-size: 11px;
    cursor: pointer;
  }
  .go-btn:hover { border-color: var(--border-panel); color: var(--border-panel); }
</style>
