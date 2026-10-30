// t92 against a server built from the tree, no route mocks, Dovecot 2.4 as the
// IMAP upstream (scripts/dovecot-t32: testuser / testpass; the seed creates
// "Entwürfe", "Корзина" and "Archive/2026").
//
// MW_T92_LEG says which Dovecot the server signs in to:
//   utf8   the `full` service: IMAP4rev2 + UTF8=ACCEPT, raw UTF-8 names on the
//          wire. Before t92 the sign-in failed as "Invalid credentials".
//   7bit   the `no-utf8` service: names in modified UTF-7 (Entw&APw-rfe). Before
//          t92 the folder-role picker showed them undecoded.
// MW_T92_SHOTS: a directory for screenshots.
//
// 1. Sign-in, the non-ASCII folders decoded in the sidebar and in Settings,
//    then a non-ASCII folder created and renamed from the sidebar menu; the
//    server's own spelling is read back through Mailbox/get.
// 2. Calendar: multi-day events as bars in the week all-day row and the month
//    grid (cut at the week edge, "+N more" past three lanes, right-to-left), and
//    free / tentative / cancelled events styled apart from busy ones with their
//    state in the accessible name.
import { test, expect } from '../csrf.ts';
import type { Locator, Page } from '@playwright/test';

const leg = process.env['MW_T92_LEG'];
const shots = process.env['MW_T92_SHOTS'];
test.skip(leg !== 'utf8' && leg !== '7bit', 'needs MW_T92_LEG=utf8|7bit (see the header)');

type Call = (name: string, args: Record<string, unknown>, using?: string[]) => Promise<Record<string, unknown>>;

async function signIn(page: Page): Promise<Call> {
  await page.goto('/');
  await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toBeVisible();
  await page.getByLabel('Email or username', { exact: true }).fill('testuser');
  await page.getByLabel('Password', { exact: true }).fill('testpass');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByRole('alert').filter({ hasText: /Invalid credentials|could not read/ })).toHaveCount(0);
  await expect(page.getByRole('toolbar', { name: 'Workspace layout' })).toBeVisible();
  const session = (await (await page.request.get('/jmap/session')).json()) as { primaryAccounts: Record<string, string> };
  const accountId = session.primaryAccounts['urn:ietf:params:jmap:mail']!;
  return async (name, args, using = ['urn:ietf:params:jmap:core', 'urn:ietf:params:jmap:mail']) => {
    const send = () => page.request.post('/jmap/api', { data: { using, methodCalls: [[name, { accountId, ...args }, 'c']] } });
    let response = await send();
    for (let attempt = 0; response.status() === 429 && attempt < 40; attempt++) {
      await new Promise((done) => setTimeout(done, 50));
      response = await send();
    }
    expect(response.ok(), `${name} ${response.status()}`).toBe(true);
    const body = (await response.json()) as { methodResponses: [string, Record<string, unknown>][] };
    expect(body.methodResponses[0]![0], JSON.stringify(body)).toBe(name);
    return body.methodResponses[0]![1];
  };
}

async function openNav(page: Page, phone: boolean): Promise<void> {
  if (!phone) return;
  const toggle = page.getByRole('button', { name: 'Open folders and apps' });
  if ((await toggle.getAttribute('aria-expanded')) !== 'true') await toggle.click();
}

async function shot(page: Page, name: string, project: string): Promise<void> {
  if (shots) await page.screenshot({ path: `${shots}/${leg}-${project}-${name}.png` });
}

interface Box { id: string; name: string; mailwomanUpstreamName?: string; mailwomanPath?: string }

test('non-ASCII folders sign in, show decoded and rename', async ({ page }, info) => {
  const phone = (page.viewportSize()?.width ?? 1280) < 600;
  const call = await signIn(page);
  await openNav(page, phone);
  const nav = page.locator('#shell-nav');
  for (const name of ['Entwürfe', 'Корзина']) await expect(nav.getByText(name, { exact: true }).first()).toBeVisible();
  // No modified UTF-7 spelling reaches the screen.
  await expect(nav).not.toContainText('&APw-');
  await expect(nav).not.toContainText('&BBoE');
  await shot(page, 'sidebar-decoded', info.project.name);

  let boxes = (await call('Mailbox/get', {})).list as Box[];
  // Folders a failed earlier run left behind.
  const stale = boxes.filter((b) => /^(Prüfung|Überprüft) /.test(b.name)).map((b) => b.id);
  if (stale.length) {
    await call('Mailbox/set', { destroy: stale, onDestroyRemoveEmails: true });
    boxes = (await call('Mailbox/get', {})).list as Box[];
  }
  const drafts = boxes.find((b) => b.name === 'Entwürfe')!;
  expect(drafts.mailwomanUpstreamName).toBe(leg === 'utf8' ? 'Entwürfe' : 'Entw&APw-rfe');
  expect(drafts.mailwomanPath).toBe('Entwürfe');

  // Create a non-ASCII folder, then rename it from the sidebar menu.
  const tag = `${info.project.name}-${Date.now() % 100000}`;
  const created = `Prüfung ${tag}`;
  const renamed = `Überprüft «${tag}» Кор`;
  const set = await call('Mailbox/set', { create: { n: { name: created } } });
  const id = (set.created as Record<string, { id: string }>).n!.id;
  try {
    const row = nav.getByRole('button', { name: created, exact: true });
    await expect(row).toBeVisible({ timeout: 20000 });
    // A touch screen has no right button: the keyboard opens the same menu.
    if (phone) {
      await row.focus();
      await page.keyboard.press('Shift+F10');
    } else await row.click({ button: 'right' });
    await page.getByRole('menuitem', { name: 'Rename…' }).click({ timeout: 15000 });
    // Inline in the sidebar on a wide screen; a dialog on a phone.
    if (phone) {
      const dialog = page.getByRole('dialog', { name: new RegExp(`Rename .*${created}`) });
      await dialog.getByLabel('Folder name').fill(renamed);
      await shot(page, 'rename-dialog', info.project.name);
      await dialog.getByRole('button', { name: 'Rename', exact: true }).click();
    } else {
      const input = nav.getByRole('textbox', { name: new RegExp(`New name for .*${created}`) });
      await input.fill(renamed);
      await shot(page, 'rename-inline', info.project.name);
      await input.press('Enter');
    }
    await openNav(page, phone);
    await expect(nav.getByText(renamed, { exact: true })).toBeVisible({ timeout: 20000 });
    await expect(nav.getByText(created, { exact: true })).toHaveCount(0);
    await shot(page, 'renamed', info.project.name);
    const after = ((await call('Mailbox/get', { ids: [id] })).list as Box[])[0]!;
    expect(after.name).toBe(renamed);
    if (leg === '7bit') {
      expect(after.mailwomanUpstreamName).toMatch(/^[\x20-\x7e]+$/);
      expect(after.mailwomanUpstreamName).toContain('&');
    } else {
      expect(after.mailwomanUpstreamName).toBe(renamed);
    }

    // Settings > folder roles list the readable path too.
    await openNav(page, phone);
    await page.locator('.sidebar__footer').getByRole('button', { name: 'Settings', exact: true }).click();
    await page.getByRole('navigation', { name: 'Settings sections' }).getByRole('button', { name: 'Folders and sidebar', exact: true }).click();
    const roleSelect = page.locator('select[id^="folder-role-"]').first();
    await expect(roleSelect.locator('option', { hasText: 'Entwürfe' })).toHaveCount(1);
    await expect(roleSelect.locator('option', { hasText: '&APw-' })).toHaveCount(0);
  } finally {
    await call('Mailbox/set', { destroy: [id], onDestroyRemoveEmails: true });
  }
});

const CAL = ['urn:ietf:params:jmap:core', 'urn:mailwoman:calendars'];

test('calendar: multi-day bars and free / tentative / cancelled styles', async ({ page }, info) => {
  const phone = (page.viewportSize()?.width ?? 1280) < 600;
  const call = await signIn(page);
  await openNav(page, phone);
  const cal = page.locator('[data-module="calendar"]');
  // Right after sign-in the first sync re-renders the sidebar, and a click that
  // lands during it can be lost (seen in Firefox): click until the module opens.
  await expect(async () => {
    await page.getByTestId('nav-calendar').click();
    await expect(cal).toBeVisible({ timeout: 3000 });
  }).toPass({ timeout: 30000 });
  await expect(cal.locator('input[type="checkbox"]').first()).toBeAttached();
  await cal.getByRole('tab', { name: 'Week', exact: true }).click();
  const cells = cal.getByTestId('all-day-cell');
  await expect(cells).toHaveCount(7);
  const days = await cells.evaluateAll((els) => els.map((e) => e.getAttribute('data-date')!));

  const calendars = (await call('Calendar/get', { ids: null }, CAL)).list as { id: string }[];
  const calendarId = calendars[0]!.id;
  // Events a failed earlier run left behind.
  const old = ((await call('CalendarEvent/get', { ids: null }, CAL)).list as { id: string; title: string }[])
    .filter((e) => /^(Conference|Trip|Extra [ABC]|Focus block|Maybe sync|Dropped standup|Busy review) (chromium|firefox|phone)-\d+$/.test(e.title))
    .map((e) => e.id);
  if (old.length) await call('CalendarEvent/set', { destroy: old }, CAL);
  const tag = `${info.project.name}-${Date.now() % 100000}`;
  const base = { calendarId, description: '', timeZone: null, locations: [], recurrenceRules: [], excludedRecurrenceDates: [], participants: {}, alerts: {} };
  const allDay = (title: string, day: number, length: number, extra: Record<string, unknown> = {}) => ({
    ...base, title: `${title} ${tag}`, start: `${days[day]}T00:00:00`, duration: `P${length}D`, showWithoutTime: true, status: 'confirmed', freeBusyStatus: 'busy', ...extra,
  });
  const timed = (title: string, hour: number, extra: Record<string, unknown>) => ({
    ...base, title: `${title} ${tag}`, start: `${days[2]}T${String(hour).padStart(2, '0')}:00:00`, duration: 'PT1H', showWithoutTime: false, status: 'confirmed', freeBusyStatus: 'busy', ...extra,
  });
  const create = {
    conf: allDay('Conference', 1, 3),
    trip: allDay('Trip', 5, 5),
    ea: allDay('Extra A', 1, 1),
    eb: allDay('Extra B', 1, 1),
    ec: allDay('Extra C', 1, 1),
    free: timed('Focus block', 9, { freeBusyStatus: 'free' }),
    tent: timed('Maybe sync', 11, { status: 'tentative' }),
    canc: timed('Dropped standup', 13, { status: 'cancelled' }),
    busy: timed('Busy review', 15, {}),
  };
  const made = (await call('CalendarEvent/set', { create }, CAL)).created as Record<string, { id: string }>;
  const ids = Object.values(made).map((m) => m.id);
  expect(ids).toHaveLength(Object.keys(create).length);
  try {
    // Reload the week from the engine.
    await cal.getByRole('button', { name: 'Next', exact: true }).first().click();
    await cal.getByRole('button', { name: 'Today', exact: true }).first().click();
    const bar = (title: string): Locator => cal.getByTestId('span-bar').filter({ hasText: `${title} ${tag}` });

    // Week all-day row: one bar per event, sized to its days; the trip is cut
    // at the week's end; day 1 holds four bars, so one is behind "+1 more".
    await expect(bar('Conference')).toHaveAttribute('data-span', '3');
    await expect(bar('Trip')).toHaveAttribute('data-span', '2');
    await expect(bar('Trip')).toHaveAttribute('data-continues', 'after');
    await expect(cells.nth(1).getByTestId('span-more')).toHaveText('+1 more');
    const confBox = (await bar('Conference').boundingBox({ timeout: 15000 }))!;
    const c1 = (await cells.nth(1).boundingBox({ timeout: 15000 }))!;
    const c3 = (await cells.nth(3).boundingBox({ timeout: 15000 }))!;
    // The bar runs from day 1's cell to day 3's (2px cell padding at each end).
    expect(Math.abs(confBox.x - (c1.x + 2))).toBeLessThan(3);
    expect(Math.abs(confBox.x + confBox.width - (c3.x + c3.width - 2))).toBeLessThan(3);

    // Free / tentative / cancelled / busy on day 2's timed events.
    const chip = (title: string): Locator => cal.getByTestId('day-column').getByRole('button', { name: new RegExp(`${title} ${tag}`) });
    const look = (l: Locator) => l.evaluate((el: Element) => {
      const s = getComputedStyle(el);
      return { bg: s.backgroundColor, border: s.borderTopWidth, borderStyle: s.borderTopStyle, deco: s.textDecorationLine, opacity: s.opacity, label: el.getAttribute('aria-label') ?? '', hatch: el.querySelector('[data-testid="event-hatch"] pattern') !== null };
    }, undefined, { timeout: 15000 });
    const free = await look(chip('Focus block'));
    const tent = await look(chip('Maybe sync'));
    const canc = await look(chip('Dropped standup'));
    const busy = await look(chip('Busy review'));
    expect(free.bg).not.toBe(busy.bg);
    expect(free.border).toBe('2px');
    expect(free.borderStyle).toBe('solid');
    expect(free.label).toMatch(/Shows as free/);
    expect(tent.hatch).toBe(true);
    expect(tent.label).toMatch(/Tentative/);
    expect(canc.deco).toContain('line-through');
    expect(Number(canc.opacity)).toBeLessThan(1);
    expect(canc.label).toMatch(/Cancelled/);
    expect(busy.hatch).toBe(false);
    expect(busy.label).not.toMatch(/Tentative|Cancelled|free/);
    await chip('Maybe sync').scrollIntoViewIfNeeded({ timeout: 15000 });
    await shot(page, 'week', info.project.name);

    // Month: the trip crosses the week boundary, so it is two bars.
    await cal.getByRole('tab', { name: 'Month', exact: true }).click();
    await expect(bar('Trip')).toHaveCount(2);
    await expect(bar('Trip').first()).toHaveAttribute('data-continues', 'after');
    await expect(bar('Trip').nth(1)).toHaveAttribute('data-continues', 'before');
    await expect(bar('Trip').nth(1)).toHaveAttribute('aria-label', new RegExp(`Trip ${tag}.* to `));
    await expect(bar('Conference')).toHaveAttribute('data-span', '3');
    await shot(page, 'month', info.project.name);

    // Right-to-left: the bar starts at its first day's right edge and grows left.
    await page.evaluate(() => document.documentElement.setAttribute('dir', 'rtl'));
    const startCell = cal.locator(`[role="gridcell"][data-date="${days[1]}"]`);
    const endCell = cal.locator(`[role="gridcell"][data-date="${days[3]}"]`);
    const rb = (await bar('Conference').boundingBox({ timeout: 15000 }))!;
    const rs = (await startCell.boundingBox({ timeout: 15000 }))!;
    const re = (await endCell.boundingBox({ timeout: 15000 }))!;
    expect(rs.x).toBeGreaterThan(re.x);
    expect(Math.abs(rb.x + rb.width - (rs.x + rs.width - 3))).toBeLessThan(4);
    expect(Math.abs(rb.x - (re.x + 3))).toBeLessThan(4);
    await shot(page, 'month-rtl', info.project.name);
    await page.evaluate(() => document.documentElement.setAttribute('dir', 'ltr'));
  } finally {
    await call('CalendarEvent/set', { destroy: ids }, CAL);
  }
});
