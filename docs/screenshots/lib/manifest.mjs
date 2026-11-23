// Manifest files: docs/screenshots/manifest/<area>.json (core: _core.json). A file
// is either an array of entries or { "defaults": {...}, "entries": [...] }; each
// entry is merged over the defaults. See README.md for the fields.
import { createHash } from 'node:crypto';
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join, relative } from 'node:path';
import { MANIFESTS, OUTPUT, REPO } from './paths.mjs';

export const VIEWPORTS = {
  desktop: { width: 1440, height: 900 },
  narrow: { width: 1024, height: 768 },
  phone: { width: 390, height: 844 },
};

export const BUDGET = { full: 220 * 1024, crop: 80 * 1024, total: 30 * 1024 * 1024 };

/** The area name of a manifest file ("_core.json" -> "core"). */
export const areaOf = (file) => file.replace(/\.json$/, '').replace(/^_/, '');

const STEP_KINDS = new Set([
  'goto', 'click', 'dblclick', 'rightclick', 'hover', 'focus', 'fill', 'type', 'press', 'check', 'uncheck',
  'select', 'waitFor', 'waitForHidden', 'wait', 'scroll', 'eval', 'setting', 'reload', 'keys',
]);
const ENTRY_KEYS = new Set([
  'id', 'page', 'stack', 'seed', 'viewport', 'theme', 'session', 'clock', 'steps', 'shot', 'callouts', 'mask',
  'alt', 'watch', 'settle', 'scale', 'isolate', 'quality', 'note', 'hide',
]);

/** A short stable hash of an entry (for --changed-since and the run metadata). */
export function entryHash(entry) {
  return createHash('sha256').update(JSON.stringify(entry)).digest('hex').slice(0, 16);
}

/** Problems with one merged entry (empty when valid). */
export function validate(entry) {
  const problems = [];
  const say = (m) => problems.push(`${entry.id ?? '(no id)'}: ${m}`);
  for (const key of Object.keys(entry)) if (!ENTRY_KEYS.has(key) && !key.startsWith('_')) say(`unknown field '${key}'`);
  if (!/^[a-z0-9][a-z0-9-]*$/.test(entry.id ?? '')) say('id must be lowercase letters, digits and dashes');
  if (entry.page !== null && (typeof entry.page !== 'string' || !entry.page.endsWith('.md'))) say("page must be a docs-relative .md path or null");
  if (!['M', 'E'].includes(entry.stack)) say("stack must be 'M' or 'E'");
  if (!Array.isArray(entry.seed)) say('seed must be an array of seed names');
  if (typeof entry.viewport === 'string' ? !VIEWPORTS[entry.viewport] : !(entry.viewport?.width > 0 && entry.viewport?.height > 0)) {
    say(`viewport must be ${Object.keys(VIEWPORTS).join('/')} or {width,height}`);
  }
  if (!['dark', 'light'].includes(entry.theme)) say("theme must be 'dark' or 'light'");
  if (!['user', 'admin', 'none'].includes(entry.session)) say("session must be 'user', 'admin' or 'none'");
  if (typeof entry.alt !== 'string' || entry.alt.trim().length < 10) say('alt text is required (a sentence saying what is shown)');
  else if (/^(a )?screenshot/i.test(entry.alt.trim())) say("alt text says what is shown; do not start with 'Screenshot'");
  if (!Array.isArray(entry.steps)) say('steps must be an array');
  else {
    entry.steps.forEach((step, i) => {
      const kinds = Object.keys(step).filter((k) => STEP_KINDS.has(k));
      if (kinds.length !== 1) say(`step ${i + 1} must have exactly one of ${[...STEP_KINDS].join(', ')}`);
    });
  }
  const shot = entry.shot;
  if (!(shot === 'full' || (shot && typeof shot === 'object' && (shot.element || shot.clip)))) say("shot must be 'full', {element} or {clip}");
  for (const c of entry.callouts ?? []) {
    if (!Number.isInteger(c.n) || c.n < 1) say('callout n must be a positive integer');
    if (!c.target) say(`callout ${c.n} needs a target locator`);
  }
  return problems;
}

/** The kind of image an entry produces, for the size budget. */
export const kindOf = (entry) => (entry.shot === 'full' ? 'full' : 'crop');

/** docs/public/screenshots/<area>/<id>.webp */
export const outputOf = (area, id) => join(OUTPUT, area, `${id}.webp`);
/** The URL a page uses: /screenshots/<area>/<id>.webp */
export const urlOf = (area, id) => `/screenshots/${area}/${id}.webp`;

const DEFAULTS = { stack: 'M', seed: [], viewport: 'desktop', theme: 'dark', session: 'user', steps: [], callouts: [], mask: [] };

/** Parse one manifest file into merged entries. */
export function readManifest(file) {
  const raw = JSON.parse(readFileSync(file, 'utf8'));
  const { defaults = {}, entries } = Array.isArray(raw) ? { entries: raw } : raw;
  if (!Array.isArray(entries)) throw new Error(`${relative(REPO, file)}: expected an array or {defaults, entries}`);
  return entries.map((e) => ({ ...DEFAULTS, ...defaults, ...e }));
}

/** Every manifest: [{ area, file, entries }]. `only` limits to some areas. */
export function loadManifests(only) {
  if (!existsSync(MANIFESTS)) return [];
  return readdirSync(MANIFESTS)
    .filter((f) => f.endsWith('.json'))
    .sort()
    .map((f) => ({ area: areaOf(f), file: join(MANIFESTS, f), entries: readManifest(join(MANIFESTS, f)) }))
    .filter((m) => !only || only.includes(m.area));
}

/** Validation problems across manifests (ids unique per area, valid fields). */
export function validateAll(manifests) {
  const problems = [];
  for (const { area, entries } of manifests) {
    const seen = new Set();
    for (const entry of entries) {
      problems.push(...validate(entry).map((p) => `${area}/${p}`));
      if (seen.has(entry.id)) problems.push(`${area}/${entry.id}: duplicate id`);
      seen.add(entry.id);
    }
  }
  return problems;
}
