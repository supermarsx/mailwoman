// Bundle-size gate (SPEC §23 / §5.2, plan §3 e8/e11): the login->inbox entry
// chunk must be < 250 KB gzip, AND pdfjs (~1 MB) must NOT ride the critical
// path — it is a lazy chunk pulled only when a PDF attachment is opened
// (`lazy(() => import('./PdfViewer.tsx'))`, plan §1.7).
//
// "Critical path" = the entry module named in dist/index.html PLUS every
// `<link rel="modulepreload">` chunk (both load on initial navigation). Lazy
// dynamic-import chunks are neither the entry <script> nor preloaded, so pdfjs
// landing in one is exactly the intended split; pdfjs in ANY critical chunk
// fails the gate.
//
// Run after `vite build`. Exits non-zero on: entry over budget, or pdfjs on the
// critical path.
import { readdir, readFile } from 'node:fs/promises';
import { gzipSync } from 'node:zlib';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const BUDGET_BYTES = 250 * 1024;
const distDir = fileURLToPath(new URL('../dist/', import.meta.url));
const assetsDir = join(distDir, 'assets');

// Distinctive tokens the pdfjs-dist library bundle emits. Matching any in a
// chunk marks it as "carries pdfjs". These are library internals, not incidental
// app references, so they only appear in the chunk that actually bundles pdfjs.
const PDFJS_FINGERPRINTS = [
  'Setting up fake worker',
  'GlobalWorkerOptions',
  'AbortException',
  'pdfjsVersion',
];

async function gzipOf(path) {
  const buf = await readFile(path);
  return { raw: buf.length, gz: gzipSync(buf).length, text: buf.toString('latin1') };
}

function kb(n) {
  return `${(n / 1024).toFixed(1)} KB`;
}

// --- locate the critical path from dist/index.html --------------------------
let html;
try {
  html = await readFile(join(distDir, 'index.html'), 'utf8');
} catch {
  console.error('check-size: dist/index.html not found — run `pnpm build` first');
  process.exit(1);
}

const entryMatch = html.match(/<script[^>]*\btype="module"[^>]*\bsrc="([^"]+)"/);
if (!entryMatch) {
  console.error('check-size: no <script type="module"> entry found in index.html');
  process.exit(1);
}
const toAssetName = (href) => href.replace(/^.*\/assets\//, '').replace(/^.*\//, '');
const entryName = toAssetName(entryMatch[1]);

const preloadNames = [...html.matchAll(/<link[^>]*\brel="modulepreload"[^>]*\bhref="([^"]+)"/g)].map(
  (m) => toAssetName(m[1]),
);
const criticalNames = new Set([entryName, ...preloadNames]);

// --- report every JS asset; classify critical vs lazy -----------------------
let assetFiles;
try {
  assetFiles = (await readdir(assetsDir)).filter((f) => f.endsWith('.js'));
} catch {
  console.error('check-size: dist/assets not found — run `pnpm build` first');
  process.exit(1);
}
if (assetFiles.length === 0) {
  console.error('check-size: no JS assets found in dist/assets');
  process.exit(1);
}

let entryGz = 0;
let criticalOnPdfjs = [];
let pdfjsLazyChunk = null;

console.log('check-size: JS assets (raw / gzip):');
for (const f of assetFiles) {
  const { raw, gz, text } = await gzipOf(join(assetsDir, f));
  const critical = criticalNames.has(f);
  const isEntry = f === entryName;
  const hasPdfjs = PDFJS_FINGERPRINTS.some((s) => text.includes(s));
  const tag = isEntry ? 'ENTRY' : critical ? 'preload' : 'lazy';
  console.log(
    `  [${tag}] ${f}: ${kb(raw)} raw, ${kb(gz)} gzip${hasPdfjs ? '  <- contains pdfjs' : ''}`,
  );
  if (isEntry) entryGz = gz;
  if (hasPdfjs && critical) criticalOnPdfjs.push(f);
  if (hasPdfjs && !critical) pdfjsLazyChunk = f;
}

// --- gate 1: entry chunk under the gzip budget ------------------------------
console.log(
  `\ncheck-size: entry ${entryName} = ${kb(entryGz)} gzip (budget ${BUDGET_BYTES / 1024} KB)`,
);
let failed = false;
if (entryGz > BUDGET_BYTES) {
  console.error(`check-size: OVER BUDGET — entry exceeds ${BUDGET_BYTES / 1024} KB gzip`);
  failed = true;
}

// --- gate 2: pdfjs must not be on the critical path -------------------------
if (criticalOnPdfjs.length > 0) {
  console.error(
    `check-size: pdfjs on the CRITICAL PATH (must be lazy) — ${criticalOnPdfjs.join(', ')}`,
  );
  failed = true;
} else if (pdfjsLazyChunk) {
  console.log(`check-size: pdfjs correctly isolated in lazy chunk ${pdfjsLazyChunk}`);
} else {
  // Viewers not yet reachable from the app graph -> pdfjs tree-shaken out
  // entirely. The "not on the critical path" invariant still holds.
  console.log('check-size: pdfjs not present in any chunk (viewers not yet in the graph) — OK');
}

if (failed) process.exit(1);
console.log('check-size: OK');
