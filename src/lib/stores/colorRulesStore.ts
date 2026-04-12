import { writable, derived } from 'svelte/store';

export interface ColorRule {
  id: string;
  enabled: boolean;
  label: string;
  // match criteria (OR-logic among non-empty fields)
  matchExtensions: string;   // comma-sep, e.g. "rs,toml,lock"
  matchNamePattern: string;  // glob-ish substring, e.g. "Makefile"
  matchMinSize: number;      // 0 = ignore
  matchMaxSize: number;      // 0 = ignore
  // style
  fg: string;    // CSS color or empty
  bg: string;    // CSS color or empty
  bold: boolean;
  italic: boolean;
}

const STORAGE_KEY = 'nyx-color-rules';

function load(): ColorRule[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : defaultRules();
  } catch {
    return defaultRules();
  }
}

function save(rules: ColorRule[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(rules));
}

function defaultRules(): ColorRule[] {
  return [
    {
      id: 'arc',
      enabled: true,
      label: 'Archives',
      matchExtensions: 'zip,tar,gz,bz2,xz,tgz,tbz2,txz,rar,7z,jar,war,apk',
      matchNamePattern: '',
      matchMinSize: 0,
      matchMaxSize: 0,
      fg: '#ffcc00',
      bg: '',
      bold: false,
      italic: false,
    },
    {
      id: 'media',
      enabled: true,
      label: 'Media',
      matchExtensions: 'mp4,mkv,avi,mov,mp3,flac,ogg,wav,aac,m4a,m4v,webm',
      matchNamePattern: '',
      matchMinSize: 0,
      matchMaxSize: 0,
      fg: '#cc88ff',
      bg: '',
      bold: false,
      italic: false,
    },
    {
      id: 'img',
      enabled: true,
      label: 'Images',
      matchExtensions: 'png,jpg,jpeg,gif,svg,webp,ico,bmp,tiff,heic',
      matchNamePattern: '',
      matchMinSize: 0,
      matchMaxSize: 0,
      fg: '#00cccc',
      bg: '',
      bold: false,
      italic: false,
    },
    {
      id: 'exec',
      enabled: true,
      label: 'Executables',
      matchExtensions: 'sh,bash,zsh,fish,ps1,bat,exe,dylib,so',
      matchNamePattern: 'Makefile,makefile,Rakefile,Guardfile',
      matchMinSize: 0,
      matchMaxSize: 0,
      fg: '#00ff88',
      bg: '',
      bold: true,
      italic: false,
    },
    {
      id: 'big',
      enabled: true,
      label: 'Large files (>100 MB)',
      matchExtensions: '',
      matchNamePattern: '',
      matchMinSize: 100 * 1024 * 1024,
      matchMaxSize: 0,
      fg: '#ff6644',
      bg: '',
      bold: false,
      italic: false,
    },
  ];
}

function createColorRulesStore() {
  const { subscribe, update, set } = writable<ColorRule[]>(load());

  function addRule() {
    update(rules => {
      const r: ColorRule = {
        id: Math.random().toString(36).slice(2, 8),
        enabled: true,
        label: 'New rule',
        matchExtensions: '',
        matchNamePattern: '',
        matchMinSize: 0,
        matchMaxSize: 0,
        fg: '#ffffff',
        bg: '',
        bold: false,
        italic: false,
      };
      const next = [...rules, r];
      save(next);
      return next;
    });
  }

  function updateRule(id: string, patch: Partial<ColorRule>) {
    update(rules => {
      const next = rules.map(r => r.id === id ? { ...r, ...patch } : r);
      save(next);
      return next;
    });
  }

  function removeRule(id: string) {
    update(rules => {
      const next = rules.filter(r => r.id !== id);
      save(next);
      return next;
    });
  }

  function reorder(fromIdx: number, toIdx: number) {
    update(rules => {
      const next = [...rules];
      const [item] = next.splice(fromIdx, 1);
      next.splice(toIdx, 0, item);
      save(next);
      return next;
    });
  }

  return { subscribe, addRule, updateRule, removeRule, reorder };
}

export const colorRulesStore = createColorRulesStore();

// ── Match helper ──────────────────────────────────────────────────────────────

export interface AppliedStyle {
  fg: string;
  bg: string;
  bold: boolean;
  italic: boolean;
}

export function applyColorRule(
  rules: ColorRule[],
  entry: { name: string; extension: string; size: number; is_dir: boolean },
): AppliedStyle | null {
  if (entry.is_dir) return null;

  for (const rule of rules) {
    if (!rule.enabled) continue;

    let matched = false;

    if (rule.matchExtensions) {
      const exts = rule.matchExtensions.split(',').map(e => e.trim().toLowerCase());
      if (exts.includes(entry.extension.toLowerCase())) matched = true;
    }

    if (!matched && rule.matchNamePattern) {
      const patterns = rule.matchNamePattern.split(',').map(p => p.trim());
      if (patterns.some(p => entry.name.toLowerCase().includes(p.toLowerCase()))) matched = true;
    }

    if (!matched && rule.matchMinSize > 0 && entry.size >= rule.matchMinSize) matched = true;
    if (!matched && rule.matchMaxSize > 0 && entry.size <= rule.matchMaxSize) matched = true;

    if (matched) {
      return { fg: rule.fg, bg: rule.bg, bold: rule.bold, italic: rule.italic };
    }
  }
  return null;
}
