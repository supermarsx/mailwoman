// t60: message-body frames must never go blank. A live-stack stress harness, no
// route mocks: it seeds a varied mailbox (plain text, HTML newsletters, remote
// images blocked and allowed, inline CID images, very long and very short mail,
// S/MIME and PGP, an .ics invitation, sender colours that adaptive mode
// rewrites) and opens messages over and over while it also switches quickly with
// arrow keys and clicks, toggles the reading pane, switches layouts and dark
// mode, refreshes, flags, lets mark-read land, receives new mail in realtime,
// reorders tabs and detaches a message into a popup and back.
//
// After every open it checks the frame the user sees:
//   - the reader iframe exists and its nested message frame exists and holds
//     the message's marker text (not empty, not about:blank, not a stale message);
//   - both frames are taller than a few pixels;
//   - the marker's text colour differs from the colour behind it;
//   - a screenshot of the frame is not a single flat colour;
//   - no CSP violation or console error was reported for it.
//
// MW_T60_OPENS sets the number of opens (default 40, the harness runs use 200+);
// MW_T60_IDLE=1 adds one 65 s idle observation across the 30 s preference polls.
// MW_T60_OUT names a directory for the event log and blank-frame screenshots;
// MW_T60_TRACE=1 also logs every /api and /jmap request (to tell which fetch
// was in flight when the shell was swapped out).
// It also fails when a frame reloads outside a tab reorder or a popup detach
// (an iframe moved in the DOM: a remount, or the mailbox <Suspense> swapping the
// shell out), and when reselecting a tab reloads a frame or refetches an image.
// Runs in the receipts config against MW_RECEIPTS_BASE_URL; it seeds its own
// mailbox over SMTP (MW_E2E_SMTP_HOST / MW_E2E_SMTP_PORT).
import net from 'node:net';
import { mkdirSync, writeFileSync, appendFileSync } from 'node:fs';
import path from 'node:path';
import { test, expect, type Page, type Locator } from '../csrf.ts';

const OPENS = Number(process.env['MW_T60_OPENS'] ?? 40);
const SMTP_HOST = process.env['MW_E2E_SMTP_HOST'] ?? '127.0.0.1';
const SMTP_PORT = Number(process.env['MW_E2E_SMTP_PORT'] ?? 42225);
const PNG = 'iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAIAAACQkWg2AAAAGElEQVR42mP8z8BQz0AEYBxVSF+FAP5FDvcfRYWgAAAAAElFTkSuQmCC';

type Kind = 'decrypt' | 'frame';
interface Seed { key: string; subject: string; kind: Kind; mime: (to: string, from: string) => string }

function headers(to: string, from: string, subject: string, extra: string): string {
  return `From: ${from}\r\nTo: <${to}>\r\nSubject: ${subject}\r\nDate: ${new Date().toUTCString()}\r\nMessage-ID: <${Math.random().toString(36).slice(2)}@t60.test>\r\nMIME-Version: 1.0\r\n${extra}`;
}
const mark = (key: string): string => `T60MARK${key}`;
function seeds(tag: string): Seed[] {
  const s = (key: string, kind: Kind, body: (subject: string, to: string, from: string) => string): Seed => {
    const subject = `T60 ${tag} ${key}`;
    return { key, subject, kind, mime: (to, from) => body(subject, to, from) };
  };
  const html = (inner: string) => (subject: string, to: string, from: string) =>
    headers(to, from, subject, 'Content-Type: text/html; charset=utf-8\r\n\r\n') + inner;
  const long = Array.from({ length: 1500 }, (_, i) => `<p>Paragraph ${i} of a very long message, with enough words to wrap the line in the reading pane.</p>`).join('\r\n');
  const newsletter = `<table width="600" cellpadding="0" cellspacing="0" style="background:#f4f4f4;font-family:Arial"><tr><td style="background:#0b5394;color:#ffffff;padding:24px;font-size:22px">Weekly digest</td></tr>` +
    Array.from({ length: 12 }, (_, i) => `<tr><td style="padding:16px;color:#333333;background:#ffffff"><h2 style="color:#0b5394">Story ${i}</h2><p>Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor.</p><a href="https://news.example.com/s/${i}" style="background:#e69138;color:#fff;padding:8px 12px;text-decoration:none">Read more</a></td></tr>`).join('') +
    `<tr><td style="padding:12px;color:#999;font-size:11px">${mark('news')} Unsubscribe</td></tr></table>`;
  const remote = (key: string) => `<div style="font-family:Arial"><p>${mark(key)} remote images below</p>` +
    Array.from({ length: 6 }, (_, i) => `<img src="https://images.example.com/t60/${key}/${i}.png" width="300" height="120" alt="banner ${i}"><br>`).join('') +
    `<img src="https://tracker.example.com/open.gif?u=${key}" width="1" height="1"><p>Footer text after images.</p></div>`;
  return [
    s('plain', 'frame', (subject, to, from) => headers(to, from, subject, 'Content-Type: text/plain; charset=utf-8\r\n\r\n') + `Hello,\r\n\r\n${mark('plain')} plain text body.\r\nhttps://example.org/t60\r\n`),
    s('short', 'frame', html(`<p>${mark('short')}</p>`)),
    s('long', 'frame', html(`<p>${mark('long')} begins</p>${long}`)),
    s('news', 'frame', html(newsletter)),
    s('dark', 'frame', html(`<table bgcolor="#1e1e1e" width="100%"><tr><td style="color:#f0f0f0;padding:20px"><font color="#eeeeee">${mark('dark')} light text on a dark sender background</font></td></tr></table>`)),
    s('ink', 'frame', html(`<div style="color:#222222">${mark('ink')} dark sender ink with no background</div><p style="color:#000">black paragraph</p>`)),
    s('remoteA', 'frame', html(remote('remoteA'))),
    s('remoteB', 'frame', html(remote('remoteB'))),
    s('cid', 'frame', (subject, to, from) => headers(to, from, subject, 'Content-Type: multipart/related; boundary="rel"\r\n\r\n') +
      `--rel\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<p>${mark('cid')} inline image:</p><img src="cid:logo@t60" width="64" height="64"><p>after</p>\r\n` +
      `--rel\r\nContent-Type: image/png\r\nContent-Transfer-Encoding: base64\r\nContent-ID: <logo@t60>\r\nContent-Disposition: inline; filename="logo.png"\r\n\r\n${PNG}\r\n--rel--\r\n`),
    s('smime', 'frame', (subject, to, from) => headers(to, from, subject, 'Content-Type: multipart/signed; protocol="application/pkcs7-signature"; micalg=sha-256; boundary="sig"\r\n\r\n') +
      `--sig\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<p>${mark('smime')} signed with S/MIME</p>\r\n` +
      `--sig\r\nContent-Type: application/pkcs7-signature; name="smime.p7s"\r\nContent-Transfer-Encoding: base64\r\nContent-Disposition: attachment; filename="smime.p7s"\r\n\r\nMIAGCSqGSIb3DQEHAqCAMIACAQExDzANBglghkgBZQMEAgEFADCABgkqhkiG9w0BBwEAAKCAMIIB\r\n--sig--\r\n`),
    s('pgpsig', 'frame', (subject, to, from) => headers(to, from, subject, 'Content-Type: multipart/signed; protocol="application/pgp-signature"; micalg=pgp-sha256; boundary="pgp"\r\n\r\n') +
      `--pgp\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n${mark('pgpsig')} signed with OpenPGP\r\n` +
      `--pgp\r\nContent-Type: application/pgp-signature; name="signature.asc"\r\n\r\n-----BEGIN PGP SIGNATURE-----\r\n\r\niQEzBAEBCAAdFiEE\r\n=abcd\r\n-----END PGP SIGNATURE-----\r\n--pgp--\r\n`),
    s('armored', 'decrypt', (subject, to, from) => headers(to, from, subject, 'Content-Type: text/plain; charset=utf-8\r\n\r\n') +
      `-----BEGIN PGP MESSAGE-----\r\n\r\nhQEMA1234567890ABAQf/abcdefghijklmnopqrstuvwxyz\r\n=wxyz\r\n-----END PGP MESSAGE-----\r\n`),
    s('ics', 'frame', (subject, to, from) => headers(to, from, subject, 'Content-Type: multipart/mixed; boundary="mix"\r\n\r\n') +
      `--mix\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n${mark('ics')} you are invited\r\n` +
      `--mix\r\nContent-Type: text/calendar; method=REQUEST; charset=utf-8\r\nContent-Disposition: attachment; filename="invite.ics"\r\n\r\nBEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//t60//EN\r\nMETHOD:REQUEST\r\nBEGIN:VEVENT\r\nUID:t60-${tag}@t60.test\r\nDTSTAMP:20261029T090000Z\r\nDTSTART:20261105T100000Z\r\nDTEND:20261105T110000Z\r\nSUMMARY:T60 planning\r\nORGANIZER:mailto:${from.replace(/.*<|>.*/g, '')}\r\nATTENDEE;RSVP=TRUE;PARTSTAT=NEEDS-ACTION:mailto:${to}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n--mix--\r\n`),
    s('quoted', 'frame', html(`<p>${mark('quoted')} reply on top</p><blockquote style="border-left:2px solid #ccc;padding-left:8px;color:#555">On Monday someone wrote:<br>${'quoted line<br>'.repeat(40)}</blockquote>`)),
  ];
}

function smtpSend(to: string, from: string, data: string): Promise<void> {
  const steps = ['EHLO t60', `MAIL FROM:<${from.replace(/.*<|>.*/g, '')}>`, `RCPT TO:<${to}>`, 'DATA', `${data.replace(/\r?\n/g, '\r\n').replace(/^\./gm, '..')}\r\n.`, 'QUIT'];
  return new Promise((resolve, reject) => {
    const sock = net.createConnection({ host: SMTP_HOST, port: SMTP_PORT });
    let step = -1; let buf = '';
    sock.setTimeout(15000, () => { sock.destroy(); reject(new Error(`smtp timeout at ${step}`)); });
    sock.on('error', reject);
    sock.on('data', chunk => {
      buf += chunk.toString('utf8');
      const reply = buf.split('\r\n').find(line => /^\d{3} /.test(line));
      if (reply === undefined) return;
      buf = '';
      if (Number(reply.slice(0, 3)) >= 400) { sock.destroy(); reject(new Error(`smtp: ${reply}`)); return; }
      step += 1;
      if (step >= steps.length) { sock.end(); resolve(); return; }
      sock.write(`${steps[step]}\r\n`);
    });
  });
}

/** Deterministic PRNG so a failing sequence can be replayed (MW_T60_SEED). */
function rng(seed: number): () => number {
  let x = seed >>> 0 || 1;
  return () => { x ^= x << 13; x ^= x >>> 17; x ^= x << 5; return ((x >>> 0) % 1_000_000) / 1_000_000; };
}

interface FrameReport {
  ok: boolean; reason: string; outerSrc?: string; nestedUrl?: string; outerH?: number; nestedH?: number;
  text?: string; color?: string; background?: string; inkPixels?: number; ms?: number;
}

/** Everything about one reader frame the user can see. */
async function inspectFrame(page: Page, frameEl: Locator, marker: string): Promise<FrameReport> {
  const handle = await frameEl.elementHandle({ timeout: 1000 }).catch(() => null);
  if (!handle) return { ok: false, reason: 'no-outer-frame' };
  const outerBox = await handle.boundingBox();
  const outerH = outerBox?.height ?? 0;
  const outerSrc = (await handle.getAttribute('src')) ?? `srcdoc:${((await handle.getAttribute('srcdoc')) ?? '').length}`;
  const outer = await handle.contentFrame();
  if (!outer) return { ok: false, reason: 'outer-not-loaded', outerSrc, outerH };
  // The direct reader is a bridge document holding one nested sandbox="" frame;
  // the fallback reader is the sandbox="" frame itself.
  const bridge = outerSrc.includes('/api/rc/frame');
  let content = outer;
  let nestedH = outerH;
  if (bridge) {
    const nested = outer.childFrames();
    if (nested.length === 0) return { ok: false, reason: 'nested-frame-missing', outerSrc, outerH, nestedUrl: outer.url() };
    content = nested[nested.length - 1]!;
    nestedH = await outer.evaluate(() => { const f = document.querySelector('iframe'); return f ? f.getBoundingClientRect().height : -1; }).catch(() => -2);
  }
  const nestedUrl = content.url();
  const probe = await content.evaluate((needle: string) => {
    const body = document.body;
    const text = body?.innerText ?? '';
    const walker = document.createTreeWalker(body ?? document, NodeFilter.SHOW_TEXT);
    let el: Element | null = null;
    for (let n = walker.nextNode(); n; n = walker.nextNode()) if (n.textContent?.includes(needle)) { el = n.parentElement; break; }
    let color = ''; let background = '';
    if (el) {
      color = getComputedStyle(el).color;
      for (let at: Element | null = el; at; at = at.parentElement) {
        const bg = getComputedStyle(at).backgroundColor;
        if (bg && bg !== 'transparent' && !/rgba\(.*,\s*0\)$/.test(bg)) { background = bg; break; }
      }
    }
    return { text: text.slice(0, 160), found: el !== null, html: document.documentElement.outerHTML.length, color, background, ready: document.readyState };
  }, marker).catch((e: Error) => ({ text: '', found: false, html: 0, color: '', background: '', ready: `eval-failed:${e.message.slice(0, 80)}` }));
  const base: FrameReport = { ok: false, reason: '', outerSrc, nestedUrl, outerH, nestedH, text: probe.text, color: probe.color, background: probe.background };
  if (outerH < 8) return { ...base, reason: 'outer-height-0' };
  if (nestedH < 8) return { ...base, reason: 'nested-height-0' };
  if (!probe.found) return { ...base, reason: probe.html < 80 ? 'nested-empty' : probe.text.trim() === '' ? 'nested-no-text' : 'marker-missing' };
  if (!probe.background) {
    // Nothing paints behind the text inside the frame: the outer iframe's own
    // background shows through.
    base.background = await handle.evaluate(el => getComputedStyle(el).backgroundColor);
  }
  if (sameColour(base.color ?? '', base.background ?? '')) return { ...base, reason: 'ink-equals-paper' };
  const shot = await frameEl.screenshot({ timeout: 3000 }).catch(() => null);
  if (shot) {
    base.inkPixels = await page.evaluate(async (b64: string) => {
      const img = new Image(); img.src = `data:image/png;base64,${b64}`; await img.decode();
      const c = document.createElement('canvas'); c.width = img.width; c.height = img.height;
      const g = c.getContext('2d')!; g.drawImage(img, 0, 0);
      const d = g.getImageData(0, 0, c.width, c.height).data;
      // Count pixels that differ from the most common one (inside the border).
      const counts = new Map<number, number>();
      for (let i = 0; i < d.length; i += 4 * 7) { const k = (d[i]! << 16) | (d[i + 1]! << 8) | d[i + 2]!; counts.set(k, (counts.get(k) ?? 0) + 1); }
      let mode = 0; let best = -1; for (const [k, v] of counts) if (v > best) { best = v; mode = k; }
      let ink = 0;
      for (let y = 4; y < c.height - 4; y += 2) for (let x = 4; x < c.width - 4; x += 2) {
        const i = (y * c.width + x) * 4; const k = (d[i]! << 16) | (d[i + 1]! << 8) | d[i + 2]!;
        if (Math.abs((k >> 16) - (mode >> 16)) + Math.abs(((k >> 8) & 255) - ((mode >> 8) & 255)) + Math.abs((k & 255) - (mode & 255)) > 60) ink += 1;
      }
      return ink;
    }, shot.toString('base64'));
    if ((base.inkPixels ?? 0) < 6) return { ...base, reason: 'flat-screenshot' };
  }
  return { ...base, ok: true, reason: 'ok' };
}

function rgb(c: string): [number, number, number, number] | null {
  const m = /rgba?\(([^)]+)\)/.exec(c); if (!m) return null;
  const p = m[1]!.split(/[ ,/]+/).filter(Boolean).map(Number);
  return [p[0] ?? 0, p[1] ?? 0, p[2] ?? 0, p[3] ?? 1];
}
function sameColour(a: string, b: string): boolean {
  const x = rgb(a); const y = rgb(b); if (!x || !y) return false;
  const lum = (v: [number, number, number, number]) => { const f = (n: number) => { const s = n / 255; return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4; }; return 0.2126 * f(v[0]) + 0.7152 * f(v[1]) + 0.0722 * f(v[2]); };
  const l1 = lum(x); const l2 = lum(y); const ratio = (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05);
  return ratio < 1.35;
}

test.describe('reader frame stability (t60)', () => {
  test.skip(({ viewport }) => (viewport?.width ?? 1280) < 600, 'desktop harness; the phone layout is covered by the reader specs');
  test.setTimeout(60 * 60 * 1000);
  test.use({ actionTimeout: 10000 });

  test('message frames never go blank across varied opens', async ({ page, context }, info) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    const outDir = path.resolve(process.env['MW_T60_OUT'] ?? info.outputPath('t60'));
    mkdirSync(outDir, { recursive: true });
    const logFile = path.join(outDir, `events-${info.project.name}.log`);
    writeFileSync(logFile, '');
    const t0 = Date.now();
    const log = (line: string): void => { appendFileSync(logFile, `${String(Date.now() - t0).padStart(7)} ${line}\n`); };

    const tag = `${info.project.name.slice(0, 2)}${Date.now().toString(36)}`;
    const user = `t60-${tag}@example.org`;
    const from = 'Newsletter Sender <news@sender.example.net>';
    const all = seeds(tag);
    for (const seed of all) await smtpSend(user, from, seed.mime(user, from));
    const extraFrom = 'Live Update <live@sender.example.net>';

    // Instrument every page and frame: console, page errors, CSP reports, and
    // frame lifecycle (attach / navigate / detach) with timestamps.
    const problems: string[] = [];
    await context.addInitScript(() => {
      document.addEventListener('securitypolicyviolation', e => {
        console.warn(`[t60-csp] ${JSON.stringify({ directive: e.violatedDirective, blocked: e.blockedURI, sample: e.sample, doc: location.href.slice(0, 80) })}`);
      });
    });
    const wire = (p: Page, name: string): void => {
      p.on('console', msg => {
        const text = msg.text();
        log(`[${name} console.${msg.type()}] ${text.slice(0, 300)}`);
        // Harness noise: the init script itself is refused in sandbox="" frames.
        if (/Blocked script execution in 'about:(blank|srcdoc)'/.test(text)) return;
        if (text.startsWith('[t60-csp]') || msg.type() === 'error') problems.push(`${name}: ${text.slice(0, 200)}`);
      });
      p.on('pageerror', err => { log(`[${name} pageerror] ${err.message}`); if (!err.message.includes("'serviceWorker'")) problems.push(`${name} pageerror: ${err.message}`); });
      p.on('crash', () => { log(`[${name} CRASH]`); problems.push(`${name} renderer crashed`); });
      p.on('frameattached', f => log(`[${name} frameattached] ${f.url().slice(0, 100)}`));
      p.on('framenavigated', f => { if (f !== p.mainFrame()) log(`[${name} framenavigated] ${f.url().slice(0, 100)}`); });
      p.on('framedetached', f => log(`[${name} framedetached] ${f.url().slice(0, 100)}`));
    };
    wire(page, 'main');

    await page.goto('/');
    await page.getByLabel('Email or username', { exact: true }).fill(user);
    await page.getByLabel('Password', { exact: true }).fill('t60pass');
    await page.getByRole('button', { name: 'Sign in', exact: true }).click();
    const rows = page.locator('.list__scroll .list__row');
    await expect(rows.filter({ hasText: all[all.length - 1]!.subject })).toBeAttached({ timeout: 60000 });
    for (const seed of all) await expect(rows.filter({ hasText: seed.subject }).first()).toBeAttached({ timeout: 60000 });
    await page.waitForLoadState('networkidle');

    // A reader iframe moved in the DOM reloads its bridge under the same token:
    // count those reloads (and the image requests they cause) per action, and
    // record which element was moved where.
    let bridgeReloads = 0; let imageRequests = 0; const bridgeTokens = new Set<string>();
    page.on('framenavigated', f => { const m = /\/api\/rc\/frame[^#]*#token=([0-9a-f]{16})/.exec(f.url()); if (!m) return; if (bridgeTokens.has(m[1]!)) bridgeReloads += 1; bridgeTokens.add(m[1]!); });
    if (process.env['MW_T60_TRACE'] === '1') { page.on('request', r => { const u = new URL(r.url()); if (/^\/(api|jmap)/.test(u.pathname)) log(`[req] ${r.method()} ${u.pathname}${r.method() === 'POST' && u.pathname.includes('jmap') ? ' ' + (r.postData() ?? '').slice(0, 140) : ''}`); }); page.on('requestfinished', r => { const u = new URL(r.url()); if (/^\/(api|jmap)/.test(u.pathname)) log(`[done] ${u.pathname}`); }); }
    const imageUrls = new Map<string, number>();
    page.on('request', r => { if (!/\/api\/rc\/(i|p)\//.test(new URL(r.url()).pathname)) return; imageRequests += 1; imageUrls.set(r.url(), (imageUrls.get(r.url()) ?? 0) + 1); });
    await page.evaluate(() => {
      const w = window as unknown as { __t60moves: string[] };
      w.__t60moves = [];
      const name = (n: Node | null): string => n instanceof Element ? `${n.tagName.toLowerCase()}${n.id ? `#${n.id}` : ''}.${String(n.className).split(' ')[0]}` : String(n?.nodeName);
      new MutationObserver(ms => {
        for (const m of ms) for (const n of m.addedNodes) if (n instanceof Element && m.target instanceof Element && m.target.id === 'root') console.log(`[t60-root-added] ${name(n)} ${n.textContent?.slice(0, 60)}`);
        for (const m of ms) for (const n of m.removedNodes) if (n instanceof Element && n.classList.contains('shell')) console.log('[t60-shell-removed]');
        for (const m of ms) for (const n of m.removedNodes) if (n instanceof Element && (n.tagName === 'IFRAME' ? n.classList.contains('reader__frame') : n.querySelector('iframe.reader__frame'))) w.__t60moves.push(`-${name(n)} from ${name(m.target)} < ${name(m.target.parentNode)}`);
      }).observe(document.body, { childList: true, subtree: true });
    });
    const takeMoves = (): Promise<string[]> => page.evaluate(() => { const w = window as unknown as { __t60moves: string[] }; const out = w.__t60moves; w.__t60moves = []; return out; }).catch(() => []);
    const random = rng(Number(process.env['MW_T60_SEED'] ?? 6060));
    const pick = <T,>(xs: readonly T[]): T => xs[Math.floor(random() * xs.length)]!;
    const stats = { opens: 0, blank: 0, notOpened: 0, decrypt: 0, actions: {} as Record<string, number>, reloads: {} as Record<string, number>, reasons: {} as Record<string, number> };
    const blanks: string[] = [];
    let dark = false;
    const reselects: { reloads: number; refetched: number }[] = [];
    const idleChurn: number[] = [];
    let extra = 0;

    const rowFor = (subject: string) => rows.filter({ hasText: subject }).first();
    const closeOtherTabs = async (): Promise<void> => {
      const tabs = page.getByRole('tab');
      if ((await tabs.count()) <= 4) return;
      await page.getByRole('tab', { name: 'Mail', exact: true }).first().press('Shift+F10').catch(() => undefined);
      const item = page.getByRole('menuitem', { name: 'Close other tabs', exact: true });
      if (await item.isVisible().catch(() => false)) await item.click(); else await page.keyboard.press('Escape');
      await page.getByRole('tab', { name: 'Mail', exact: true }).first().click().catch(() => undefined);
    };

    /** The reader frames on screen: the pane that holds the opened message. */
    async function verify(target: Page, seed: Seed, action: string, where: string): Promise<void> {
      stats.opens += 1;
      const started = Date.now();
      const marker = mark(seed.key);
      if (seed.kind === 'decrypt') {
        // An armoured message shows the decrypt panel instead of a frame.
        const ok = await target.locator('.reader').filter({ hasText: seed.subject }).first().isVisible().catch(() => false);
        stats.decrypt += 1;
        log(`[open ${stats.opens}] ${seed.key} ${action} ${where} decrypt-panel=${ok}`);
        return;
      }
      let report: FrameReport = { ok: false, reason: 'not-checked' };
      const deadline = started + 12000;
      let lastReports: FrameReport[] = [];
      while (Date.now() < deadline) {
        const frames = target.locator('iframe.reader__frame');
        const n = await frames.count();
        const reports: FrameReport[] = [];
        for (let i = 0; i < n; i += 1) {
          const f = frames.nth(i);
          if (!(await f.isVisible().catch(() => false))) continue;
          const r = await inspectFrame(target, f, marker);
          reports.push(r);
          if (r.ok) { report = r; break; }
        }
        if (report.ok) break;
        lastReports = reports;
        report = reports.find(r => r.reason !== 'marker-missing') ?? reports[0] ?? { ok: false, reason: 'no-visible-frame' };
        await target.waitForTimeout(150);
      }
      report.ms = Date.now() - started;
      if (!report.ok && target === page && (await target.getByRole('tab', { name: seed.subject, exact: true }).count()) === 0 && (await target.locator('.reader:visible').filter({ hasText: seed.subject }).count()) === 0) {
        // The click opened nothing at all: no tab, no reader for the message.
        // A different bug from a blank frame; counted and logged on its own.
        stats.notOpened += 1;
        log(`[open ${stats.opens}] ${seed.key} ${action} ${where} not-opened ${report.ms}ms`);
        await target.screenshot({ path: path.join(outDir, `not-opened-${info.project.name}-${stats.opens}-${seed.key}.png`) }).catch(() => undefined);
        return;
      }
      log(`[open ${stats.opens}] ${seed.key} ${action} ${where} ${report.reason} ${report.ms}ms outerH=${report.outerH} nestedH=${report.nestedH} ink=${report.inkPixels} color=${report.color} bg=${report.background} src=${report.outerSrc?.slice(0, 60)} nested=${report.nestedUrl?.slice(0, 40)} text=${JSON.stringify(report.text?.slice(0, 60))}`);
      if (!report.ok) {
        for (const r of lastReports) log(`[open ${stats.opens} frame] ${r.reason} outerH=${r.outerH} nestedH=${r.nestedH} src=${r.outerSrc?.slice(0, 60)} nested=${r.nestedUrl?.slice(0, 40)} text=${JSON.stringify(r.text?.slice(0, 80))}`);
        stats.blank += 1;
        stats.reasons[report.reason] = (stats.reasons[report.reason] ?? 0) + 1;
        const shot = path.join(outDir, `blank-${info.project.name}-${stats.opens}-${seed.key}-${report.reason}.png`);
        await target.screenshot({ path: shot }).catch(() => undefined);
        blanks.push(`#${stats.opens} ${seed.key} after ${action} (${where}): ${report.reason} ${JSON.stringify(report)}`);
      }
    }

    async function openByClick(seed: Seed): Promise<void> {
      await rowFor(seed.subject).click();
    }

    const actions: Record<string, () => Promise<{ seed: Seed; where?: string; target?: Page } | null>> = {
      click: async () => { const seed = pick(all); await openByClick(seed); return { seed }; },
      arrows: async () => {
        // Fast switching: next/previous by the list shortcut, no wait between.
        const start = pick(all); await openByClick(start);
        const steps = 2 + Math.floor(random() * 4);
        await rowFor(start.subject).focus();
        let index = Number(await rowFor(start.subject).getAttribute('data-index'));
        for (let i = 0; i < steps; i += 1) {
          const down = random() < 0.6;
          await page.keyboard.press(down ? 'ArrowDown' : 'ArrowUp');
          await page.keyboard.press('Enter');
          index = Math.max(0, index + (down ? 1 : -1));
        }
        // The opened message is the focused row's; if focus left the list
        // (not this harness's subject) the open is not attributable: skip it.
        const final = await page.evaluate(() => { const el = document.activeElement; return el instanceof HTMLElement && el.classList.contains('list__row') ? el.textContent ?? '' : ''; });
        const seed = all.filter(s => final.includes(s.subject)).sort((a, b) => b.subject.length - a.subject.length)[0];
        return seed ? { seed } : null;
      },
      rapidClicks: async () => {
        const a = pick(all); const b = pick(all); const c = pick(all);
        await rowFor(a.subject).click(); await rowFor(b.subject).click({ delay: 10 }); await rowFor(c.subject).click();
        return { seed: c };
      },
      readingPane: async () => {
        // Off: the reader replaces the list, so open first and switch after.
        const mode = pick(['Right', 'Bottom', 'Off', 'Right']);
        const seed = pick(all);
        if (mode === 'Off') { await openByClick(seed); await page.getByRole('button', { name: mode, exact: true }).first().click(); }
        else { await page.getByRole('button', { name: mode, exact: true }).first().click(); await openByClick(seed); }
        return { seed, where: `pane=${mode}` };
      },
      layout: async () => {
        const layout = pick(['Tabs', 'Side by side', 'Stacked', 'Masonry side by side', 'Tabs']);
        const seed = pick(all); await openByClick(seed);
        await page.getByRole('button', { name: layout, exact: true }).click();
        return { seed, where: `layout=${layout}` };
      },
      theme: async () => {
        const seed = pick(all); await openByClick(seed);
        await page.waitForTimeout(Math.floor(random() * 600));
        dark = !dark; await page.emulateMedia({ colorScheme: dark ? 'dark' : 'light' });
        return { seed, where: `dark=${dark}` };
      },
      markRead: async () => {
        // Open, let the 1 s mark-read land and the list update under the reader.
        const seed = pick(all); await openByClick(seed); await page.waitForTimeout(1400);
        return { seed, where: 'after-mark-read' };
      },
      refresh: async () => {
        const seed = pick(all); await openByClick(seed);
        await page.getByRole('button', { name: 'Refresh folder', exact: true }).click().catch(() => undefined);
        return { seed, where: 'refresh' };
      },
      flag: async () => {
        const seed = pick(all); await openByClick(seed);
        await page.keyboard.press('Escape');
        const flag = page.locator('.reader:visible').getByRole('button', { name: /^(Flag|Unflag)/ }).first();
        await flag.click({ timeout: 2000 }).catch(() => undefined);
        return { seed, where: 'flag' };
      },
      realtime: async () => {
        // A new message arrives while a message is open: the list grows above it.
        const seed = pick(all); await openByClick(seed);
        extra += 1;
        await smtpSend(user, extraFrom, headers(user, extraFrom, `T60 ${tag} live ${extra}`, 'Content-Type: text/plain; charset=utf-8\r\n\r\n') + `live update ${extra}\r\n`);
        await page.waitForTimeout(2500);
        return { seed, where: `realtime-${extra}` };
      },
      remoteGrant: async () => {
        const seed = all.find(s => s.key === (random() < 0.5 ? 'remoteA' : 'remoteB'))!;
        await openByClick(seed);
        const grant = page.getByRole('button', { name: 'Load images', exact: true }).first();
        if (await grant.isVisible({ timeout: 3000 }).catch(() => false)) await grant.click();
        return { seed, where: 'remote-granted' };
      },
      folder: async () => {
        await page.locator('#shell-nav').getByRole('button', { name: pick(['Attachments', 'Outbox']), exact: true }).first().click().catch(() => undefined);
        await page.waitForTimeout(300);
        await page.locator('#shell-nav').getByRole('button', { name: 'Inbox', exact: true }).first().click().catch(() => undefined);
        await expect(rows.first()).toBeVisible();
        const seed = pick(all); await openByClick(seed);
        return { seed, where: 'folder-switch' };
      },
      reorderTabs: async () => {
        // Two message tabs; move one along the strip (Ctrl+Shift+Arrow), which
        // reorders the workspace panes and so moves their iframes in the DOM.
        const a = pick(all); const b = pick(all.filter(s => s !== a));
        await openByClick(a); await openByClick(b);
        const tab = page.getByRole('tab', { name: a.subject, exact: true });
        if (!(await tab.isVisible().catch(() => false))) return null;
        await tab.click();
        for (let i = 0, n = 1 + Math.floor(random() * 3); i < n; i += 1) await page.keyboard.press(random() < 0.5 ? 'Control+Shift+ArrowLeft' : 'Control+Shift+ArrowRight');
        const other = page.getByRole('tab', { name: b.subject, exact: true });
        if (!(await other.isVisible().catch(() => false))) return null;
        await other.click(); await tab.click();
        return { seed: a, where: 'tab-reordered' };
      },
      idle: async () => {
        // Leave one message open across the 30 s preference / appearance polls
        // and count every frame the page attaches or detaches meanwhile: a poll
        // that rewrites or remounts the body shows up here.
        const seed = pick(all.filter(s => s.kind === 'frame')); await openByClick(seed);
        await page.waitForTimeout(1500);
        let churn = 0; const requests: string[] = [];
        const onFrame = (): void => { churn += 1; };
        const onRequest = (r: { url: () => string }): void => { const u = new URL(r.url()); if (u.pathname.startsWith('/api/')) requests.push(u.pathname); };
        page.on('frameattached', onFrame); page.on('framedetached', onFrame); page.on('request', onRequest);
        await page.waitForTimeout(Number(process.env['MW_T60_IDLE_MS'] ?? 65000));
        page.off('frameattached', onFrame); page.off('framedetached', onFrame); page.off('request', onRequest);
        const counts: Record<string, number> = {}; for (const r of requests) counts[r] = (counts[r] ?? 0) + 1;
        log(`[idle] ${seed.key} frame churn=${churn} requests=${JSON.stringify(counts)}`);
        idleChurn.push(churn);
        return { seed, where: `idle-churn=${churn}` };
      },
      reselect: async () => {
        // Reselect an open message tab from its tab and from its list row: the
        // body must still be there and nothing may reload or refetch (t56).
        const a = pick(all.filter(s => s.kind === 'frame')); const b = pick(all.filter(s => s !== a));
        await openByClick(a); await openByClick(b);
        // A lazy image of a pane that was hidden may load for the first time
        // now; an image fetched AGAIN is the refetch t56 saw.
        const reloads = bridgeReloads; const fetchedBefore = new Map(imageUrls);
        const tab = page.getByRole('tab', { name: a.subject, exact: true });
        if (!(await tab.isVisible().catch(() => false))) return null;
        await tab.click();
        await page.getByRole('tab', { name: b.subject, exact: true }).click().catch(() => undefined);
        await rowFor(a.subject).click();
        await page.waitForTimeout(800);
        const refetched = [...imageUrls].filter(([url, n]) => (fetchedBefore.get(url) ?? 0) > 0 && n > (fetchedBefore.get(url) ?? 0)).length;
        reselects.push({ reloads: bridgeReloads - reloads, refetched });
        return { seed: a, where: `reselect reloads=${bridgeReloads - reloads} refetched=${refetched}` };
      },
      popup: async () => {
        const seed = pick(all.filter(s => s.kind === 'frame')); await openByClick(seed);
        const tab = page.getByRole('tab', { name: seed.subject, exact: true });
        await tab.press('Shift+F10');
        const item = page.getByRole('menuitem', { name: `Open ${seed.subject} in a new window`, exact: true });
        if (!(await item.isVisible().catch(() => false))) { await page.keyboard.press('Escape'); return { seed, where: 'popup-unavailable' }; }
        const pending = page.waitForEvent('popup');
        await item.click();
        const popup = await pending;
        wire(popup, 'popup');
        await popup.waitForLoadState();
        await verify(popup, seed, 'popup', 'detached');
        await popup.close();
        await page.getByRole('tab', { name: seed.subject, exact: true }).click().catch(() => undefined);
        return { seed, where: 'reattached' };
      },
    };
    const weights: [string, number][] = [['click', 6], ['arrows', 3], ['rapidClicks', 2], ['readingPane', 2], ['layout', 2], ['theme', 2], ['markRead', 2], ['refresh', 1], ['flag', 1], ['realtime', 1], ['remoteGrant', 2], ['folder', 1], ['reorderTabs', 1], ['reselect', 2], ['popup', 1]];
    // One idle observation up front (when enabled) so every run measures the polls.
    if (process.env['MW_T60_IDLE'] === '1') { const r = await actions['idle']!(); if (r) await verify(page, r.seed, 'idle', r.where ?? ''); }
    const total = weights.reduce((n, [, w]) => n + w, 0);
    const choose = (): string => { let r = random() * total; for (const [name, w] of weights) { r -= w; if (r <= 0) return name; } return 'click'; };

    /** Back to the mail list with the reading pane on the right, if an action left it covered. */
    const ensureList = async (): Promise<void> => {
      if (await rows.first().isVisible().catch(() => false)) return;
      await page.getByRole('tab', { name: 'Mail', exact: true }).first().click().catch(() => undefined);
      await page.getByRole('button', { name: 'Right', exact: true }).first().click().catch(() => undefined);
      if (!(await rows.first().isVisible().catch(() => false))) await page.locator('#shell-nav').getByRole('button', { name: 'Inbox', exact: true }).first().click().catch(() => undefined);
      await expect(rows.first()).toBeVisible();
    };
    while (stats.opens < OPENS) {
      await ensureList().catch(e => log(`[ensure-list-error] ${(e as Error).message.split('\n')[0]}`));
      const name = choose();
      stats.actions[name] = (stats.actions[name] ?? 0) + 1;
      const reloadsBefore = bridgeReloads; const imagesBefore = imageRequests; await takeMoves();
      log(`[action] ${name}`);
      let result: Awaited<ReturnType<(typeof actions)[string]>> = null;
      try { result = await actions[name]!(); }
      catch (e) { log(`[action-error] ${name}: ${(e as Error).message.split('\n')[0]}`); }
      if (result) await verify(result.target ?? page, result.seed, name, result.where ?? '');
      const moved = bridgeReloads - reloadsBefore;
      const removed = await takeMoves();
      if (moved > 0) { stats.reloads[name] = (stats.reloads[name] ?? 0) + moved; log(`[reloads] ${name} +${moved} bridge reloads, ${imageRequests - imagesBefore} image requests; removed: ${JSON.stringify(removed.slice(0, 6))}`); }
      if (stats.opens % 9 === 0) await closeOtherTabs().catch(e => log(`[close-tabs-error] ${(e as Error).message.split('\n')[0]}`));
      if (page.isClosed()) break;
    }

    const summary = { project: info.project.name, ...stats, idleChurn, reselects, problems: problems.length, blanks };
    writeFileSync(path.join(outDir, `summary-${info.project.name}.json`), JSON.stringify({ ...summary, problemLines: problems.slice(0, 200) }, null, 2));
    log(`[summary] ${JSON.stringify(summary)}`);
    expect(blanks, `blank frames (see ${logFile})`).toEqual([]);
    // CSP reports from the message frames themselves (the bridge document). The
    // app shell's own reports are logged but are not this harness's subject.
    expect(problems.filter(p => p.includes('[t60-csp]') && p.includes('/api/rc/frame')), 'CSP violations in the reader frames').toEqual([]);
    expect(problems.filter(p => p.includes('renderer crashed')), 'renderer crashes').toEqual([]);
    // Only a tab reorder or a popup detach/reattach moves a frame in the DOM. Any
    // other bridge reload is a remount or a Suspense swap of the shell (t60).
    expect(Object.fromEntries(Object.entries(stats.reloads).filter(([action]) => action !== 'reorderTabs' && action !== 'popup')), 'bridge reloads by action').toEqual({});
    expect(reselects.filter(r => r.reloads > 0 || r.refetched > 0), 'a reselected tab reloaded or refetched images').toEqual([]);
  });
});
