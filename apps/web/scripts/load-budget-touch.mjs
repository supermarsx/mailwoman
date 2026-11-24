// `pnpm run size:touch`: confirm check-load-budget's invalidation numbers with
// real rebuilds, the t39x audit's method. For each probe in load-budget.json
// that names a source module, build once untouched and once with one statement
// appended to that module (MW_LOAD_TOUCH, scripts/chunkPlan.ts touchPlugin),
// then compare the hashed file names under assets/.
//
// Slow (one build per probe plus one), so not part of `pnpm run size`. It
// overwrites the build caches in node_modules/.cache/mailwoman-build: run
// `pnpm build` again before `pnpm run size`.
//
// Fails when a probe renames more hashed files than its maxRenamed, renames a
// file of a forbidden chunk or a worker, or when the renamed JS chunks differ
// from the static prediction.
import { spawnSync } from 'node:child_process';
import { copyFileSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { invalidated, listDist } from './check-load-budget.mjs';
import { readLocaleSources, splitEntries } from './catalogPacks.ts';

const web = fileURLToPath(new URL('..', import.meta.url));
const budget = JSON.parse(readFileSync(new URL('./load-budget.json', import.meta.url), 'utf8'));
const reportPath = join(web, 'node_modules/.cache/mailwoman-build/chunk-plan.json');
const vite = join(web, 'node_modules/vite/bin/vite.js');
const work = mkdtempSync(join(tmpdir(), 'mw-load-touch-'));
const probes = (budget.probes ?? []).filter((probe) => probe.module !== undefined && !probe.module.includes('?'));
const catalogProbes = (budget.probes ?? []).filter((probe) => probe.catalog !== undefined);

function build(name, touch) {
  const outDir = join(work, name);
  const env = { ...process.env };
  delete env.MW_LOAD_TOUCH;
  if (touch) env.MW_LOAD_TOUCH = touch;
  console.log(`size:touch: building ${name}${touch ? ` (${touch} + 1 line)` : ''}`);
  const result = spawnSync(process.execPath, [vite, 'build', '--outDir', outDir, '--emptyOutDir', '--logLevel', 'error'], { cwd: web, env, stdio: 'inherit' });
  if (result.status !== 0) throw new Error(`size:touch: build ${name} failed`);
  return outDir;
}

const allHashed = (dir) => new Set(listDist(dir).filter((file) => file.startsWith('assets/') && /-[A-Za-z0-9_-]{8}\.[a-z0-9]+$/.test(file)));
const hashed = (dir) => new Set(listDist(dir).filter((file) => file.startsWith('assets/') && /\.(?:m?js|css)$/.test(file)));

let failed = false;
try {
  const base = build('base');
  copyFileSync(reportPath, join(work, 'base-report.json'));
  const report = JSON.parse(readFileSync(join(work, 'base-report.json'), 'utf8'));
  const before = hashed(base);
  const otherJs = (budget.otherJs ?? []).map((pattern) => new RegExp(pattern));
  probes.forEach((probe, index) => {
    const start = Object.keys(report.chunks).find((file) => report.chunks[file].modules.includes(probe.module));
    if (start === undefined) { console.error(`size:touch: FAIL ${probe.module} is in no chunk`); failed = true; return; }
    const predicted = invalidated(report.chunks, start);
    const after = hashed(build(`probe-${index}`, probe.module));
    const renamed = [...before].filter((file) => !after.has(file));
    const js = renamed.filter((file) => report.chunks[file]);
    const workers = renamed.filter((file) => /\.m?js$/.test(file) && !report.chunks[file]);
    const css = renamed.filter((file) => file.endsWith('.css'));
    const plans = js.map((file) => report.chunks[file].plan).sort();
    console.log(`size:touch: ${probe.module}: ${renamed.length} hashed files renamed (JS chunks: ${plans.join(', ') || 'none'}; CSS: ${css.length}; other JS: ${workers.join(', ') || 'none'}); predicted ${predicted.size}`);
    const missed = js.filter((file) => !predicted.has(file));
    const extra = [...predicted].filter((file) => !js.includes(file));
    if (missed.length + extra.length > 0) {
      failed = true;
      console.error(`size:touch: FAIL static prediction differs: renamed but not predicted ${missed.map((f) => report.chunks[f].plan).join(', ') || '-'}; predicted but kept ${extra.map((f) => report.chunks[f].plan).join(', ') || '-'}`);
    }
    if (probe.maxRenamed !== undefined && renamed.length > probe.maxRenamed) { failed = true; console.error(`size:touch: FAIL ${renamed.length} > maxRenamed ${probe.maxRenamed}`); }
    const forbidden = plans.filter((plan) => (probe.forbid ?? []).includes(plan));
    if (forbidden.length > 0) { failed = true; console.error(`size:touch: FAIL renamed forbidden chunks ${forbidden.join(', ')}`); }
    const workerRenames = workers.filter((file) => otherJs.some((pattern) => pattern.test(file)) && /worker/i.test(file));
    if (probe.forbidWorkers && workerRenames.length > 0) { failed = true; console.error(`size:touch: FAIL renamed worker files ${workerRenames.join(', ')}`); }
  });
  // One-line edits of an English translation file. Editing a message every other
  // locale translates renames the English pack only; a new English message is
  // merged into every locale's pack as the fallback, so it renames them all.
  // No JavaScript is renamed either way (the pack table is in index.html).
  const sources = readLocaleSources(join(web, 'locales'));
  const catalogBuild = (probe, edit, label) => {
    const [locale, namespace] = probe.catalog.split('/');
    const file = join(web, 'locales', locale, `${namespace}.ftl`);
    const original = readFileSync(file, 'utf8');
    let after;
    try {
      writeFileSync(file, edit(original));
      after = allHashed(build(label));
    } finally {
      writeFileSync(file, original);
    }
    return [...baseAll].filter((name) => !after.has(name));
  };
  const baseAll = allHashed(base);
  for (const [index, probe] of catalogProbes.entries()) {
    const [locale, namespace] = probe.catalog.split('/');
    const mine = splitEntries(sources.get(locale).get(namespace));
    const translated = [...mine].find(([id, block]) => [...sources.keys()].every((other) => other === locale || (splitEntries(sources.get(other).get(namespace) ?? '').get(id) ?? block) !== block));
    if (translated === undefined) { failed = true; console.error(`size:touch: FAIL ${probe.catalog} has no message every locale translates`); continue; }
    const [id] = translated;
    const edited = catalogBuild(probe, (text) => text.replace(new RegExp(`^(${id}[ \t]*=[^\r\n]*)`, 'm'), '$1 (touch)'), `catalog-edit-${index}`);
    console.log(`size:touch: locales/${probe.catalog}.ftl, ${id} edited in English: ${edited.length} hashed files renamed (${edited.join(', ') || 'none'})`);
    if (probe.maxRenamed !== undefined && edited.length > probe.maxRenamed) { failed = true; console.error(`size:touch: FAIL ${edited.length} > maxRenamed ${probe.maxRenamed}`); }
    const added = catalogBuild(probe, (text) => `${text}
mw-load-touch-probe = probe
`, `catalog-add-${index}`);
    console.log(`size:touch: locales/${probe.catalog}.ftl, one new English message: ${added.length} hashed files renamed (${added.filter((name) => /\.json$/.test(name)).length} packs)`);
    if (probe.maxRenamedNewMessage !== undefined && added.length > probe.maxRenamedNewMessage) { failed = true; console.error(`size:touch: FAIL ${added.length} > maxRenamedNewMessage ${probe.maxRenamedNewMessage}`); }
    for (const [label, list] of [['edit', edited], ['new message', added]]) {
      const js = list.filter((name) => /\.m?js$/.test(name));
      if (js.length > 0) { failed = true; console.error(`size:touch: FAIL a translation ${label} renamed JavaScript: ${js.join(', ')}`); }
    }
  }
} finally {
  rmSync(work, { recursive: true, force: true });
  console.log('size:touch: the build caches now describe a probe build; run `pnpm build` before `pnpm run size`.');
}
if (failed) process.exit(1);
console.log('size:touch: OK');
