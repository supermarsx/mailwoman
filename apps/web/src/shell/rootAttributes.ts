// Root `<html>` attribute wiring for i18n, accessibility preferences, and colour scheme
// (plan §6 e0, SPEC §17.1/§24).
//
// Preferences driven onto `document.documentElement`:
//   • `lang` + `dir` — the active locale and its writing direction, so the whole
//     document (and the browser's own bidi handling, spellcheck, form controls)
//     reads correctly and mirrors under RTL.
//   • `data-reduced-motion` — a root flag mirroring `prefers-reduced-motion` so
//     both CSS (`:root[data-reduced-motion] …`) and JS (feature checks) can gate
//     animations. The theme layer ALSO switches motion tokens purely via the
//     media query (themes.css.ts); this flag is the JS-observable companion.
//   • `data-reduced-texture` — optional reduced-transparency/data preferences,
//     with the browser's Save-Data setting as an additional data-saver signal.
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

/** Optional preferences stay out of static CSS so unsupported media features
 * do not produce stylesheet parse warnings. Unknown queries simply do not match. */
export function watchReducedTexture(): () => void {
  if (!hasDoc()) return () => undefined;
  const root = document.documentElement;
  const queries = ['(prefers-reduced-transparency: reduce)', '(prefers-reduced-data: reduce)']
    .map(mediaQuery).filter((query): query is MediaQueryList => query !== null);
  type Connection = {readonly saveData?: boolean; addEventListener?: EventTarget['addEventListener']; removeEventListener?: EventTarget['removeEventListener']};
  const connection = typeof navigator === 'undefined' ? undefined : (navigator as Navigator & {connection?: Connection}).connection;
  const apply = (): void => {
    root.toggleAttribute('data-reduced-texture', connection?.saveData === true || queries.some(query => query.matches === true));
  };
  const cleanup: (() => void)[] = [];
  for (const query of queries) {
    if (typeof query.addEventListener === 'function' && typeof query.removeEventListener === 'function') {
      query.addEventListener('change', apply);
      cleanup.push(() => query.removeEventListener('change', apply));
    } else if (typeof query.addListener === 'function' && typeof query.removeListener === 'function') {
      query.addListener(apply);
      cleanup.push(() => query.removeListener(apply));
    }
  }
  if (typeof connection?.addEventListener === 'function' && typeof connection.removeEventListener === 'function') {
    connection.addEventListener('change', apply);
    cleanup.push(() => connection.removeEventListener?.('change', apply));
  }
  apply();
  return () => {for (const stop of cleanup) stop(); root.removeAttribute('data-reduced-texture');};
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
