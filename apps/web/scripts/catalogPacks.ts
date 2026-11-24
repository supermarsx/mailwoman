// Translation packs: the production build ships two files per locale instead of one
// file per namespace. An `app` pack holds every namespace a signed-in user can
// need; an `admin` pack holds the namespaces only the admin console loads and is
// fetched when that console opens. A pack is `{namespace: ftl text}`, so each
// namespace stays its own FluentResource at runtime.
//
// The English fallback is merged here, at build time: in a non-English pack every
// message or term that the locale does not define in ANY of its namespaces is
// appended, with its English text, to the pack's copy of that namespace. The
// runtime then needs only the active locale's pack. This reproduces the lookup
// the registry performs on the chain [locale, en]: a message id is taken from the
// locale when any of its namespaces defines it, otherwise from English.
import {readdirSync, readFileSync} from 'node:fs';
import {join} from 'node:path';

export type PackKind = 'app' | 'admin';
export const SOURCE_LOCALE = 'en';

/**
 * Namespaces only the admin console loads. Reviewed by hand: a namespace that is
 * not listed is an app namespace, so a new `admin-*.ftl` file ships in the app
 * pack until it is added here (catalogPacks.test.mjs lists the unclassified names).
 */
export const ADMIN_NAMESPACES: readonly string[] = [
  'admin', 'admin-admins', 'admin-analytics', 'admin-branding', 'admin-breaches', 'admin-cache',
  'admin-compliance', 'admin-consents', 'admin-console-url-mail', 'admin-cookies', 'admin-dashboard', 'admin-data-requests',
  'admin-database', 'admin-diagnostics', 'admin-directory', 'admin-domain-backends', 'admin-domain-guided',
  'admin-domains', 'admin-encryption', 'admin-erasure', 'admin-features', 'admin-geoip', 'admin-governance',
  'admin-hosting', 'admin-identity', 'admin-inbound', 'admin-maintenance', 'admin-metrics-persisted',
  'admin-passkeys', 'admin-personal', 'admin-policy', 'admin-privacy-speed', 'admin-requests',
  'admin-restrictions', 'admin-retention', 'admin-role-purpose', 'admin-security', 'admin-settings', 'admin-ssh', 'admin-stats',
  'admin-telemetry', 'admin-users', 'admin-ux', 'admin-webhooks',
];

/**
 * `admin-*` namespaces that screens outside the admin console load, and the pack
 * they were given. admin-shell (18 KB) is in the app pack (the empty-folder dialog
 * uses it). admin-settings (302 KB in English, about 70 KB gzip of every app pack)
 * stays in the admin pack, so Settings > Transfer fetches the admin pack when it
 * opens; moving it here is one line plus this list.
 */
export const APP_RESIDENT_ADMIN_NAMESPACES: readonly string[] = ['admin-shell'];
export const ADMIN_PACK_NAMESPACES_LOADED_OUTSIDE_ADMIN: readonly string[] = ['admin-settings'];

const adminSet = new Set(ADMIN_NAMESPACES);
export function packKind(namespace: string): PackKind {
  return adminSet.has(namespace) ? 'admin' : 'app';
}

const ENTRY_START = /^(-?[A-Za-z][A-Za-z0-9_-]*)[ \t]*=/;

/**
 * The entries (messages and terms) of one Fluent source, id -> verbatim block.
 * An entry starts at a line whose first column holds `id =` and runs to the next
 * line that starts in the first column; continuation lines (patterns, attributes,
 * selectors) are always indented in Fluent, and blank lines inside a pattern are
 * followed by an indented line.
 */
export function splitEntries(source: string): Map<string, string> {
  const lines = source.split(/\r?\n/);
  const out = new Map<string, string>();
  let id: string | null = null;
  let block: string[] = [];
  const flush = (): void => {
    if (id === null) return;
    while (block.length > 1 && block[block.length - 1]!.trim() === '') block.pop();
    if (!out.has(id)) out.set(id, block.join('\n'));
    id = null;
    block = [];
  };
  for (const line of lines) {
    const start = ENTRY_START.exec(line);
    if (start) {
      flush();
      id = start[1]!;
      block = [line];
    } else if (id !== null && (line.trim() === '' || /^[ \t]/.test(line))) {
      block.push(line);
    } else {
      flush();
    }
  }
  flush();
  return out;
}

export type LocaleSources = Map<string, Map<string, string>>;

/** locale -> namespace -> ftl text, for every locale directory under `localesDir`. */
export function readLocaleSources(localesDir: string): LocaleSources {
  const out: LocaleSources = new Map();
  const locales = readdirSync(localesDir, {withFileTypes: true}).filter((entry) => entry.isDirectory()).map((entry) => entry.name).sort();
  for (const locale of locales) {
    const files = new Map<string, string>();
    for (const name of readdirSync(join(localesDir, locale)).filter((file) => file.endsWith('.ftl')).sort()) {
      files.set(name.slice(0, -'.ftl'.length), readFileSync(join(localesDir, locale, name), 'utf8'));
    }
    out.set(locale, files);
  }
  return out;
}

/** Every namespace of a locale after the English merge: its own files plus English-only files. */
export function packNamespaces(sources: LocaleSources, locale: string): string[] {
  const names = new Set<string>([...(sources.get(locale)?.keys() ?? []), ...(sources.get(SOURCE_LOCALE)?.keys() ?? [])]);
  return [...names].sort();
}

/**
 * One locale's namespaces with the English fallback merged in. English itself is
 * returned unchanged (as the text of each file).
 */
export function mergeEnglishFallback(sources: LocaleSources, locale: string): Map<string, string> {
  const own = sources.get(locale) ?? new Map<string, string>();
  const english = sources.get(SOURCE_LOCALE) ?? new Map<string, string>();
  const merged = new Map<string, string>();
  if (locale === SOURCE_LOCALE) {
    for (const [namespace, text] of english) merged.set(namespace, text);
    return merged;
  }
  const defined = new Set<string>();
  for (const text of own.values()) for (const id of splitEntries(text).keys()) defined.add(id);
  for (const namespace of packNamespaces(sources, locale)) {
    let text = own.get(namespace) ?? '';
    const missing: string[] = [];
    for (const [id, block] of splitEntries(english.get(namespace) ?? '')) if (!defined.has(id)) missing.push(block);
    if (missing.length > 0) text = `${text}${text === '' || text.endsWith('\n') ? '' : '\n'}\n${missing.join('\n')}\n`;
    merged.set(namespace, text);
  }
  return merged;
}

export interface Pack {
  locale: string;
  kind: PackKind;
  /** Namespace -> ftl text, keys sorted. */
  files: Record<string, string>;
}

/** The packs of every locale: app and admin, in locale order. */
export function buildPacks(sources: LocaleSources): Pack[] {
  const packs: Pack[] = [];
  for (const locale of sources.keys()) {
    const merged = mergeEnglishFallback(sources, locale);
    for (const kind of ['app', 'admin'] as const) {
      const files: Record<string, string> = {};
      for (const namespace of [...merged.keys()].sort()) if (packKind(namespace) === kind) files[namespace] = merged.get(namespace)!;
      packs.push({locale, kind, files});
    }
  }
  return packs;
}

/** Deterministic file content of a pack. */
export function serializePack(pack: Pack): string {
  return JSON.stringify(pack.files);
}

/** The asset name a pack is emitted under; the build adds `-<hash>` before `.json`. */
export function packAssetName(locale: string, kind: PackKind): string {
  return `catalog.${locale}.${kind}.json`;
}

/** Build output of an app pack (`assets/catalog.<locale>.app-<hash>.json`): precached; admin packs are not. */
export function isAppCatalogPack(path: string): boolean {
  return /^\/assets\/catalog\.[^./]+\.app-[^/]+\.json$/.test(path);
}
