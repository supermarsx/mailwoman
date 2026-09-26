import { test, expect, type Page } from '@playwright/test';
import { engineLogin, gotoModule, reloadToShell, uid } from './pim-helpers.ts';

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
});
