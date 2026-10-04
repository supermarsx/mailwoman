import { readFile } from 'node:fs/promises';
import { test, expect, type Page } from '@playwright/test';
import { engineLogin, gotoModule, reloadToShell, uid } from './pim-helpers.ts';
import { ENGINE_CREDS } from './helpers.ts';

/**
 * V3 Calendar live E2E (plan §3 e12): drive the real Calendar module against the
 * engine's auto-seeded default calendar. Create an event through the editor and
 * see it render in the week view; a recurring event expands to multiple
 * instances; overlapping events raise a conflict badge; a created event survives
 * a full reload (proving the engine round-trip, not local state).
 *
 * The engine↔web contract gap e12 escalated is fixed (t5-e13): the web calendar
 * module reads `CalendarEvent/expand` instances from `response.list`, its true
 * masters from `CalendarEvent/get {ids:null}`, and `Calendar/detectConflicts`
 * pairs from `response.list` ({eventA,eventB,…}). These four specs (render,
 * recurrence, conflict badge, reload persistence) are the live proof.
 *
 * 26.20 t28-e5 adds the three contracts no spec exercised, which is why each
 * was broken against the real engine while the mock-backed unit tests passed:
 * `.ics` import (`blob` in, `{imported,count}` out), export (`{blob}` out), and
 * subscribe-by-URL (through the server's sync driver, `/api/calendar/subscribe`).
 */

const calendar = (page: Page) => page.locator('[data-module="calendar"]');

/** Open the Calendar module and wait until its seeded default calendar loads. */
async function openCalendar(page: Page): Promise<void> {
  await gotoModule(page, 'calendar');
  // The default "Calendar" collection is seeded on the first `Calendar/get`; wait
  // for its sidebar row so the event editor has a target calendar (no race).
  await expect(calendar(page).locator('input[type="checkbox"]').first()).toBeVisible();
}

/**
 * Create an event through the real editor. Leaves the start at the default (now,
 * i.e. this week) so it renders in the default week view, and optionally makes it
 * a daily recurrence.
 */
/**
 * `datetime-local` wants `YYYY-MM-DDTHH:mm` in LOCAL time, which is also the
 * timezone the week grid is laid out in — so this must not go through
 * `toISOString()`, whose UTC shift can move the date across a day boundary.
 */
function localDateTimeValue(d: Date): string {
  const p = (n: number): string => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}T${p(d.getHours())}:${p(d.getMinutes())}`;
}

async function createEvent(page: Page, title: string, opts: { daily?: boolean } = {}): Promise<void> {
  await page.getByRole('button', { name: 'New event' }).click();
  const dialog = page.getByRole('dialog', { name: 'New event' });
  await expect(dialog).toBeVisible();
  await dialog.getByLabel('Title').fill(title);
  if (opts.daily === true) {
    // Start the rule in the PAST, so the expansion covers the whole visible week
    // on every day of the year.
    //
    // Leaving the default start ("now") made this date-latent, and it duly went
    // red on Saturday 2026-09-26. The week grid starts on the locale's first day
    // — `localeWeekStart()` is 0 (Sunday) for the `en-US` the CI browser runs
    // with — so Saturday is the LAST visible day, and a daily rule starting that
    // day has exactly one instance in the window. `> 1` then fails. It passed the
    // day before (Friday gives Fri + Sat = 2) and would have passed on any other
    // weekday, which is why it looked intermittent: the window was one day wide,
    // one day a week.
    //
    // An unbounded daily rule anchored three days back always yields a full
    // week's worth of instances no matter which weekday the run lands on, and it
    // exercises expansion across the whole window rather than only its tail. A
    // wider VIEW would not have fixed this — month view has the same defect at a
    // month boundary, just 12 days a year instead of 52.
    const anchor = new Date();
    anchor.setDate(anchor.getDate() - 3);
    anchor.setHours(9, 0, 0, 0);
    await dialog.getByLabel('Start', { exact: true }).fill(localDateTimeValue(anchor));
    await dialog.getByLabel('Repeats').check();
    await dialog.getByLabel('Frequency').selectOption('daily');
  }
  await dialog.getByRole('button', { name: 'Save' }).click();
  await expect(dialog).toBeHidden();
}

test.describe('Calendar module through the real UI (engine mode)', () => {
  test('the module loads reachable + the event editor opens and saves', async ({ page }) => {
    // The part that IS wired live: the seeded default calendar loads, the editor
    // opens, accepts a title, and Save closes it (the `CalendarEvent/set` create
    // reaches the engine). Rendering the created event back is the fixme'd gap.
    await engineLogin(page);
    await openCalendar(page);

    await page.getByRole('button', { name: 'New event' }).click();
    const dialog = page.getByRole('dialog', { name: 'New event' });
    await expect(dialog).toBeVisible();
    await dialog.getByLabel('Title').fill(`Standup ${uid()}`);
    await dialog.getByRole('button', { name: 'Save' }).click();
    await expect(dialog).toBeHidden();
  });

  test('create an event → it renders in the week view', async ({ page }) => {
    await engineLogin(page);
    await openCalendar(page);

    const title = `Standup ${uid()}`;
    await createEvent(page, title);

    // The event chip (time + title) renders in the week grid, engine-expanded.
    await expect(calendar(page).getByText(title).first()).toBeVisible();
  });

  test('a recurring (daily) event expands to multiple instances', async ({ page }) => {
    await engineLogin(page);
    await openCalendar(page);

    const title = `Daily sync ${uid()}`;
    await createEvent(page, title, { daily: true });

    // A daily rule expands across the visible week → more than one instance.
    const instances = calendar(page).getByText(title);
    await expect(instances.first()).toBeVisible();
    expect(await instances.count()).toBeGreaterThan(1);
  });

  test('overlapping events raise a conflict badge', async ({ page }) => {
    await engineLogin(page);
    await openCalendar(page);

    // Two events left at the default start (now) overlap; engine conflict
    // detection flags the pair and the view renders a conflict badge.
    const tag = uid();
    await createEvent(page, `Overlap A ${tag}`);
    await createEvent(page, `Overlap B ${tag}`);

    await expect(calendar(page).getByText('conflict').first()).toBeVisible();
  });

  test('a created event persists across a full reload (engine round-trip)', async ({ page }) => {
    await engineLogin(page);
    await openCalendar(page);

    const title = `Persisted ${uid()}`;
    await createEvent(page, title);
    await expect(calendar(page).getByText(title).first()).toBeVisible();

    await reloadToShell(page);
    await openCalendar(page);
    await expect(calendar(page).getByText(title).first()).toBeVisible();
  });

  test('import an .ics file → its event renders and survives a reload', async ({ page }) => {
    await engineLogin(page);
    await openCalendar(page);

    const title = `Imported review ${uid()}`;
    // Precondition: nothing with this title is on the calendar yet.
    await expect(calendar(page).getByText(title)).toHaveCount(0);

    // A floating (no TZID, no Z) event at 10:00 today, so it lands in the
    // default week view on any day and in any runner time zone.
    const d = new Date();
    const p = (n: number): string => String(n).padStart(2, '0');
    const ymd = `${d.getFullYear()}${p(d.getMonth() + 1)}${p(d.getDate())}`;
    const ics = [
      'BEGIN:VCALENDAR',
      'VERSION:2.0',
      'PRODID:-//Mailwoman e2e//EN',
      'BEGIN:VEVENT',
      `UID:imp-${uid()}@e2e.test`,
      `SUMMARY:${title}`,
      `DTSTART:${ymd}T100000`,
      `DTEND:${ymd}T110000`,
      'END:VEVENT',
      'END:VCALENDAR',
      '',
    ].join('\r\n');
    await calendar(page)
      .getByLabel('Import calendar file')
      .setInputFiles({ name: 'review.ics', mimeType: 'text/calendar', buffer: Buffer.from(ics, 'utf8') });

    // The engine reports one created event and the view renders it.
    await expect(calendar(page).getByTestId('calendar-feedback')).toHaveText('Imported 1 event.');
    await expect(calendar(page).getByText(title).first()).toBeVisible();

    await reloadToShell(page);
    await openCalendar(page);
    await expect(calendar(page).getByText(title).first()).toBeVisible();
  });

  test('a file that is not a calendar is refused and says so', async ({ page }) => {
    await engineLogin(page);
    await openCalendar(page);

    await calendar(page)
      .getByLabel('Import calendar file')
      .setInputFiles({ name: 'notes.txt', mimeType: 'text/plain', buffer: Buffer.from('not a calendar', 'utf8') });

    const feedback = calendar(page).getByTestId('calendar-feedback');
    await expect(feedback).toHaveAttribute('role', 'alert');
    await expect(feedback).toContainText('Nothing was imported');
  });

  test('export downloads an .ics document containing a created event', async ({ page }) => {
    await engineLogin(page);
    await openCalendar(page);

    const title = `Exported ${uid()}`;
    await createEvent(page, title);
    await expect(calendar(page).getByText(title).first()).toBeVisible();

    const downloadPromise = page.waitForEvent('download');
    await calendar(page).getByRole('button', { name: 'Export' }).click();
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toBe('mailwoman-calendar.ics');
    const path = await download.path();
    const body = await readFile(path, 'utf8');
    // Not the literal "undefined" a missing response field serializes to.
    expect(body.startsWith('BEGIN:VCALENDAR')).toBe(true);
    expect(body).toContain(`SUMMARY:${title}`);
    expect(body.trimEnd().endsWith('END:VCALENDAR')).toBe(true);
  });

  test('subscribing to a feed the server cannot fetch adds no calendar and says so', async ({ page }) => {
    await engineLogin(page);
    await openCalendar(page);

    const rows = calendar(page).locator('aside li');
    const before = await rows.count();
    expect(before).toBeGreaterThan(0);

    // The sync driver fetches through the SSRF-hardened fetcher, which refuses a
    // loopback target — so this fails in the server, deterministically, and the
    // app must not be left with an empty overlay calendar.
    await calendar(page).getByLabel('Calendar URL').fill('http://127.0.0.1:9/none.ics');
    await calendar(page).getByRole('button', { name: 'Subscribe' }).click();

    const feedback = calendar(page).getByTestId('calendar-feedback');
    await expect(feedback).toHaveAttribute('role', 'alert');
    await expect(feedback).toContainText('could not be fetched');
    await reloadToShell(page);
    await openCalendar(page);
    await expect(calendar(page).locator('aside li')).toHaveCount(before);
  });

  test('the task list is not listed as a calendar', async ({ page }) => {
    await engineLogin(page);
    // Visiting Tasks first guarantees the seeded VTODO list exists.
    await gotoModule(page, 'tasks');
    await expect(page.locator('[data-module="tasks"]').getByRole('button', { name: /Tasks/ }).first()).toBeVisible();
    await openCalendar(page);
    // Each calendar row carries a "Toggle <name>" checkbox; the task list has none.
    await expect(calendar(page).locator('aside li', { hasText: /^\W*Tasks\b/ })).toHaveCount(0);
    await expect(calendar(page).getByLabel(/^Toggle \W*Tasks\W*$/)).toHaveCount(0);
  });

  test('an invitation addressed to the account shows Accept / Decline; one that is not, does not', async ({ page }) => {
    await engineLogin(page);
    await openCalendar(page);

    // The engine finds the user's own participant entry by an exact match of the
    // account identity against the participant key (`event_respond`), and the
    // identity is the LOGIN NAME — against Greenmail the bare `testuser`. So the
    // invitation below addresses the attendee by that name. (An invitation sent
    // to `testuser@example.org` is not recognised by the engine as the user's;
    // that is a server-side limit, recorded in the t28-e5 log.)
    const tag = uid();
    const mine = `Invited ${tag}`;
    const theirs = `Not invited ${tag}`;
    const d = new Date();
    const p = (n: number): string => String(n).padStart(2, '0');
    const ymd = `${d.getFullYear()}${p(d.getMonth() + 1)}${p(d.getDate())}`;
    const vevent = (title: string, hour: string, attendee: string): string[] => [
      'BEGIN:VEVENT',
      `UID:inv-${title.replace(/\s+/g, '-')}@e2e.test`,
      `SUMMARY:${title}`,
      `DTSTART:${ymd}T${hour}0000`,
      `DTEND:${ymd}T${hour}3000`,
      'ORGANIZER;CN=Boss:mailto:boss@example.org',
      `ATTENDEE;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:${attendee}`,
      'END:VEVENT',
    ];
    const ics = [
      'BEGIN:VCALENDAR',
      'VERSION:2.0',
      'PRODID:-//Mailwoman e2e//EN',
      ...vevent(mine, '14', ENGINE_CREDS.username),
      ...vevent(theirs, '16', 'someone.else@example.org'),
      'END:VCALENDAR',
      '',
    ].join('\r\n');
    await calendar(page)
      .getByLabel('Import calendar file')
      .setInputFiles({ name: 'invites.ics', mimeType: 'text/calendar', buffer: Buffer.from(ics, 'utf8') });
    await expect(calendar(page).getByTestId('calendar-feedback')).toHaveText('Imported 2 events.');

    // Not a participant → no response controls.
    await calendar(page).getByText(theirs).first().click();
    const editor = page.getByRole('dialog', { name: 'Edit event' });
    await expect(editor).toBeVisible();
    await expect(editor.getByRole('button', { name: 'Accept' })).toHaveCount(0);
    await editor.getByRole('button', { name: 'Cancel' }).click();
    await expect(editor).toBeHidden();

    // A participant keyed by the account identity → the controls are there, and
    // accepting is stored by the engine (it survives a reload).
    await calendar(page).getByText(mine).first().click();
    await expect(editor).toBeVisible();
    await expect(editor.getByRole('group', { name: 'Invitation' })).toContainText('needs-action');
    await editor.getByRole('button', { name: 'Accept' }).click();
    await expect(editor).toBeHidden();

    await reloadToShell(page);
    await openCalendar(page);
    await calendar(page).getByText(mine).first().click();
    await expect(editor.getByRole('group', { name: 'Invitation' })).toContainText('accepted');
  });
});
