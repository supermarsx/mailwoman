// Named seeds: demo/seeds.json (core) and demo/extra/<area>.json (one file per
// writer). A seed is an object with any of these keys, applied in this order:
//
//   include        other seed names, applied first
//   stacks         ["M"], ["E"] or both (default both); a capture on another stack fails
//   mail           dataset file(s) under demo/ (mail/basic.json): folders + messages,
//                  put into the mail backend before the server starts
//   adminStepUp    true: confirm the administrator's password (POST <admin>/step-up) first
//   userStepUp     true: confirm the demo user's password (POST /api/security/step-up) first
//   userTotp       true: enrol an authenticator (TOTP) for the demo user; the harness
//                  keeps the secret and can type codes (step "totp")
//   adminSettings  { "<registry key>": value }  PUT <admin>/settings/<key> {globalValue}
//   admin          [{ "method", "path" (under the admin path), "body" }]
//   userSettings   { "<registry key>": value }  PUT /api/settings/<key> {value}
//   user           [{ "method", "path", "body" }] as the signed-in demo user
//   jmap           [{ "using": [...], "calls": [[method, args, id], ...] }] as the user
//   pim            dataset file(s) under demo/ (pim.json): calendars, events, tasks,
//                  notes, contacts, written through the user's JMAP session (stack E)
//
// Every value may use {{me}} (the demo address), {{admin}} (the admin path),
// {{person:<key>}} (an address from people.json) and {{date:+3d}} / {{date:-2h}}
// (ISO instant relative to the capture clock, 2026-03-10T09:30:00Z).
import { createHmac } from 'node:crypto';
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { DEMO, log, waitFor } from './paths.mjs';
import { imapDate, imapSession, quote } from './imap.mjs';
import { loadMail, toJmap, toMime } from './mime.mjs';
import { api, userAgent } from './browser.mjs';

export const CLOCK = '2026-03-10T09:30:00Z';

/** Every seed definition by name, with the file that defines it. */
export function loadSeedDefs() {
  const files = [join(DEMO, 'seeds.json')];
  const extra = join(DEMO, 'extra');
  if (existsSync(extra)) for (const f of readdirSync(extra).sort()) if (f.endsWith('.json')) files.push(join(extra, f));
  const defs = new Map();
  for (const file of files) {
    const data = JSON.parse(readFileSync(file, 'utf8'));
    for (const [name, def] of Object.entries(data.seeds ?? {})) {
      if (defs.has(name)) throw new Error(`seed '${name}' is defined twice (${defs.get(name).file} and ${file})`);
      defs.set(name, { ...def, name, file });
    }
  }
  return defs;
}

/** The seeds to apply for `names`, includes first, each once. */
export function expandSeeds(names, defs = loadSeedDefs()) {
  const out = [];
  const visit = (name, path) => {
    if (out.some((d) => d.name === name)) return;
    if (path.includes(name)) throw new Error(`seed include cycle: ${[...path, name].join(' -> ')}`);
    const def = defs.get(name);
    if (!def) throw new Error(`unknown seed '${name}' (define it in docs/screenshots/demo/seeds.json or demo/extra/<area>.json)`);
    for (const inc of def.include ?? []) visit(inc, [...path, name]);
    out.push(def);
  };
  for (const n of names) visit(n, []);
  return out;
}

const list = (v) => (v === undefined ? [] : Array.isArray(v) ? v : [v]);

function relDate(spec) {
  const m = /^([+-])(\d+)([mhd])$/.exec(spec);
  if (!m) throw new Error(`bad {{date:${spec}}} (use +3d, -2h, +30m)`);
  const ms = { m: 60e3, h: 3600e3, d: 86400e3 }[m[3]] * Number(m[2]) * (m[1] === '-' ? -1 : 1);
  return new Date(Date.parse(CLOCK) + ms).toISOString().replace('.000Z', 'Z');
}

/** Substitute {{...}} placeholders in every string of `value`. */
export function expand(value, stack) {
  const people = JSON.parse(readFileSync(join(DEMO, 'people.json'), 'utf8'));
  const sub = (s) =>
    s.replace(/\{\{([^}]+)\}\}/g, (all, key) => {
      if (key === 'me') return stack.user.username;
      if (key === 'admin') return stack.adminPath;
      if (key.startsWith('person:')) return people.people[key.slice(7)]?.email ?? all;
      if (key.startsWith('date:')) return relDate(key.slice(5));
      return all;
    });
  const walk = (v) => (typeof v === 'string' ? sub(v) : Array.isArray(v) ? v.map(walk) : v && typeof v === 'object' ? Object.fromEntries(Object.entries(v).map(([k, x]) => [sub(k), walk(x)])) : v);
  return walk(value);
}

// ── mail (before the server starts) ────────────────────────────────────────────

async function mockJmap(stack, calls) {
  const auth = `Basic ${Buffer.from(`${stack.user.username}:${stack.user.password}`).toString('base64')}`;
  const r = await fetch(`${stack.jmapUrl}/jmap`, {
    method: 'POST',
    headers: { authorization: auth, 'content-type': 'application/json' },
    body: JSON.stringify({ using: ['urn:ietf:params:jmap:core', 'urn:ietf:params:jmap:mail'], methodCalls: calls }),
  });
  if (!r.ok) throw new Error(`mock JMAP ${r.status}`);
  return (await r.json()).methodResponses;
}

async function seedMockMail(stack, datasets) {
  // The mock starts with three test messages; the demo mailbox replaces them.
  const [[, q]] = await mockJmap(stack, [['Email/query', { filter: { inMailbox: 'mb-inbox' } }, 'q']]);
  if (q.ids.length) await mockJmap(stack, [['Email/set', { destroy: q.ids }, 'd']]);
  let count = 0;
  for (const { folders, messages } of datasets) {
    const [[, got]] = await mockJmap(stack, [['Mailbox/get', {}, 'g']]);
    const ids = new Map(got.list.map((m) => [m.name.toLowerCase(), m.id]));
    const missing = [...new Set([...folders, ...messages.map((m) => m.folder)])].filter((f) => !ids.has(f.toLowerCase()));
    if (missing.length) {
      const [[, set]] = await mockJmap(stack, [['Mailbox/set', { create: Object.fromEntries(missing.map((f, i) => [`f${i}`, { name: f }])) }, 's']]);
      missing.forEach((f, i) => ids.set(f.toLowerCase(), set.created[`f${i}`].id));
    }
    // The mock lists newest-created first, so create oldest first.
    for (const m of messages) {
      const [[name, res]] = await mockJmap(stack, [['Email/set', { create: { m: toJmap(m, ids.get(m.folder.toLowerCase())) } }, 'c']]);
      if (name === 'error' || !res.created?.m) throw new Error(`mock Email/set for ${m.id}: ${JSON.stringify(res)}`);
      count += 1;
    }
  }
  log(`seeded ${count} messages into the mock`);
}

const IMAP_NAMES = { inbox: 'INBOX' };

async function seedImapMail(stack, datasets) {
  const s = await imapSession(stack.imap);
  let count = 0;
  try {
    await s.command(`LOGIN ${quote(stack.user.username)} ${quote(stack.user.password)}`);
    const created = new Set(['INBOX']);
    const folder = (name) => IMAP_NAMES[name.toLowerCase()] ?? name;
    for (const { folders, messages } of datasets) {
      for (const f of ['Sent', ...folders, ...messages.map((m) => m.folder)].map(folder)) {
        if (created.has(f)) continue;
        await s.command(`CREATE ${quote(f)}`).catch((e) => {
          if (!/exist/i.test(e.message)) throw e;
        });
        created.add(f);
      }
      for (const m of messages) {
        const flags = [...(m.unread ? [] : ['\\Seen']), ...(m.flagged ? ['\\Flagged'] : []), ...(m.draft ? ['\\Draft'] : []), ...(m.keywords ?? [])];
        await s.command(`APPEND ${quote(folder(m.folder))} (${flags.join(' ')}) ${quote(imapDate(m.date))}`, toMime(m));
        count += 1;
      }
    }
    await s.command('LOGOUT').catch(() => {});
  } finally {
    s.close();
  }
  log(`seeded ${count} messages over IMAP`);
}

/** The mail part of `names`, as the stack's `beforeServer` hook. */
export function mailSeeder(names) {
  const seeds = expandSeeds(names);
  return async (stack) => {
    for (const seed of seeds) {
      if (seed.stacks && !seed.stacks.includes(stack.kind)) throw new Error(`seed '${seed.name}' does not run on stack ${stack.kind}`);
    }
    const datasets = seeds.flatMap((s) => list(s.mail)).map(loadMail);
    stack.seededMessages = datasets.reduce((n, d) => n + d.messages.length, 0);
    if (!datasets.length) return;
    if (stack.kind === 'M') await seedMockMail(stack, datasets);
    else await seedImapMail(stack, datasets);
  };
}

// ── everything else (after the server is up) ───────────────────────────────────

/** POST JMAP calls through the server as the signed-in user. */
export async function userJmap(client, using, calls) {
  const r = await client.must('POST', '/jmap/api', { using, methodCalls: calls });
  for (const [name, args, id] of r.json.methodResponses) {
    if (name === 'error') throw new Error(`JMAP ${id}: ${JSON.stringify(args)}`);
    for (const key of ['notCreated', 'notUpdated', 'notDestroyed']) {
      if (args?.[key] && Object.keys(args[key]).length) throw new Error(`JMAP ${name} ${key}: ${JSON.stringify(args[key])}`);
    }
  }
  return r.json.methodResponses;
}

async function waitForSync(client, stack) {
  if (stack.kind !== 'E' || !stack.seededMessages) return;
  await waitFor(
    async () => {
      const [[, got]] = await userJmap(client, ['urn:ietf:params:jmap:core', 'urn:ietf:params:jmap:mail'], [['Mailbox/get', { properties: ['totalEmails'] }, 'g']]);
      return got.list.reduce((n, m) => n + (m.totalEmails ?? 0), 0) >= stack.seededMessages;
    },
    'the engine to sync the seeded mail',
    120_000,
  );
}

const MAIL_USING = ['urn:ietf:params:jmap:core', 'urn:ietf:params:jmap:mail'];

// ── second factor ──────────────────────────────────────────────────────────────

function base32Decode(text) {
  const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
  let bits = '';
  for (const c of text.replace(/=+$/, '').toUpperCase()) {
    const v = alphabet.indexOf(c);
    if (v < 0) continue;
    bits += v.toString(2).padStart(5, '0');
  }
  const bytes = [];
  for (let i = 0; i + 8 <= bits.length; i += 8) bytes.push(parseInt(bits.slice(i, i + 8), 2));
  return Buffer.from(bytes);
}

/** RFC 6238 code (SHA-1, 6 digits, 30 s) for `secret` (base32) at time step `step`. */
export function totpCode(secret, step) {
  const counter = Buffer.alloc(8);
  counter.writeBigUInt64BE(BigInt(step));
  const mac = createHmac('sha1', base32Decode(secret)).update(counter).digest();
  const offset = mac[mac.length - 1] & 0x0f;
  const value = (mac.readUInt32BE(offset) & 0x7fffffff) % 1_000_000;
  return String(value).padStart(6, '0');
}

/**
 * A code for the demo user's authenticator that the server has not seen yet: the
 * server refuses a code from a 30-second step it already accepted, so this waits
 * for the next step when needed.
 */
export async function nextTotpCode(stack) {
  if (!stack.totp) throw new Error("no authenticator is enrolled; add a seed with \"userTotp\": true");
  let step = Math.floor(Date.now() / 30_000);
  if (step <= stack.totp.lastStep) {
    await new Promise((r) => setTimeout(r, (stack.totp.lastStep + 1) * 30_000 - Date.now() + 250));
    step = Math.floor(Date.now() / 30_000);
  }
  stack.totp.lastStep = step;
  return totpCode(stack.totp.secret, step);
}

/** Enrol an authenticator (TOTP) for the demo user and keep its secret on the stack. */
async function enrolTotp(client, stack) {
  const begin = await client.must('POST', '/api/account/2fa/totp/begin', {});
  const secret = begin.json?.secret;
  if (!secret) throw new Error(`authenticator enrolment returned no secret: ${begin.text.slice(0, 200)}`);
  stack.totp = { secret, lastStep: -1 };
  await client.must('POST', '/api/account/2fa/totp/confirm', { code: await nextTotpCode(stack) });
}

/**
 * Remove what the harness's own sign-ins leave behind: the "new browser or device"
 * security notices the server mails to the account, and the notification-centre
 * entries. They carry today's date and are not part of the demo.
 */
export async function tidy(client, stack, { expectNotice = true } = {}) {
  const notices = async () => (await userJmap(client, MAIL_USING, [['Email/query', { filter: { text: 'from:security@mailwoman.invalid' } }, 'q']]))[0][1];
  let q = await notices();
  // On stack E the notice is appended in the background shortly after sign-in.
  for (let i = 0; stack.kind === 'E' && !q.ids?.length && i < (expectNotice ? 20 : 3); i += 1) {
    await new Promise((r) => setTimeout(r, 500));
    q = await notices();
  }
  if (q.ids?.length) await userJmap(client, MAIL_USING, [['Email/set', { destroy: q.ids }, 'd']]);
  const list = await client.call('GET', '/api/account/notification-center');
  for (const notice of list.json?.notices ?? []) {
    if (notice.state !== 'dismissed') await client.call('POST', '/api/account/notification-center', { id: notice.id, action: 'dismiss' });
  }
}

/**
 * Apply the post-start parts of seeds `names` to `stack`, then keep the signed-in
 * cookies of the demo user and the administrator (`stack.userState`,
 * `stack.adminState`): captures reuse them, so taking a screenshot is not a new
 * sign-in (which would mail another security notice and add a notification).
 */
export async function applySeeds(browser, stack, names) {
  const seeds = expandSeeds(names).map((s) => expand(s, stack));
  const adminContext = await browser.newContext({ userAgent: userAgent() });
  try {
    const client = api(adminContext, stack.base);
    await client.signInAdmin(stack);
    for (const seed of seeds) {
      // Some console writes (outbound proxies, single sign-on, Nextcloud) need a
      // recent password confirmation of the administrator first.
      if (seed.adminStepUp) await client.must('POST', `${stack.adminPath}/step-up`, { password: stack.admin.password });
      for (const [key, value] of Object.entries(seed.adminSettings ?? {})) {
        await client.must('PUT', `${stack.adminPath}/settings/${key}`, { globalValue: value });
      }
      for (const req of seed.admin ?? []) await client.must(req.method ?? 'POST', `${stack.adminPath}${req.path}`, req.body);
    }
    stack.adminState = await adminContext.storageState();
  } finally {
    await adminContext.close();
  }
  const context = await browser.newContext({ userAgent: userAgent() });
  try {
    const client = api(context, stack.base);
    stack.me = await client.signInUser(stack.user);
    await waitForSync(client, stack);
    for (const seed of seeds) {
      if (seed.userStepUp || seed.userTotp) await client.must('POST', '/api/security/step-up', { password: stack.user.password });
      if (seed.userTotp) await enrolTotp(client, stack);
      for (const [key, value] of Object.entries(seed.userSettings ?? {})) {
        await client.must('PUT', `/api/settings/${key}`, { value });
      }
      for (const req of seed.user ?? []) await client.must(req.method ?? 'POST', req.path, req.body);
      for (const batch of seed.jmap ?? []) await userJmap(client, batch.using, batch.calls);
      if (seed.pim) {
        const { seedPim } = await import('./pim.mjs');
        for (const file of list(seed.pim)) await seedPim(client, stack, file);
      }
    }
    await tidy(client, stack);
    stack.userState = await context.storageState();
  } finally {
    await context.close();
  }
}
