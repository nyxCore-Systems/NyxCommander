<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { processStore } from '$lib/stores/processStore';
  import type { ProcessInfo } from '$lib/stores/processStore';

  const dispatch = createEventDispatcher<{ close: void }>();

  $: state = $processStore;

  let overlayEl: HTMLElement;

  onMount(() => { processStore.start(); overlayEl?.focus(); });
  onDestroy(() => processStore.stop());

  function handleKey(e: KeyboardEvent) {
    e.stopPropagation();
    if (e.key === 'Escape') { dispatch('close'); return; }
    if (e.key === 'ArrowDown') { e.preventDefault(); moveCursor(1); }
    if (e.key === 'ArrowUp')   { e.preventDefault(); moveCursor(-1); }
    if (e.key === 'Delete' || e.key === 'k') {
      if (state.selectedPid !== null) confirmKill();
    }
  }

  function moveCursor(delta: number) {
    if (state.processes.length === 0) return;
    const idx = state.selectedPid === null
      ? (delta > 0 ? 0 : state.processes.length - 1)
      : state.processes.findIndex(p => p.pid === state.selectedPid);
    const next = Math.max(0, Math.min(state.processes.length - 1, idx + delta));
    processStore.selectPid(state.processes[next].pid);
  }

  let killConfirm = false;
  let killTarget: ProcessInfo | null = null;

  function confirmKill() {
    const proc = state.processes.find(p => p.pid === state.selectedPid);
    if (!proc) return;
    killTarget = proc;
    killConfirm = true;
  }

  async function doKill() {
    if (state.selectedPid === null) return;
    await processStore.killSelected(state.selectedPid);
    killConfirm = false;
    killTarget = null;
  }

  function fmtMem(kb: number): string {
    if (kb >= 1024 * 1024) return `${(kb / 1024 / 1024).toFixed(1)}G`;
    if (kb >= 1024)        return `${(kb / 1024).toFixed(1)}M`;
    return `${kb}K`;
  }

  function fmtMemBar(used: number, total: number): string {
    if (total === 0) return '----------';
    const pct = Math.round((used / total) * 10);
    return '█'.repeat(pct) + '░'.repeat(10 - pct);
  }

  function fmtCpuBar(pct: number): string {
    const filled = Math.round(Math.min(pct, 100) / 10);
    return '█'.repeat(filled) + '░'.repeat(10 - filled);
  }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="overlay" on:keydown={handleKey} tabindex="-1" bind:this={overlayEl}>
  <div class="explorer" role="dialog" tabindex="-1" aria-label="Process Explorer">

    <div class="title-bar">
      <span class="title-text">NYX.PROCESSES</span>
      <span class="close-btn" on:click={() => dispatch('close')} on:keydown={e => e.key === 'Enter' && dispatch('close')} role="button" tabindex="0">✕</span>
    </div>

    <!-- System stats header -->
    {#if state.stats}
      {@const s = state.stats}
      <div class="stats-bar">
        <span class="stat">
          CPU <span class="bar cpu-bar">{fmtCpuBar(s.cpu_usage)}</span>
          <span class="stat-val">{s.cpu_usage.toFixed(1)}%</span>
        </span>
        <span class="stat">
          MEM <span class="bar mem-bar">{fmtMemBar(s.used_memory_kb, s.total_memory_kb)}</span>
          <span class="stat-val">{fmtMem(s.used_memory_kb)} / {fmtMem(s.total_memory_kb)}</span>
        </span>
        <span class="stat-count">{s.process_count} procs</span>
      </div>
    {:else}
      <div class="stats-bar"><span class="loading-text">Collecting data…</span></div>
    {/if}

    <!-- Column headers -->
    <div class="col-headers">
      <span class="col-pid">PID</span>
      <span class="col-name">Name</span>
      <span class="col-cpu">CPU%</span>
      <span class="col-mem">Memory</span>
      <span class="col-status">Status</span>
    </div>

    <!-- Process list -->
    <div class="proc-list">
      {#if state.loading && state.processes.length === 0}
        <div class="message">Loading processes…</div>
      {:else if state.error}
        <div class="message error">{state.error}</div>
      {:else}
        {#each state.processes as proc (proc.pid)}
          <!-- svelte-ignore a11y-click-events-have-key-events -->
          <div
            class="proc-row"
            class:selected={state.selectedPid === proc.pid}
            on:click={() => processStore.selectPid(proc.pid)}
            on:dblclick={confirmKill}
          >
            <span class="col-pid dim">{proc.pid}</span>
            <span class="col-name">{proc.name}</span>
            <span class="col-cpu" class:hot={proc.cpu > 10}>{proc.cpu.toFixed(1)}%</span>
            <span class="col-mem">{fmtMem(proc.memory_kb)}</span>
            <span class="col-status dim">{proc.status}</span>
          </div>
        {/each}
      {/if}
    </div>

    <!-- Footer -->
    <div class="footer">
      <span class="hint">↑↓ navigate</span>
      <span class="hint">K / Del kill</span>
      <span class="hint">Esc close</span>
      {#if state.selectedPid !== null}
        <button class="kill-btn" on:click={confirmKill}>Kill [{state.selectedPid}]</button>
      {/if}
    </div>

  </div>
</div>

<!-- Kill confirmation modal -->
{#if killConfirm && killTarget}
  <div class="confirm-overlay">
    <div class="confirm-box" role="dialog" tabindex="-1">
      <div class="confirm-title">Kill Process?</div>
      <div class="confirm-msg">
        Send SIGKILL to <span class="proc-name">{killTarget.name}</span> (PID {killTarget.pid})?
      </div>
      <div class="confirm-footer">
        <button class="btn-cancel" on:click={() => { killConfirm = false; killTarget = null; }}>Cancel</button>
        <button class="btn-kill" on:click={doKill}>Kill</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .explorer {
    display: flex;
    flex-direction: column;
    width: 820px;
    max-width: 95vw;
    height: 70vh;
    background: var(--bg-panel);
    border: 1px solid var(--border-panel);
    font-family: 'Courier New', monospace;
    font-size: 11px;
    overflow: hidden;
  }

  .title-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 8px;
    height: 22px;
    background: var(--bg-header);
    border-bottom: 1px solid var(--border-panel);
    flex-shrink: 0;
  }

  .title-text {
    color: var(--border-panel);
    font-weight: bold;
    font-size: 11px;
    letter-spacing: 0.1em;
  }

  .close-btn {
    color: var(--text-dim);
    cursor: pointer;
    font-size: 12px;
    padding: 0 2px;
  }
  .close-btn:hover { color: var(--text-danger); }

  .stats-bar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 4px 8px;
    background: var(--bg);
    border-bottom: 1px solid var(--border-dim);
    flex-shrink: 0;
    height: 26px;
    font-size: 10px;
    color: var(--text-dim);
  }

  .stat { display: flex; align-items: center; gap: 4px; }
  .bar { color: var(--border-panel); letter-spacing: -1px; }
  .stat-val { color: var(--text); }
  .stat-count { margin-left: auto; color: var(--text-dim); }
  .loading-text { color: var(--text-dim); }

  .col-headers {
    display: flex;
    align-items: center;
    padding: 0 8px;
    height: 20px;
    background: var(--bg-header);
    border-bottom: 1px solid var(--border-dim);
    flex-shrink: 0;
    color: var(--text-dim);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .proc-list {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--bg);
  }

  .proc-row {
    display: flex;
    align-items: center;
    padding: 0 8px;
    height: 22px;
    cursor: pointer;
    border-bottom: 1px solid transparent;
    color: var(--text);
  }

  .proc-row:hover { background: var(--bg-header); }
  .proc-row.selected {
    background: var(--bg-row-sel);
    color: var(--text-sel);
    border-bottom-color: var(--border-dim);
  }

  .col-pid    { width: 60px;  flex-shrink: 0; text-align: right; padding-right: 8px; }
  .col-name   { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .col-cpu    { width: 60px;  flex-shrink: 0; text-align: right; padding-right: 8px; }
  .col-mem    { width: 72px;  flex-shrink: 0; text-align: right; padding-right: 8px; }
  .col-status { width: 80px;  flex-shrink: 0; }

  .dim { color: var(--text-dim); }
  .hot { color: var(--text-danger); }
  .proc-row.selected .dim,
  .proc-row.selected .col-cpu { color: var(--text-sel); }

  .footer {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 8px;
    height: 22px;
    background: var(--bg-header);
    border-top: 1px solid var(--border-dim);
    flex-shrink: 0;
    font-size: 10px;
    color: var(--text-dim);
  }

  .hint { color: var(--text-dim); }

  .kill-btn {
    margin-left: auto;
    background: transparent;
    border: 1px solid var(--text-danger);
    color: var(--text-danger);
    font-family: 'Courier New', monospace;
    font-size: 10px;
    padding: 1px 8px;
    cursor: pointer;
  }
  .kill-btn:hover { background: var(--text-danger); color: var(--bg); }

  .message {
    padding: 12px 8px;
    color: var(--text-dim);
  }
  .message.error { color: var(--text-danger); }

  /* Kill confirmation */
  .confirm-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }

  .confirm-box {
    background: var(--bg-panel);
    border: 1px solid var(--text-danger);
    padding: 16px 20px;
    font-family: 'Courier New', monospace;
    min-width: 320px;
  }

  .confirm-title {
    color: var(--text-danger);
    font-size: 12px;
    font-weight: bold;
    letter-spacing: 0.08em;
    margin-bottom: 10px;
  }

  .confirm-msg {
    color: var(--text);
    font-size: 11px;
    margin-bottom: 14px;
  }

  .proc-name { color: var(--border-panel); }

  .confirm-footer {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }

  .btn-cancel, .btn-kill {
    font-family: 'Courier New', monospace;
    font-size: 11px;
    padding: 3px 14px;
    cursor: pointer;
    border: 1px solid;
  }

  .btn-cancel {
    background: transparent;
    border-color: var(--border-dim);
    color: var(--text-dim);
  }
  .btn-cancel:hover { border-color: var(--text); color: var(--text); }

  .btn-kill {
    background: var(--text-danger);
    border-color: var(--text-danger);
    color: var(--bg);
  }
  .btn-kill:hover { filter: brightness(1.2); }
</style>
