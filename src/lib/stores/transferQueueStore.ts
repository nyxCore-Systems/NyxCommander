import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export type TransferStatus = 'pending' | 'running' | 'done' | 'error' | 'paused';
export type TransferKind = 'copy' | 'move';

export interface TransferJob {
  id: string;
  kind: TransferKind;
  srcs: string[];
  dstDir: string;
  status: TransferStatus;
  error?: string;
  // Progress is approximate — we track job-level, not byte-level
  startedAt?: number;
  finishedAt?: number;
}

function createTransferQueueStore() {
  const { subscribe, update } = writable<TransferJob[]>([]);

  function enqueue(kind: TransferKind, srcs: string[], dstDir: string): string {
    const id = Math.random().toString(36).slice(2, 10);
    update(jobs => [...jobs, {
      id, kind, srcs, dstDir, status: 'pending',
    }]);
    _run(id, kind, srcs, dstDir);
    return id;
  }

  async function _run(id: string, kind: TransferKind, srcs: string[], dstDir: string) {
    update(jobs => jobs.map(j => j.id === id
      ? { ...j, status: 'running', startedAt: Date.now() }
      : j));
    try {
      if (kind === 'copy') {
        await invoke('copy_items', { srcs, dstDir });
      } else {
        await invoke('move_items', { srcs, dstDir });
      }
      update(jobs => jobs.map(j => j.id === id
        ? { ...j, status: 'done', finishedAt: Date.now() }
        : j));
    } catch (e) {
      update(jobs => jobs.map(j => j.id === id
        ? { ...j, status: 'error', error: String(e), finishedAt: Date.now() }
        : j));
    }
  }

  function dismiss(id: string) {
    update(jobs => jobs.filter(j => j.id !== id));
  }

  function dismissCompleted() {
    update(jobs => jobs.filter(j => j.status !== 'done'));
  }

  return { subscribe, enqueue, dismiss, dismissCompleted };
}

export const transferQueue = createTransferQueueStore();
