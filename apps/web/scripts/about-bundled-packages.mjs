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

import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
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
