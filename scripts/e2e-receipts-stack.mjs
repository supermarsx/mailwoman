#!/usr/bin/env node
/* global process, console, fetch, setTimeout */
// The stack the nested Playwright config apps/web/e2e/receipts runs against
// (t39 C4). Everything it serves is built from the current tree: no saved build,
// no route mocks.
//
//   - Greenmail 2.1.4 in Docker: testuser/testpass@example.org preseeded, sign-in
//     checks off, so parent@example.org and child@child.test (the linked-mailbox
//     specs) are created on first sign-in;
//   - apps/web/e2e/fixtures/delivery-smtp.py as the server's SMTP submission
//     endpoint: it records every accepted message in MW_DSN_SMTP_CAPTURE
//     (accepted.jsonl + <sha256>.eml) and relays it to Greenmail, so a sent
//     message also arrives in the recipient's mailbox;
//   - the mailwoman binary in engine mode over that Greenmail, with the admin
//     console on (root / hunter2), mailbox linking on and the sign-in retry
//     cooldown at 0;
//   - the fixed 30-message seed from apps/web/e2e/restyle/seed.mjs.
//
// MW_STACK_IMAP_SERVER=dovecot (t93) swaps Greenmail for Dovecot 2.4.4 with the
// scripts/dovecot-t32 account (testuser/testpass, plaintext IMAP, UIDPLUS and
// IMAP4rev2). Specs that need UIDPLUS (account Drafts) run on that leg and skip
// on Greenmail; the env file says which one is up (MW_RECEIPTS_IMAP_SERVER).
// That leg has no SMTP server: the capture fixture records submissions without
// relaying them, and there is no 30-message seed.
//
// Build, start, run:
//   cargo build -p mw-server --bin mailwoman
//   pnpm -C apps/web build
//   node scripts/e2e-receipts-stack.mjs          # stays in the foreground; Ctrl-C stops it
//   # in another shell (the env file holds the values the specs read):
//   set -a; . apps/web/test-results/receipts-stack.env; set +a
//   pnpm -C apps/web exec playwright test -c e2e/receipts/playwright.config.ts --workers=1
//
// When it is ready it writes the env file and prints one line starting READY.
// Environment (all optional):
//   MW_BIN             server binary   (default $CARGO_TARGET_DIR or <repo>/target, /debug/mailwoman[.exe])
//   MW_WEB_DIR         built web app   (default <repo>/apps/web/dist)
//   MW_STACK_PORT      server port     (default 42209, the receipts config default)
//   MW_STACK_SMTP      Greenmail SMTP  (default 42225)
//   MW_STACK_IMAP      Greenmail IMAP  (default 42243)
//   MW_STACK_CAPTURE   capture SMTP    (default 42226)
//   MW_STACK_ENV       env file        (default <repo>/apps/web/test-results/receipts-stack.env)
//   MW_DSN_SMTP_CAPTURE capture dir    (default <repo>/apps/web/test-results/receipts-smtp; emptied at start)
//   MW_STACK_IMAP_SERVER greenmail (default) or dovecot
//   MW_STACK_CAPTURE_RELAY off disables local Greenmail delivery (wire-only special fixtures)
//   MW_CAPTURE_SMTPUTF8 1 enables UTF-8 SMTP capture and capability-toggle marker for special tests
//   PYTHON             interpreter for the capture fixture (default python3, python on Windows)
// It stops the server and the capture fixture it started and removes its
// Greenmail container on Ctrl-C, SIGTERM, or when any of them exits.

import { spawn, execFileSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, rmSync, writeFileSync, existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve, dirname } from 'node:path';
import { randomBytes } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import net from 'node:net';

const repo = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const exe = process.platform === 'win32' ? 'mailwoman.exe' : 'mailwoman';
const bin = process.env.MW_BIN ?? join(process.env.CARGO_TARGET_DIR ?? join(repo, 'target'), 'debug', exe);
const web = resolve(process.env.MW_WEB_DIR ?? join(repo, 'apps/web/dist'));
const port = Number(process.env.MW_STACK_PORT ?? 42209);
const smtp = Number(process.env.MW_STACK_SMTP ?? 42225);
const imap = Number(process.env.MW_STACK_IMAP ?? 42243);
const capturePort = Number(process.env.MW_STACK_CAPTURE ?? 42226);
const envFile = resolve(process.env.MW_STACK_ENV ?? join(repo, 'apps/web/test-results/receipts-stack.env'));
const capture = resolve(process.env.MW_DSN_SMTP_CAPTURE ?? join(repo, 'apps/web/test-results/receipts-smtp'));
const python = process.env.PYTHON ?? (process.platform === 'win32' ? 'python' : 'python3');
const imapServer = process.env.MW_STACK_IMAP_SERVER ?? 'greenmail';
if (imapServer !== 'greenmail' && imapServer !== 'dovecot') {
  console.error(`MW_STACK_IMAP_SERVER must be greenmail or dovecot, not ${imapServer}`);
  process.exit(2);
}
const dovecot = imapServer === 'dovecot';
const container = `mw-receipts-${imapServer}-${port}`;
const admin = { username: 'root', password: 'hunter2' };
// t76: the console path the stack serves (a new database would otherwise get a
// generated one) and hands to the specs as MW_E2E_ADMIN_PATH.
const adminPath = process.env.MW_E2E_ADMIN_PATH ?? '/admin';
const local = `http://127.0.0.1:${port}`;
// The browser-facing origin: WebAuthn (the passkeys specs) needs a host name,
// not an IP literal, and localhost is a secure context over plain HTTP.
const origin = `http://localhost:${port}`;

for (const [what, path] of [['server binary', bin], ['web build', join(web, 'index.html')]]) {
  if (!existsSync(path)) {
    console.error(`missing ${what}: ${path}`);
    process.exit(2);
  }
}

function docker(...args) {
  return execFileSync('docker', args, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
}

const children = [];
let stopping = false;
function stop(code = 0) {
  if (stopping) return;
  stopping = true;
  for (const child of children) {
    try { child.kill(); } catch { /* gone */ }
  }
  try { docker('rm', '-f', container); } catch { /* gone */ }
  try { rmSync(envFile, { force: true }); } catch { /* gone */ }
  process.exit(code);
}
process.on('SIGINT', () => stop(0));
process.on('SIGTERM', () => stop(0));

function start(name, command, args, env) {
  const child = spawn(command, args, { stdio: ['ignore', 'inherit', 'inherit'], env: { ...process.env, ...env } });
  child.on('exit', (code) => { if (!stopping) { console.error(`${name} exited (${code})`); stop(1); } });
  child.on('error', (error) => { if (!stopping) { console.error(`${name}: ${error.message}`); stop(1); } });
  children.push(child);
  return child;
}

async function waitFor(check, what, ms = 180_000) {
  const end = Date.now() + ms;
  for (;;) {
    try { if (await check()) return; } catch { /* not yet */ }
    if (Date.now() > end) { console.error(`timed out waiting for ${what}`); stop(1); }
    await new Promise((r) => setTimeout(r, 500));
  }
}

const listening = (p) => new Promise((ok) => {
  const socket = net.connect(p, '127.0.0.1');
  socket.once('connect', () => { socket.destroy(); ok(true); });
  socket.once('error', () => ok(false));
});

rmSync(envFile, { force: true });
rmSync(capture, { recursive: true, force: true });
mkdirSync(capture, { recursive: true });
mkdirSync(dirname(envFile), { recursive: true });

try { docker('rm', '-f', container); } catch { /* not there */ }
if (dovecot) {
  const conf = join(repo, 'scripts/dovecot-t32');
  docker('run', '-d', '--name', container, '-p', `127.0.0.1:${imap}:143`,
    '-v', `${join(conf, 'zz-t32.conf')}:/etc/dovecot/conf.d/zz-t32.conf:ro`,
    '-v', `${join(conf, 'mailwoman-users')}:/etc/dovecot/mailwoman-users:ro`,
    'dovecot/dovecot:2.4.4');
  // A listening port is not enough: wait for the greeting.
  await waitFor(() => new Promise((ok) => {
    const socket = net.connect(imap, '127.0.0.1');
    socket.once('data', (chunk) => { socket.destroy(); ok(chunk.toString().startsWith('* OK')); });
    socket.once('error', () => ok(false));
    // Docker's port proxy accepts and then closes while Dovecot is starting.
    socket.once('close', () => ok(false));
    socket.setTimeout(2000, () => { socket.destroy(); ok(false); });
  }), 'dovecot');
} else {
  docker('run', '-d', '--name', container,
    '-p', `127.0.0.1:${smtp}:3025`, '-p', `127.0.0.1:${imap}:3143`,
    '-e', 'GREENMAIL_OPTS=-Dgreenmail.setup.test.all -Dgreenmail.hostname=0.0.0.0 -Dgreenmail.users=testuser:testpass@example.org -Dgreenmail.auth.disabled',
    'greenmail/standalone:2.1.4');
  await waitFor(async () => (await listening(smtp)) && (await listening(imap)), 'greenmail');
}

start('capture SMTP', python, [join(repo, 'apps/web/e2e/fixtures/delivery-smtp.py'), String(capturePort), capture],
  dovecot || process.env.MW_STACK_CAPTURE_RELAY === 'off' ? { MW_CAPTURE_RELAY: '' } : { MW_CAPTURE_RELAY: `127.0.0.1:${smtp}` });
await waitFor(() => listening(capturePort), 'the capture SMTP fixture');

const data = mkdtempSync(join(tmpdir(), 'mw-receipts-stack-'));
const server = start('server', bin, ['serve'], {
  MW_MODE: 'engine',
  MW_BIND: `127.0.0.1:${port}`,
  MW_DB_PATH: join(data, 'mw.db'),
  MW_UPLOAD_DIR: join(data, 'uploads'),
  MW_SERVER_KEY: randomBytes(32).toString('hex'),
  MW_WEB_DIR: web,
  MW_UPSTREAM_DEFAULT: `imap://127.0.0.1:${imap}`,
  MW_ENGINE_TLS: 'plaintext',
  MW_SMTP_HOST: '127.0.0.1',
  MW_SMTP_PORT: String(capturePort),
  MW_SMTP_SECURITY: 'plaintext',
  MW_COOKIE_SECURE: 'false',
  MW_CSRF_STRICT: 'true',
  MW_ADMIN_USER: admin.username,
  MW_ADMIN_PASSWORD: admin.password,
  // t76: a new database would get a generated console path.
  MW_ADMIN_PATH: adminPath,
  MW_ALLOWED_FROMS: 'testuser@example.org',
  MW_PUBLIC_URL: origin,
  MW_WEBAUTHN_RP_ID: 'localhost',
  MW_WEBAUTHN_ORIGIN: origin,
  MW_REALTIME_RECHECK_SECS: '5',
  RUST_LOG: process.env.RUST_LOG ?? 'warn',
});
await waitFor(async () => (await fetch(`${local}/healthz`)).ok, 'the server');

// Mailbox linking is off by default; the admin console switches it on. The
// retry cooldown is set to 0 because the suite signs in many times in a row.
const login = await fetch(`${local}${adminPath}/login`, {
  method: 'POST', headers: { 'content-type': 'application/json', origin: local }, body: JSON.stringify(admin),
});
if (!login.ok) { console.error(`admin login: ${login.status} ${await login.text()}`); stop(1); }
const cookie = login.headers.getSetCookie().map((c) => c.split(';')[0]).join('; ');
const csrf = (await login.clone().json().catch(() => ({}))).csrfToken;
for (const [key, value] of [['accounts.linkingEnabled', true], ['signin.retryCooldownSecs', 0]]) {
  const put = await fetch(`${local}${adminPath}/settings/${key}`, {
    method: 'PUT',
    headers: { 'content-type': 'application/json', origin: local, cookie, ...(csrf ? { 'x-csrf-token': csrf } : {}) },
    body: JSON.stringify({ globalValue: value }),
  });
  if (!put.ok) { console.error(`PUT ${key}: ${put.status} ${await put.text()}`); stop(1); }
}

// 30 fixed messages in testuser's INBOX (list-keyboard-live needs at least 30).
if (!dovecot) {
  execFileSync(process.execPath, [join(repo, 'apps/web/e2e/restyle/seed.mjs')], {
    stdio: 'inherit',
    env: { ...process.env, MW_E2E_SMTP_HOST: '127.0.0.1', MW_E2E_SMTP_PORT: String(smtp), MW_E2E_IMAP_PORT: String(imap) },
  });
}

const env = {
  MW_RECEIPTS_BASE_URL: origin,
  MW_RECEIPTS_IMAP_SERVER: imapServer,
  // The Dovecot leg has no SMTP server to deliver into.
  ...(dovecot ? {} : { MW_RECEIPTS_SMTP: `127.0.0.1:${smtp}`, MW_E2E_SMTP_HOST: '127.0.0.1', MW_E2E_SMTP_PORT: String(smtp) }),
  MW_E2E_IMAP_PORT: String(imap),
  MW_T49_IMAP: `127.0.0.1:${imap}`,
  // Forward slashes, so a POSIX shell can source the file on Windows too.
  MW_DSN_SMTP_CAPTURE: capture.replaceAll('\\', '/'),
  MW_ADMIN_USER: admin.username,
  MW_ADMIN_PASSWORD: admin.password,
  MW_E2E_ADMIN_PATH: adminPath,
  // The same server for the passkeys and restyle configs.
  PLAYWRIGHT_PASSKEY_BASE_URL: origin,
  PLAYWRIGHT_ENGINE_BASE_URL: origin,
};
writeFileSync(envFile, Object.entries(env).map(([k, v]) => `${k}=${v}\n`).join(''));
console.log(`READY ${Object.entries(env).map(([k, v]) => `${k}=${v}`).join(' ')} (env file ${envFile}, server pid ${server.pid}, data ${data})`);
