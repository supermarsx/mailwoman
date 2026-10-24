// Tests for gen-third-party.mjs. Run: node --test scripts/about/*.test.mjs
// No cargo, pnpm or node_modules needed: the inputs are built in a temp directory.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import {
  allowedBy,
  build,
  diffDocs,
  httpsLink,
  normaliseLicence,
  normaliseText,
  outputFiles,
  parseSpdx,
  unknownIds,
} from './gen-third-party.mjs';

function fixture() {
  const root = mkdtempSync(join(tmpdir(), 'mw-tp-'));
  const reg = join(root, 'registry');
  const crate = (name, version, files) => {
    const dir = join(reg, `${name}-${version}`);
    mkdirSync(dir, { recursive: true });
    for (const [f, text] of Object.entries(files)) writeFileSync(join(dir, f), text);
    writeFileSync(join(dir, 'Cargo.toml'), '');
    return join(dir, 'Cargo.toml');
  };
  const src = 'registry+https://github.com/rust-lang/crates.io-index';
  const pkg = (name, version, extra) => ({ id: `${name} ${version}`, name, version, source: src, ...extra });
  const metadata = {
    workspace_members: ['mw-server 0.0.0', 'mw-test-gate 0.0.0'],
    packages: [
      { id: 'mw-server 0.0.0', name: 'mw-server', version: '0.0.0', source: null },
      { id: 'mw-test-gate 0.0.0', name: 'mw-test-gate', version: '0.0.0', source: null },
      // two files, identical MIT text with CRLF in one: stored once
      pkg('alpha', '1.0.0', {
        license: 'MIT OR Apache-2.0',
        repository: 'git+https://github.com/example/alpha.git',
        manifest_path: crate('alpha', '1.0.0', { 'LICENSE-MIT': 'MIT text\r\nline two\r\n', COPYING: 'MIT text\nline two\n' }),
      }),
      // no licence file: standard text; homepage not https
      pkg('beta', '2.0.0', {
        license: 'MIT/Apache-2.0',
        homepage: 'http://beta.example.org/',
        manifest_path: crate('beta', '2.0.0', {}),
      }),
      // unknown licence id
      pkg('gamma', '0.1.0', { license: 'Foo-1.0', manifest_path: crate('gamma', '0.1.0', { LICENSE: 'Foo licence' }) }),
      // build dependency: listed
      pkg('builder', '1.0.0', { license: 'ISC', manifest_path: crate('builder', '1.0.0', { LICENSE: 'ISC text' }) }),
      // dev dependency only: not listed
      pkg('devonly', '1.0.0', { license: 'MIT', manifest_path: crate('devonly', '1.0.0', { LICENSE: 'x' }) }),
      // only reachable from a non-shipped workspace crate: not listed
      pkg('testonly', '1.0.0', { license: 'MIT', manifest_path: crate('testonly', '1.0.0', { LICENSE: 'y' }) }),
    ],
    resolve: {
      nodes: [
        {
          id: 'mw-server 0.0.0',
          deps: [
            { pkg: 'alpha 1.0.0', dep_kinds: [{ kind: null }] },
            { pkg: 'beta 2.0.0', dep_kinds: [{ kind: null }] },
            { pkg: 'gamma 0.1.0', dep_kinds: [{ kind: null }] },
            { pkg: 'builder 1.0.0', dep_kinds: [{ kind: 'build' }] },
            { pkg: 'devonly 1.0.0', dep_kinds: [{ kind: 'dev' }] },
          ],
        },
        { id: 'mw-test-gate 0.0.0', deps: [{ pkg: 'testonly 1.0.0', dep_kinds: [{ kind: null }] }] },
        { id: 'alpha 1.0.0', deps: [] },
        { id: 'beta 2.0.0', deps: [] },
        { id: 'gamma 0.1.0', deps: [] },
        { id: 'builder 1.0.0', deps: [] },
        { id: 'devonly 1.0.0', deps: [] },
        { id: 'testonly 1.0.0', deps: [] },
      ],
    },
  };
  // web side: one package with a git:// repository on an unknown host, one pinned to an OS
  const nm = join(root, 'node_modules');
  const npm = (name, pj, files = {}) => {
    const dir = join(nm, name);
    mkdirSync(dir, { recursive: true });
    writeFileSync(join(dir, 'package.json'), JSON.stringify({ name, ...pj }));
    for (const [f, t] of Object.entries(files)) writeFileSync(join(dir, f), t);
    return dir;
  };
  const pnpmList = {
    MIT: [
      {
        name: 'web-a',
        versions: ['1.2.3'],
        paths: [npm('web-a', { repository: 'git://git.example.org/web-a.git' }, { LICENSE: 'MIT text\nline two\n' })],
        license: 'MIT',
        homepage: 'https://web-a.example.org/',
      },
      {
        name: 'native-win32-x64',
        versions: ['1.0.0'],
        paths: [npm('native-win32-x64', { os: ['win32'], cpu: ['x64'] }, { LICENSE: 'z' })],
        license: 'MIT',
      },
    ],
  };
  writeFileSync(join(root, 'Cargo.lock'), 'version = 4\n');
  mkdirSync(join(root, 'apps/web'), { recursive: true });
  writeFileSync(join(root, 'apps/web/pnpm-lock.yaml'), "lockfileVersion: '9.0'\n");
  writeFileSync(join(root, 'deny.toml'), '[licenses]\nallow = [\n    "MIT", # comment\n    "ISC",\n]\n');
  const run = () => build({ root, metadatas: [metadata], pnpmList, cargoHome: join(root, 'no-cargo-home') });
  return { root, run, cleanup: () => rmSync(root, { recursive: true, force: true }) };
}

test('only crates reached through normal and build dependencies of a shipped root are listed', () => {
  const f = fixture();
  try {
    const names = f.run().crates.packages.map((p) => p.name);
    assert.deepEqual(names, ['alpha', 'beta', 'builder', 'gamma']);
  } finally {
    f.cleanup();
  }
});

test('an unknown licence is flagged', () => {
  const f = fixture();
  try {
    const { flags } = f.run();
    assert.ok(flags.some((l) => l.startsWith('gamma@0.1.0: unknown licence Foo-1.0')), flags.join('\n'));
    assert.ok(!flags.some((l) => l.startsWith('alpha@')), 'a known expression is not flagged');
    assert.deepEqual(unknownIds('MIT OR Apache-2.0 WITH LLVM-exception'), []);
    assert.deepEqual(unknownIds('MIT AND Bar-2.0'), ['Bar-2.0']);
    assert.deepEqual(unknownIds('MIT OR ('), ['MIT OR (']);
  } finally {
    f.cleanup();
  }
});

test('a non-https link is dropped and flagged; forge links are rewritten to https', () => {
  const f = fixture();
  try {
    const { crates, web, flags } = f.run();
    const beta = crates.packages.find((p) => p.name === 'beta');
    assert.equal(beta.homepage, undefined);
    assert.ok(flags.includes('beta@2.0.0: homepage dropped (not https): http://beta.example.org/'), flags.join('\n'));
    assert.equal(crates.packages.find((p) => p.name === 'alpha').repository, 'https://github.com/example/alpha');
    const webA = web.packages.find((p) => p.name === 'web-a');
    assert.equal(webA.repository, undefined);
    assert.equal(webA.homepage, 'https://web-a.example.org');
    assert.ok(flags.some((l) => l.startsWith('web-a@1.2.3: repository dropped (not https)')));
  } finally {
    f.cleanup();
  }
  assert.equal(httpsLink('git://github.com/prosemirror/prosemirror-state.git'), 'https://github.com/prosemirror/prosemirror-state');
  assert.equal(httpsLink('github:owner/repo'), 'https://github.com/owner/repo');
  assert.equal(httpsLink('owner/repo'), 'https://github.com/owner/repo');
  assert.equal(httpsLink('http://example.com'), null);
  assert.equal(httpsLink('ssh://git@example.com/x.git'), null);
  assert.equal(httpsLink('javascript:alert(1)'), null);
  assert.equal(httpsLink('https://user:pw@example.com/'), null);
});

test('identical licence texts are stored once; a package without a file gets the marked standard text', () => {
  const f = fixture();
  try {
    const { crates, web, texts } = f.run();
    const alpha = crates.packages.find((p) => p.name === 'alpha');
    assert.deepEqual(alpha.texts.map((t) => t.file), ['COPYING', 'LICENSE-MIT']);
    assert.equal(alpha.texts[0].id, alpha.texts[1].id);
    assert.equal(web.packages.find((p) => p.name === 'web-a').texts[0].id, alpha.texts[0].id);
    assert.equal(texts[alpha.texts[0].id], 'MIT text\nline two');
    const beta = crates.packages.find((p) => p.name === 'beta');
    assert.deepEqual(beta.texts.map((t) => t.standard), ['MIT', 'Apache-2.0']);
    assert.match(texts[beta.texts[0].id], /^MIT License/);
    assert.match(texts[beta.texts[1].id], /Apache License/);
  } finally {
    f.cleanup();
  }
});

test('licence texts are written in 16 shards by the first character of their id', () => {
  const f = fixture();
  try {
    const out = f.run();
    const files = outputFiles(out);
    assert.equal(files.length, 2 + 16);
    const shards = files.slice(2);
    for (const [path, doc] of shards) {
      const h = path.match(/texts\/([0-9a-f])\.json$/)[1];
      for (const id of Object.keys(doc)) assert.equal(id[0], h);
    }
    const all = Object.assign({}, ...shards.map(([, d]) => d));
    assert.deepEqual(all, out.texts);
  } finally {
    f.cleanup();
  }
});

test('packages pinned to one OS or CPU are left out of the web list', () => {
  const f = fixture();
  try {
    assert.deepEqual(f.run().web.packages.map((p) => p.name), ['web-a']);
  } finally {
    f.cleanup();
  }
});

test('--check reports drift when a lockfile or the dependency set changes', () => {
  const f = fixture();
  try {
    const before = f.run();
    assert.deepEqual(diffDocs('crates', before.crates, f.run().crates), [], 'unchanged input, no drift');
    writeFileSync(join(f.root, 'Cargo.lock'), 'version = 4\n# changed\n');
    const after = f.run();
    assert.deepEqual(diffDocs('crates', before.crates, after.crates), ['crates: Cargo.lock changed since the list was generated']);
    const dropped = { ...after.crates, packages: after.crates.packages.filter((p) => p.name !== 'beta') };
    assert.deepEqual(diffDocs('crates', dropped, after.crates), ['crates: added beta@2.0.0']);
    assert.deepEqual(diffDocs('web', null, after.web), ['web: missing']);
  } finally {
    f.cleanup();
  }
});

test('policy: an expression no allow-list alternative satisfies is reported', () => {
  const f = fixture();
  try {
    const { policy } = f.run();
    // alpha and beta are MIT OR Apache-2.0 (MIT allowed), builder ISC: fine; gamma is not.
    assert.deepEqual(policy.cargoDeny, ['gamma@0.1.0 (Foo-1.0)']);
  } finally {
    f.cleanup();
  }
  const allow = new Set(['MIT', 'Apache-2.0 WITH LLVM-exception']);
  assert.equal(allowedBy('Apache-2.0 WITH LLVM-exception OR GPL-3.0', allow), true);
  assert.equal(allowedBy('MIT AND GPL-3.0', allow), false);
  assert.equal(allowedBy('(MIT OR GPL-3.0) AND MIT', allow), true);
});

test('licence expressions and texts are normalised', () => {
  assert.equal(normaliseLicence('MIT/Apache-2.0'), 'MIT OR Apache-2.0');
  assert.equal(normaliseLicence('(MIT OR Apache-2.0)'), 'MIT OR Apache-2.0');
  assert.equal(normaliseLicence('(MIT OR X) AND (Y OR Z)'), '(MIT OR X) AND (Y OR Z)');
  assert.deepEqual(parseSpdx('GPL-2.0+'), { id: 'GPL-2.0' });
  assert.equal(normaliseText('﻿a  \r\nb\t\r\n\r\n'), 'a\nb');
});
