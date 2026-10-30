// The feature profile editor's model (t98), kept free of the DOM so it is
// tested on its own: grouping and search, per-group "allow all / disallow all /
// clear", the diff against the server default, and reading typed key inputs.

import { PROFILE_FORMAT, PROFILE_FORMAT_VERSION, type FeatureCatalogue, type FeatureDef, type ProfileKeyDef, type ProfileValue } from './api.ts';

export type Values = Record<string, ProfileValue>;

/** The editor groups when the server sends none (an older server). */
export const DEFAULT_CATEGORIES = ['compose', 'reading', 'views', 'organise', 'pim', 'data', 'accounts', 'integrations', 'developer', 'personal'];

export interface FeatureGroup {
  category: string;
  features: FeatureDef[];
}

/** Every key a feature row edits. */
export function keysOf(f: FeatureDef): string[] {
  return [...f.capabilityKeys, ...f.limitKeys, ...(f.settingKeys ?? [])];
}

/** Group the catalogue by category in the server's order; `match` filters rows. */
export function groupFeatures(catalogue: FeatureCatalogue, match: (f: FeatureDef) => boolean): FeatureGroup[] {
  const order = catalogue.categories ?? DEFAULT_CATEGORIES;
  const groups = new Map<string, FeatureDef[]>();
  for (const f of catalogue.catalogue) {
    if (keysOf(f).length === 0 || !match(f)) continue;
    const c = f.category ?? 'other';
    groups.set(c, [...(groups.get(c) ?? []), f]);
  }
  const known = order.filter((c) => groups.has(c)).map((c) => ({ category: c, features: groups.get(c)! }));
  const rest = [...groups.keys()].filter((c) => !order.includes(c)).map((c) => ({ category: c, features: groups.get(c)! }));
  return [...known, ...rest];
}

/** Lower-cased text a search matches against. */
export function haystack(...parts: (string | undefined)[]): string {
  return parts.filter((p): p is string => typeof p === 'string').join(' ').toLowerCase();
}

/** Allow, disallow or clear every boolean switch of `features`. Limits and
 *  other values are only cleared by `clear`. Locks are kept. */
export function applyToGroup(values: Values, features: readonly FeatureDef[], mode: 'on' | 'off' | 'clear'): Values {
  const next = { ...values };
  for (const f of features) {
    for (const key of f.capabilityKeys) {
      if (mode === 'clear') delete next[key];
      else next[key] = { value: mode === 'on', locked: values[key]?.locked ?? false };
    }
    if (mode === 'clear') for (const key of [...f.limitKeys, ...(f.settingKeys ?? [])]) delete next[key];
  }
  return next;
}

const same = (a: unknown, b: unknown): boolean => JSON.stringify(a) === JSON.stringify(b);

/** Which of the profile's values differ from the server default (what a user
 *  with no profile gets), and which repeat it. */
export function diffAgainstDefault(values: Values, defaults: Record<string, unknown> | undefined): { differs: string[]; same: string[] } {
  const differs: string[] = [];
  const equal: string[] = [];
  for (const [key, v] of Object.entries(values)) {
    if (defaults && key in defaults && same(defaults[key], v.value) && !v.locked) equal.push(key);
    else differs.push(key);
  }
  return { differs: differs.sort(), same: equal.sort() };
}

/** Whether `key` is held by the profile with a value other than the default. */
export function keyDiffers(values: Values, defaults: Record<string, unknown> | undefined, key: string): boolean {
  const v = values[key];
  if (v === undefined) return false;
  return !(defaults && key in defaults && same(defaults[key], v.value) && !v.locked);
}

/** The text a key's input shows for a value. */
export function inputText(kind: ProfileKeyDef['kind'] | undefined, value: unknown): string {
  if (value === undefined || value === null) return '';
  if (kind?.type === 'stringList' || kind?.type === 'intList') return Array.isArray(value) ? value.join(', ') : '';
  if (typeof value === 'string') return value;
  return JSON.stringify(value);
}

/** Read a key's input: `undefined` = not set (empty input), `Error` = invalid. */
export function readInput(kind: ProfileKeyDef['kind'] | undefined, text: string): unknown {
  const raw = text.trim();
  if (raw === '') return undefined;
  switch (kind?.type) {
    case 'int': {
      if (!/^-?\d+$/.test(raw)) return new Error('int');
      const n = Number(raw);
      if ((kind.min !== undefined && n < kind.min) || (kind.max !== undefined && n > kind.max)) return new Error('range');
      return n;
    }
    case 'float': {
      const n = Number(raw);
      return Number.isFinite(n) ? n : new Error('float');
    }
    case 'stringList':
      return raw.split(',').map((s) => s.trim()).filter((s) => s !== '');
    case 'intList': {
      const parts = raw.split(',').map((s) => s.trim()).filter((s) => s !== '');
      return parts.every((p) => /^-?\d+$/.test(p)) ? parts.map(Number) : new Error('intList');
    }
    case 'bool':
      return raw === 'true' ? true : raw === 'false' ? false : new Error('bool');
    default:
      return raw;
  }
}

/** Client-side shape check of an imported file before the server validates it. */
export function readDocument(text: string): { ok: true; doc: unknown } | { ok: false; reason: 'json' | 'format' | 'version' } {
  let doc: unknown;
  try {
    doc = JSON.parse(text);
  } catch {
    return { ok: false, reason: 'json' };
  }
  if (doc === null || typeof doc !== 'object' || (doc as { format?: unknown }).format !== PROFILE_FORMAT) return { ok: false, reason: 'format' };
  if ((doc as { version?: unknown }).version !== PROFILE_FORMAT_VERSION) return { ok: false, reason: 'version' };
  return { ok: true, doc };
}

/** A file name for an exported profile. */
export function exportFileName(name: string): string {
  const base = name.normalize('NFKD').toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '').slice(0, 48) || 'profile';
  return `feature-profile-${base}.json`;
}
