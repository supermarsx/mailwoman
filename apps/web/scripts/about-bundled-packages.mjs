// Bundle recorder for the third-party list (t32-e75; t32-cfg.md §4.11).
//
// A Vite plugin that notes every node_modules package whose code rollup put in the
// output (a module with rendered bytes in some chunk, or an emitted asset that came
// from a package), across the main build and its worker builds, and writes them to
// node_modules/.cache/mailwoman-about/bundled-packages.json when the main build
// closes. scripts/about/gen-third-party.mjs reads that file: the web list is what the
// bundle holds, not what is installed.
//
// The record is written outside dist/ on purpose: dist/ is embedded in the server
// and served as is, and the record names local paths.

import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';

const found = new Map(); // package dir -> { name, version, dir }

/** The package root of a module id under node_modules, or null. */
export function packageDirOf(id) {
  const path = String(id).replace(/^\0/, '').split('?')[0].replace(/\\/g, '/');
  const at = path.lastIndexOf('/node_modules/');
  if (at < 0) return null;
  const rest = path.slice(at + '/node_modules/'.length).split('/');
  const name = rest[0]?.startsWith('@') ? `${rest[0]}/${rest[1]}` : rest[0];
  if (!name || name.startsWith('.')) return null;
  return `${path.slice(0, at)}/node_modules/${name}`;
}

function note(id) {
  const dir = packageDirOf(id);
  if (!dir || found.has(dir)) return;
  try {
    const pj = JSON.parse(readFileSync(join(dir, 'package.json'), 'utf8'));
    found.set(dir, { name: pj.name, version: pj.version, dir });
  } catch {
    // not a package root (a nested dist/package.json is never reached here)
  }
}

/** @param {{ main?: boolean }} [opts] main: the instance in `plugins` (writes the record) */
export function bundledPackages(opts = {}) {
  let root = process.cwd();
  return {
    name: 'mailwoman-bundled-packages',
    apply: 'build',
    configResolved(config) {
      root = config.root;
      if (opts.main) found.clear();
    },
    generateBundle(_options, bundle) {
      for (const item of Object.values(bundle)) {
        if (item.type === 'chunk') {
          for (const [id, info] of Object.entries(item.modules)) if (info.renderedLength > 0) note(id);
        } else {
          for (const src of item.originalFileNames ?? []) note(resolve(root, src));
        }
      }
      if (opts.main) {
        const components = [...found.values()].map(({name, version}) => ({
          type: 'library', 'bom-ref': `npm:${name}@${version}`, name, version,
          purl: `pkg:npm/${name.startsWith('@') ? `%40${name.slice(1)}` : name}@${encodeURIComponent(version)}`,
          properties: [{name:'mailwoman:coverage',value:'Package code or assets observed in this Vite build'}],
        })).sort((a,b) => a['bom-ref'].localeCompare(b['bom-ref'],'en'));
        // The full PDF viewer is served as vendored static files, outside Rollup.
        const pdfPath=join(root,'public/pdfjs/manifest.json');
        const pdf=existsSync(pdfPath)?JSON.parse(readFileSync(pdfPath,'utf8')):null;
        if (pdf&&!components.some(c=>c.name==='pdfjs-dist')) components.push({
          type:'library','bom-ref':`npm:pdfjs-dist@${pdf.version}`,name:'pdfjs-dist',version:pdf.version,
          purl:`pkg:npm/pdfjs-dist@${pdf.version}`,hashes:[{alg:'SHA-256',content:pdf.archiveSha256}],
          properties:[{name:'mailwoman:coverage',value:'Vendored PDF.js distribution; archive checksum and per-file manifest; additional notices retained'}],
        });
        this.emitFile({type:'asset',fileName:'software-bom.frontend.json',source:JSON.stringify({
          bomFormat:'CycloneDX',specVersion:'1.6',version:1,
          metadata:{component:{type:'application',name:'Mailwoman web',version:JSON.parse(readFileSync(join(root,'package.json'),'utf8')).version},properties:[{name:'mailwoman:coverage',value:'Bundled npm packages; vendored assets, system libraries and backend inventory are separate'}]},components,
        },null,2)});
      }
    },
    closeBundle() {
      if (!opts.main) return;
      const out = join(root, 'node_modules/.cache/mailwoman-about/bundled-packages.json');
      mkdirSync(dirname(out), { recursive: true });
      const packages = [...found.values()].sort((a, b) => a.name.localeCompare(b.name, 'en') || a.version.localeCompare(b.version, 'en'));
      writeFileSync(out, `${JSON.stringify({ schema: 'mailwoman-bundled-packages/1', packages }, null, 2)}\n`);
    },
  };
}
