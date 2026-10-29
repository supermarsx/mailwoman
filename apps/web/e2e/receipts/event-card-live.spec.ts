// t53: the reader's Outlook-style event card against a server built from the
// tree, over Greenmail, no route mocks (scripts/e2e-receipts-stack.mjs).
//
//   1. an organizer's REQUEST arrives by SMTP; the card shows it; Accept with a
//      comment stores the event and sends a REPLY from the invited address,
//      which arrives in the organizer's Greenmail INBOX;
//   2. the organizer's CANCEL arrives; "Remove from calendar" removes it;
//   3. a plain .ics attachment with two events: "Add all" adds both, and the
//      attachment chip's "Open event" preview renders the same card.
//
// Runs in chromium and firefox, each at 1440 and 390 px wide (the `phone`
// project is skipped: the widths are set here). MW_EVENT_SHOTS: a directory
// for screenshots.
import net from 'node:net';
import type { Page } from '@playwright/test';
import { test, expect } from '../csrf.ts';
import { injectViaSmtp, messageRow } from '../helpers.ts';

const shots = process.env['MW_EVENT_SHOTS'];
const ME = 'testuser@example.org';
const IMAP_HOST = process.env['MW_E2E_SMTP_HOST'] ?? '127.0.0.1';
const IMAP_PORT = Number(process.env['MW_E2E_IMAP_PORT'] ?? 42243);
const CAL = 'urn:mailwoman:calendars';


function stamp(d: Date): string {
  return d.toISOString().replace(/[-:]/g, '').replace(/\.\d{3}/, '');
}

function itip(method: 'REQUEST' | 'CANCEL', uid: string, organizer: string, title: string, start: Date): string {
  const end = new Date(start.getTime() + 3_600_000);
  return [
    'BEGIN:VCALENDAR', 'VERSION:2.0', 'PRODID:-//Mailwoman e2e//EN', `METHOD:${method}`,
    'BEGIN:VEVENT', `UID:${uid}`, 'SEQUENCE:0', `DTSTAMP:${stamp(new Date())}`,
    `DTSTART:${stamp(start)}`, `DTEND:${stamp(end)}`, `SUMMARY:${title}`,
    `LOCATION:https://meet.jit.si/mw-${uid.split('@')[0]}`,
    'DESCRIPTION:Agenda:\\n1. Status\\n2. Plans https://example.com/notes',
    `ORGANIZER;CN=Olga Organizer:mailto:${organizer}`,
    `ATTENDEE;CN=Olga Organizer;PARTSTAT=ACCEPTED:mailto:${organizer}`,
    `ATTENDEE;CN=Test User;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:${ME}`,
    'ATTENDEE;CN=Pat;PARTSTAT=TENTATIVE:mailto:pat@example.net',
    ...(method === 'CANCEL' ? ['STATUS:CANCELLED'] : []),
    'END:VEVENT', 'END:VCALENDAR', '',
  ].join('\r\n');
}

/** The sources of the messages in `address`'s Greenmail INBOX whose subject has `subject`. */
function imapFetch(address: string, subject: string): Promise<string[]> {
  return new Promise((resolve, reject) => {
    const sock = net.createConnection({ host: IMAP_HOST, port: IMAP_PORT });
    sock.setEncoding('latin1');
    let buf = '';
    let step = 0;
    let ids: string[] = [];
    const out: string[] = [];
    const send = (line: string): void => { step += 1; buf = ''; sock.write(`a${step} ${line}\r\n`); };
    sock.setTimeout(15_000, () => { sock.destroy(); reject(new Error('IMAP timed out')); });
    sock.on('error', reject);
    sock.on('data', (chunk: string) => {
      buf += chunk;
      if (step === 0) { if (buf.includes('\r\n')) send(`LOGIN "${address}" "${address}"`); return; }
      const done = new RegExp(`(^|\\r\\n)a${step} (OK|NO|BAD)[^\\r\\n]*\\r\\n$`).exec(buf);
      if (!done) return;
      if (done[2] !== 'OK') { sock.end(); resolve([]); return; }
      if (step === 1) return send('SELECT INBOX');
      if (step === 2) return send(`SEARCH SUBJECT "${subject}"`);
      if (step === 3) ids = (/\* SEARCH([^\r\n]*)/.exec(buf)?.[1] ?? '').trim().split(/\s+/).filter(Boolean);
      else {
        const lit = /\{(\d+)\}\r\n/.exec(buf);
        if (lit) out.push(buf.slice(lit.index + lit[0].length, lit.index + lit[0].length + Number(lit[1])));
      }
      const next = ids[step - 3];
      if (next !== undefined) return send(`FETCH ${next} BODY.PEEK[]`);
      sock.end();
      resolve(out);
    });
  });
}

async function signIn(page: Page): Promise<string> {
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill('testuser');
  await page.getByLabel('Password', { exact: true }).fill('testpass');
  const login = page.waitForResponse((r) => r.url().endsWith('/api/login') && r.request().method() === 'POST');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  const { accountId } = await (await login).json() as { accountId: string };
  await expect(page.getByRole('button', { name: 'Compose', exact: true }).first()).toBeVisible();
  return accountId;
}

async function openMessage(page: Page, subject: string): Promise<void> {
  await expect(async () => {
    await page.goto('/');
    // The workspace keeps the message tabs opened before: go back to the list.
    const mail = page.getByRole('tab', { name: 'Mail', exact: true });
    await expect(page.getByRole('button', { name: 'Compose', exact: true }).first()).toBeVisible();
    await page.waitForTimeout(500);
    if (await mail.count() > 0 && (await mail.getAttribute('aria-selected')) !== 'true') {
      await mail.dispatchEvent('click');
      await expect(mail).toHaveAttribute('aria-selected', 'true');
    }
    await expect(page.getByRole('region', { name: 'Messages', exact: true })).toHaveAttribute('aria-busy', 'false');
    await expect(messageRow(page, subject)).toBeVisible({ timeout: 5_000 });
  }).toPass({ timeout: 90_000 });
  await messageRow(page, subject).click();
  await expect(page.getByRole('heading', { name: subject }).first()).toBeVisible();
}

async function events(page: Page, accountId: string): Promise<Array<{ uid: string; title: string; status: string }>> {
  // One retry: a read racing the client's own session refresh can be refused once.
  for (let attempt = 0; ; attempt++) {
    const res = await page.request.post('/jmap/api', { data: { using: ['urn:ietf:params:jmap:core', CAL], methodCalls: [['CalendarEvent/get', { accountId, ids: null }, 'g']] } });
    if (res.ok()) return (await res.json()).methodResponses[0][1].list;
    if (attempt > 0) throw new Error(`CalendarEvent/get: ${res.status()} ${await res.text()}`);
    await page.waitForTimeout(1000);
  }
}

async function noHorizontalScroll(page: Page): Promise<void> {
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1)).toBe(true);
}

for (const width of [1440, 390]) {
  test(`event card: accept, cancel and add a calendar file at ${width}px`, async ({ page }, info) => {
    test.skip(info.project.name === 'phone', 'the widths are set by the test');
    await page.setViewportSize({ width, height: width > 800 ? 900 : 844 });
    const browser = info.project.name;
    const tag = `${browser}-${width}-${Date.now()}`;
    const organizer = `olga-${tag}@example.org`.toLowerCase();
    const uid = `t53-${tag}@example.org`;
    const title = `Planning ${tag}`;
    const start = new Date(Date.now() + 2 * 86_400_000);
    start.setUTCHours(14, 0, 0, 0);
    const shot = async (name: string): Promise<void> => {
      if (shots) await page.screenshot({ path: `${shots}/${browser}-${width}-${name}.png`, fullPage: false });
    };

    // ── 1. REQUEST → Accept → REPLY at the organizer ──────────────────────────
    const inviteSubject = `Invitation: ${title}`;
    await injectViaSmtp({
      from: `Olga Organizer <${organizer}>`, subject: inviteSubject, text: 'You are invited.',
      attachments: [{ filename: 'invite.ics', contentType: 'text/calendar; method=REQUEST; charset=utf-8', content: itip('REQUEST', uid, organizer, title, start) }],
    });
    const accountId = await signIn(page);
    const sent: string[] = [];
    page.on('request', (r) => { if ((r.postData() ?? '').includes('EmailSubmission/set')) sent.push(r.url()); });
    await openMessage(page, inviteSubject);
    const card = page.getByRole('region', { name: 'Calendar invitation' });
    await expect(card.getByRole('heading', { name: title })).toBeVisible();
    await expect(card.getByText('Meeting invitation')).toBeVisible();
    await expect(card.getByRole('link', { name: 'Join Jitsi Meet' })).toHaveAttribute('href', /^https:\/\/meet\.jit\.si\//);
    await expect(card.getByText(`You were invited as`)).toBeVisible();
    await expect(card.locator('[data-card="attendee-summary"]')).toContainText('2 attendees');
    await expect(page.getByRole('region', { name: /Your calendar on/ })).toBeVisible();
    expect((await events(page, accountId)).some((e) => e.uid === uid)).toBe(false);
    await card.getByRole('heading', { name: title }).scrollIntoViewIfNeeded().catch(() => undefined);
    await noHorizontalScroll(page);
    await expect(card.getByRole('button', { name: 'Accept' })).toBeVisible();
    await shot('1-request');

    await card.getByLabel('Comment (optional)').fill('Looking forward to it');
    await card.getByRole('button', { name: 'Accept' }).click();
    await expect(card.locator('[data-card="result"]')).toContainText(`Accepted. Your response was sent to`);
    await expect(card.locator('[data-card="badge"]')).toHaveText('Accepted');
    const stored = (await events(page, accountId)).find((e) => e.uid === uid);
    expect(stored?.title).toBe(title);
    await shot('2-accepted');

    let reply = '';
    await expect(async () => {
      const found = await imapFetch(organizer, title);
      expect(found.length).toBeGreaterThan(0);
      reply = found.join('\n').replace(/\r\n[ \t]/g, '');
    }).toPass({ timeout: 60_000 });
    expect(reply).toContain('METHOD:REPLY');
    expect(reply).toMatch(new RegExp(`^From: ${ME}`, 'mi'));
    expect(reply).toContain(`UID:${uid}`);
    expect(reply).toMatch(new RegExp(`PARTSTAT=ACCEPTED[^\\r\\n]*:mailto:${ME}`, 'i'));
    expect(reply).toContain('COMMENT:Looking forward to it');
    expect(reply).not.toContain('pat@example.net');

    // The event is in the calendar, on its day.
    await card.getByRole('link', { name: 'Open in calendar' }).click();
    await expect(page.locator('[data-module="calendar"]').getByText(title).first()).toBeVisible();
    await shot('3-calendar');

    // ── 2. CANCEL → Remove from calendar ─────────────────────────────────────
    const cancelSubject = `Cancelled: ${title}`;
    await injectViaSmtp({
      from: `Olga Organizer <${organizer}>`, subject: cancelSubject, text: 'The meeting is cancelled.',
      attachments: [{ filename: 'cancel.ics', contentType: 'text/calendar; method=CANCEL; charset=utf-8', content: itip('CANCEL', uid, organizer, title, start) }],
    });
    await openMessage(page, cancelSubject);
    await expect(card.getByText('The organizer cancelled this event.')).toBeVisible();
    await expect(card.getByText('Cancelled meeting')).toBeVisible();
    await shot('4-cancel');
    await card.getByRole('button', { name: 'Remove from calendar' }).click();
    await expect(card.locator('[data-card="result"]')).toHaveText('Removed from your calendar.');
    expect((await events(page, accountId)).some((e) => e.uid === uid)).toBe(false);

    // ── 3. A plain .ics attachment: Add all, and the chip's preview ──────────
    const fileSubject = `Talks ${tag}`;
    const day = new Date(start.getTime() + 86_400_000);
    const talk = (n: number): string[] => {
      const s = new Date(day.getTime() + n * 3_600_000);
      return ['BEGIN:VEVENT', `UID:talk-${n}-${tag}@example.org`, `DTSTAMP:${stamp(new Date())}`, `DTSTART:${stamp(s)}`,
        `DTEND:${stamp(new Date(s.getTime() + 1_800_000))}`, `SUMMARY:Talk ${n} ${tag}`, 'END:VEVENT'];
    };
    const file = ['BEGIN:VCALENDAR', 'VERSION:2.0', 'PRODID:-//Mailwoman e2e//EN', ...talk(1), ...talk(2), 'END:VCALENDAR', ''].join('\r\n');
    await injectViaSmtp({
      from: 'Conference <events@example.net>', subject: fileSubject, text: 'The programme is attached.',
      attachments: [{ filename: 'talks.ics', contentType: 'application/octet-stream', content: file }],
    });
    await openMessage(page, fileSubject);
    const fileCard = page.getByRole('region', { name: 'Calendar event' }).first();
    await expect(fileCard.getByText('2 calendar events')).toBeVisible();
    await shot('5-file');
    await fileCard.getByRole('button', { name: 'Add all (2)' }).click();
    await expect(fileCard.locator('[data-card="result"]')).toHaveText('2 events added to your calendar.');
    const held = (await events(page, accountId)).map((e) => e.uid);
    expect(held).toEqual(expect.arrayContaining([`talk-1-${tag}@example.org`, `talk-2-${tag}@example.org`]));

    // The chip opens the same card in the attachment preview.
    await page.getByRole('option', { name: /talks\.ics/ }).click();
    const preview = page.locator('[data-viewer-kind="calendar"]').getByRole('region', { name: 'Calendar event' });
    await expect(preview.getByText('In your calendar').first()).toBeVisible();
    await shot('6-preview');
    await noHorizontalScroll(page);
    expect(sent).toEqual([]);
  });
}
