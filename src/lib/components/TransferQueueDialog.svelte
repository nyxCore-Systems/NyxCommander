<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { transferQueue, type TransferJob } from '$lib/stores/transferQueueStore';

  const dispatch = createEventDispatcher<{ close: void }>();

  $: jobs = $transferQueue;

  function handleKey(e: KeyboardEvent) {
    e.stopPropagation();
    if (e.key === 'Escape') dispatch('close');
  }

  function statusIcon(s: TransferJob['status']): string {
    if (s === 'pending') return '◌';
    if (s === 'running') return '▶';
    if (s === 'done') return '✓';
    if (s === 'error') return '✕';
    if (s === 'paused') return '⏸';
    return '?';
  }

  function statusColor(s: TransferJob['status']): string {
    if (s === 'done') return 'var(--text-selected)';
    if (s === 'error') return 'var(--danger)';
    if (s === 'running') return 'var(--accent)';
    return 'var(--text-dim)';
  }

  function elapsed(j: TransferJob): string {
    if (!j.startedAt) return '';
    const end = j.finishedAt ?? Date.now();
    const secs = Math.round((end - j.startedAt) / 1000);
    return secs < 60 ? `${secs}s` : `${Math.floor(secs / 60)}m${secs % 60}s`;
  }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="overlay" on:keydown={handleKey}>
  <div class="dialog">
    <div class="titlebar">
      <span class="title">Transfer Queue</span>
      <div class="hdr-actions">
        <button on:click={() => transferQueue.dismissCompleted()}>Clear done</button>
        <button class="close-btn" on:click={() => dispatch('close')}>✕</button>
      </div>
    </div>

    {#if jobs.length === 0}
      <div class="empty">No transfers in queue.</div>
    {:else}
      <div class="job-list">
        {#each jobs as job (job.id)}
          <div class="job-row" class:done={job.status === 'done'} class:err={job.status === 'error'}>
            <span class="status-icon" style="color: {statusColor(job.status)}">{statusIcon(job.status)}</span>
            <div class="job-info">
              <div class="job-title">
                <span class="kind">{job.kind.toUpperCase()}</span>
                <span class="srcs">{job.srcs.length} item{job.srcs.length !== 1 ? 's' : ''}</span>
                <span class="arrow">→</span>
                <span class="dst">{job.dstDir}</span>
              </div>
              {#if job.error}
                <div class="job-error">{job.error}</div>
              {:else}
                <div class="job-detail">
                  {job.srcs.slice(0, 3).map(p => p.split('/').pop()).join(', ')}
                  {job.srcs.length > 3 ? `… +${job.srcs.length - 3} more` : ''}
                </div>
              {/if}
            </div>
            <span class="elapsed">{elapsed(job)}</span>
            <button class="dismiss" on:click={() => transferQueue.dismiss(job.id)}>✕</button>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed; inset: 0; background: rgba(0,0,0,0.75);
    display: flex; align-items: center; justify-content: center; z-index: 100;
  }
  .dialog {
    background: var(--bg-panel); border: 1px solid var(--border-panel);
    width: min(560px, 95vw); max-height: 70vh;
    display: flex; flex-direction: column; font-family: 'Courier New', monospace; font-size: 12px;
  }
  .titlebar {
    display: flex; align-items: center; justify-content: space-between;
    padding: 6px 10px; background: var(--bg-header); border-bottom: 1px solid var(--border-dim);
  }
  .title { color: var(--accent); font-size: 11px; text-transform: uppercase; letter-spacing: 0.06em; }
  .hdr-actions { display: flex; gap: 8px; }
  .close-btn { background: none; border: none; color: var(--text-dim); cursor: pointer; font-size: 14px; }
  .close-btn:hover { color: var(--danger); }
  button {
    background: var(--bg); border: 1px solid var(--border-dim); color: var(--text-dim);
    font-family: inherit; font-size: 10px; padding: 2px 8px; cursor: pointer;
  }
  button:hover { border-color: var(--accent); color: var(--accent); }

  .empty { padding: 24px; text-align: center; color: var(--text-dim); }

  .job-list { flex: 1; overflow-y: auto; scrollbar-width: thin; scrollbar-color: var(--scrollbar-thumb) var(--bg); }
  .job-row {
    display: flex; align-items: center; gap: 8px;
    padding: 8px 10px; border-bottom: 1px solid var(--border-dim);
  }
  .job-row:hover { background: var(--bg-header); }
  .job-row.done { opacity: 0.6; }
  .job-row.err { background: rgba(255,34,68,0.05); }

  .status-icon { font-size: 14px; flex-shrink: 0; width: 18px; text-align: center; }
  .job-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 3px; }
  .job-title { display: flex; align-items: center; gap: 6px; font-size: 11px; overflow: hidden; }
  .kind { color: var(--accent); font-weight: bold; flex-shrink: 0; }
  .srcs { color: var(--text-dim); flex-shrink: 0; }
  .arrow { color: var(--border-dim); }
  .dst { color: var(--text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .job-detail { font-size: 10px; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .job-error { font-size: 10px; color: var(--danger); }
  .elapsed { color: var(--text-dim); font-size: 10px; flex-shrink: 0; }
  .dismiss { background: none; border: none; color: var(--text-dim); cursor: pointer; font-size: 11px; padding: 0 3px; }
  .dismiss:hover { color: var(--danger); border: none; }
</style>
