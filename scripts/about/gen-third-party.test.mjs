// Tests for gen-third-party.mjs. Run: node --test scripts/about/*.test.mjs
// No cargo, pnpm, docker or node_modules needed: the inputs are built in a temp directory.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import * as gen from './gen-third-party.mjs';

const SRC = 'registry+https://github.com/rust-lang/crates.io-index';
const DISTROLESS = 'gcr.io/distroless/cc-debian12:nonroot@sha256:aaaa';
const MUSL_BUILD = 'rust:1.98.1-alpine@sha256:bbbb';

function write(path, text) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, text);
}

/**
 * A clean little repository: the run has no problems until a test adds one.
 *   server (linux):   mw-server -> alpha, beta, serde_derive (proc-macro) -> syn; builder (build); devonly (dev)
 *   desktop:          mailwoman-desktop -> alpha, winonly
 *   not shipped:      mw-test-gate -> testonly
 *   web bundle:       web-a, pdfjs-dist (node_modules also holds not-bundled)
 */
function fixture() {
  const root = mkdtempSync(join(tmpdir(), 'mw-tp-'));
  const reg = join(root, 'registry');
  const crateDir = (name, version, files) => {
    const dir = join(reg, `${name}-${version}`);
    for (const [f, text] of Object.entries(files)) write(join(dir, f), text);
    write(join(dir, 'Cargo.toml'), '');
    return join(dir, 'Cargo.toml');
  };
  const pkgs = new Map();
  const nodes = new Map();
  const ws = (name) => {
    const id = `${name} 0.0.0`;
    pkgs.set(id, { id, name, version: '0.0.0', source: null, targets: [{ kind: ['lib'] }] });
    nodes.set(id, { id, deps: [] });
    return id;
  };
  const crate = (name, version, extra = {}, files = { LICENSE: `${name} licence text` }) => {
    const id = `${name} ${version}`;
    const { procMacro, ...rest } = extra;
    pkgs.set(id, {
      id,
      name,
      version,
      source: SRC,
      license: 'MIT',
      targets: [{ kind: [procMacro ? 'proc-macro' : 'lib'] }],
      manifest_path: crateDir(name, version, files),
      ...rest,
    });
    nodes.set(id, { id, deps: [] });
    return id;
  };
  const dep = (from, to, kind = null) => nodes.get(from).deps.push({ pkg: to, dep_kinds: [{ kind }] });

  const server = ws('mw-server');
  const desktop = ws('mailwoman-desktop');
  const gate = ws('mw-test-gate');
  const alpha = crate(
    'alpha',
    '1.0.0',
    { license: 'MIT OR Apache-2.0', repository: 'git+https://github.com/example/alpha.git' },
    { 'LICENSE-MIT': 'MIT text\r\nline two\r\n', COPYING: 'MIT text\nline two\n' },
  );
  const beta = crate('beta', '2.0.0', { license: 'MIT/Apache-2.0', homepage: 'http://beta.example.org/' }, {});
  const derive = crate('serde_derive', '1.0.0', { procMacro: true });
  const syn = crate('syn', '2.0.0');
  const builder = crate('builder', '1.0.0', { license: 'ISC' });
  const devonly = crate('devonly', '1.0.0');
  const testonly = crate('testonly', '1.0.0');
  const winonly = crate('winonly', '0.1.0');
  dep(server, alpha);
  dep(server, beta);
  dep(server, derive);
  dep(derive, syn);
  dep(server, builder, 'build');
  dep(server, devonly, 'dev');
  dep(gate, testonly);
  dep(desktop, alpha);
  dep(desktop, winonly);
  const metadata = () => ({
    workspace_members: [server, desktop, gate],
    packages: [...pkgs.values()],
    resolve: { nodes: [...nodes.values()] },
  });

  // web: node_modules with two bundled packages and one that rollup did not include
  const npm = (name, pj, files = { LICENSE: `${name} licence` }) => {
    const dir = join(root, 'apps/web/node_modules', name);
    write(join(dir, 'package.json'), JSON.stringify({ name, ...pj }));
    for (const [f, t] of Object.entries(files)) write(join(dir, f), t);
    return dir;
  };
  const bundled = {
    schema: 'mailwoman-bundled-packages/1',
    packages: [
      { name: 'web-a', version: '1.2.3', dir: npm('web-a', { version: '1.2.3', license: 'MIT', repository: 'git://git.example.org/web-a.git', homepage: 'https://web-a.example.org/' }, { LICENSE: 'MIT text\nline two\n' }) },
      { name: 'pdfjs-dist', version: '6.1.200', dir: npm('pdfjs-dist', { version: '6.1.200', license: 'Apache-2.0', repository: 'https://github.com/mozilla/pdf.js' }) },
    ],
  };
  npm('not-bundled', { version: '9.9.9', license: 'MIT' });

  // assets and public files
  write(join(root, 'apps/web/src/assets/fonts/jost/OFL.txt'), 'Copyright Jost\n\nSIL OPEN FONT LICENSE Version 1.1\n');
  write(join(root, 'apps/web/src/assets/fonts/jost/SOURCE.txt'), 'Jost.\n\nPackage:   @fontsource-variable/jost 5.3.0 (npm registry)\n');
  write(join(root, 'apps/web/src/assets/fonts/jost/jost-latin.woff2'), 'x');
  write(join(root, 'license.md'), 'MIT License\n\nCopyright (c) 2026 Mailwoman Contributors\n');
  for (const f of ['icons/icon-192.png', 'favicon.svg', 'favicon.ico', 'apple-touch-icon.png', 'themes/grain.svg', 'pdf.worker.mjs', 'sw.js', 'manifest.webmanifest']) {
    write(join(root, 'apps/web/public', f), 'x');
  }

  write(join(root, 'Cargo.lock'), 'version = 4\n');
  write(join(root, 'crates/mw-media-wasm/Cargo.lock'), 'version = 4\n');
  write(join(root, 'apps/web/pnpm-lock.yaml'), "lockfileVersion: '9.0'\n");
  write(join(root, 'deny.toml'), '[licenses]\nallow = [\n    "MIT", # comment\n    "ISC",\n    "Apache-2.0",\n]\n');
  write(
    join(root, 'Dockerfile'),
    `FROM rust:1 AS build\nFROM ${DISTROLESS} AS runtime\nFROM ${MUSL_BUILD} AS build-musl\nFROM scratch AS runtime-static\n`,
  );
  const image = {
    readFrom: 'mailwoman:test',
    packages: [
      {
        name: 'libc6',
        version: '2.36-9+deb12u14',
        homepage: 'https://www.gnu.org/software/libc/libc.html',
        copyright: 'Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/\n\nFiles: *\nLicense: LGPL-2.1+\n\nFiles: x\nLicense: GPL-2+\n',
      },
    ],
  };

  const input = {
    root,
    cargo: [
      { metadata: metadata(), roots: { 'mw-server': 'server' } },
      { metadata: metadata(), roots: { 'mailwoman-desktop': 'desktop' } },
    ],
    bundled,
    cargoHome: join(root, 'no-cargo-home'),
    image,
    previous: null,
  };
  return {
    root,
    input,
    crate,
    dep,
    metadata,
    npm,
    rebuild: () => {
      input.cargo = input.cargo.map((c) => ({ ...c, metadata: metadata() }));
    },
    run: (over = {}) => gen.build({ ...input, ...over }),
    cleanup: () => rmSync(root, { recursive: true, force: true }),
  };
}

const names = (list) => list.map((e) => e.name);

function withFixture(fn) {
  const f = fixture();
  try {
    fn(f);
  } finally {
    f.cleanup();
  }
}

test('the clean fixture has no problems', () => {
  withFixture((f) => {
    const out = f.run();
    assert.deepEqual(out.problems, []);
  });
});

test('shipped crates follow normal dependencies; proc-macros and build dependencies are build tools; shells are their own list', () => {
  withFixture((f) => {
    const { doc } = f.run();
    assert.deepEqual(names(doc.rust), ['alpha', 'beta']);
    assert.deepEqual(names(doc.buildTools), ['builder', 'serde_derive', 'syn']);
    assert.deepEqual(names(doc.shells), ['winonly']);
    assert.deepEqual(doc.rust.find((e) => e.name === 'alpha').artifacts, ['server', 'desktop']);
    const all = names([...doc.rust, ...doc.shells, ...doc.buildTools]);
    assert.ok(!all.includes('devonly') && !all.includes('testonly'));
  });
});

test('a crate absent from the platform-filtered metadata is not listed', () => {
  withFixture((f) => {
    // mw-server -> winonly is a windows-only edge: present in the unfiltered graph,
    // absent from what `cargo metadata --filter-platform <linux>` reports.
    f.dep('mw-server 0.0.0', 'winonly 0.1.0');
    f.rebuild();
    const unfiltered = f.run();
    assert.ok(unfiltered.doc.rust.some((e) => e.name === 'winonly'), 'control: the edge is walked when present');
    const linux = f.metadata();
    linux.resolve.nodes = linux.resolve.nodes.map((n) =>
      n.id === 'mw-server 0.0.0' ? { ...n, deps: n.deps.filter((d) => d.pkg !== 'winonly 0.1.0') } : n,
    );
    f.input.cargo[0] = { ...f.input.cargo[0], metadata: linux };
    const { doc } = f.run();
    assert.equal(doc.rust.some((e) => e.name === 'winonly'), false);
    assert.deepEqual(names(doc.shells), ['winonly']);
  });
});

test('web packages are the ones rollup bundled, with metadata from their package.json', () => {
  withFixture((f) => {
    const { doc, flags } = f.run();
    assert.deepEqual(names(doc.web), ['pdfjs-dist', 'web-a']);
    const a = doc.web.find((e) => e.name === 'web-a');
    assert.equal(a.version, '1.2.3');
    assert.equal(a.licence, 'MIT');
    assert.equal(a.repository, undefined);
    assert.ok(flags.some((l) => l.startsWith('web-a@1.2.3: repository dropped (not https)')));
  });
});

test('a missing bundle record is a problem, not an empty list', () => {
  withFixture((f) => {
    const { problems } = f.run({ bundled: null });
    assert.ok(problems.some((p) => /bundle record/.test(p)), problems.join('\n'));
  });
});

test('an entry with no licence text and a licence outside deny.toml are problems (crates and web)', () => {
  withFixture((f) => {
    const odd = f.crate('odd', '0.1.0', { license: 'Foo-1.0' }, {});
    f.dep('mw-server 0.0.0', odd);
    f.rebuild();
    f.input.bundled.packages.push({ name: 'gpl-thing', version: '1.0.0', dir: f.npm('gpl-thing', { version: '1.0.0', license: 'GPL-3.0-only' }) });
    const { problems, flags } = f.run();
    assert.ok(problems.includes('odd@0.1.0: no licence text'), problems.join('\n'));
    assert.ok(problems.includes("odd@0.1.0: licence Foo-1.0 is outside deny.toml's [licenses] allow list"), problems.join('\n'));
    assert.ok(problems.includes("gpl-thing@1.0.0: licence GPL-3.0-only is outside deny.toml's [licenses] allow list"), problems.join('\n'));
    assert.ok(flags.some((l) => l.startsWith('odd@0.1.0: unknown licence Foo-1.0')));
  });
});

test('every font or image under src/assets and public must be listed', () => {
  withFixture((f) => {
    write(join(f.root, 'apps/web/src/assets/images/photo.png'), 'x');
    write(join(f.root, 'apps/web/public/stray.woff2'), 'x');
    const { problems } = f.run();
    assert.ok(problems.includes('apps/web/src/assets/images/photo.png: not covered by any entry'), problems.join('\n'));
    assert.ok(problems.includes('apps/web/public/stray.woff2: not covered by any entry'), problems.join('\n'));
  });
});

test('the public pdf.worker.mjs copy needs pdfjs-dist in the web list', () => {
  withFixture((f) => {
    f.input.bundled.packages = f.input.bundled.packages.filter((p) => p.name !== 'pdfjs-dist');
    const { problems } = f.run();
    assert.ok(problems.some((p) => p.startsWith('apps/web/public/pdf.worker.mjs: ')), problems.join('\n'));
  });
});

test('assets: Jost carries the version from SOURCE.txt and its OFL.txt; icons and theme textures are first-party', () => {
  withFixture((f) => {
    const { doc, texts } = f.run();
    const jost = doc.assets.find((e) => e.name === 'Jost');
    assert.equal(jost.version, '5.3.0');
    assert.equal(jost.texts[0].file, 'OFL.txt');
    assert.match(texts[jost.texts[0].id], /OPEN FONT LICENSE/);
    assert.equal(doc.assets.find((e) => e.kind === 'icons').licence, 'MIT');
    assert.ok(doc.assets.some((e) => e.name === 'Theme textures'));
  });
});

test('the runtime images are listed: distroless packages with their copyright files, musl and the CA bundle for FROM scratch', () => {
  withFixture((f) => {
    const { doc, texts } = f.run();
    assert.equal(doc.images.runtime.from, DISTROLESS);
    assert.equal(doc.images.runtimeStatic.buildFrom, MUSL_BUILD);
    const libc = doc.runtimeImage.find((e) => e.name === 'libc6');
    assert.equal(libc.version, '2.36-9+deb12u14');
    assert.equal(libc.licence, 'LGPL-2.1+ AND GPL-2+');
    assert.equal(libc.texts[0].file, '/usr/share/doc/libc6/copyright');
    assert.match(texts[libc.texts[0].id], /LGPL-2\.1\+/);
    assert.deepEqual(libc.images, ['runtime']);
    const musl = doc.runtimeImage.find((e) => e.name === 'musl');
    assert.equal(musl.licence, 'MIT');
    assert.deepEqual(musl.images, ['runtime-static']);
    assert.equal(musl.texts[0].standard, 'MIT');
  });
});

test('without --image the previous image entries are kept while the Dockerfile still names the same images; a changed digest is a problem', () => {
  withFixture((f) => {
    const first = f.run();
    const previous = { doc: first.doc, texts: first.texts };
    const kept = f.run({ image: null, previous });
    assert.deepEqual(kept.doc.runtimeImage, first.doc.runtimeImage);
    assert.deepEqual(kept.problems, []);
    write(join(f.root, 'Dockerfile'), `FROM gcr.io/distroless/cc-debian12:nonroot@sha256:cccc AS runtime\nFROM ${MUSL_BUILD} AS build-musl\n`);
    const drift = f.run({ image: null, previous });
    assert.ok(drift.problems.some((p) => /runtime image changed/.test(p)), drift.problems.join('\n'));
  });
});

test('identical licence texts are stored once; a package without a file gets the marked standard text', () => {
  withFixture((f) => {
    const { doc, texts } = f.run();
    const alpha = doc.rust.find((p) => p.name === 'alpha');
    assert.deepEqual(alpha.texts.map((t) => t.file), ['COPYING', 'LICENSE-MIT']);
    assert.equal(alpha.texts[0].id, alpha.texts[1].id);
    assert.equal(doc.web.find((p) => p.name === 'web-a').texts[0].id, alpha.texts[0].id);
    assert.equal(texts[alpha.texts[0].id], 'MIT text\nline two');
    const beta = doc.rust.find((p) => p.name === 'beta');
    assert.deepEqual(beta.texts.map((t) => t.standard), ['MIT', 'Apache-2.0']);
    assert.match(texts[beta.texts[0].id], /^MIT License/);
  });
});

test('the vendored SPDX set covers the allow-listed licences named in §4.11', () => {
  for (const id of ['MIT', 'Apache-2.0', 'BSD-2-Clause', 'BSD-3-Clause', 'ISC', 'MPL-2.0', 'Unicode-3.0', 'OFL-1.1', 'Zlib', 'CC0-1.0', 'BSL-1.0']) {
    assert.equal(typeof gen.STANDARD_TEXTS?.[id], 'string', id);
    assert.ok(gen.STANDARD_TEXTS[id].length > 200, id);
  }
});

test('one output document plus 16 text shards', () => {
  withFixture((f) => {
    const out = f.run();
    const files = gen.outputFiles(out);
    assert.equal(files[0][0], 'apps/web/src/about/third-party.json');
    assert.equal(files.length, 1 + 16);
    for (const [path, doc] of files.slice(1)) {
      const h = path.match(/texts\/([0-9a-f])\.json$/)[1];
      for (const id of Object.keys(doc)) assert.equal(id[0], h);
    }
    assert.deepEqual(Object.assign({}, ...files.slice(1).map(([, d]) => d)), out.texts);
    assert.deepEqual(Object.keys(out.doc.generatedFrom), ['cargoLock', 'mediaWasmLock', 'pnpmLock']);
  });
});

test('--check reports drift when a lockfile or the dependency set changes', () => {
  withFixture((f) => {
    const before = f.run();
    assert.deepEqual(gen.diffDocs('doc', before.doc, f.run().doc), [], 'unchanged input, no drift');
    write(join(f.root, 'Cargo.lock'), 'version = 4\n# changed\n');
    const after = f.run();
    assert.deepEqual(gen.diffDocs('doc', before.doc, after.doc), ['doc: Cargo.lock changed since the list was generated']);
    const dropped = { ...after.doc, rust: after.doc.rust.filter((p) => p.name !== 'beta') };
    assert.deepEqual(gen.diffDocs('doc', dropped, after.doc), ['doc: rust: added beta@2.0.0']);
    assert.deepEqual(gen.diffDocs('doc', null, after.doc), ['doc: missing']);
  });
});

test('links: non-https dropped, forge links rewritten', () => {
  assert.equal(gen.httpsLink('git://github.com/prosemirror/prosemirror-state.git'), 'https://github.com/prosemirror/prosemirror-state');
  assert.equal(gen.httpsLink('github:owner/repo'), 'https://github.com/owner/repo');
  assert.equal(gen.httpsLink('owner/repo'), 'https://github.com/owner/repo');
  assert.equal(gen.httpsLink('http://example.com'), null);
  assert.equal(gen.httpsLink('ssh://git@example.com/x.git'), null);
  assert.equal(gen.httpsLink('javascript:alert(1)'), null);
  assert.equal(gen.httpsLink('https://user:pw@example.com/'), null);
});

test('licence expressions and texts are normalised; allow-list semantics', () => {
  assert.equal(gen.normaliseLicence('MIT/Apache-2.0'), 'MIT OR Apache-2.0');
  assert.equal(gen.normaliseLicence('(MIT OR Apache-2.0)'), 'MIT OR Apache-2.0');
  assert.equal(gen.normaliseLicence('(MIT OR X) AND (Y OR Z)'), '(MIT OR X) AND (Y OR Z)');
  assert.deepEqual(gen.parseSpdx('GPL-2.0+'), { id: 'GPL-2.0' });
  assert.equal(gen.normaliseText('﻿a  \r\nb\t\r\n\r\n'), 'a\nb');
  assert.deepEqual(gen.unknownIds('MIT OR Apache-2.0 WITH LLVM-exception'), []);
  assert.deepEqual(gen.unknownIds('MIT AND Bar-2.0'), ['Bar-2.0']);
  const allow = new Set(['MIT', 'Apache-2.0 WITH LLVM-exception']);
  assert.equal(gen.allowedBy('Apache-2.0 WITH LLVM-exception OR GPL-3.0', allow), true);
  assert.equal(gen.allowedBy('MIT AND GPL-3.0', allow), false);
  assert.equal(gen.allowedBy('(MIT OR GPL-3.0) AND MIT', allow), true);
});

test('bundle recorder: package roots from module ids, and the record written by the main instance', async () => {
  const rec = await import('../../apps/web/scripts/about-bundled-packages.mjs');
  assert.equal(
    rec.packageDirOf(String.raw`F:\w\node_modules\.pnpm\solid-js@1.9.14\node_modules\solid-js\dist\solid.js?v=1`),
    'F:/w/node_modules/.pnpm/solid-js@1.9.14/node_modules/solid-js',
  );
  assert.equal(rec.packageDirOf('/w/node_modules/@fluent/bundle/esm/index.js'), '/w/node_modules/@fluent/bundle');
  assert.equal(rec.packageDirOf('/w/src/App.tsx'), null);
  assert.equal(rec.packageDirOf('\0vite/preload-helper.js'), null);

  const root = mkdtempSync(join(tmpdir(), 'mw-rec-'));
  try {
    const pkg = (name, version) => {
      const dir = join(root, 'node_modules', name);
      write(join(dir, 'package.json'), JSON.stringify({ name, version }));
      return dir.replace(/\\/g, '/');
    };
    const used = pkg('used', '1.0.0');
    const shaken = pkg('shaken', '2.0.0');
    const fromWorker = pkg('in-worker', '3.0.0');
    const font = pkg('@scope/font', '4.0.0');
    const main = rec.bundledPackages({ main: true });
    const worker = rec.bundledPackages();
    main.configResolved({ root });
    worker.configResolved({ root });
    worker.generateBundle({}, { 'w.js': { type: 'chunk', modules: { [`${fromWorker}/index.js`]: { renderedLength: 10 } } } });
    main.generateBundle({}, {
      'a.js': { type: 'chunk', modules: { [`${used}/index.js`]: { renderedLength: 5 }, [`${shaken}/index.js`]: { renderedLength: 0 }, [`${root}/src/x.ts`]: { renderedLength: 9 } } },
      'f.woff2': { type: 'asset', originalFileNames: ['node_modules/@scope/font/files/f.woff2'] },
    });
    void font;
    main.closeBundle();
    const out = JSON.parse(readFileSync(join(root, 'node_modules/.cache/mailwoman-about/bundled-packages.json'), 'utf8'));
    assert.deepEqual(out.packages.map((p) => `${p.name}@${p.version}`), ['@scope/font@4.0.0', 'in-worker@3.0.0', 'used@1.0.0']);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
