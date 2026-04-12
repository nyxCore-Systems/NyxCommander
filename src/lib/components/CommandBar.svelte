<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let cwd: string;

  const dispatch = createEventDispatcher<{ close: void }>();

  let input = '';
  let output = '';
  let running = false;
  let inputEl: HTMLInputElement;
  let outputEl: HTMLElement;
  let history: string[] = [];
  let histIdx = -1;

  onMount(() => inputEl?.focus());

  async function run() {
    if (!input.trim()) return;
    history = [input, ...history.slice(0, 49)];
    histIdx = -1;
    running = true;
    const cmd = input;
    input = '';
    output += `\n$ ${cmd}\n`;
    try {
      const result = await invoke<string>('exec_shell', { cmd, cwd });
      output += result;
    } catch (e) {
      output += `Error: ${e}\n`;
    }
    running = false;
    // scroll to bottom
    setTimeout(() => {
      if (outputEl) outputEl.scrollTop = outputEl.scrollHeight;
    }, 10);
  }

  function handleKey(e: KeyboardEvent) {
    e.stopPropagation();
    if (e.key === 'Enter') { run(); return; }
    if (e.key === 'Escape') { dispatch('close'); return; }
    if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (histIdx < history.length - 1) {
        histIdx++;
        input = history[histIdx];
      }
      return;
    }
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (histIdx > 0) {
        histIdx--;
        input = history[histIdx];
      } else {
        histIdx = -1;
        input = '';
      }
    }
  }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="cmd-bar" on:keydown={handleKey}>
  {#if output}
    <div class="output" bind:this={outputEl}>{output.trimStart()}</div>
  {/if}
  <div class="input-row">
    <span class="prompt">{cwd} $</span>
    <input
      bind:this={inputEl}
      bind:value={input}
      class="cmd-input"
      placeholder={running ? 'running…' : 'shell command…'}
      disabled={running}
      spellcheck="false"
      autocomplete="off"
    />
  </div>
</div>

<style>
  .cmd-bar {
    display: flex;
    flex-direction: column;
    background: var(--bg);
    border-top: 1px solid var(--border-dim);
    flex-shrink: 0;
    max-height: 200px;
  }

  .output {
    flex: 1;
    overflow-y: auto;
    padding: 4px 8px;
    font-family: 'Courier New', monospace;
    font-size: 11px;
    color: var(--text-dim);
    white-space: pre-wrap;
    word-break: break-all;
    max-height: 140px;
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--bg);
  }

  .input-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    border-top: 1px solid var(--border-dim);
  }

  .prompt {
    font-family: 'Courier New', monospace;
    font-size: 11px;
    color: var(--accent);
    white-space: nowrap;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 0;
  }

  .cmd-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text);
    font-family: 'Courier New', monospace;
    font-size: 11px;
    caret-color: var(--accent);
  }
  .cmd-input::placeholder { color: var(--text-dim); }
  .cmd-input:disabled { opacity: 0.5; }
</style>
