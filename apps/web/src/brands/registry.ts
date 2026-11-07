// Brand lookup index. Only names, aliases and host suffixes live here (small, in the
// main chunk); the vector paths are in `paths.generated.ts` and load on demand
// (see `loadBrandPath`). Marks are bundled locally: nothing is fetched from a CDN or
// a favicon service, so lookups never leak which providers an operator uses.
import {BRANDS} from './registry.generated.ts';
import {LOADERS} from './loaders.generated.ts';
import type {BrandCategory, BrandEntry} from './types.ts';

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
export function brandFor(key: string | undefined | null, categories?: readonly BrandCategory[]): BrandEntry | undefined {
  if (!key) return undefined;
  const k = norm(key);
  const hit = byKey.get(k) ?? brandForHost(k);
  return hit && (!categories || categories.includes(hit.category)) ? hit : undefined;
}

/**
 * Categories for operator-chosen provider labels (single sign-on, AI profiles): names such as
 * "Delta" or "Target" are real brands, but never what a sign-in provider called that means.
 */
export const PROVIDER_CATEGORIES: readonly BrandCategory[] = ['identity', 'mail', 'productivity', 'tech', 'cloud', 'storage', 'app', 'security', 'ai', 'analytics', 'push', 'social'];

/**
 * Resolve a host name, URL or e-mail address to a brand by domain suffix:
 * `imap.gmail.com`, `https://s3.eu-west-1.amazonaws.com/b`, `me@outlook.com`.
 * The longest matching suffix wins. Unknown hosts return undefined.
 */
export function brandForHost(input: string | undefined | null): BrandEntry | undefined {
  if (!input) return undefined;
  // A list shows hundreds of rows from a handful of senders: memoise per input (bounded).
  if (hostMemo.has(input)) return hostMemo.get(input);
  const hit = resolveHost(input);
  if (hostMemo.size >= 4000) hostMemo.clear();
  hostMemo.set(input, hit);
  return hit;
}
const hostMemo = new Map<string, BrandEntry | undefined>();

function resolveHost(input: string): BrandEntry | undefined {
  // Scheme off, then the authority (up to the first / ? #), then any user@ prefix: so a lookalike
  // hidden in a path or query ("evil.io/?u=@paypal.com") is never read as the host.
  let host = input.trim().toLowerCase().replace(/^[a-z][a-z0-9+.-]*:\/\//, '').replace(/[/?#].*$/, '');
  host = host.slice(host.lastIndexOf('@') + 1).replace(/:\d+$/, '').replace(/\.$/, '');
  if (!host) return undefined;
  const labels = host.split('.');
  for (let i = 0; i < labels.length - 1; i += 1) {
    const hit = hostIndex.get(labels.slice(i).join('.'));
    if (hit) return hit;
  }
  return undefined;
}

const chunks = new Map<string, Promise<Readonly<Record<string, string>>>>();
/** Load (once per category chunk) the vector path for a brand id. Resolves undefined when the brand has no bundled mark. */
export function loadBrandPath(id: string): Promise<string | undefined> {
  const entry = byKey.get(id);
  const load = entry && !entry.noMark ? LOADERS[entry.category] : undefined;
  if (!entry || !load) return Promise.resolve(undefined);
  let chunk = chunks.get(entry.category);
  if (!chunk) { chunk = load().then(module => module.PATHS); chunks.set(entry.category, chunk); }
  return chunk.then(paths => paths[entry.id]);
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
export function brandInText(text: string | undefined | null, categories?: readonly BrandCategory[]): BrandEntry | undefined {
  if (!text) return undefined;
  const exact = brandFor(text, categories);
  if (exact) return exact;
  phrases ??= BRANDS.flatMap(entry => [entry.name, ...entry.aliases].map(norm).filter(p => p.length >= 4 && !AMBIGUOUS.has(p)).map(p => ({entry, p})))
    .sort((a, b) => b.p.length - a.p.length)
    .map(({entry, p}) => ({entry, re: new RegExp(String.raw`(^|[^\p{L}\p{N}])` + escapeRe(p) + String.raw`($|[^\p{L}\p{N}])`, 'u')}));
  const hay = norm(text);
  return phrases.find(({entry, re}) => (!categories || categories.includes(entry.category)) && re.test(hay))?.entry;
}
