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
// A third, CSP gate rides along here because this is the repo's only post-build
// check of the emitted bundle (t24-e13): no compiled template may carry a literal
// `style="…"` attribute. The shell ships `style-src 'self'` with no
// `'unsafe-inline'` (dropped deliberately in 26.16), and a literal style attribute
// is subject to that directive, so the browser silently refuses to apply it.
//
// It is easy to reintroduce by accident. Solid applies a DYNAMIC `style={{…}}`
// value through the CSSOM (`el.style.setProperty`), which `style-src` does not
// govern — but a STATIC one (and the static subset of a partly-dynamic object) is
// hoisted by the compiler into the template HTML, where it does. The two look
// identical in the JSX, which is why 26.16's reasoning that "every inline style in
// the SPA is Solid object-form, applied through the CSSOM" held for the sites its
// author checked and was false for five others. Only the built output tells them
// apart, so the check lives here.
//
// Run after `vite build`. Exits non-zero on: entry over budget, pdfjs on the
// critical path, or a literal style attribute in an app chunk.
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
let inlineStyleChunks = [];

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
  // Skip the vendored pdfjs chunk: it is third-party code we do not author, and
  // its only match sits inside a `data:image/svg+xml` URI, which is not a DOM
  // attribute on our page and so is not governed by `style-src`.
  if (!hasPdfjs) {
    const found = [...new Set(text.match(/style="[^"]*"/g) ?? [])];
    if (found.length > 0) inlineStyleChunks.push({ file: f, found });
  }
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

// --- gate 3: no literal style attribute (blocked by `style-src 'self'`) -----
if (inlineStyleChunks.length > 0) {
  console.error(
    '\ncheck-size: literal style="…" attribute in a compiled template — the shell CSP\n' +
      "  (`style-src 'self'`, no 'unsafe-inline') refuses to apply it, so the element\n" +
      '  renders unstyled. Move the STATIC declarations to a CSS class; keep only the\n' +
      '  dynamic ones in `style={{…}}` (Solid applies those via the CSSOM, which\n' +
      '  `style-src` does not govern).',
  );
  for (const { file, found } of inlineStyleChunks) {
    console.error(`  ${file}: ${found.join(' ')}`);
  }
  failed = true;
} else {
  console.log('check-size: no literal style attributes in compiled templates — OK');
}

if (failed) process.exit(1);
console.log('check-size: OK');
