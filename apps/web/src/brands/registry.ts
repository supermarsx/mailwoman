// Brand lookup index. Only names, aliases and host suffixes live here (small, in the
// main chunk); the vector paths are in `paths.generated.ts` and load on demand
// (see `loadBrandPath`). Marks are bundled locally: nothing is fetched from a CDN or
// a favicon service, so lookups never leak which providers an operator uses.
import {BRANDS} from './registry.generated.ts';
import type {BrandEntry} from './types.ts';

export type {BrandEntry, BrandCategory} from './types.ts';
export {BRANDS};

const norm = (value: string): string => value.trim().toLowerCase().replace(/\s+/g, ' ');
const byKey = new Map<string, BrandEntry>();
const hostIndex = new Map<string, BrandEntry>();
for (const entry of BRANDS) {
  byKey.set(entry.id, entry);
  byKey.set(norm(entry.name), entry);
}
// Aliases never override an id or a display name.
for (const entry of BRANDS) for (const alias of entry.aliases) if (!byKey.has(norm(alias))) byKey.set(norm(alias), entry);
for (const entry of BRANDS) for (const host of entry.hosts) hostIndex.set(host, entry);

/** Resolve by id, display name, alias, or a bare domain/host (`gmail.com`). */
export function brandFor(key: string | undefined | null): BrandEntry | undefined {
  if (!key) return undefined;
  const k = norm(key);
  return byKey.get(k) ?? brandForHost(k);
}

/**
 * Resolve a host name, URL or e-mail address to a brand by domain suffix:
 * `imap.gmail.com`, `https://s3.eu-west-1.amazonaws.com/b`, `me@outlook.com`.
 * The longest matching suffix wins. Unknown hosts return undefined.
 */
export function brandForHost(input: string | undefined | null): BrandEntry | undefined {
  if (!input) return undefined;
  let host = input.trim().toLowerCase();
  const at = host.lastIndexOf('@');
  if (at >= 0) host = host.slice(at + 1);
  host = host.replace(/^[a-z][a-z0-9+.-]*:\/\//, '').replace(/[/?#].*$/, '').replace(/:\d+$/, '').replace(/\.$/, '');
  if (!host) return undefined;
  const labels = host.split('.');
  for (let i = 0; i < labels.length - 1; i += 1) {
    const hit = hostIndex.get(labels.slice(i).join('.'));
    if (hit) return hit;
  }
  return undefined;
}

let pathModule: Promise<Readonly<Record<string, string>>> | undefined;
/** Load (once) the vector path for a brand id. Resolves undefined for unknown ids. */
export function loadBrandPath(id: string): Promise<string | undefined> {
  pathModule ??= import('./paths.generated.ts').then(module => module.PATHS);
  return pathModule.then(paths => paths[id]);
}

const AMBIGUOUS = new Set(['live', 'teams', 'spaces', 'element', 'gpt', 'proton']);
let phrases: {entry: BrandEntry; re: RegExp}[] | undefined;
const escapeRe = (s: string): string => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
/**
 * Find a brand named inside free text such as a provider label chosen by an
 * operator ("Acme Okta", "Google Workspace login"). Whole-word matches on display
 * names and aliases of four or more characters (minus a few common words), longest first, so short
 * words ("box", "live") never match by accident. Exact keys and hosts win first.
 */
export function brandInText(text: string | undefined | null): BrandEntry | undefined {
  if (!text) return undefined;
  const exact = brandFor(text);
  if (exact) return exact;
  phrases ??= BRANDS.flatMap(entry => [entry.name, ...entry.aliases].map(norm).filter(p => p.length >= 4 && !AMBIGUOUS.has(p)).map(p => ({entry, p})))
    .sort((a, b) => b.p.length - a.p.length)
    .map(({entry, p}) => ({entry, re: new RegExp(String.raw`(^|[^\p{L}\p{N}])` + escapeRe(p) + String.raw`($|[^\p{L}\p{N}])`, 'u')}));
  const hay = norm(text);
  return phrases.find(({re}) => re.test(hay))?.entry;
}
