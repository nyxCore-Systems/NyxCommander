import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export type RemoteType = 'ssh' | 'sftp' | 'ftp' | 'cifs';

export interface RemoteConn {
  id: string;
  name: string;
  type: RemoteType;
  host: string;
  port: number;
  user: string;
  password?: string;
  keyPath?: string;
  remotePath: string;
}

export const DEFAULT_PORTS: Record<RemoteType, number> = {
  ssh: 22, sftp: 22, ftp: 21, cifs: 445,
};

const STORAGE_KEY = 'nyx-remotes';

function createRemotesStore() {
  const saved = browser ? localStorage.getItem(STORAGE_KEY) : null;
  const initial: RemoteConn[] = saved ? (JSON.parse(saved) as RemoteConn[]) : [];

  const { subscribe, update } = writable<RemoteConn[]>(initial);

  function persist(conns: RemoteConn[]) {
    if (browser) localStorage.setItem(STORAGE_KEY, JSON.stringify(conns));
  }

  function add(conn: Omit<RemoteConn, 'id'>) {
    update(conns => {
      const next = [...conns, { ...conn, id: crypto.randomUUID() }];
      persist(next);
      return next;
    });
  }

  function edit(id: string, patch: Partial<Omit<RemoteConn, 'id'>>) {
    update(conns => {
      const next = conns.map(c => (c.id === id ? { ...c, ...patch } : c));
      persist(next);
      return next;
    });
  }

  function remove(id: string) {
    update(conns => {
      const next = conns.filter(c => c.id !== id);
      persist(next);
      return next;
    });
  }

  return { subscribe, add, edit, remove };
}

export const remotesStore = createRemotesStore();
