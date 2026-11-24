#!/usr/bin/env node
// Checks a REAL production web build against every consumer that asks for its
// files by path: the HTML entry points, the chunks' own imports, the
// stylesheets, the web manifest, the service worker's integrity manifest and
// offline list, the precompressed variants the server negotiates, the pinned
// PDF.js inventory the server embeds, the socket-delivered public client assets
// (when that feature's layout file exists), and the Rust sources' compile-time
// includes of apps/web files against the Dockerfiles that build them.
//
// Nothing here is a hand-written list of expected file names. Each expectation
// is read from the consumer (the built HTML/JS/CSS/sw.js, the PDF manifest, the
// shared layout JSON the server includes, the Rust sources, the Dockerfiles),
// and each is checked against the files that are actually in the build.
//
//   pnpm -C apps/web build && node apps/web/scripts/check-build-assets.mjs [distDir]
//
// Exit status 1 lists every problem; 0 prints what was checked.
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { dirname, join, posix, relative, resolve, sep } from 'node:path';
import { brotliDecompressSync, gunzipSync } from 'node:zlib';

const web = resolve(import.meta.dirname, '..');
const repo = resolve(web, '../..');
const dist = resolve(process.argv[2] ?? join(web, 'dist'));
const problems = [];
const counts = {};
const count = (name, n = 1) => { counts[name] = (counts[name] ?? 0) + n; };
const fail = (message) => problems.push(message);
const sha = (algorithm, bytes, encoding = 'hex') => createHash(algorithm).update(bytes).digest(encoding);
const walk = (dir) => readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
  const full = join(dir, entry.name);
  return entry.isDirectory() ? walk(full) : [full];
});
const rel = (file) => relative(dist, file).split(sep).join('/');
const file = (path) => join(dist, ...path.split('/'));
const has = (path) => existsSync(file(path)) && statSync(file(path)).isFile();
const read = (path) => readFileSync(file(path));

if (!existsSync(join(dist, 'index.html'))) {
  console.error(`check-build-assets: ${dist} has no index.html; run the production build first (pnpm -C apps/web build).`);
  process.exit(1);
}
const all = walk(dist).map(rel);
const present = new Set(all);

/** A same-build reference: resolve `ref` against the file that names it. */
function resolveRef(from, ref) {
  const clean = ref.split(/[?#]/)[0];
  if (!clean || /^(?:[a-z][a-z\d+.-]*:|\/\/|#)/i.test(ref)) return undefined;
  const target = clean.startsWith('/') ? clean.slice(1) : posix.normalize(posix.join(posix.dirname(from), clean));
  return target.startsWith('../') ? `../${target}` : target;
}
function expect(from, ref, what) {
  const target = resolveRef(from, ref);
  if (target === undefined) return;
  count(what);
  if (!present.has(target)) fail(`${what}: ${from} references ${ref} -> ${target}, which is not in the build`);
}

// ── HTML entry points: every ./ reference exists, SRI matches the bytes ─────
for (const page of all.filter((p) => p.endsWith('.html') && !p.startsWith('pdfjs/'))) {
  const html = read(page).toString('utf8');
  for (const tag of html.match(/<(?:script|link)\b[^>]*>/g) ?? []) {
    const ref = /\s(?:src|href)="([^"]+)"/.exec(tag)?.[1];
    if (!ref) continue;
    expect(page, ref, 'html reference');
    const integrity = /\sintegrity="sha384-([^"]+)"/.exec(tag)?.[1];
    const target = resolveRef(page, ref);
    if (integrity && target && present.has(target) && sha('sha384', read(target), 'base64') !== integrity) {
      fail(`html integrity: ${page} pins ${ref} with an SRI hash that does not match the built file`);
    }
  }
}

// ── Chunks: static imports, dynamic imports, preload maps, new URL(...) ─────
const literal = String.raw`"((?:\\.|[^"\\])*)"|'((?:\\.|[^'\\])*)'`;
const forms = [
  new RegExp(String.raw`\bfrom\s*(?:${literal})`, 'g'),
  new RegExp(String.raw`\bimport\s*\(\s*(?:${literal})\s*\)`, 'g'),
  new RegExp(String.raw`\bimport\s*(?:${literal})`, 'g'),
  new RegExp(String.raw`new URL\(\s*(?:${literal})\s*,\s*import\.meta\.url\s*\)`, 'g'),
  new RegExp(String.raw`\bimportScripts\(\s*(?:${literal})\s*\)`, 'g'),
];
for (const chunk of all.filter((p) => /\.(?:m?js)$/.test(p) && !p.startsWith('pdfjs/') && p !== 'pdf.worker.mjs')) {
  const source = read(chunk).toString('utf8');
  for (const form of forms) {
    for (const match of source.matchAll(form)) {
      const ref = match[1] ?? match[2];
      if (!ref || !/^\.{0,2}\/|^[\w.-]+\.(?:m?js|css|wasm|ftl|json|woff2?|png|svg|jpg|webp|ico)$/.test(ref)) continue;
      expect(chunk, ref, 'chunk reference');
    }
  }
  // Vite's preload map. With the relative base its entries resolve against the
  // chunk that holds the map.
  const deps = /__vite__mapDeps=\([^)]*?\[([^\]]*)\]/.exec(source)?.[1];
  for (const dep of deps?.match(/"[^"]+"/g) ?? []) expect(chunk, JSON.parse(dep), 'preload map entry');
}

// ── Stylesheets: url(...) ────────────────────────────────────────────────────
for (const sheet of all.filter((p) => p.endsWith('.css') && !p.startsWith('pdfjs/'))) {
  for (const match of read(sheet).toString('utf8').matchAll(/url\(\s*(?:"([^"]+)"|'([^']+)'|([^)'"\s]+))\s*\)/g)) {
    const ref = match[1] ?? match[2] ?? match[3];
    if (ref.startsWith('data:')) continue;
    expect(sheet, ref, 'stylesheet url');
  }
}

// ── Web manifest icons ───────────────────────────────────────────────────────
if (!has('manifest.webmanifest')) fail('manifest.webmanifest: missing (index.html and the server preload it)');
else for (const icon of JSON.parse(read('manifest.webmanifest').toString('utf8')).icons ?? []) expect('manifest.webmanifest', icon.src, 'manifest icon');

// ── Service worker: integrity manifest, offline list, imported scripts ───────
let digests = {};
if (!has('sw.js')) fail('sw.js: missing (the app registers it and the server preloads it)');
else {
  const sw = read('sw.js').toString('utf8');
  for (const marker of ['/* @mw-asset-digests */ null', '/* @mw-offline-assets */ []', '/* @mw-offline-background */']) {
    if (sw.includes(marker)) fail(`sw.js: build placeholder ${marker} was never filled`);
  }
  const value = (name, open, close) => {
    const start = sw.indexOf(open, sw.indexOf(`const ${name} =`));
    return JSON.parse(sw.slice(start, sw.indexOf(close, start) + 1));
  };
  try {
    digests = value('ASSET_DIGESTS', '{', '}');
  } catch {
    fail('sw.js: the asset digest manifest could not be parsed');
  }
  let offline = [];
  try {
    offline = value('OFFLINE_ASSETS', '[', ']');
  } catch {
    fail('sw.js: the offline asset list could not be parsed');
  }
  for (const [path, digest] of Object.entries(digests)) {
    count('sw digest');
    const target = path.slice(1);
    if (!present.has(target)) fail(`sw.js digest: ${path} is not in the build (the worker would refuse or miss it)`);
    else if (path !== '/index.html' && sha('sha256', read(target)) !== digest) fail(`sw.js digest: ${path} does not match the built bytes`);
  }
  for (const path of offline) {
    count('sw offline entry');
    if (!(path in digests)) fail(`sw.js offline list: ${path} has no integrity digest`);
  }
  // t59: sw.js is served under `require-trusted-types-for 'script'`; a plain
  // string passed to importScripts is refused there and the worker never installs.
  for (const match of sw.matchAll(/\bimportScripts\(\s*["'`]([^"'`]*)/g)) fail(`sw.js: importScripts("${match[1]}") passes a plain string; Trusted Types refuses it (use scripts/swTrustedImport.ts)`);
  for (const match of sw.matchAll(/\}\)\("(\.\/assets\/[^"]+)"\);/g)) expect('sw.js', match[1], 'sw import');
}

// ── Precompressed variants: base file exists and the bytes decode to it ──────
for (const variant of all.filter((p) => p.endsWith('.gz') || p.endsWith('.br'))) {
  count('precompressed variant');
  const base = variant.slice(0, -3);
  if (!present.has(base)) { fail(`precompressed: ${variant} has no ${base}`); continue; }
  try {
    const decoded = variant.endsWith('.gz') ? gunzipSync(read(variant)) : brotliDecompressSync(read(variant));
    if (!decoded.equals(read(base))) fail(`precompressed: ${variant} does not decode to ${base}`);
  } catch {
    fail(`precompressed: ${variant} is not valid ${variant.endsWith('.gz') ? 'gzip' : 'brotli'}`);
  }
}

// ── Pinned PDF.js inventory (the server embeds public/pdfjs; the SW pins dist) ─
const pdfManifest = join(web, 'public', 'pdfjs', 'manifest.json');
if (existsSync(pdfManifest)) {
  const pinned = JSON.parse(readFileSync(pdfManifest, 'utf8')).files ?? {};
  for (const [name, digest] of Object.entries(pinned)) {
    count('pdfjs file');
    const path = `pdfjs/${name}`;
    if (!present.has(path)) fail(`pdfjs: ${path} is pinned in public/pdfjs/manifest.json but not in the build`);
    else if (sha('sha256', read(path)) !== digest) fail(`pdfjs: ${path} does not match its pinned digest`);
  }
}
for (const path of ['pdf.worker.mjs', 'workspace-popup.html']) {
  count('public copy');
  const source = join(web, 'public', path);
  if (!present.has(path)) fail(`${path}: missing from the build`);
  else if (existsSync(source) && !readFileSync(source).equals(read(path))) fail(`${path}: the build copy differs from public/${path} (the server embeds the public one)`);
}

// ── Socket-delivered public client assets (shared layout with the server) ────
const layoutFile = join(web, 'src', 'api', 'public-client-assets-layout.json');
if (existsSync(layoutFile)) {
  const layout = JSON.parse(readFileSync(layoutFile, 'utf8'));
  const fill = (template, keyHash, digest, codec = '') => template.replace('{keyHash}', keyHash).replace('{digest}', digest).replace('{codec}', codec);
  const expected = new Set();
  if (!present.has(layout.inventory)) fail(`public client assets: ${layout.inventory} is missing`);
  else {
    const inventory = JSON.parse(read(layout.inventory).toString('utf8'));
    for (const ref of inventory.assets ?? []) {
      count('public client asset');
      const keyHash = sha('sha256', ref.key);
      const manifestPath = fill(layout.manifest, keyHash, ref.digest);
      expected.add(manifestPath);
      if (!present.has(manifestPath)) { fail(`public client assets: ${ref.key} has no manifest at ${manifestPath} (the route answers 404)`); continue; }
      const manifest = JSON.parse(read(manifestPath).toString('utf8'));
      for (const codec of layout.codecs) {
        const representation = manifest[codec];
        if (!representation) continue;
        const path = fill(layout.representation, keyHash, ref.digest, codec);
        expected.add(path);
        if (!present.has(path)) fail(`public client assets: ${ref.key} ${codec} is not at ${path} (the route answers 404)`);
        else if (read(path).length !== representation.bytes || sha('sha256', read(path)) !== representation.digest) fail(`public client assets: ${path} does not match its manifest`);
      }
      // The raw file the service worker may cache must be digest-pinned there.
      const raw = resolveRef('index.html', ref.url);
      if (!raw || !present.has(raw)) fail(`public client assets: ${ref.key} raw file ${ref.url} is not in the build`);
      else if (digests[`/${raw}`] !== ref.digest) fail(`public client assets: sw.js does not pin ${raw} to the asset digest`);
    }
  }
  for (const template of [layout.manifest, layout.representation]) {
    const directory = template.slice(0, template.lastIndexOf('/') + 1);
    for (const path of all.filter((p) => p.startsWith(directory) && !p.endsWith('.gz') && !p.endsWith('.br'))) {
      if (!expected.has(path)) fail(`public client assets: ${path} is in the build but no inventory entry names it`);
    }
  }
}

// ── Rust compile-time includes of apps/web files vs. the Dockerfiles ─────────
// A Docker build compiles the server from an explicit list of COPY lines; an
// include_str!/include_bytes! of a file that list leaves out fails that build.
const dockerfiles = ['Dockerfile', 'scripts/perf/Dockerfile'].filter((p) => existsSync(join(repo, p)));
const copies = dockerfiles.map((p) => [p, [...readFileSync(join(repo, p), 'utf8').matchAll(/^COPY\s+(\S+)\s+\/src\/(\S+)\s*$/gm)].filter((m) => m[1] === m[2]).map((m) => m[1].replace(/\/$/, ''))]);
const rustFiles = walk(join(repo, 'crates')).filter((p) => p.endsWith('.rs') && p.split(sep).includes('src'));
for (const source of rustFiles) {
  const text = readFileSync(source, 'utf8');
  const testStart = text.search(/#\[cfg\(test\)\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s/);
  for (const match of text.matchAll(/include_(?:str|bytes)!\(\s*"([^"]*apps\/web\/[^"]*)"/g)) {
    if (testStart >= 0 && match.index > testStart) continue;
    const target = relative(repo, resolve(dirname(source), match[1])).split(sep).join('/');
    count('rust include');
    if (!existsSync(join(repo, target))) fail(`rust include: ${relative(repo, source).split(sep).join("/")} includes ${target}, which does not exist`);
    for (const [dockerfile, list] of copies) {
      if (!list.some((copied) => target === copied || target.startsWith(`${copied}/`))) fail(`rust include: ${relative(repo, source).split(sep).join("/")} includes ${target}, but ${dockerfile} does not COPY it into /src (the image build fails)`);
    }
  }
}

if (problems.length) {
  console.error(`check-build-assets: ${problems.length} problem(s) in ${dist}`);
  for (const problem of problems) console.error(`  - ${problem}`);
  process.exit(1);
}
console.log(`check-build-assets: OK ${dist}`);
for (const [name, n] of Object.entries(counts)) console.log(`  ${name}: ${n}`);
