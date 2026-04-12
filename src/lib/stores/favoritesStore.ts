import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export interface Favorite {
  id: string;
  name: string;
  path: string;
}

const STORAGE_KEY = 'nyx-favorites';

function createFavoritesStore() {
  const saved = browser ? localStorage.getItem(STORAGE_KEY) : null;
  const initial: Favorite[] = saved ? (JSON.parse(saved) as Favorite[]) : [];

  const { subscribe, update } = writable<Favorite[]>(initial);

  function persist(favs: Favorite[]) {
    if (browser) localStorage.setItem(STORAGE_KEY, JSON.stringify(favs));
  }

  function add(path: string, name?: string) {
    update(favs => {
      if (favs.some(f => f.path === path)) return favs;
      const label = name ?? (path.split('/').filter(Boolean).pop() ?? path);
      const next = [...favs, { id: crypto.randomUUID(), name: label, path }];
      persist(next);
      return next;
    });
  }

  function remove(id: string) {
    update(favs => {
      const next = favs.filter(f => f.id !== id);
      persist(next);
      return next;
    });
  }

  function rename(id: string, name: string) {
    update(favs => {
      const next = favs.map(f => (f.id === id ? { ...f, name } : f));
      persist(next);
      return next;
    });
  }

  return { subscribe, add, remove, rename };
}

export const favoritesStore = createFavoritesStore();
