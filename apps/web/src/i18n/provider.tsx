// `LocaleProvider` — the Solid context that boots + drives the i18n runtime.
//
// The `t()` accessor and the bundle registry live at module scope (registry.ts),
// so components can `import { t }` without threading context. This provider owns
// the *lifecycle*: negotiate the active locale, register the critical catalog,
// push `lang`/`dir` onto `<html>`, and mirror reduced-motion. It also exposes a
// context (`useI18n`) for the reactive bits a component wants to READ (current
// locale, direction) or DO (switch locale).
//
// Mount it once at the app root, wrapping the whole tree:
//   render(() => <LocaleProvider><App/></LocaleProvider>, root)

import {
  createContext,
  useContext,
  createEffect,
  onCleanup,
  type JSX,
  type Accessor,
} from 'solid-js';
import {
  negotiateLocale,
  resolveDir,
  isKnownLocale,
  SOURCE_LOCALE,
  type Locale,
  type Dir,
} from './locales.ts';
import { t, activeLocaleSignal, setActiveLocale, type TArgs } from './registry.ts';
import { reloadForActiveChain } from './catalog.ts';
import { syncRootLangDir, watchReducedMotion } from '../shell/rootAttributes.ts';

export interface I18nContext {
  /** Format a message id with optional args (the same `t` exported globally). */
  t: (id: string, args?: TArgs) => string;
  /** The active locale (reactive). */
  locale: Accessor<Locale>;
  /** The active writing direction (reactive). */
  dir: Accessor<Dir>;
  /** Switch the active locale; reloads touched catalogs + repaints. */
  setLocale: (locale: Locale) => void;
}

const Ctx = createContext<I18nContext>();

/** Preferred locales from the browser, falling back to `['en']` off-browser. */
function requestedLocales(): readonly string[] {
  if (typeof navigator !== 'undefined' && Array.isArray(navigator.languages)) {
    return navigator.languages;
  }
  return [SOURCE_LOCALE];
}

export interface LocaleProviderProps {
  children: JSX.Element;
  /** Force a locale (tests/storybook/shell override); otherwise negotiated. */
  locale?: Locale;
}

export function LocaleProvider(props: LocaleProviderProps): JSX.Element {
  const initial: Locale = props.locale ?? negotiateLocale(requestedLocales());
  setActiveLocale(initial);

  const locale = activeLocaleSignal;
  const dir = (): Dir => resolveDir(locale());

  const setLocale = (next: Locale): void => {
    if (!isKnownLocale(next)) return;
    setActiveLocale(next);
    void reloadForActiveChain();
  };

  // Push lang/dir onto <html> whenever the locale changes.
  createEffect(() => {
    syncRootLangDir(locale(), dir());
  });

  // Mirror prefers-reduced-motion onto the root for the lifetime of the app.
  const stop = watchReducedMotion();
  onCleanup(stop);

  const value: I18nContext = { t, locale, dir, setLocale };
  return <Ctx.Provider value={value}>{props.children}</Ctx.Provider>;
}

/** Read the i18n context. Throws if used outside a `<LocaleProvider>`. */
export function useI18n(): I18nContext {
  const ctx = useContext(Ctx);
  if (ctx === undefined) {
    throw new Error('useI18n must be used within <LocaleProvider>');
  }
  return ctx;
}
