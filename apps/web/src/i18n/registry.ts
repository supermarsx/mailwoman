// Reactive Fluent bundle registry + the `t(id, args)` accessor (plan §4).
//
// There is no official Solid binding for Fluent, so this is the thin wrapper: a
// MODULE-LEVEL reactive store of one `FluentBundle` per locale in the active
// fallback chain. `t()` reads it, so components re-render when a lazily-loaded
// `.ftl` catalog registers new messages. Keeping the store at module scope (not
// only in a context) means:
//   • `t` is a plain importable function — ergonomic for every screen/component,
//   • the test helper can seed the `en` bundle synchronously with no provider,
//   • `LocaleProvider` just DRIVES this store (sets the locale, loads catalogs).
//
// Fluent formatting is synchronous once a bundle holds the message; before a
// catalog loads, `t()` falls back down the chain and finally returns the message
// id itself (visible + debuggable, never throws).

import { createSignal } from 'solid-js';
import { FluentBundle, FluentResource, type FluentVariable } from '@fluent/bundle';
import { mapBundleSync } from '@fluent/sequence';
import { SOURCE_LOCALE, fallbackChain, type Locale } from './locales.ts';

/** Args accepted by `t()` — Fluent variables (string | number | Date | …). */
export type TArgs = Record<string, FluentVariable>;

// `useIsolating: false`: we do NOT want Fluent wrapping every placeable in
// FSI/PDI (that would inject invisible control chars into literal-text test
// assertions and copy/paste). Untrusted values are isolated explicitly via
// `bidi.ts#isolate` at the call site instead (SPEC §24).
function makeBundle(locale: Locale): FluentBundle {
  return new FluentBundle(locale, { useIsolating: false });
}

// One accumulating bundle per locale. Lazily created; catalogs `addResource`
// into these as modules load.
const bundlesByLocale = new Map<Locale, FluentBundle>();

// The reactive fallback chain of bundles for the ACTIVE locale, [active, en].
// A fresh array reference is published on every mutation so Solid re-runs `t`.
const [chain, setChain] = createSignal<FluentBundle[]>([]);
const [activeLocale, setActiveLocaleSignal] = createSignal<Locale>(SOURCE_LOCALE);

function bundleFor(locale: Locale): FluentBundle {
  let b = bundlesByLocale.get(locale);
  if (b === undefined) {
    b = makeBundle(locale);
    bundlesByLocale.set(locale, b);
  }
  return b;
}

/** Republish the active chain (new array ref) so `t()` consumers re-render. */
function publishChain(): void {
  setChain(fallbackChain(activeLocale()).map(bundleFor));
}

/** The currently active locale (reactive). */
export function activeLocaleSignal(): Locale {
  return activeLocale();
}

/** Switch the active locale; (re)builds the reactive fallback chain. */
export function setActiveLocale(locale: Locale): void {
  setActiveLocaleSignal(locale);
  publishChain();
}

/**
 * Register a parsed `.ftl` source into a locale's bundle. Idempotent-ish: Fluent
 * ignores duplicate message ids already present (first definition wins) and
 * reports them in the returned errors, which we swallow — a module loaded twice
 * is a no-op, not a crash. Publishes the chain if the target locale is live.
 */
export function registerResource(locale: Locale, source: string): void {
  const bundle = bundleFor(locale);
  const errors = bundle.addResource(new FluentResource(source), { allowOverrides: false });
  // Junk/parse errors are non-fatal: a malformed line is skipped, the rest of
  // the catalog still loads. Duplicate-id "overrides" are expected on re-load.
  void errors;
  if (fallbackChain(activeLocale()).includes(locale)) publishChain();
}

/**
 * Format message `id` with `args`, walking the active fallback chain. Returns the
 * message id itself if no bundle in the chain defines it (catalog not loaded yet,
 * or a genuinely missing key) — never throws, so a missing string degrades to a
 * visible token rather than a blank screen.
 */
export function t(id: string, args?: TArgs): string {
  const bundles = chain();
  if (bundles.length > 0) {
    const bundle = mapBundleSync(bundles, id);
    if (bundle !== null) {
      const message = bundle.getMessage(id);
      if (message?.value != null) {
        const errors: Error[] = [];
        const out = bundle.formatPattern(message.value, args, errors);
        return out;
      }
    }
  }
  return id;
}

/**
 * TEST/BOOTSTRAP ONLY: seed a locale's bundle synchronously from raw `.ftl`
 * sources and make it the active locale. Used by the `en`-bundle test helper so
 * literal-text assertions resolve against `en` with no async/provider.
 */
export function seedLocaleSync(locale: Locale, sources: Iterable<string>): void {
  const bundle = bundleFor(locale);
  for (const source of sources) {
    bundle.addResource(new FluentResource(source), { allowOverrides: true });
  }
  setActiveLocale(locale);
}

/** TEST ONLY: forget every loaded bundle (isolation between suites if needed). */
export function _resetRegistry(): void {
  bundlesByLocale.clear();
  setActiveLocaleSignal(SOURCE_LOCALE);
  setChain([]);
}
