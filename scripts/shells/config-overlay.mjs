#!/usr/bin/env node
// The per-build Tauri config overlay for the desktop shell (t32-cfg §4.18), written
// by scripts/build-shells.{sh,ps1} and passed to `tauri build --config`. Tauri
// merges it over apps/desktop/src-tauri/tauri.conf.json (JSON merge patch).
//
// Inputs (environment; the scripts have already checked the branding values):
//   MW_SHELL_PRODUCT_NAME    -> productName
//   MW_SHELL_IDENTIFIER      -> identifier
//   MW_SHELL_UPDATER_URL     the update feed: https, normally
//                            https://host/path/{{target}}/{{arch}}/{{current_version}}
//   MW_SHELL_UPDATER_PUBKEY  the minisign public key, as `tauri signer generate`
//                            prints it (base64 of the whole .pub file)
//
// Both updater inputs set -> `plugins.updater` with the key and the feed, and
// `bundle.createUpdaterArtifacts: true`, so installers come with signed update
// bundles. Either one missing -> no updater block: the committed config has none,
// and the shell registers the updater plugin only when the block is there
// (`updater_configured`, apps/desktop/src-tauri/src/lib.rs). Setting only one of
// the two is reported, and the build goes on without an updater.
//
// Prints the overlay as JSON on stdout (`{}` when there is nothing to change) and
// what it decided on stderr. Exits 1 on an invalid updater input.

import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const PLACEHOLDERS = ['{{target}}', '{{arch}}', '{{current_version}}'];

/** Why `url` cannot be the update feed, or `null` when it can. */
export function feedProblem(url) {
  if (/\s/.test(url)) return 'must not contain spaces';
  let parsed;
  try {
    parsed = new URL(url);
  } catch {
    return 'is not a URL';
  }
  if (parsed.protocol !== 'https:') return 'must start with https://';
  if (parsed.username !== '' || parsed.password !== '') return 'must not contain a user name or password';
  if (parsed.hostname === '') return 'has no host';
  return null;
}

/** Why `key` is not a minisign public key in Tauri's form, or `null`. */
export function pubkeyProblem(key) {
  if (!/^[A-Za-z0-9+/]+={0,2}$/.test(key)) {
    return 'must be the base64 text that `tauri signer generate` prints for the public key';
  }
  const text = Buffer.from(key, 'base64').toString('utf8');
  const lines = text.split(/\r?\n/).filter((l) => l !== '');
  if (!lines[0]?.startsWith('untrusted comment:') || !/^RW[A-Za-z0-9+/]+={0,2}$/.test(lines[1] ?? '')) {
    return 'is not a minisign public key (base64 of the whole .pub file, as `tauri signer generate` prints it)';
  }
  return null;
}

/**
 * The overlay for `env`, plus the notes to print. Throws an Error with the
 * message to show on an invalid updater input.
 */
export function buildOverlay(env) {
  const overlay = {};
  const notes = [];
  if (env.MW_SHELL_PRODUCT_NAME) overlay.productName = env.MW_SHELL_PRODUCT_NAME;
  if (env.MW_SHELL_IDENTIFIER) overlay.identifier = env.MW_SHELL_IDENTIFIER;

  const url = env.MW_SHELL_UPDATER_URL ?? '';
  const key = (env.MW_SHELL_UPDATER_PUBKEY ?? '').trim();
  if (url !== '' && key !== '') {
    const badUrl = feedProblem(url);
    if (badUrl) throw new Error(`MW_SHELL_UPDATER_URL ${badUrl}: ${url}`);
    const badKey = pubkeyProblem(key);
    if (badKey) throw new Error(`MW_SHELL_UPDATER_PUBKEY ${badKey}`);
    overlay.bundle = { createUpdaterArtifacts: true };
    overlay.plugins = { updater: { pubkey: key, endpoints: [url], windows: { installMode: 'passive' } } };
    notes.push(`updater on: ${url}`);
    const missing = PLACEHOLDERS.filter((p) => !url.includes(p));
    if (missing.length > 0) notes.push(`note: the feed URL has no ${missing.join(', ')}; every build asks the same address`);
  } else if (url !== '' || key !== '') {
    const unset = url === '' ? 'MW_SHELL_UPDATER_URL' : 'MW_SHELL_UPDATER_PUBKEY';
    notes.push(`updater off: ${unset} is not set (both are needed); this build will not update itself`);
  } else {
    notes.push('updater off: MW_SHELL_UPDATER_URL and MW_SHELL_UPDATER_PUBKEY are not set');
  }
  return { overlay, notes };
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  try {
    const { overlay, notes } = buildOverlay(process.env);
    for (const n of notes) process.stderr.write(`[build-shells] ${n}\n`);
    process.stdout.write(JSON.stringify(overlay));
  } catch (e) {
    process.stderr.write(`[build-shells] ${e.message}\n`);
    process.exit(1);
  }
}
