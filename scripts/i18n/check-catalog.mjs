// i18n catalog-completeness gate (1.0 hardening, SPEC §24 / ROADMAP-1.0 L21).
//
// Two things, in one pass:
//   1. RESOLVE GATE (hard-fail): every static-literal `t('id')` used in the web
//      app must resolve to a message id defined in the `en` source catalog.
//      A `t('mail-foo')` with no `mail-foo =` in locales/en/*.ftl is a bug that
//      ships a raw id to the user — this FAILS the build.
//   2. COVERAGE REPORT (informational): for each of the 12 shipped locales, how
//      many of the `en` message ids it defines. Non-`en` catalogs are populated
//      by human translators via Weblate (out of autonomous scope), so a partial
//      or empty non-`en` catalog is EXPECTED and never fails — missing keys fall
//      back to `en` at runtime. This is a visibility signal for translators, not
//      a gate.
//
// `en` is the single source of truth. The runtime (`src/i18n/registry.ts`)
// returns the id itself when a key is missing, so an unresolved id is silent at
// runtime — this gate makes it loud at build time.
//
// Usage: `node scripts/i18n/check-catalog.mjs` (exit non-zero on unresolved id).

import { readdir, readFile } from 'node:fs/promises';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = fileURLToPath(new URL('../../', import.meta.url));
const localesDir = join(repoRoot, 'apps/web/locales');
const srcDir = join(repoRoot, 'apps/web/src');

// The 12 shipped locales (mirror of src/i18n/locales.ts LOCALES). `en` is source.
const LOCALES = ['en', 'de', 'fr', 'es', 'pt-BR', 'nl', 'it', 'pl', 'ru', 'uk', 'zh', 'ja'];
const SOURCE = 'en';

/** Recursively list files under `dir` matching `filter`. */
async function walk(dir, filter, out = []) {
  let entries;
  try {
    entries = await readdir(dir, { withFileTypes: true });
  } catch {
    return out;
  }
  for (const e of entries) {
    const p = join(dir, e.name);
    if (e.isDirectory()) {
      if (e.name === 'node_modules' || e.name === 'dist') continue;
      await walk(p, filter, out);
    } else if (filter(e.name)) {
      out.push(p);
    }
  }
  return out;
}

/**
 * Parse the top-level Fluent message ids from a `.ftl` source. Message ids start
 * at column 0 (attributes and select variants are indented, so `startsWith`
 * whitespace filters them); Fluent TERMS begin with `-` and are not `t()`-addressable.
 * Comments (`#`) and blank lines are skipped.
 */
function parseFtlIds(source) {
  const ids = new Set();
  for (const rawLine of source.split(/\r?\n/)) {
    if (rawLine.length === 0 || /^\s/.test(rawLine)) continue; // indented / blank
    if (rawLine.startsWith('#') || rawLine.startsWith('-')) continue; // comment / term
    const m = rawLine.match(/^([a-zA-Z][a-zA-Z0-9_-]*)\s*=/);
    if (m) ids.add(m[1]);
  }
  return ids;
}

/** All message ids defined for a locale (union across its module catalogs). */
async function idsForLocale(locale) {
  const dir = join(localesDir, locale);
  const files = await walk(dir, (n) => n.endsWith('.ftl'));
  const ids = new Set();
  for (const f of files) {
    const src = await readFile(f, 'utf8');
    for (const id of parseFtlIds(src)) ids.add(id);
  }
  return ids;
}

/**
 * Collect every STATIC-LITERAL `t('id' | "id")` used in the app source, with the
 * file + line for diagnostics. Dynamic ids (`t(variable)`, `t(`x-${y}`)`) are
 * intentionally skipped — they cannot be statically resolved and are the callers'
 * responsibility. Test files and the i18n runtime itself (which deliberately
 * references non-existent ids to prove the id-fallback) are excluded.
 */
async function collectUsedIds() {
  const files = await walk(
    srcDir,
    (n) => (n.endsWith('.tsx') || n.endsWith('.ts')) && !/\.test\.tsx?$/.test(n),
  );
  // `t('kebab-id')` — the id charset matches the .ftl grammar. Also catches the
  // aliased `import { t } from '../test/i18n'` form (same call shape).
  const callRe = /\bt\(\s*(['"])([a-zA-Z][a-zA-Z0-9_-]*)\1/g;
  const used = new Map(); // id -> [{ file, line }]
  for (const f of files) {
    // Skip the i18n runtime dir (its tests/among sources probe missing ids on purpose).
    if (f.includes(join('src', 'i18n'))) continue;
    const src = await readFile(f, 'utf8');
    const lines = src.split(/\r?\n/);
    lines.forEach((line, i) => {
      let m;
      callRe.lastIndex = 0;
      while ((m = callRe.exec(line)) !== null) {
        const id = m[2];
        if (!used.has(id)) used.set(id, []);
        used.get(id).push({ file: relative(repoRoot, f), line: i + 1 });
      }
    });
  }
  return used;
}

// ---------------------------------------------------------------------------

const enIds = await idsForLocale(SOURCE);
if (enIds.size === 0) {
  console.error('check-catalog: no `en` message ids found — is locales/en/ populated?');
  process.exit(1);
}
console.log(`check-catalog: en source catalog defines ${enIds.size} message ids.`);

// --- gate: every static t('id') resolves in en -----------------------------
const used = await collectUsedIds();
const unresolved = [];
for (const [id, sites] of used) {
  if (!enIds.has(id)) unresolved.push({ id, sites });
}
console.log(`check-catalog: app uses ${used.size} distinct static t() ids.`);

// --- coverage report (informational) ---------------------------------------
console.log('\ncheck-catalog: per-locale coverage vs en (translated via Weblate; non-en is informational):');
const enList = [...enIds];
for (const locale of LOCALES) {
  const ids = locale === SOURCE ? enIds : await idsForLocale(locale);
  const present = enList.filter((id) => ids.has(id)).length;
  const pct = ((present / enList.length) * 100).toFixed(1);
  const bar = locale === SOURCE ? 'source' : `${present}/${enList.length} (${pct}%)`;
  console.log(`  ${locale.padEnd(6)} ${bar}`);
}

// --- verdict ---------------------------------------------------------------
if (unresolved.length > 0) {
  console.error(`\ncheck-catalog: FAIL — ${unresolved.length} t() id(s) do not resolve in the en catalog:`);
  for (const { id, sites } of unresolved) {
    const first = sites[0];
    console.error(`  • ${id}  (${first.file}:${first.line}${sites.length > 1 ? ` +${sites.length - 1} more` : ''})`);
  }
  console.error('\nAdd the id to the appropriate locales/en/<module>.ftl, or fix the call site.');
  process.exit(1);
}

console.log('\ncheck-catalog: OK — every static t() id resolves in the en source catalog.');
