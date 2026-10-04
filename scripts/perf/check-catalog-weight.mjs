// Fluent-catalog-weight guard (SPEC §23 / plan §6 t8-e5-perf, risk #3).
//
// The 250 KB entry budget (apps/web/scripts/check-size.mjs) measures the
// login→inbox critical path. i18n catalogs must NOT silently ride it: per
// src/i18n/catalog.ts only `en/common.ftl` is statically imported (rides the
// entry, intended), while every other `locales/<loc>/<module>.ftl` is a lazy
// `import.meta.glob(..., '?raw')` chunk pulled on demand. If a translator adds a
// static import — or a feature area eagerly bundles its catalog — a dozen
// locales of strings can quietly inflate the entry.
//
// This guard asserts that invariant directly, WITHOUT re-implementing the size
// gate:
//   1. Only `en/common.ftl` may appear on the critical path. Any OTHER catalog
//      file found in a critical chunk FAILS — it leaked off the lazy path.
//   2. The eager `en/common.ftl` source stays under a small ceiling so the one
//      allowed critical catalog can't balloon.
//   3. The entry chunk stays < 250 KB gzip (defence-in-depth; reported so a
//      catalog-driven regression is visible even if run standalone).
//
// The critical path is the entry `<script type="module">` of dist/index.html,
// any `<link rel="modulepreload">` beside it, and every chunk those import
// STATICALLY, transitively. The closure is computed from the built files, not
// assumed, and the run prints how many chunks each of the three contributed.
//
// Detection is per catalog FILE, by lines of that file which occur in no other
// catalog file. A `?raw` import embeds the file's text in the chunk, so such a
// line identifies exactly which chunk carries which `<locale>/<module>.ftl`;
// `t('id')` call sites carry ids, never a whole catalog line. Two properties
// keep the detector from going blind without anyone noticing:
//   * a catalog with no line of its own cannot be told apart from another one,
//     and FAILS rather than being skipped;
//   * every catalog must be found in at least one chunk (the lazy glob in
//     catalog.ts bundles all of them) and `en/common` must be found on the
//     critical path (it is statically imported). A fingerprint that matches
//     nothing FAILS: it means the build is older than locales/ or the detector
//     cannot see that catalog, and a leak of it would be invisible.
//
// Run after `pnpm -C apps/web build`:  node scripts/perf/check-catalog-weight.mjs

import { readFile, readdir } from 'node:fs/promises';
import { gzipSync } from 'node:zlib';
import { join, basename } from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = fileURLToPath(new URL('.', import.meta.url));
const webRoot = join(scriptDir, '..', '..', 'apps', 'web');
const distDir = join(webRoot, 'dist');
const assetsDir = join(distDir, 'assets');
const localesDir = join(webRoot, 'locales');

const ENTRY_BUDGET_BYTES = 250 * 1024;
// The single eager catalog (en/common). A generous ceiling on its source — it is
// meant for genuinely cross-cutting strings only (buttons, states, errors).
const EAGER_COMMON_CEILING_BYTES = 8 * 1024;
// The one catalog file that is statically imported (src/i18n/catalog.ts).
const EAGER_CATALOG = 'en/common';
// How many of a catalog's own lines to look for. One would do; several let the
// match survive an edit to any single line between the build and the check.
const FINGERPRINT_LINES = 5;
// Shorter lines are too likely to occur in application code by coincidence.
const MIN_FINGERPRINT_LENGTH = 16;

const kb = (n) => `${(n / 1024).toFixed(1)} KB`;

let failed = false;
const fail = (msg) => {
  console.error(`check-catalog-weight: ${msg}`);
  failed = true;
};

// --- critical-path roots from dist/index.html (entry + any modulepreload) ----
let html;
try {
  html = await readFile(join(distDir, 'index.html'), 'utf8');
} catch {
  console.error('check-catalog-weight: dist/index.html not found — run `pnpm -C apps/web build` first');
  process.exit(1);
}
const toAssetName = (href) => href.replace(/^.*\/assets\//, '').replace(/^.*\//, '');
const entryMatch = html.match(/<script[^>]*\btype="module"[^>]*\bsrc="([^"]+)"/);
if (!entryMatch) {
  console.error('check-catalog-weight: no <script type="module"> entry in index.html');
  process.exit(1);
}
const entryName = toAssetName(entryMatch[1]);
const preloadNames = [...html.matchAll(/<link[^>]*\brel="modulepreload"[^>]*\bhref="([^"]+)"/g)].map(
  (m) => toAssetName(m[1]),
);

// --- load every JS chunk once ------------------------------------------------
let assetFiles;
try {
  assetFiles = (await readdir(assetsDir)).filter((f) => f.endsWith('.js'));
} catch {
  console.error('check-catalog-weight: dist/assets not found — run `pnpm -C apps/web build` first');
  process.exit(1);
}
// The bundler writes non-ASCII as `\uXXXX` / `\xXX` escapes inside the string
// that holds a catalog (the em-dash becomes `\u2014`). Undo those so a catalog
// line compares equal whether or not the bundler escaped it.
const unescapeJs = (text) =>
  text.replace(/\\u\{([0-9a-fA-F]{1,6})\}|\\u([0-9a-fA-F]{4})|\\x([0-9a-fA-F]{2})/g, (_, a, b, c) =>
    String.fromCodePoint(parseInt(a ?? b ?? c, 16)),
  );
const chunks = await Promise.all(
  assetFiles.map(async (f) => {
    const raw = await readFile(join(assetsDir, f));
    return { f, gzip: gzipSync(raw).length, text: unescapeJs(raw.toString('utf8')) };
  }),
);
const chunkByName = new Map(chunks.map((ch) => [ch.f, ch]));
if (!chunkByName.has(entryName)) {
  console.error(`check-catalog-weight: entry ${entryName} named by index.html is not in dist/assets`);
  process.exit(1);
}

// --- critical path: the roots plus everything they statically import ---------
// `import x from"./a.js"`, `import"./a.js"` and `export … from"./a.js"` are
// static; `import("./a.js")` is not, and is deliberately not followed.
const STATIC_IMPORT = /(?:\bfrom|\bimport)\s*["']([^"']+\.js)["']/g;
const rootNames = new Set([entryName, ...preloadNames]);
const criticalNames = new Set();
const pending = [...rootNames];
while (pending.length > 0) {
  const name = pending.pop();
  if (criticalNames.has(name)) continue;
  const chunk = chunkByName.get(name);
  if (!chunk) continue;
  criticalNames.add(name);
  for (const m of chunk.text.matchAll(STATIC_IMPORT)) pending.push(toAssetName(m[1]));
}

// --- enumerate every catalog file --------------------------------------------
async function ftlFiles() {
  const out = [];
  let locales;
  try {
    locales = await readdir(localesDir, { withFileTypes: true });
  } catch {
    return out;
  }
  for (const d of locales) {
    if (!d.isDirectory()) continue;
    const locale = d.name;
    let files;
    try {
      files = await readdir(join(localesDir, locale));
    } catch {
      continue;
    }
    for (const f of files) {
      if (f.endsWith('.ftl')) out.push({ locale, module: basename(f, '.ftl'), path: join(localesDir, locale, f) });
    }
  }
  return out;
}

const catalogs = await ftlFiles();
if (catalogs.length === 0) {
  console.error('check-catalog-weight: no catalogs found under apps/web/locales — nothing to guard');
  process.exit(1);
}

// --- gate 1: fingerprint every catalog file, then place it --------------------
// A line is usable only if the bundler embeds it verbatim: quotes, backticks,
// backslashes and `$` may be escaped depending on the kind of string literal it
// chooses, so lines containing them are not used.
const usable = (line) => line.length >= MIN_FINGERPRINT_LENGTH && !/[`'"\\$]/.test(line);
const owners = new Map(); // line -> Set of the catalog keys that contain it
for (const c of catalogs) {
  c.key = `${c.locale}/${c.module}`;
  const lines = (await readFile(c.path, 'utf8')).split(/\r?\n/).map((l) => l.trimEnd());
  c.lines = [...new Set(lines)].filter(usable);
  for (const line of c.lines) {
    if (!owners.has(line)) owners.set(line, new Set());
    owners.get(line).add(c.key);
  }
}

for (const c of catalogs) {
  // Each fingerprint line occurs in this file and no other, so a match cannot
  // be mistaken for a different module or a different locale.
  c.fingerprints = c.lines.filter((l) => owners.get(l).size === 1).slice(0, FINGERPRINT_LINES);
  c.carrying = [];
  c.inCritical = [];
  if (c.fingerprints.length === 0) {
    fail(
      `UNFINGERPRINTABLE — ${c.key}.ftl has no line of its own (every line of ${MIN_FINGERPRINT_LENGTH}+ ` +
        `characters also occurs in another catalog), so a leak of it could not be told apart from a leak of ` +
        `another. Give it a header line naming its locale and module.`,
    );
    continue;
  }
  c.carrying = chunks.filter((ch) => c.fingerprints.some((fp) => ch.text.includes(fp))).map((ch) => ch.f);
  c.inCritical = c.carrying.filter((f) => criticalNames.has(f));
  if (c.carrying.length === 0) {
    fail(
      `NOT FOUND — ${c.key}.ftl is in no built chunk. src/i18n/catalog.ts bundles every catalog, so either ` +
        `dist/ is older than locales/ (rebuild) or this guard can no longer see that catalog.`,
    );
  }
  if (c.inCritical.length > 0 && c.key !== EAGER_CATALOG) {
    fail(
      `LEAK — ${c.key}.ftl rides the CRITICAL path (${c.inCritical.join(', ')}). ` +
        `Only ${EAGER_CATALOG} may be eager; load this one with loadCatalog('${c.module}').`,
    );
  }
}

// The positive control. en/common is statically imported, so it must be seen on
// the critical path; if it is not, nothing else would be seen there either.
const eager = catalogs.find((c) => c.key === EAGER_CATALOG);
if (eager && eager.carrying.length > 0 && eager.inCritical.length === 0) {
  fail(
    `${EAGER_CATALOG}.ftl was not found on the critical path, where src/i18n/catalog.ts statically imports ` +
      `it. It is this guard's positive control: a leak could not be seen there either.`,
  );
}

console.log(
  `check-catalog-weight: critical path = ${criticalNames.size} chunk(s): the entry ${entryName}, ` +
    `${preloadNames.length} modulepreload link(s), ${criticalNames.size - rootNames.size} more by static import`,
);
console.log(`check-catalog-weight: placement of ${catalogs.length} catalog files, by module:`);
const byModule = new Map();
for (const c of catalogs) {
  if (!byModule.has(c.module)) byModule.set(c.module, []);
  byModule.get(c.module).push(c);
}
for (const [module, files] of [...byModule].sort(([a], [b]) => a.localeCompare(b))) {
  const critical = files.filter((c) => c.inCritical.length > 0);
  const lazy = files.filter((c) => c.carrying.some((f) => !criticalNames.has(f)));
  const unplaced = files.filter((c) => c.carrying.length === 0);
  const parts = [`${lazy.length} in lazy chunks`];
  if (critical.length > 0) {
    parts.push(`CRITICAL: ${critical.map((c) => `${c.locale} in ${c.inCritical.join(', ')}`).join('; ')}`);
  }
  if (unplaced.length > 0) parts.push(`UNPLACED: ${unplaced.map((c) => c.locale).join(', ')}`);
  console.log(`  ${module} (${files.length} locale${files.length === 1 ? '' : 's'}): ${parts.join(' · ')}`);
}

// --- gate 2: the one eager catalog (en/common) stays small -------------------
try {
  const enCommon = await readFile(join(localesDir, 'en', 'common.ftl'));
  const gz = gzipSync(enCommon).length;
  console.log(
    `\ncheck-catalog-weight: eager en/common.ftl = ${enCommon.length} B raw, ${gz} B gzip ` +
      `(ceiling ${EAGER_COMMON_CEILING_BYTES} B gzip)`,
  );
  if (gz > EAGER_COMMON_CEILING_BYTES) {
    fail('en/common.ftl exceeds the eager-catalog ceiling — move strings to a lazy module catalog');
  }
} catch {
  fail('apps/web/locales/en/common.ftl missing (expected the eager critical catalog)');
}

// --- gate 3: entry gzip < 250 KB (defence-in-depth; "entry didn't regress") --
const entryGz = chunkByName.get(entryName).gzip;
console.log(`check-catalog-weight: entry ${entryName} = ${kb(entryGz)} gzip (budget ${ENTRY_BUDGET_BYTES / 1024} KB)`);
if (entryGz > ENTRY_BUDGET_BYTES) fail('entry over the 250 KB gzip budget');

if (failed) {
  console.error('\ncheck-catalog-weight: FAIL — see the messages above (SPEC §23 catalog invariant).');
  process.exit(1);
}
console.log('\ncheck-catalog-weight: OK — en/common is the only catalog on the critical path; every other one is lazy.');
