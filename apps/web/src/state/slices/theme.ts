// Theme slice (plan §3 e4): design-token theme/density/accent/font state, the
// `data-theme`/`data-density` + inline `--mw-accent`/`--mw-ui-font` runtime
// switch, and the ribbon layout preset. Persisted to localStorage for V2.
//
// PERSISTENCE: no per-user settings endpoint exists yet, so prefs live in
// localStorage. Engine-side settings sync is an e9/e10 follow-up — swap the
// `loadPrefs`/`savePrefs` pair to move the backend without touching the rest.
//
// Importing the style modules here (for their side effects) is what pulls the
// vanilla-extract themes + @font-face + print CSS into the bundle graph, since
// the store is reachable from `main.tsx`.
import '../../theme/themes.css.ts';
import '../../styles/fonts.css.ts';
import '../../styles/print.css.ts';

import { createSignal, type Accessor } from 'solid-js';
import type { ThemeName, Density } from '../../theme/contract.css.ts';
import type { SliceContext } from './context.ts';

export type LayoutMode = 'default' | 'ribbon';
export type UiFont = 'default' | 'system' | 'serif' | 'mono';

/** UI-font override stacks; `null` = remove the override (use the theme font). */
const FONT_STACKS: Record<UiFont, string | null> = {
  default: null,
  system: 'system-ui, -apple-system, "Segoe UI", Roboto, sans-serif',
  serif: '"Newsreader", Georgia, "Times New Roman", serif',
  mono: '"JetBrains Mono", ui-monospace, Consolas, monospace',
};

const THEME_VALUES: readonly ThemeName[] = [
  'light',
  'dark',
  'hc-light',
  'hc-dark',
  'amoled',
  'grove-light',
  'grove-dark',
];
const DENSITY_VALUES: readonly Density[] = ['compact', 'cozy', 'relaxed'];

interface Prefs {
  theme: ThemeName;
  density: Density;
  accent: string;
  font: UiFont;
  layout: LayoutMode;
  ribbonCollapsed: boolean;
}

const STORAGE_KEY = 'mw.theme.prefs';

/** The theme portion of `AppState`. */
export interface ThemeSlice {
  theme: Accessor<ThemeName>;
  density: Accessor<Density>;
  accent: Accessor<string>;
  uiFont: Accessor<UiFont>;
  layout: Accessor<LayoutMode>;
  ribbonCollapsed: Accessor<boolean>;
  setTheme(t: ThemeName): void;
  setDensity(d: Density): void;
  setAccent(hex: string): void;
  setUiFont(f: UiFont): void;
  setLayout(l: LayoutMode): void;
  setRibbonCollapsed(v: boolean): void;
}

function systemTheme(): ThemeName {
  if (typeof window !== 'undefined' && typeof window.matchMedia === 'function') {
    try {
      if (window.matchMedia('(prefers-color-scheme: dark)').matches) return 'dark';
    } catch {
      /* jsdom without matchMedia */
    }
  }
  return 'light';
}

function defaults(): Prefs {
  return {
    theme: systemTheme(),
    density: 'cozy',
    accent: '',
    font: 'default',
    layout: 'default',
    ribbonCollapsed: false,
  };
}

function loadPrefs(): Prefs {
  const base = defaults();
  if (typeof localStorage === 'undefined') return base;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw === null) return base;
    const p = JSON.parse(raw) as Partial<Prefs>;
    return {
      theme: THEME_VALUES.includes(p.theme as ThemeName) ? (p.theme as ThemeName) : base.theme,
      density: DENSITY_VALUES.includes(p.density as Density) ? (p.density as Density) : base.density,
      accent: typeof p.accent === 'string' ? p.accent : base.accent,
      font: p.font !== undefined && p.font in FONT_STACKS ? p.font : base.font,
      layout: p.layout === 'ribbon' ? 'ribbon' : 'default',
      ribbonCollapsed: p.ribbonCollapsed === true,
    };
  } catch {
    return base;
  }
}

function savePrefs(p: Prefs): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(p));
  } catch {
    /* private mode / quota — prefs are best-effort */
  }
}

/** Reflect prefs onto :root (attributes + inline var overrides). */
function apply(p: Prefs): void {
  if (typeof document === 'undefined') return;
  const root = document.documentElement;
  root.setAttribute('data-theme', p.theme);
  root.setAttribute('data-density', p.density);
  if (p.accent !== '') root.style.setProperty('--mw-accent', p.accent);
  else root.style.removeProperty('--mw-accent');
  const stack = FONT_STACKS[p.font];
  if (stack !== null) root.style.setProperty('--mw-ui-font', stack);
  else root.style.removeProperty('--mw-ui-font');
}

export function createThemeSlice(_ctx: SliceContext): ThemeSlice {
  const initial = loadPrefs();
  apply(initial);

  const [theme, setThemeSig] = createSignal<ThemeName>(initial.theme);
  const [density, setDensitySig] = createSignal<Density>(initial.density);
  const [accent, setAccentSig] = createSignal(initial.accent);
  const [uiFont, setUiFontSig] = createSignal<UiFont>(initial.font);
  const [layout, setLayoutSig] = createSignal<LayoutMode>(initial.layout);
  const [ribbonCollapsed, setRibbonCollapsedSig] = createSignal(initial.ribbonCollapsed);

  function snapshot(): Prefs {
    return {
      theme: theme(),
      density: density(),
      accent: accent(),
      font: uiFont(),
      layout: layout(),
      ribbonCollapsed: ribbonCollapsed(),
    };
  }

  function persist(): void {
    const p = snapshot();
    apply(p);
    savePrefs(p);
  }

  return {
    theme,
    density,
    accent,
    uiFont,
    layout,
    ribbonCollapsed,
    setTheme(t) {
      setThemeSig(t);
      persist();
    },
    setDensity(d) {
      setDensitySig(d);
      persist();
    },
    setAccent(hex) {
      setAccentSig(hex);
      persist();
    },
    setUiFont(f) {
      setUiFontSig(f);
      persist();
    },
    setLayout(l) {
      setLayoutSig(l);
      persist();
    },
    setRibbonCollapsed(v) {
      setRibbonCollapsedSig(v);
      persist();
    },
  };
}
