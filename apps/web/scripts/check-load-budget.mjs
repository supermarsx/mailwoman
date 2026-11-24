// Load budget (t52w part 5): what each screen downloads, how the chunks are
// layered, and how many hashed files one changed line renames. The numbers live
// in scripts/load-budget.json so a change to them shows up in review.
//
// Run after `vite build` (part of `pnpm run size`). It reads the chunk report
// that scripts/chunkPlan.ts writes during the build
// (node_modules/.cache/mailwoman-build/chunk-plan.json), dist/index.html,
// dist/sw.js and the file sizes in dist. It fails on:
//  1. routes: JS / CSS file count or brotli bytes over budget, where a route's
//     files are the static import closure of the entry chunk plus the chunk of
//     the route's root module (and of its parent route, for panes opened from
//     the mailbox), plus the stylesheets index.html links;
//  2. totals: JS chunks, all JS files, CSS files, catalog files, service-worker
//     precache entries and bytes, sw.js bytes;
//  3. layering: a dynamic import target no chunk rule names, a static import of the entry chunk (except the named
//     exceptions), a vendor chunk importing app code, a data chunk importing
//     anything, a chunk importing a chunk of its own or a higher layer (leaf
//     chunks excepted), a chunk the plan does not name, a static import cycle;
//  4. route boundaries: a route's root module must be in a dynamically loaded
//     chunk outside its parent's static closure, and a route not marked
//     "console" must not load an admin shell or admin section chunk statically;
//  5. invalidation: for each probe module, the hashed files renamed when it
//     changes = its chunk + every chunk that imports it, transitively, by
//     static or dynamic import (Rollup's file hash covers the names of the
//     chunks it imports). Over budget, or a forbidden chunk in the set, fails.
// `pnpm run size:touch` (scripts/load-budget-touch.mjs) confirms 5 with real
// rebuilds.
//
// Usage: node scripts/check-load-budget.mjs [distDir] [chunkReport] [budget]
//        add --json to print the computed numbers as JSON.
import { Buffer } from 'node:buffer';
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { brotliCompressSync, gzipSync, constants as zlib } from 'node:zlib';

/** Static closure over chunk imports. */
export function staticClosure(chunks, roots) {
  const found = new Set();
  const queue = [...roots];
  while (queue.length > 0) {
    const name = queue.pop();
    if (found.has(name) || !chunks[name]) continue;
    found.add(name);
    queue.push(...chunks[name].imports);
  }
  return found;
}

/** Chunks renamed when `start` changes: it and all transitive importers (static and dynamic). */
export function invalidated(chunks, start) {
  const importers = new Map();
  for (const [name, chunk] of Object.entries(chunks)) {
    for (const target of [...chunk.imports, ...chunk.dynamicImports]) {
      if (!importers.has(target)) importers.set(target, new Set());
      importers.get(target).add(name);
    }
  }
  const found = new Set();
  const queue = [start];
  while (queue.length > 0) {
    const name = queue.pop();
    if (found.has(name)) continue;
    found.add(name);
    queue.push(...(importers.get(name) ?? []));
  }
  return found;
}

/** Static import cycles between chunks (each reported once, as a path). */
export function staticCycles(chunks) {
  const cycles = [];
  const state = new Map();
  const stack = [];
  const visit = (name) => {
    state.set(name, 1);
    stack.push(name);
    for (const next of chunks[name]?.imports ?? []) {
      if (!chunks[next]) continue;
      if (state.get(next) === 1) cycles.push([...stack.slice(stack.indexOf(next)), next]);
      else if (!state.has(next)) visit(next);
    }
    stack.pop();
    state.set(name, 2);
  };
  for (const name of Object.keys(chunks).sort()) if (!state.has(name)) visit(name);
  return cycles;
}

const assetPath = (href) => href.replace(/^\.\//, '');

/** Files index.html loads before any script runs. */
export function htmlFiles(html) {
  const entry = /<script[^>]*\btype="module"[^>]*\bsrc="([^"]+)"/.exec(html)?.[1];
  const preload = [...html.matchAll(/<link[^>]*\brel="modulepreload"[^>]*\bhref="([^"]+)"/g)].map((m) => assetPath(m[1]));
  const css = [...html.matchAll(/<link[^>]*\brel="stylesheet"[^>]*\bhref="([^"]+)"/g)].map((m) => assetPath(m[1]));
  return { entry: entry === undefined ? null : assetPath(entry), preload, css };
}

/**
 * The whole check, on data already read. `sizes(path)` returns {raw, br, gz}
 * for a dist-relative path. Returns {failures, stats}.
 */
export function checkLoadBudget({ report, html, sw, sizes, distFiles, budget }) {
  const failures = [];
  const fail = (message) => failures.push(message);
  const chunks = report.chunks;
  const policy = report.policy;
  // One admin section is one chunk, `admin-section.<path>`: it follows the rules of `admin-section`.
  const fam = (plan) => (typeof plan === 'string' && plan.startsWith('admin-section.') ? 'admin-section' : plan);
  const layer = (plan) => policy.layers[fam(plan)];
  const consoleOnly = new Set(policy.admin ?? []);
  const leaf = new Set(policy.leaf);
  const data = new Set(policy.data);
  const vendor = new Set(policy.vendor);
  const entryExceptions = new Set(policy.entryImportExceptions);
  const stats = { routes: {}, totals: {}, probes: {}, chunks: {} };

  // --- the entry -----------------------------------------------------------
  const page = htmlFiles(html);
  const entries = Object.keys(chunks).filter((name) => chunks[name].isEntry);
  if (entries.length !== 1) fail(`expected one entry chunk, found ${entries.length}`);
  const entry = entries[0];
  if (entry !== undefined && page.entry !== entry) fail(`index.html loads ${page.entry}, but the entry chunk is ${entry}`);
  if (entry !== undefined && chunks[entry].plan !== 'entry') fail(`the entry chunk ${entry} is not the plan's entry chunk`);

  // --- unplanned lazy boundaries ------------------------------------------
  for (const target of report.unrootedDynamicTargets ?? []) fail(`unplanned: ${target} is a dynamic import target no ROOTS rule in scripts/chunkPlan.ts names (it was built into the unplanned chunk)`);

  // --- layering ------------------------------------------------------------
  for (const [name, chunk] of Object.entries(chunks)) {
    const plan = fam(chunk.plan);
    if (plan === null || layer(plan) === undefined) { fail(`layering: ${name} is not a chunk the plan names (modules of several chunks or an unplanned chunk)`); continue; }
    if (data.has(plan) && chunk.imports.length + chunk.dynamicImports.length > 0) fail(`layering: data chunk ${plan} imports ${[...chunk.imports, ...chunk.dynamicImports].join(', ')}`);
    for (const target of chunk.imports) {
      const to = fam(chunks[target]?.plan);
      if (to === undefined) { fail(`layering: ${plan} imports ${target}, which is not in the chunk report`); continue; }
      if (to === null || layer(to) === undefined) continue;
      if (to === 'entry') { if (!entryExceptions.has(plan)) fail(`layering: ${plan} imports the entry chunk`); continue; }
      if (vendor.has(plan) && !vendor.has(to)) fail(`layering: vendor chunk ${plan} imports app chunk ${to}`);
      if (plan === 'entry' || leaf.has(plan)) continue;
      if (layer(to) >= layer(plan)) fail(`layering: ${plan} (layer ${layer(plan)}) imports ${to} (layer ${layer(to)})`);
    }
  }
  // Rollup's CIRCULAR_CHUNK warning (made fatal in vite.config.ts) covers
  // re-export cycles; this covers any static loop between emitted chunks.
  for (const cycle of staticCycles(chunks)) fail(`circular chunk imports: ${cycle.map((name) => chunks[name].plan ?? name).join(' -> ')}`);

  // --- sizes ---------------------------------------------------------------
  const sum = (files) => {
    let raw = 0, br = 0, gz = 0;
    for (const file of files) { const s = sizes(file); raw += s.raw; br += s.br; gz += s.gz; }
    return { raw, br, gz };
  };
  for (const [name, chunk] of Object.entries(chunks)) {
    const s = sizes(name);
    stats.chunks[chunk.plan ?? name] = { file: name, raw: s.raw, br: s.br, css: chunk.css.length };
  }
  const entryClosure = entry === undefined ? new Set() : staticClosure(chunks, [entry]);
  const critical = new Set([...entryClosure, ...page.preload.filter((file) => chunks[file])]);
  const criticalCss = new Set(page.css);
  stats.critical = { js: critical.size, css: criticalCss.size, ...sum([...critical, ...criticalCss]) };
  const lim = budget.critical ?? {};
  if (lim.maxJs !== undefined && critical.size > lim.maxJs) fail(`critical path: ${critical.size} JS files > ${lim.maxJs}`);
  if (lim.maxGzipBytes !== undefined && stats.critical.gz > lim.maxGzipBytes) fail(`critical path: ${stats.critical.gz} gzip bytes > ${lim.maxGzipBytes}`);
  if (lim.maxBrBytes !== undefined && stats.critical.br > lim.maxBrBytes) fail(`critical path: ${stats.critical.br} brotli bytes > ${lim.maxBrBytes}`);

  // --- routes --------------------------------------------------------------
  const moduleChunk = new Map();
  for (const [name, chunk] of Object.entries(chunks)) for (const module of chunk.modules) moduleChunk.set(module, name);
  const routeFiles = new Map();
  const resolveRoute = (route, seen = new Set()) => {
    if (routeFiles.has(route)) return routeFiles.get(route);
    const def = budget.routes[route];
    if (def === undefined || seen.has(route)) { fail(`route ${route} is not defined (or defined in a loop)`); return { js: new Set(), css: new Set() }; }
    seen.add(route);
    const parent = def.after === undefined ? { js: critical, css: criticalCss } : resolveRoute(def.after, seen);
    const rootChunk = moduleChunk.get(def.root);
    const js = new Set(parent.js);
    if (rootChunk === undefined) fail(`route ${route}: root module ${def.root} is in no chunk`);
    else {
      if (parent.js.has(rootChunk)) fail(`route ${route}: ${def.root} is in ${chunks[rootChunk].plan}, which ${def.after ?? 'the entry'} loads statically (the route must stay a dynamic import)`);
      if (!chunks[rootChunk].isDynamicEntry) fail(`route ${route}: chunk ${chunks[rootChunk].plan} of ${def.root} is not a dynamic entry`);
      for (const file of staticClosure(chunks, [rootChunk])) js.add(file);
    }
    const css = new Set(parent.css);
    for (const file of js) for (const sheet of chunks[file]?.css ?? []) css.add(sheet);
    const result = { js, css };
    routeFiles.set(route, result);
    const added = { js: [...js].filter((f) => !parent.js.has(f)), css: [...css].filter((f) => !parent.css.has(f)) };
    const total = sum([...js, ...css]);
    const adds = sum([...added.js, ...added.css]);
    stats.routes[route] = {
      js: js.size, css: css.size, raw: total.raw, br: total.br,
      ...(def.after === undefined ? {} : { addsJs: added.js.length, addsCss: added.css.length, addsRaw: adds.raw, addsBr: adds.br }),
      chunks: [...js].map((file) => chunks[file].plan).sort(),
    };
    const r = stats.routes[route];
    if (!def.console) {
      const admin = [...js].filter((file) => consoleOnly.has(fam(chunks[file].plan)));
      if (admin.length > 0) fail(`route ${route}: its static closure contains admin console chunks ${admin.map((file) => chunks[file].plan).join(', ')}`);
    }
    const counted = def.after === undefined ? { js: r.js, css: r.css, br: r.br } : { js: r.addsJs, css: r.addsCss, br: r.addsBr };
    if (def.maxJs !== undefined && counted.js > def.maxJs) fail(`route ${route}: ${counted.js} JS files > ${def.maxJs}`);
    if (def.maxCss !== undefined && counted.css > def.maxCss) fail(`route ${route}: ${counted.css} CSS files > ${def.maxCss}`);
    if (def.maxBrBytes !== undefined && counted.br > def.maxBrBytes) fail(`route ${route}: ${counted.br} brotli bytes > ${def.maxBrBytes}`);
    return result;
  };
  for (const route of Object.keys(budget.routes)) resolveRoute(route);

  // --- totals --------------------------------------------------------------
  const assets = distFiles.filter((file) => file.startsWith('assets/'));
  const swList = /const OFFLINE_ASSETS = (\[[^\n]*?\]);/.exec(sw)?.[1];
  let precache = [];
  if (swList === undefined) fail('sw.js: no OFFLINE_ASSETS list');
  else precache = JSON.parse(swList);
  const precacheBytes = precache.reduce((total, path) => total + (distFiles.includes(path.slice(1)) ? sizes(path.slice(1)).raw : 0), 0);
  stats.totals = {
    jsChunks: Object.keys(chunks).length,
    nonAdminChunks: Object.values(chunks).filter((chunk) => fam(chunk.plan) !== 'admin-section').length,
    adminSectionChunks: Object.values(chunks).filter((chunk) => fam(chunk.plan) === 'admin-section').length,
    jsFiles: assets.filter((file) => /\.m?js$/.test(file)).length,
    cssFiles: assets.filter((file) => file.endsWith('.css')).length,
    catalogFiles: assets.filter((file) => file.endsWith('.ftl')).length,
    precacheEntries: precache.length,
    precacheBytes,
    swBytes: Buffer.byteLength(sw),
  };
  for (const [key, max] of Object.entries(budget.totals ?? {})) {
    const value = stats.totals[key];
    if (value === undefined) fail(`totals: unknown key ${key}`);
    else if (value > max) fail(`totals: ${key} ${value} > ${max}`);
  }
  const unreported = assets.filter((file) => /\.m?js$/.test(file) && !chunks[file] && !(budget.otherJs ?? []).some((pattern) => new RegExp(pattern).test(file)));
  if (unreported.length > 0) fail(`JS files in dist that are neither in the chunk report nor in otherJs (stale report?): ${unreported.join(', ')}`);
  for (const file of Object.keys(chunks)) if (!distFiles.includes(file)) fail(`chunk report names ${file}, which dist does not have (stale report?)`);

  // --- invalidation --------------------------------------------------------
  for (const probe of budget.probes ?? []) {
    const start = moduleChunk.get(probe.module);
    if (start === undefined) { fail(`probe ${probe.module}: in no chunk`); continue; }
    const renamed = invalidated(chunks, start);
    const plans = [...renamed].map((file) => chunks[file].plan).sort();
    stats.probes[probe.module] = { renamed: renamed.size, chunks: plans, revalidated: ['index.html', 'sw.js'] };
    if (probe.maxRenamed !== undefined && renamed.size > probe.maxRenamed) fail(`probe ${probe.module}: renames ${renamed.size} hashed files > ${probe.maxRenamed} (${plans.join(', ')})`);
    const forbidden = plans.filter((plan) => (probe.forbid ?? []).includes(plan));
    if (forbidden.length > 0) fail(`probe ${probe.module}: renames ${forbidden.join(', ')}`);
  }
  return { failures, stats };
}

/** Sizes for dist files: raw, brotli (the build's .br sidecar when present), gzip. */
export function distSizes(distDir) {
  const cache = new Map();
  return (file) => {
    if (cache.has(file)) return cache.get(file);
    const path = join(distDir, file);
    const bytes = readFileSync(path);
    const br = existsSync(`${path}.br`) ? statSync(`${path}.br`).size : brotliCompressSync(bytes, { params: { [zlib.BROTLI_PARAM_QUALITY]: 11 } }).length;
    const result = { raw: bytes.length, br, gz: gzipSync(bytes, { level: 9 }).length };
    cache.set(file, result);
    return result;
  };
}

export function listDist(distDir) {
  const out = [];
  const walk = (dir) => {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const full = join(dir, entry.name);
      if (entry.isDirectory()) walk(full);
      else if (entry.isFile() && !/\.(?:br|gz|map)$/.test(entry.name)) out.push(relative(distDir, full).split(sep).join('/'));
    }
  };
  walk(distDir);
  return out;
}

const kb = (n) => `${(n / 1024).toFixed(1)} KB`;

export function printStats(stats) {
  console.log('check-load-budget: chunks (raw / br, CSS files):');
  for (const [plan, s] of Object.entries(stats.chunks).sort((a, b) => b[1].br - a[1].br)) console.log(`  ${plan.padEnd(14)} ${kb(s.raw).padStart(11)} ${kb(s.br).padStart(10)}  css ${s.css}  ${s.file}`);
  console.log(`check-load-budget: critical path ${stats.critical.js} JS + ${stats.critical.css} CSS, ${kb(stats.critical.raw)} raw, ${kb(stats.critical.br)} br, ${kb(stats.critical.gz)} gzip`);
  for (const [route, r] of Object.entries(stats.routes)) {
    const adds = r.addsJs === undefined ? '' : `  (adds ${r.addsJs} JS + ${r.addsCss} CSS, ${kb(r.addsBr)} br)`;
    console.log(`check-load-budget: route ${route.padEnd(9)} ${r.js} JS + ${r.css} CSS, ${kb(r.raw)} raw, ${kb(r.br)} br${adds}`);
  }
  console.log(`check-load-budget: totals ${JSON.stringify(stats.totals)}`);
  for (const [module, p] of Object.entries(stats.probes)) console.log(`check-load-budget: a change in ${module} renames ${p.renamed} hashed files (${p.chunks.join(', ')}) and changes index.html + sw.js`);
}

export function readInputs(distDir, reportPath) {
  if (!existsSync(reportPath)) throw new Error(`no chunk report at ${reportPath}: run \`pnpm build\` first`);
  const report = JSON.parse(readFileSync(reportPath, 'utf8'));
  if (report.schema !== 1) throw new Error('chunk report: unknown schema');
  return {
    report,
    html: readFileSync(join(distDir, 'index.html'), 'utf8'),
    sw: readFileSync(join(distDir, 'sw.js'), 'utf8'),
    sizes: distSizes(distDir),
    distFiles: listDist(distDir),
  };
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const args = process.argv.slice(2).filter((arg) => arg !== '--json');
  const distDir = args[0] ?? fileURLToPath(new URL('../dist/', import.meta.url));
  const reportPath = args[1] ?? fileURLToPath(new URL('../node_modules/.cache/mailwoman-build/chunk-plan.json', import.meta.url));
  const budgetPath = args[2] ?? fileURLToPath(new URL('./load-budget.json', import.meta.url));
  const budget = JSON.parse(readFileSync(budgetPath, 'utf8'));
  const { failures, stats } = checkLoadBudget({ ...readInputs(distDir, reportPath), budget });
  if (process.argv.includes('--json')) console.log(JSON.stringify(stats, null, 1));
  else printStats(stats);
  if (failures.length > 0) {
    for (const message of failures) console.error(`check-load-budget: FAIL ${message}`);
    process.exit(1);
  }
  if (!process.argv.includes('--json')) console.log('check-load-budget: OK');
}
