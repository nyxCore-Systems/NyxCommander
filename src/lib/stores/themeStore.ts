import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export type BuiltinThemeName = 'neon' | 'classic' | 'matrix' | 'amber';
export type ThemeName = BuiltinThemeName | string;

export interface Theme {
  name: ThemeName;
  label: string;
  vars: Record<string, string>;
}

// All CSS custom properties in declaration order (used by the theme editor)
export const THEME_VAR_GROUPS: Array<{ label: string; vars: string[] }> = [
  {
    label: 'Backgrounds',
    vars: ['--bg', '--bg-panel', '--bg-header', '--bg-dialog', '--bg-fnbar', '--bg-fnkey',
           '--bg-row-hover', '--bg-row-cursor', '--bg-row-sel'],
  },
  {
    label: 'Borders & Accents',
    vars: ['--border', '--border-dim', '--border-panel'],
  },
  {
    label: 'Text',
    vars: ['--text', '--text-dim', '--text-dir', '--text-sel', '--text-danger',
           '--text-fn-num', '--text-fn-label'],
  },
  {
    label: 'State Indicators',
    vars: ['--cursor-outline', '--sel-outline', '--scrollbar-thumb'],
  },
];

const CUSTOM_KEY = 'nyx-custom-themes';

export const THEMES: Theme[] = [
  {
    name: 'neon',
    label: 'Neon',
    vars: {
      '--bg':              '#0a0e27',
      '--bg-panel':        '#0d1130',
      '--bg-header':       '#141430',
      '--bg-row-hover':    '#1a1a4a',
      '--bg-row-cursor':   '#1e2860',
      '--bg-row-sel':      '#062a10',
      '--bg-dialog':       '#0d1130',
      '--bg-fnbar':        '#060818',
      '--bg-fnkey':        '#1a1a3e',
      '--border':          '#00d4ff',
      '--border-dim':      '#1a1a3e',
      '--border-panel':    '#00d4ff',
      '--text':            '#e0e8ff',
      '--text-dim':        '#6878aa',
      '--text-dir':        '#00d4ff',
      '--text-sel':        '#00ff88',
      '--text-danger':     '#ff3355',
      '--text-fn-num':     '#00d4ff',
      '--text-fn-label':   '#c0c8e8',
      '--cursor-outline':  '#00d4ff',
      '--sel-outline':     '#00ff88',
      '--scrollbar-thumb': '#2a2a5e',
    },
  },
  {
    name: 'classic',
    label: 'Classic',
    vars: {
      '--bg':              '#000080',
      '--bg-panel':        '#000098',
      '--bg-header':       '#0000aa',
      '--bg-row-hover':    '#0000b8',
      '--bg-row-cursor':   '#006080',
      '--bg-row-sel':      '#006000',
      '--bg-dialog':       '#000080',
      '--bg-fnbar':        '#000070',
      '--bg-fnkey':        '#00007a',
      '--border':          '#00aaff',
      '--border-dim':      '#000060',
      '--border-panel':    '#00aaff',
      '--text':            '#ffffff',
      '--text-dim':        '#8080ff',
      '--text-dir':        '#00ffff',
      '--text-sel':        '#ffff00',
      '--text-danger':     '#ff4444',
      '--text-fn-num':     '#000000',
      '--text-fn-label':   '#ffffff',
      '--cursor-outline':  '#00ffff',
      '--sel-outline':     '#ffff00',
      '--scrollbar-thumb': '#000060',
    },
  },
  {
    name: 'matrix',
    label: 'Matrix',
    vars: {
      '--bg':              '#000000',
      '--bg-panel':        '#001400',
      '--bg-header':       '#001a00',
      '--bg-row-hover':    '#002200',
      '--bg-row-cursor':   '#003800',
      '--bg-row-sel':      '#004400',
      '--bg-dialog':       '#001200',
      '--bg-fnbar':        '#000800',
      '--bg-fnkey':        '#001800',
      '--border':          '#00ff41',
      '--border-dim':      '#003800',
      '--border-panel':    '#00ff41',
      '--text':            '#00e030',
      '--text-dim':        '#005a10',
      '--text-dir':        '#00ff88',
      '--text-sel':        '#ffffff',
      '--text-danger':     '#ff0000',
      '--text-fn-num':     '#00ff41',
      '--text-fn-label':   '#00cc30',
      '--cursor-outline':  '#00ff41',
      '--sel-outline':     '#ffffff',
      '--scrollbar-thumb': '#003800',
    },
  },
  {
    name: 'amber',
    label: 'Amber',
    vars: {
      '--bg':              '#0d0800',
      '--bg-panel':        '#160d00',
      '--bg-header':       '#1c1000',
      '--bg-row-hover':    '#2a1a00',
      '--bg-row-cursor':   '#3d2200',
      '--bg-row-sel':      '#1a2a00',
      '--bg-dialog':       '#160d00',
      '--bg-fnbar':        '#0a0600',
      '--bg-fnkey':        '#201200',
      '--border':          '#ff9900',
      '--border-dim':      '#3d2200',
      '--border-panel':    '#ff9900',
      '--text':            '#ffcc66',
      '--text-dim':        '#885500',
      '--text-dir':        '#ffaa00',
      '--text-sel':        '#ccff44',
      '--text-danger':     '#ff3300',
      '--text-fn-num':     '#ff9900',
      '--text-fn-label':   '#cc8800',
      '--cursor-outline':  '#ff9900',
      '--sel-outline':     '#ccff44',
      '--scrollbar-thumb': '#3d2200',
    },
  },
];

// ─── Custom themes store ──────────────────────────────────────────────────────

function loadCustomThemes(): Theme[] {
  if (!browser) return [];
  try {
    return JSON.parse(localStorage.getItem(CUSTOM_KEY) ?? '[]') as Theme[];
  } catch {
    return [];
  }
}

export const customThemes = writable<Theme[]>(loadCustomThemes());

export function saveCustomTheme(theme: Theme) {
  customThemes.update(all => {
    const filtered = all.filter(t => t.name !== theme.name);
    const next = [...filtered, theme];
    if (browser) localStorage.setItem(CUSTOM_KEY, JSON.stringify(next));
    return next;
  });
}

export function deleteCustomTheme(name: string) {
  customThemes.update(all => {
    const next = all.filter(t => t.name !== name);
    if (browser) localStorage.setItem(CUSTOM_KEY, JSON.stringify(next));
    return next;
  });
}

// ─── Active theme store ───────────────────────────────────────────────────────

const STORAGE_KEY = 'nyx-theme';

function createThemeStore() {
  const saved = browser ? localStorage.getItem(STORAGE_KEY) : null;
  const initial =
    THEMES.find(t => t.name === saved) ??
    (loadCustomThemes().find(t => t.name === saved)) ??
    THEMES[0];

  const { subscribe, set, update } = writable<Theme>(initial);

  function applyTheme(theme: Theme) {
    if (!browser) return;
    const root = document.documentElement;
    for (const [k, v] of Object.entries(theme.vars)) {
      root.style.setProperty(k, v);
    }
    localStorage.setItem(STORAGE_KEY, theme.name);
  }

  function cycle() {
    update(current => {
      const idx = THEMES.findIndex(t => t.name === current.name);
      const next = THEMES[(idx + 1) % THEMES.length];
      applyTheme(next);
      return next;
    });
  }

  function setTheme(name: ThemeName) {
    // search built-ins first, then custom
    let theme = THEMES.find(t => t.name === name);
    if (!theme) {
      theme = loadCustomThemes().find(t => t.name === name);
    }
    if (!theme) return;
    set(theme);
    applyTheme(theme);
  }

  return { subscribe, cycle, setTheme, applyTheme };
}

export const themeStore = createThemeStore();
