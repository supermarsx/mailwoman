// t78: the Events view against a server built from the tree, over Greenmail,
// no route mocks (scripts/e2e-receipts-stack.mjs).
//
// Seven messages arrive by SMTP: an invitation (text/calendar), an updated
// invitation sent as application/octet-stream named invite.ics, a reply, a
// cancellation, a counter-proposal, a past PUBLISH calendar file
// (application/ics) and a plain message that only mentions an .ics file. The
// Events entry under the folders lists the six calendar messages, not the
// seventh; each row carries its hint (calendar mark, state, start) at the
// height of an Inbox row; the entry has a count bubble; "Upcoming only" drops
// the past event; a click opens the message with t53's event card (in the
// reading pane at 1440, covering the list at 390).
//
// Runs in chromium and firefox, each at 1440 and 390 px wide (the `phone`
// project is skipped: the widths are set here). MW_EVENT_SHOTS: a directory
// for screenshots.
import type { Page } from '@playwright/test';
import { test, expect } from '../csrf.ts';
import { drawerButton, injectViaSmtp, messageRow, messageSlot } from '../helpers.ts';

const shots = process.env['MW_EVENT_SHOTS'];
const ME = 'testuser@example.org';

function stamp(d: Date): string {
  return d.toISOString().replace(/[-:]/g, '').replace(/\.\d{3}/, '');
}

function ics(method: string | null, uid: string, organizer: string, title: string, start: Date, sequence = 0): string {
  const end = new Date(start.getTime() + 3_600_000);
  return [
    'BEGIN:VCALENDAR', 'VERSION:2.0', 'PRODID:-//Mailwoman e2e//EN', ...(method ? [`METHOD:${method}`] : []),
    'BEGIN:VEVENT', `UID:${uid}`, `SEQUENCE:${sequence}`, `DTSTAMP:${stamp(new Date())}`,
    `DTSTART:${stamp(start)}`, `DTEND:${stamp(end)}`, `SUMMARY:${title}`,
    `ORGANIZER;CN=Olga Organizer:mailto:${organizer}`,
    `ATTENDEE;CN=Test User;PARTSTAT=${method === 'REPLY' ? 'ACCEPTED' : 'NEEDS-ACTION'};RSVP=TRUE:mailto:${ME}`,
    ...(method === 'CANCEL' ? ['STATUS:CANCELLED'] : []),
    'END:VEVENT', 'END:VCALENDAR', '',
  ].join('\r\n');
}

async function signIn(page: Page): Promise<void> {
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill('testuser');
  await page.getByLabel('Password', { exact: true }).fill('testpass');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Compose', exact: true }).first()).toBeVisible();
}

function nav(page: Page) {
  return page.getByRole('navigation', { name: 'Mailboxes' });
}

/** Click a sidebar entry, through the drawer at phone width. */
async function sidebar(page: Page, name: string, narrow: boolean): Promise<void> {
  const entry = nav(page).getByRole('button', { name, exact: true });
  if (narrow && !(await entry.isVisible())) await drawerButton(page).click();
  await entry.click();
}

async function listSettled(page: Page): Promise<void> {
  await expect(page.getByRole('region', { name: 'Messages', exact: true })).toHaveAttribute('aria-busy', 'false');
}

for (const width of [1440, 390]) {
  test(`events view: calendar mail with hints, counts, upcoming only and the card at ${width}px`, async ({ page }, info) => {
    test.skip(info.project.name === 'phone', 'the widths are set by the test');
    const narrow = width < 761;
    await page.setViewportSize({ width, height: narrow ? 844 : 900 });
    const browser = info.project.name;
    const tag = `${browser}-${width}-${Date.now()}`;
    const organizer = `olga-${tag}@example.org`.toLowerCase();
    const soon = new Date(Date.now() + 3 * 86_400_000);
    soon.setUTCHours(14, 0, 0, 0);
    const past = new Date(Date.UTC(2021, 2, 1, 9, 0, 0));
    const shot = async (name: string): Promise<void> => {
      if (shots) await page.screenshot({ path: `${shots}/${browser}-${width}-${name}.png`, fullPage: false });
    };

    const mails: Array<{ key: string; state: string | null; subject: string; type: string; file: string; body: string }> = [
      { key: 'invite', state: 'invitation', subject: `Invitation ${tag}`, type: 'text/calendar; method=REQUEST; charset=utf-8', file: 'invite.ics', body: ics('REQUEST', `a-${tag}@x`, organizer, `Kickoff ${tag}`, soon) },
      { key: 'update', state: 'updated', subject: `Updated ${tag}`, type: 'application/octet-stream', file: 'invite.ics', body: ics('REQUEST', `b-${tag}@x`, organizer, `Moved ${tag}`, soon, 2) },
      { key: 'reply', state: 'reply', subject: `Accepted ${tag}`, type: 'text/calendar; method=REPLY; charset=utf-8', file: 'reply.ics', body: ics('REPLY', `c-${tag}@x`, organizer, `Sync ${tag}`, soon) },
      { key: 'cancel', state: 'cancelled', subject: `Cancelled ${tag}`, type: 'text/calendar; method=CANCEL; charset=utf-8', file: 'cancel.ics', body: ics('CANCEL', `d-${tag}@x`, organizer, `Dropped ${tag}`, soon) },
      { key: 'counter', state: 'counter', subject: `New time ${tag}`, type: 'text/calendar; method=COUNTER; charset=utf-8', file: 'counter.ics', body: ics('COUNTER', `e-${tag}@x`, organizer, `Counter ${tag}`, soon) },
      { key: 'publish', state: 'event', subject: `Old event ${tag}`, type: 'application/ics', file: 'old.ics', body: ics('PUBLISH', `f-${tag}@x`, organizer, `Old ${tag}`, past) },
    ];
    for (const m of mails) {
      await injectViaSmtp({ from: `Olga Organizer <${organizer}>`, subject: m.subject, text: 'Calendar mail.', attachments: [{ filename: m.file, contentType: m.type, content: m.body }] });
    }
    const plainSubject = `Mentions ics ${tag}`;
    await injectViaSmtp({ from: `Olga Organizer <${organizer}>`, subject: plainSubject, text: 'I will send invite.ics later. BEGIN:VCALENDAR is how it starts.' });

    await signIn(page);
    // The Inbox first: the plain message has arrived, and its row height is the reference.
    await expect(async () => {
      await sidebar(page, 'Inbox', narrow);
      await listSettled(page);
      await expect(messageRow(page, plainSubject)).toBeVisible({ timeout: 3_000 });
    }).toPass({ timeout: 90_000 });
    const inboxRowHeight = (await messageSlot(page, plainSubject).boundingBox())?.height ?? 0;
    expect(inboxRowHeight).toBeGreaterThan(20);

    // The Events view: every calendar message, each with its hint; not the plain one.
    await expect(async () => {
      await sidebar(page, 'Events', narrow);
      await listSettled(page);
      for (const m of mails) await expect(messageRow(page, m.subject)).toBeVisible({ timeout: 3_000 });
    }).toPass({ timeout: 90_000 });
    await expect(messageRow(page, plainSubject)).toHaveCount(0);
    for (const m of mails) {
      const hint = messageRow(page, m.subject).getByTestId('row-event-hint');
      await expect(hint).toHaveAttribute('data-event-state', m.state ?? 'event');
      await expect(hint.locator('svg')).toHaveCount(1);
      const height = (await messageSlot(page, m.subject).boundingBox())?.height ?? 0;
      expect(Math.abs(height - inboxRowHeight), `${m.key} row height ${height} vs inbox ${inboxRowHeight}`).toBeLessThanOrEqual(0.5);
      // The hint stays inside the preview line: it does not push the row's content out.
      const row = await messageRow(page, m.subject).boundingBox();
      const box = await hint.boundingBox();
      expect(box && row && box.y + box.height <= row.y + row.height + 0.5).toBe(true);
    }
    await expect(messageRow(page, mails[0]!.subject).getByTestId('row-event-hint')).toContainText('Invitation');
    await expect(messageRow(page, mails[3]!.subject).getByTestId('row-event-hint')).toContainText('Cancelled');
    await expect(messageRow(page, mails[5]!.subject).getByTestId('row-event-hint')).toContainText('2021');
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1)).toBe(true);
    await shot('1-events');

    // The sidebar entry's count bubble (the folder count setting defaults to unread / total).
    if (narrow) await drawerButton(page).click();
    const entry = nav(page).getByRole('button', { name: 'Events', exact: true });
    await expect(entry).toHaveAttribute('aria-current', 'true');
    await expect(entry.locator('.sidebar__badge')).toBeVisible();
    await expect(entry.locator('.sidebar__badge')).toHaveText(/\d+\s*\/\s*\d+/);
    await shot('2-sidebar');
    if (narrow) await page.keyboard.press('Escape');

    // Upcoming only: the 2021 calendar file goes, the coming ones stay.
    const upcoming = page.getByTestId('events-upcoming');
    await upcoming.click();
    await expect(upcoming).toHaveAttribute('aria-pressed', 'true');
    await listSettled(page);
    await expect(messageRow(page, mails[5]!.subject)).toHaveCount(0);
    await expect(messageRow(page, mails[0]!.subject)).toBeVisible();
    await shot('3-upcoming');
    await upcoming.click();
    await expect(upcoming).toHaveAttribute('aria-pressed', 'false');
    await expect(messageRow(page, mails[5]!.subject)).toBeVisible();

    // Opening a message: t53's card, under the reading-pane rules (t74).
    await messageRow(page, mails[0]!.subject).click();
    const card = page.getByRole('region', { name: 'Calendar invitation' });
    await expect(card.getByRole('heading', { name: `Kickoff ${tag}` })).toBeVisible();
    if (!narrow) {
      await expect(page.getByTestId('reading-pane')).toBeVisible();
      await expect(messageRow(page, mails[1]!.subject)).toBeVisible();
    }
    await shot('4-open');
  });
}
