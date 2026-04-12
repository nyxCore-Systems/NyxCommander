import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface ProcessInfo {
  pid: number;
  name: string;
  cpu: number;
  memory_kb: number;
  status: string;
}

export interface SystemStats {
  total_memory_kb: number;
  used_memory_kb: number;
  cpu_usage: number;
  process_count: number;
}

interface ProcessState {
  processes: ProcessInfo[];
  stats: SystemStats | null;
  loading: boolean;
  error: string;
  selectedPid: number | null;
}

function createProcessStore() {
  const { subscribe, update } = writable<ProcessState>({
    processes: [],
    stats: null,
    loading: false,
    error: '',
    selectedPid: null,
  });

  let refreshTimer: ReturnType<typeof setInterval> | null = null;

  async function refresh() {
    try {
      const [processes, stats] = await Promise.all([
        invoke<ProcessInfo[]>('list_processes'),
        invoke<SystemStats>('get_system_stats'),
      ]);
      update(s => ({ ...s, processes, stats, error: '', loading: false }));
    } catch (e) {
      update(s => ({ ...s, error: String(e), loading: false }));
    }
  }

  function start() {
    update(s => ({ ...s, loading: true, selectedPid: null }));
    refresh();
    refreshTimer = setInterval(refresh, 2000);
  }

  function stop() {
    if (refreshTimer !== null) {
      clearInterval(refreshTimer);
      refreshTimer = null;
    }
    update(s => ({ ...s, processes: [], stats: null, selectedPid: null }));
  }

  function selectPid(pid: number | null) {
    update(s => ({ ...s, selectedPid: pid }));
  }

  async function killSelected(selectedPid: number) {
    try {
      await invoke('kill_process', { pid: selectedPid });
      update(s => ({ ...s, selectedPid: null }));
      await refresh();
    } catch (e) {
      update(s => ({ ...s, error: `Kill failed: ${e}` }));
    }
  }

  return { subscribe, start, stop, selectPid, killSelected, refresh };
}

export const processStore = createProcessStore();
