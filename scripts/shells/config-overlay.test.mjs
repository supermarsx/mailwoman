// Tests for scripts/shells/config-overlay.mjs (t32-cfg §4.18): the desktop shell
// gets an updater only when both the feed URL and the public key are given, and
// bad inputs stop the build before the long SPA build. Run: node --test scripts/shells/*.test.mjs
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import { buildOverlay, feedProblem, pubkeyProblem } from './config-overlay.mjs';

const SCRIPT = fileURLToPath(new URL('./config-overlay.mjs', import.meta.url));
const FEED = 'https://updates.example.org/mail/{{target}}/{{arch}}/{{current_version}}';
// What `tauri signer generate` prints for a public key: base64 of the .pub file.
const PUBKEY = Buffer.from(
  'untrusted comment: minisign public key: 3A1B2C3D4E5F6A7B\nRWR7am9+3Xv1yQ8bNL9yHZB5V0hJf4tC1nYw7WcQe2m0kP3sT6uVxYz9\n',
).toString('base64');

function run(env) {
  const clean = Object.fromEntries(Object.entries(process.env).filter(([k]) => !k.startsWith('MW_SHELL_')));
  return spawnSync(process.execPath, [SCRIPT], { env: { ...clean, ...env }, encoding: 'utf8' });
}

test('both inputs give the updater block and signed update bundles', () => {
  const { overlay } = buildOverlay({ MW_SHELL_UPDATER_URL: FEED, MW_SHELL_UPDATER_PUBKEY: PUBKEY });
  assert.deepEqual(overlay, {
    bundle: { createUpdaterArtifacts: true },
    plugins: { updater: { pubkey: PUBKEY, endpoints: [FEED], windows: { installMode: 'passive' } } },
  });
});

test('neither input gives no updater block', () => {
  const { overlay, notes } = buildOverlay({});
  assert.deepEqual(overlay, {});
  assert.match(notes.join('\n'), /updater off/);
});

test('one input alone gives no updater block and names the missing one', () => {
  const onlyUrl = buildOverlay({ MW_SHELL_UPDATER_URL: FEED });
  assert.deepEqual(onlyUrl.overlay, {});
  assert.match(onlyUrl.notes.join('\n'), /MW_SHELL_UPDATER_PUBKEY is not set/);
  const onlyKey = buildOverlay({ MW_SHELL_UPDATER_PUBKEY: PUBKEY });
  assert.deepEqual(onlyKey.overlay, {});
  assert.match(onlyKey.notes.join('\n'), /MW_SHELL_UPDATER_URL is not set/);
});

test('branding is carried with or without the updater', () => {
  const { overlay } = buildOverlay({
    MW_SHELL_PRODUCT_NAME: 'Example Mail',
    MW_SHELL_IDENTIFIER: 'org.example.mail',
    MW_SHELL_UPDATER_URL: FEED,
  });
  assert.deepEqual(overlay, { productName: 'Example Mail', identifier: 'org.example.mail' });
});

test('the feed must be https without credentials', () => {
  assert.equal(feedProblem(FEED), null);
  assert.match(feedProblem('http://updates.example.org/x'), /https/);
  assert.match(feedProblem('https://user:pw@updates.example.org/x'), /user name or password/);
  assert.match(feedProblem('https://updates.example.org/a b'), /spaces/);
  assert.match(feedProblem('updates.example.org'), /not a URL/);
  assert.throws(
    () => buildOverlay({ MW_SHELL_UPDATER_URL: 'http://updates.example.org/x', MW_SHELL_UPDATER_PUBKEY: PUBKEY }),
    /MW_SHELL_UPDATER_URL must start with https/,
  );
});

test('the key must be the base64 minisign public key file', () => {
  assert.equal(pubkeyProblem(PUBKEY), null);
  // The bare key line, not the base64 of the whole file.
  assert.notEqual(pubkeyProblem('RWR7am9+3Xv1yQ8bNL9yHZB5V0hJf4tC1nYw7WcQe2m0kP3sT6uVxYz9'), null);
  assert.notEqual(pubkeyProblem('HUMAN_PROVIDE_MINISIGN_UPDATER_PUBKEY'), null);
  assert.notEqual(pubkeyProblem(Buffer.from('untrusted comment: x\nnot a key\n').toString('base64')), null);
  assert.throws(
    () => buildOverlay({ MW_SHELL_UPDATER_URL: FEED, MW_SHELL_UPDATER_PUBKEY: 'not-base64!' }),
    /MW_SHELL_UPDATER_PUBKEY/,
  );
});

test('a feed without the placeholders is accepted with a note', () => {
  const { overlay, notes } = buildOverlay({
    MW_SHELL_UPDATER_URL: 'https://updates.example.org/latest.json',
    MW_SHELL_UPDATER_PUBKEY: PUBKEY,
  });
  assert.deepEqual(overlay.plugins.updater.endpoints, ['https://updates.example.org/latest.json']);
  assert.match(notes.join('\n'), /no \{\{target\}\}, \{\{arch\}\}, \{\{current_version\}\}/);
});

test('as a program: JSON on stdout, notes on stderr, exit 1 on a bad input', () => {
  const on = run({ MW_SHELL_UPDATER_URL: FEED, MW_SHELL_UPDATER_PUBKEY: PUBKEY });
  assert.equal(on.status, 0, on.stderr);
  assert.deepEqual(JSON.parse(on.stdout).plugins.updater.endpoints, [FEED]);
  assert.match(on.stderr, /\[build-shells\] updater on/);

  const off = run({});
  assert.equal(off.status, 0, off.stderr);
  assert.equal(off.stdout, '{}');

  const bad = run({ MW_SHELL_UPDATER_URL: 'http://x.example', MW_SHELL_UPDATER_PUBKEY: PUBKEY });
  assert.equal(bad.status, 1);
  assert.equal(bad.stdout, '');
  assert.match(bad.stderr, /must start with https/);
});
