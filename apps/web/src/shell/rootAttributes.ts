// Root `<html>` attribute wiring for i18n + reduced-motion (plan §6 e0, SPEC §24).
//
// Two concerns, both driven onto `document.documentElement`:
//   • `lang` + `dir` — the active locale and its writing direction, so the whole
//     document (and the browser's own bidi handling, spellcheck, form controls)
//     reads correctly and mirrors under RTL.
//   • `data-reduced-motion` — a root flag mirroring `prefers-reduced-motion` so
//     both CSS (`:root[data-reduced-motion] …`) and JS (feature checks) can gate
//     animations. The theme layer ALSO switches motion tokens purely via the
//     media query (themes.css.ts); this flag is the JS-observable companion.
//
// SSR/jsdom-safe: every DOM touch is guarded, so importing this in a unit test
// (or a non-browser build) is inert.

import type { Dir } from '../i18n/locales.ts';

const hasDoc = (): boolean => typeof document !== 'undefined';

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
  if (!hasDoc() || typeof matchMedia !== 'function') return () => undefined;
  const root = document.documentElement;
  const mq = matchMedia('(prefers-reduced-motion: reduce)');
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
