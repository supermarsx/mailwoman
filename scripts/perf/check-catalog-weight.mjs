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
//   1. Only the `common` module may appear on the critical path (entry +
//      modulepreload). Any OTHER module's catalog found in a critical chunk
//      FAILS — it leaked off the lazy path.
//   2. The eager `en/common.ftl` source stays under a small ceiling so the one
//      allowed critical catalog can't balloon.
//   3. The entry chunk stays < 250 KB gzip (defence-in-depth; reported so a
//      catalog-driven regression is visible even if run standalone).
//
// Detection is by each catalog's VERBATIM header comment (`# Mailwoman — …`),
// which the `?raw` import embeds byte-for-byte and which never appears in app
// code — so it pinpoints exactly which chunk bundles a given catalog with no
// false positives from `t('id')` call sites (those carry ids, not the raw file).
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
const ALLOWED_CRITICAL_MODULES = new Set(['common']);

const kb = (n) => `${(n / 1024).toFixed(1)} KB`;

// --- critical path from dist/index.html (entry + modulepreload) --------------
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
const criticalNames = new Set([entryName, ...preloadNames]);

// --- load every JS chunk once (utf8, so embedded catalog text compares clean) -
let assetFiles;
try {
  assetFiles = (await readdir(assetsDir)).filter((f) => f.endsWith('.js'));
} catch {
  console.error('check-catalog-weight: dist/assets not found — run `pnpm -C apps/web build` first');
  process.exit(1);
}
const chunks = await Promise.all(
  assetFiles.map(async (f) => ({ f, text: await readFile(join(assetsDir, f), 'utf8') })),
);

// --- enumerate every catalog and fingerprint it by its header comment --------
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

let failed = false;

// A catalog's header comment (`# Mailwoman — <module> …`) is identical across
// every locale of that module (translations of the same source), so the header
// fingerprint identifies the MODULE, not the (locale, module) pair. That is
// exactly the right altitude: the invariant is "only the `common` MODULE may be
// eager; every feature MODULE must be lazy" — a leak of any locale of `mail`
// onto the entry is caught as `mail` on the critical path. Per-locale weight is
// backstopped by the 250 KB entry-gzip budget below. So we check once per
// module.
function fingerprintOf(src) {
  // Bundlers escape non-ASCII (the em-dash `—` → `—`), so match on the
  // header's longest ASCII-only run — present verbatim wherever `?raw` bundles
  // the catalog, and distinctive per module.
  const headerLine = src.split(/\r?\n/).find((l) => l.startsWith('#'));
  if (!headerLine) return undefined;
  return (headerLine.match(/[\x20-\x7E]{12,}/g) ?? []).sort((a, b) => b.length - a.length)[0];
}

const byModule = new Map();
for (const c of catalogs) {
  const m = byModule.get(c.module) ?? { module: c.module, locales: [] };
  m.locales.push(c.locale);
  if (!m.path) m.path = c.path;
  byModule.set(c.module, m);
}

console.log('check-catalog-weight: catalog module placement (critical path = entry + modulepreload):');
for (const m of [...byModule.values()].sort((a, b) => a.module.localeCompare(b.module))) {
  const src = await readFile(m.path, 'utf8');
  const fingerprint = fingerprintOf(src);
  if (!fingerprint) {
    console.log(`  ${m.module}: (no ASCII header fingerprint — skipped)`);
    continue;
  }
  const carrying = chunks.filter((ch) => ch.text.includes(fingerprint)).map((ch) => ch.f);
  const inCritical = carrying.filter((f) => criticalNames.has(f));
  const where =
    carrying.length === 0
      ? 'lazy/absent (off critical path)'
      : inCritical.length > 0
        ? `CRITICAL: ${inCritical.join(', ')}`
        : `lazy chunk: ${carrying.join(', ')}`;
  console.log(`  ${m.module} (${m.locales.length} locale${m.locales.length === 1 ? '' : 's'}): ${where}`);
  if (inCritical.length > 0 && !ALLOWED_CRITICAL_MODULES.has(m.module)) {
    console.error(
      `check-catalog-weight: LEAK — the '${m.module}' catalog rides the CRITICAL path (${inCritical.join(', ')}). ` +
        `Feature catalogs must be lazy (loadCatalog('${m.module}')), not statically imported.`,
    );
    failed = true;
  }
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
    console.error('check-catalog-weight: en/common.ftl exceeds the eager-catalog ceiling — move strings to a lazy module catalog');
    failed = true;
  }
} catch {
  console.error('check-catalog-weight: apps/web/locales/en/common.ftl missing (expected the eager critical catalog)');
  failed = true;
}

// --- gate 3: entry gzip < 250 KB (defence-in-depth; "entry didn't regress") --
const entryChunk = chunks.find((ch) => ch.f === entryName);
if (entryChunk) {
  const entryGz = gzipSync(Buffer.from(entryChunk.text, 'utf8')).length;
  console.log(
    `check-catalog-weight: entry ${entryName} = ${kb(entryGz)} gzip (budget ${ENTRY_BUDGET_BYTES / 1024} KB)`,
  );
  if (entryGz > ENTRY_BUDGET_BYTES) {
    console.error('check-catalog-weight: entry over the 250 KB gzip budget');
    failed = true;
  }
}

if (failed) {
  console.error('\ncheck-catalog-weight: FAIL — i18n catalog weight regressed onto the critical path (SPEC §23).');
  process.exit(1);
}
console.log('\ncheck-catalog-weight: OK — only en/common is eager; all feature catalogs stay lazy.');
