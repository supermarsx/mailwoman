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
import { copyFileSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { invalidated, listDist } from './check-load-budget.mjs';

const web = fileURLToPath(new URL('..', import.meta.url));
const budget = JSON.parse(readFileSync(new URL('./load-budget.json', import.meta.url), 'utf8'));
const reportPath = join(web, 'node_modules/.cache/mailwoman-build/chunk-plan.json');
const vite = join(web, 'node_modules/vite/bin/vite.js');
const work = mkdtempSync(join(tmpdir(), 'mw-load-touch-'));
const probes = (budget.probes ?? []).filter((probe) => !probe.module.includes('?'));

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
} finally {
  rmSync(work, { recursive: true, force: true });
  console.log('size:touch: the build caches now describe a probe build; run `pnpm build` before `pnpm run size`.');
}
if (failed) process.exit(1);
console.log('size:touch: OK');
