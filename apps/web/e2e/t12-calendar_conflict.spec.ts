import { test, expect, type Page } from '@playwright/test';
import { engineLogin, gotoModule, uid } from './pim-helpers.ts';

/**
 * 26.12 calendar conflict resolver + schedule view + attendee-role UI live E2E
 * (audit #7 / #15 web, SPEC §11, plan §2 Batch-4). Drives the REAL calendar module
 * against the engine's auto-seeded default calendar:
 *
 *   • two overlapping events raise a conflict → the "Resolve conflicts" toolbar
 *     button opens the SIDE-BY-SIDE resolver (earlier vs later panels) with the
 *     free/busy grid (the previously-unused `queryFreeBusy`, now consumed);
 *   • a resolution action (reschedule) APPLIES through the engine and reduces the
 *     conflict count;
 *   • the Schedule view renders DISTINCTLY (its own `schedule-view`, not aliasing
 *     the Agenda list);
 *   • the attendee ROLE / CUTYPE pickers work and the event saves through the engine.
 */

const calendar = (page: Page) => page.locator('[data-module="calendar"]');

async function openCalendar(page: Page): Promise<void> {
  await gotoModule(page, 'calendar');
  await expect(calendar(page).locator('input[type="checkbox"]').first()).toBeVisible();
}

/** A fixed mid-day datetime-local string for TODAY (09:30), so overlapping events sit
 *  well inside a single day — the resolver's free/busy grid spans a real hour range
 *  regardless of the wall-clock time the test runs at (a "now" default near midnight
 *  would cross into the next day and collapse the grid's hour window). */
function midDayToday(): string {
  const d = new Date();
  const p = (n: number): string => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}T09:30`;
}

/** Create an event through the real editor at a fixed mid-day start (this week). */
async function createEvent(page: Page, title: string): Promise<void> {
  await page.getByRole('button', { name: 'New event' }).click();
  const dialog = page.getByRole('dialog', { name: 'New event' });
  await expect(dialog).toBeVisible();
  await dialog.getByLabel('Title').fill(title);
  await dialog.getByLabel('Start').fill(midDayToday());
  await dialog.getByRole('button', { name: 'Save' }).click();
  await expect(dialog).toBeHidden();
}

/** Read the integer conflict count off the "Resolve N conflicts" toolbar button. */
async function conflictCount(page: Page): Promise<number> {
  const btn = page.getByRole('button', { name: /Resolve \d+ conflict/ });
  if ((await btn.count()) === 0) return 0;
  const text = (await btn.first().innerText()).match(/\d+/);
  return text ? Number(text[0]) : 0;
}

test('side-by-side conflict resolver: two overlapping events → resolve applies + free/busy grid', async ({
  page,
}) => {
  test.setTimeout(90_000);
  const tag = uid();

  await engineLogin(page);
  await openCalendar(page);

  // Two events left at the default start overlap → the engine flags a conflict.
  await createEvent(page, `Conflict A ${tag}`);
  await createEvent(page, `Conflict B ${tag}`);

  const resolveBtn = page.getByRole('button', { name: /Resolve \d+ conflict/ });
  await expect(resolveBtn).toBeVisible();
  const before = await conflictCount(page);
  expect(before).toBeGreaterThan(0);

  // Open the side-by-side resolver.
  await resolveBtn.click();
  const resolver = page.getByRole('dialog', { name: 'Resolve conflicts' });
  await expect(resolver).toBeVisible();

  // If other conflicts exist (accumulated state), select the pair with MY two events so
  // the side-by-side + free/busy grid reflect the events created just now (times "now").
  const picker = resolver.locator('#resolver-pair');
  if ((await picker.count()) > 0) {
    const value = await picker.locator('option', { hasText: tag }).first().getAttribute('value');
    if (value !== null) await picker.selectOption(value);
  }

  // Side-by-side comparison: the earlier + later panels both render.
  await expect(resolver.locator('[data-testid="resolver-earlier"]')).toBeVisible();
  await expect(resolver.locator('[data-testid="resolver-later"]')).toBeVisible();

  // The free/busy grid (consuming queryFreeBusy) renders as a real table.
  await expect(resolver.locator('[data-testid="freebusy-grid"]')).toBeVisible();

  // Apply a resolution: reschedule the later event to start when the earlier ends.
  // This goes through the engine (CalendarEvent/set update) and clears that pair.
  await resolver.getByRole('button', { name: 'Reschedule later event' }).click();

  // The resolution APPLIED: the total conflict count dropped (the resolved pair no
  // longer overlaps). Poll — the update round-trips through the engine + re-detects.
  await expect
    .poll(async () => conflictCount(page), { timeout: 20_000, intervals: [500, 1000, 2000] })
    .toBeLessThan(before);

  // Close the resolver.
  await resolver.getByRole('button', { name: 'Close' }).click();
  await expect(resolver).toBeHidden();
});

test('the Schedule view renders distinctly from the Agenda view', async ({ page }) => {
  test.setTimeout(60_000);

  await engineLogin(page);
  await openCalendar(page);

  // Schedule tab → the distinct schedule feed (its own testid), NOT the agenda list.
  await calendar(page).getByRole('tab', { name: 'Schedule' }).click();
  await expect(calendar(page).locator('[data-testid="schedule-view"]')).toBeVisible();
  await expect(
    calendar(page).getByRole('list', { name: 'Agenda' }),
  ).toHaveCount(0);

  // Agenda tab → the agenda list, and the schedule feed is gone (no aliasing).
  await calendar(page).getByRole('tab', { name: 'Agenda' }).click();
  await expect(calendar(page).getByRole('list', { name: 'Agenda' })).toBeVisible();
  await expect(calendar(page).locator('[data-testid="schedule-view"]')).toHaveCount(0);
});

test('attendee ROLE / CUTYPE pickers work and the event saves through the engine', async ({ page }) => {
  test.setTimeout(60_000);
  const tag = uid();
  const attendee = `guest-${tag}@example.com`;

  await engineLogin(page);
  await openCalendar(page);

  await page.getByRole('button', { name: 'New event' }).click();
  const dialog = page.getByRole('dialog', { name: 'New event' });
  await expect(dialog).toBeVisible();
  await dialog.getByLabel('Title').fill(`Roles ${tag}`);

  // Add an attendee → its ROLE + CUTYPE pickers appear.
  await dialog.getByLabel('Add attendee').fill(attendee);
  await dialog.getByRole('button', { name: 'Add', exact: true }).click();

  // The attendee row rendered with its email.
  await expect(dialog.getByText(attendee)).toBeVisible();
  // The ROLE/CUTYPE selects' accessible names embed the email wrapped in bidi-isolate
  // marks (isolate()), so match on the stable "Role for"/"Type for" attribute prefix.
  const rolePicker = dialog.locator('select[aria-label^="Role for"]');
  const cutypePicker = dialog.locator('select[aria-label^="Type for"]');
  await expect(rolePicker).toBeVisible();
  await expect(cutypePicker).toBeVisible();

  // Set a non-default role (Optional) + type (Room) and confirm the pickers hold it.
  await rolePicker.selectOption('optional');
  await cutypePicker.selectOption('room');
  await expect(rolePicker).toHaveValue('optional');
  await expect(cutypePicker).toHaveValue('room');

  // Save → the event (with roles) persists through the engine; the dialog closes.
  await dialog.getByRole('button', { name: 'Save' }).click();
  await expect(dialog).toBeHidden();
});
