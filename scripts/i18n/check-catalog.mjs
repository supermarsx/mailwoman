// i18n catalog-completeness gate (1.0 hardening, SPEC §24 / ROADMAP-1.0 L21).
//
// Two things, in one pass:
//   1. RESOLVE GATE (hard-fail): every static-literal `t('id')` used in the web
//      app must resolve to a message id defined in the `en` source catalog.
//      A `t('mail-foo')` with no `mail-foo =` in locales/en/*.ftl is a bug that
//      ships a raw id to the user — this FAILS the build.
//   2. LOCALE-SET GATE (hard-fail): the locales this script reports on are read
//      from `src/i18n/locales.ts` LOCALES — the list the app negotiates against —
//      and must be exactly the directories under `apps/web/locales/`. A locale
//      registered without a catalog dir, or a catalog dir the app never offers,
//      FAILS.
//   3. COVERAGE GATE (t32-admin §2.9): every `en` message id must be defined
//      in every shipped locale OR listed in that locale's `pending` array in
//      apps/web/locales/TRANSLATION-STATUS.json (pending ids render in English
//      through the runtime fallback chain). Ids a locale defines that `en` does
//      not, and pending entries the locale already translates or `en` dropped,
//      are reported too. `scripts/i18n/status.mjs` regenerates the file.
//      Default mode: coverage problems are WARNINGS (t31/t32 lanes add en ids
//      while t32 runs, and t32-e9 regenerates the file). `--strict` (or
//      MW_I18N_STRICT=1) makes them FAIL; t32 stage A4 makes that the default.
//      A missing or malformed TRANSLATION-STATUS.json always FAILS.
//
// `en` is the single source of truth. The runtime (`src/i18n/registry.ts`)
// returns the id itself when a key is missing, so an unresolved id is silent at
// runtime — this gate makes it loud at build time.
//
// Usage: `node scripts/i18n/check-catalog.mjs [--strict]` (exit non-zero on an
// unresolved id, a locale-set mismatch, a bad status file, or — strict — a
// coverage gap).

import { readdir, readFile } from 'node:fs/promises';
import { join, relative } from 'node:path';
import {
  SOURCE,
  idsForLocale,
  localesDir,
  readStatus,
  repoRoot,
  shippedLocales,
  srcDir,
  statusPath,
  walk,
} from './lib/catalog.mjs';

const STRICT = process.argv.includes('--strict') || process.env.MW_I18N_STRICT === '1';

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

// --- gate: the registered locales are exactly the catalog directories -------
const LOCALES = await shippedLocales();
const localeDirs = (await readdir(localesDir, { withFileTypes: true }))
  .filter((d) => d.isDirectory())
  .map((d) => d.name);
const withoutDir = LOCALES.filter((l) => !localeDirs.includes(l));
const unregistered = localeDirs.filter((l) => !LOCALES.includes(l));

// --- coverage gate: each en id translated or pending ------------------------
let status;
try {
  status = await readStatus();
} catch (e) {
  console.error(`\ncheck-catalog: FAIL — ${relative(repoRoot, statusPath)} is not valid JSON: ${e.message}`);
  process.exit(1);
}
if (status === null || typeof status.locales !== 'object' || status.locales === null) {
  console.error(
    `\ncheck-catalog: FAIL — ${relative(repoRoot, statusPath)} is missing or has no "locales" object; ` +
      'run node scripts/i18n/status.mjs',
  );
  process.exit(1);
}

console.log('\ncheck-catalog: per-locale coverage vs en (translated / pending / missing from both):');
const enList = [...enIds];
const coverage = [];
for (const locale of LOCALES) {
  if (locale === SOURCE) continue;
  const ids = await idsForLocale(locale);
  const entry = status.locales[locale];
  const pending = new Set(Array.isArray(entry?.pending) ? entry.pending : []);
  const present = enList.filter((id) => ids.has(id));
  const missing = enList.filter((id) => !ids.has(id) && !pending.has(id));
  const extra = [...ids].filter((id) => !enIds.has(id));
  const stalePending = [...pending].filter((id) => ids.has(id) || !enIds.has(id));
  coverage.push({ locale, missing, extra, stalePending, noEntry: entry === undefined });
  const pct = ((present.length / enList.length) * 100).toFixed(1);
  console.log(
    `  ${locale.padEnd(6)} ${present.length}/${enList.length} (${pct}%) translated, ` +
      `${pending.size} pending, ${missing.length} missing` +
      (entry ? ` — source ${entry.source}, reviewed ${entry.reviewed === true ? 'yes' : 'no'}` : ' — no status entry'),
  );
}
const statusExtra = Object.keys(status.locales).filter((l) => !LOCALES.includes(l) || l === SOURCE);
const coverageProblems = coverage.filter(
  (c) => c.noEntry || c.missing.length > 0 || c.extra.length > 0 || c.stalePending.length > 0,
);
const coverageOk = coverageProblems.length === 0 && statusExtra.length === 0;

// --- verdict ---------------------------------------------------------------
if (withoutDir.length > 0 || unregistered.length > 0) {
  console.error('\ncheck-catalog: FAIL — src/i18n/locales.ts LOCALES and apps/web/locales/ disagree:');
  for (const l of withoutDir) console.error(`  • '${l}' is in LOCALES but has no apps/web/locales/${l}/ directory`);
  for (const l of unregistered) console.error(`  • apps/web/locales/${l}/ exists but '${l}' is not in LOCALES`);
  process.exit(1);
}
if (!coverageOk) {
  const out = STRICT ? console.error : console.warn;
  out(
    STRICT
      ? '\ncheck-catalog: FAIL — locale coverage does not match TRANSLATION-STATUS.json (strict):'
      : '\nWARN locale coverage does not match TRANSLATION-STATUS.json (report-only until t32 stage A4):',
  );
  const sample = (list) => list.slice(0, 5).join(', ') + (list.length > 5 ? `, … (${list.length})` : '');
  for (const l of statusExtra) out(`  • the status file lists '${l}', which is not a shipped non-en locale`);
  for (const c of coverageProblems) {
    if (c.noEntry) out(`  • ${c.locale}: no entry in the status file`);
    if (c.missing.length > 0) {
      out(`  • ${c.locale}: ${c.missing.length} en id(s) neither translated nor pending: ${sample(c.missing)}`);
    }
    if (c.extra.length > 0) out(`  • ${c.locale}: ${c.extra.length} id(s) not in en: ${sample(c.extra)}`);
    if (c.stalePending.length > 0) {
      out(`  • ${c.locale}: ${c.stalePending.length} pending id(s) already translated or gone from en: ${sample(c.stalePending)}`);
    }
  }
  out('  Run node scripts/i18n/status.mjs to regenerate the pending lists.');
  if (STRICT) process.exit(1);
}
if (unresolved.length > 0) {
  console.error(`\ncheck-catalog: FAIL — ${unresolved.length} t() id(s) do not resolve in the en catalog:`);
  for (const { id, sites } of unresolved) {
    const first = sites[0];
    console.error(`  • ${id}  (${first.file}:${first.line}${sites.length > 1 ? ` +${sites.length - 1} more` : ''})`);
  }
  console.error('\nAdd the id to the appropriate locales/en/<module>.ftl, or fix the call site.');
  process.exit(1);
}

console.log(
  `\ncheck-catalog: OK — every static t() id resolves in the en source catalog; ` +
    `all ${LOCALES.length} registered locales have a catalog directory` +
    (coverageOk ? '; every en id is translated or pending in every locale.' : '; coverage warnings above.'),
);
