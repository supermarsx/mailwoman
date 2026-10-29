#!/usr/bin/env node
/* global process, console, fetch, setTimeout */
// Fixture for receipts/linked-accounts-live.spec.ts (t46): a native engine-mode
// server built from the current tree, over a private Greenmail that holds both
// mailboxes the spec signs in to (parent@example.org, child@child.test), with
// mailbox linking switched on. No route mocks and no saved build.
//
//   cargo build -p mw-server --bin mailwoman
//   pnpm --dir apps/web build
//   node apps/web/e2e/receipts/linked-live-fixture.mjs
//   # in another shell, with the two URLs it prints:
//   MW_RECEIPTS_BASE_URL=... MW_RECEIPTS_SMTP=... npx playwright test \
//     -c apps/web/e2e/receipts/playwright.config.ts linked-accounts-live
//
// Environment (all optional):
//   MW_BIN          server binary            (default <repo>/target/debug/mailwoman[.exe],
//                                             or $CARGO_TARGET_DIR/debug/...)
//   MW_WEB_DIR      built web app            (default <repo>/apps/web/dist)
//   MW_FIXTURE_PORT server port              (default 46180)
//   MW_FIXTURE_SMTP / MW_FIXTURE_IMAP  host ports for Greenmail (default 46625 / 46643)
// Ctrl-C stops the server it started and removes its Greenmail container.

import { spawn, execFileSync } from 'node:child_process';
import { mkdtempSync, existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve, dirname } from 'node:path';
import { randomBytes } from 'node:crypto';
import { fileURLToPath } from 'node:url';

const repo = resolve(dirname(fileURLToPath(import.meta.url)), '../../../..');
const exe = process.platform === 'win32' ? 'mailwoman.exe' : 'mailwoman';
const bin = process.env.MW_BIN ?? join(process.env.CARGO_TARGET_DIR ?? join(repo, 'target'), 'debug', exe);
const web = process.env.MW_WEB_DIR ?? join(repo, 'apps/web/dist');
const port = Number(process.env.MW_FIXTURE_PORT ?? 46180);
const smtp = Number(process.env.MW_FIXTURE_SMTP ?? 46625);
const imap = Number(process.env.MW_FIXTURE_IMAP ?? 46643);
const container = `mw-linked-live-greenmail-${port}`;
const admin = { username: 'root', password: 'linked-fixture-admin' };
// t76: the console path this fixture serves (a new database would get a generated one).
const adminPath = process.env.MW_E2E_ADMIN_PATH ?? '/admin';
const base = `http://127.0.0.1:${port}`;

for (const [what, path] of [['server binary', bin], ['web build', join(web, 'index.html')]]) {
  if (!existsSync(path)) {
    console.error(`missing ${what}: ${path}`);
    process.exit(2);
  }
}

function docker(...args) {
  return execFileSync('docker', args, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
}
try { docker('rm', '-f', container); } catch { /* not there */ }
docker('run', '-d', '--name', container,
  '-p', `127.0.0.1:${smtp}:3025`, '-p', `127.0.0.1:${imap}:3143`,
  '-e', 'GREENMAIL_OPTS=-Dgreenmail.setup.test.all -Dgreenmail.hostname=0.0.0.0 -Dgreenmail.auth.disabled',
  'greenmail/standalone:2.1.4');

const data = mkdtempSync(join(tmpdir(), 'mw-linked-live-'));
const server = spawn(bin, ['serve'], {
  stdio: ['ignore', 'inherit', 'inherit'],
  env: {
    ...process.env,
    MW_MODE: 'engine',
    MW_BIND: `127.0.0.1:${port}`,
    MW_DB_PATH: join(data, 'mw.db'),
    MW_SERVER_KEY: randomBytes(32).toString('hex'),
    MW_WEB_DIR: web,
    MW_UPSTREAM_DEFAULT: `imap://127.0.0.1:${imap}`,
    MW_ENGINE_TLS: 'plaintext',
    MW_ADMIN_USER: admin.username,
    MW_ADMIN_PASSWORD: admin.password,
    // t76: a new database would get a generated console path.
    MW_ADMIN_PATH: adminPath,
    MW_REALTIME_RECHECK_SECS: '5',
    RUST_LOG: process.env.RUST_LOG ?? 'warn',
  },
});
let stopping = false;
function stop(code = 0) {
  if (stopping) return;
  stopping = true;
  try { server.kill(); } catch { /* gone */ }
  try { docker('rm', '-f', container); } catch { /* gone */ }
  process.exit(code);
}
process.on('SIGINT', () => stop(0));
process.on('SIGTERM', () => stop(0));
server.on('exit', (code) => { if (!stopping) { console.error(`server exited (${code})`); stop(1); } });

async function waitFor(check, what, ms = 120_000) {
  const end = Date.now() + ms;
  for (;;) {
    try { if (await check()) return; } catch { /* not yet */ }
    if (Date.now() > end) { console.error(`timed out waiting for ${what}`); stop(1); }
    await new Promise((r) => setTimeout(r, 500));
  }
}
await waitFor(async () => (await fetch(`${base}/`)).ok, 'the server');

// Mailbox linking is off by default; the admin console switches it on.
const login = await fetch(`${base}${adminPath}/login`, {
  method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(admin),
});
if (!login.ok) { console.error(`admin login: ${login.status} ${await login.text()}`); stop(1); }
const cookie = login.headers.getSetCookie().map((c) => c.split(';')[0]).join('; ');
const csrf = (await login.clone().json().catch(() => ({}))).csrfToken;
for (const [key, value] of [['accounts.linkingEnabled', true], ['signin.retryCooldownSecs', 0]]) {
  const put = await fetch(`${base}${adminPath}/settings/${key}`, {
    method: 'PUT',
    headers: { 'content-type': 'application/json', cookie, ...(csrf ? { 'x-csrf-token': csrf } : {}) },
    body: JSON.stringify({ globalValue: value }),
  });
  if (!put.ok) { console.error(`PUT ${key}: ${put.status} ${await put.text()}`); stop(1); }
}
console.log(`READY MW_RECEIPTS_BASE_URL=${base} MW_RECEIPTS_SMTP=127.0.0.1:${smtp} (server pid ${server.pid}, data ${data})`);
