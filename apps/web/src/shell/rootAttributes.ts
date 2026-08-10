// Root `<html>` attribute wiring for i18n, reduced-motion, and colour scheme
// (plan §6 e0, SPEC §17.1/§24).
//
// Three concerns, all driven onto `document.documentElement`:
//   • `lang` + `dir` — the active locale and its writing direction, so the whole
//     document (and the browser's own bidi handling, spellcheck, form controls)
//     reads correctly and mirrors under RTL.
//   • `data-reduced-motion` — a root flag mirroring `prefers-reduced-motion` so
//     both CSS (`:root[data-reduced-motion] …`) and JS (feature checks) can gate
//     animations. The theme layer ALSO switches motion tokens purely via the
//     media query (themes.css.ts); this flag is the JS-observable companion.
//   • `data-appearance` + the `color-scheme` style — whether the active theme
//     paints a light or a dark page, so the UA themes its own surfaces
//     (scrollbars, form controls, the canvas behind the app) to match.
//
// The colour-scheme WATCHER here is the OS-follow primitive: the theme slice
// subscribes to it so `prefers-color-scheme` changes take effect live rather
// than at the next reload.
//
// SSR/jsdom-safe: every DOM touch is guarded, so importing this in a unit test
// (or a non-browser build) is inert.

import type { Dir } from '../i18n/locales.ts';

const hasDoc = (): boolean => typeof document !== 'undefined';

const DARK_QUERY = '(prefers-color-scheme: dark)';

/** Matching helper that survives jsdom/older shells without `matchMedia`. */
function mediaQuery(query: string): MediaQueryList | null {
  if (typeof matchMedia !== 'function') return null;
  try {
    return matchMedia(query);
  } catch {
    /* jsdom without a matchMedia implementation */
    return null;
  }
}

/** Set `<html lang dir>` from the active locale + resolved direction. */
export function syncRootLangDir(lang: string, dir: Dir): void {
  if (!hasDoc()) return;
  const root = document.documentElement;
  root.setAttribute('lang', lang);
  root.setAttribute('dir', dir);
}

/**
 * Reflect `prefers-reduced-motion: reduce` onto `<html data-reduced-motion>` and
 * keep it live. Returns a cleanup that removes the listener. No-op (returns a
 * noop cleanup) where `matchMedia` is unavailable (jsdom, older shells).
 */
export function watchReducedMotion(): () => void {
  if (!hasDoc()) return () => undefined;
  const root = document.documentElement;
  const mq = mediaQuery('(prefers-reduced-motion: reduce)');
  if (mq === null) return () => undefined;
  const apply = (): void => {
    if (mq.matches) root.setAttribute('data-reduced-motion', '');
    else root.removeAttribute('data-reduced-motion');
  };
  apply();
  // `addEventListener('change', …)` is the modern API; guard for old engines.
  if (typeof mq.addEventListener === 'function') {
    mq.addEventListener('change', apply);
    return () => mq.removeEventListener('change', apply);
  }
  return () => undefined;
}

/**
 * Current `prefers-color-scheme: dark` state. `false` where the query is
 * unavailable (jsdom, non-browser builds) — light is the safe assumption.
 */
export function prefersDarkScheme(): boolean {
  return mediaQuery(DARK_QUERY)?.matches === true;
}

/**
 * Subscribe to OS colour-scheme changes. `onChange` fires with the new value
 * every time the platform flips, which is what makes the `system` theme mode
 * follow live instead of only at boot. Returns a cleanup; a no-op cleanup where
 * `matchMedia` (or change notification) is unavailable.
 */
export function watchColorScheme(onChange: (dark: boolean) => void): () => void {
  const mq = mediaQuery(DARK_QUERY);
  if (mq === null || typeof mq.addEventListener !== 'function') return () => undefined;
  const handler = (ev: MediaQueryListEvent): void => onChange(ev.matches);
  mq.addEventListener('change', handler);
  return () => mq.removeEventListener('change', handler);
}

/**
 * Reflect the active theme's light/dark nature onto the root element:
 * `data-appearance` for CSS/JS, and the `color-scheme` style so the UA paints
 * its own widgets (scrollbars, date pickers, the canvas behind the app) to
 * match. Without this a dark theme still gets a white browser canvas on
 * overscroll and light-rendered native controls.
 */
export function syncRootAppearance(appearance: 'light' | 'dark'): void {
  if (!hasDoc()) return;
  const root = document.documentElement;
  root.setAttribute('data-appearance', appearance);
  root.style.setProperty('color-scheme', appearance);
}
