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
// Build-owned AST attestation identifies Solid factory calls before minification;
// ordinary email HTML strings are not application DOM templates. Missing proof
// fails closed for every rendered module, including libraries and lazy chunks.
//
// Run after `vite build`. Exits non-zero on: entry over budget, pdfjs on the
// critical path, or a literal style attribute in an app chunk. An optional first
// argument names another dist directory (default `apps/web/dist`).
import { readdir, readFile } from 'node:fs/promises';
import { gzipSync } from 'node:zlib';
import { join,resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import {createHash} from 'node:crypto';
import {mailboxPaneGraphProof} from './bundleGraphContract.mjs';
import {solidTemplateGraphProof,solidTemplateAssetCoverage} from './solidTemplateContract.mjs';

const BUDGET_BYTES = 250 * 1024;
const distDir = process.argv[2] ?? fileURLToPath(new URL('../dist/', import.meta.url));
const assetsDir = join(distDir, 'assets');
// The graph is private build evidence, not an asset served to the browser. A
// custom dist may provide its matching graph as the second positional argument.
const graphPath=process.argv[3]??fileURLToPath(new URL('../node_modules/.cache/mailwoman-build/bundle-graph.json',import.meta.url));
const graphBytes=await readFile(graphPath);
if(graphBytes.length>16*1024*1024)throw Error('Bundle graph byte bound');
const graph=JSON.parse(graphBytes.toString('utf8'));
if(resolve(graph.outDir)!==resolve(distDir))throw Error('Bundle graph belongs to another output directory');
for(const [name,chunk]of Object.entries(graph.chunks)){
 const bytes=await readFile(join(distDir,name));
 if(bytes.length!==chunk.byteLength||createHash('sha256').update(bytes).digest('hex')!==chunk.sha256)throw Error('Bundle graph does not match current chunk bytes');
}

const templateProof=solidTemplateGraphProof(graph);

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
if(graph.chunks[`assets/${entryName}`]?.entry!==true)throw Error('Rendered graph does not identify the actual HTML module entry');
console.log('check-size: deferred mailbox UI graph',JSON.stringify(mailboxPaneGraphProof(graph,[...criticalNames].map(name=>`assets/${name}`))));

// --- report every JS asset; classify critical vs lazy -----------------------
let assetFiles;
try {
  const listJS=async(dir,prefix='')=>{
    const result=[];
    for(const entry of await readdir(dir,{withFileTypes:true})){
      if(entry.isDirectory())result.push(...await listJS(join(dir,entry.name),`${prefix}${entry.name}/`));
      else if(entry.isFile()&&/\.(?:m?js)$/.test(entry.name))result.push(`${prefix}${entry.name}`);
      else if(entry.isSymbolicLink())throw Error('Unexpected output symlink');
      if(result.length>20000)throw Error('JS asset count bound');
    }
    return result;
  };
  assetFiles = await listJS(assetsDir);
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
const inlineStyleChunks = [...templateProof.violations,...solidTemplateAssetCoverage(graph,assetFiles.map(name=>`assets/${name}`))];
for(const [name,proof]of Object.entries(graph.solidTemplates.assets)){
 const bytes=await readFile(join(distDir,name));
 if(bytes.length!==proof.byteLength||createHash('sha256').update(bytes).digest('hex')!==proof.sha256)throw Error('Emitted asset proof does not match current bytes');
}

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

// --- gate 3: no literal style attribute (blocked by `style-src 'self'`) -----
if (inlineStyleChunks.length > 0) {
  console.error(
    '\ncheck-size: literal style attribute in a compiled template — the shell CSP\n' +
      "  (`style-src 'self'`, no 'unsafe-inline') refuses to apply it, so the element\n" +
      '  renders unstyled. Move the STATIC declarations to a CSS class; keep only the\n' +
      '  dynamic ones in `style={{…}}` (Solid applies those via the CSSOM, which\n' +
      '  `style-src` does not govern).',
  );
  for (const { file, module, styleAttributes } of inlineStyleChunks) {
    console.error(`  ${file}: ${module} (${styleAttributes} style attributes)`);
  }
  failed = true;
} else {
  console.log('check-size: no literal style attributes in compiled templates — OK');
}

if (failed) process.exit(1);
console.log('check-size: OK');
