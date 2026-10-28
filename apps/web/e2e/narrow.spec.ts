import { readFileSync } from 'node:fs';
import { test, expect, devices, type Browser, type Locator, type Page } from '@playwright/test';
import {
  ENGINE_CREDS,
  composeSurface,
  drawerButton as menuButton,
  engineLogin,
  enginePhoneLogin as phoneLogin,
  injectViaSmtp,
  messageRow,
  messageSlot,
  readerContent,
  sidebarInbox,
  waitForInboxMessageViaDrawer as waitForRowViaDrawer,
} from './helpers.ts';

/**
 * Narrow-viewport layout (t28-e6), end-to-end against the REAL engine stack
 * (mw-server in MW_MODE=engine over Greenmail — the `phone` project, a Pixel 7
 * viewport).
 *
 * Before this layout existed, `styles/app.css` hid `.reader` unconditionally at
 * ≤ 760 px, so at phone width — in the browser and in both Tauri shells, which
 * load the same bundle — a message could be listed but never read. These specs
 * pin the replacement: one pane at a time (list → reader → Back), the sidebar as
 * a drawer, Compose as a full-screen sheet; and, at a desktop viewport, the
 * three-pane layout exactly where it was.
 *
 * Row actions (t29-e5, a menu since 998dcdb9): the row's "More actions" button
 * opens the row's context menu (Snooze presets, Labels, Follow-up, …). The
 * second group of cases opens it with real taps, checks its items are
 * finger-sized and on top, and that Snooze, Label and Follow-up each do what
 * they say, on screen and again after the list is fetched afresh.
 *
 * `engineLogin` and `waitForInboxMessage` from helpers.ts wait on the sidebar's
 * Inbox button, which at this width lives in the closed drawer, so the phone
 * cases sign in and poll through the phone-width variants in helpers.ts
 * (`enginePhoneLogin`, `waitForInboxMessageViaDrawer`, `drawerButton`), imported
 * here under the short names the cases use.
 */

// One shared Greenmail account: serial, like the other engine-mode specs.
test.describe.configure({ mode: 'serial' });

const BREAKPOINT = 761;

/** Deliver one plain-text message whose body carries `marker`. */
async function seed(subject: string, marker: string): Promise<void> {
  await injectViaSmtp({ from: 'Narrow Sender <narrow@example.org>', subject, text: marker });
}

/** Assert `target` lies wholly inside the page viewport (not merely attached,
 *  and not merely intersecting it). Coordinates are main-frame even for a
 *  locator inside the reader iframe. Retried briefly because the drawer slides
 *  in over 180 ms; a box that never comes fully on screen still fails. */
async function expectInsideViewport(page: Page, target: Locator, what: string): Promise<void> {
  const size = page.viewportSize();
  expect(size, 'the project sets a viewport').not.toBeNull();
  await expect(async () => {
    const box = await target.boundingBox();
    expect(box, `${what} has a layout box`).not.toBeNull();
    expect(box!.width, `${what} has width`).toBeGreaterThan(0);
    expect(box!.height, `${what} has height`).toBeGreaterThan(0);
    expect(box!.x, `${what} left edge`).toBeGreaterThanOrEqual(0);
    expect(box!.y, `${what} top edge`).toBeGreaterThanOrEqual(0);
    expect(box!.x + box!.width, `${what} right edge`).toBeLessThanOrEqual(size!.width);
    expect(box!.y + box!.height, `${what} bottom edge`).toBeLessThanOrEqual(size!.height);
  }).toPass({ timeout: 3_000 });
}

test.describe('narrow viewport (phone project)', () => {
  test('list → open a message → its body is on screen → Back returns to the list', async ({ page }) => {
    // Precondition: this really is the narrow layout.
    expect(page.viewportSize()!.width).toBeLessThan(BREAKPOINT);

    const stamp = Date.now();
    const subject = `Narrow read ${stamp}`;
    const marker = `narrow-body-${stamp}`;
    await seed(subject, marker);
    await phoneLogin(page);
    await waitForRowViaDrawer(page, subject);

    // Precondition: the list is what is on screen, and no reader is.
    const row = messageRow(page, subject);
    const reader = page.locator('.reader');
    await expect(row).toBeVisible();
    await expect(reader).toBeHidden();
    await expect(page.locator('iframe.reader__frame')).toHaveCount(0);
    // The row's whole box is the tap target: the unstyled per-row action
    // cluster, which otherwise overlaps the following row, is not shown.
    await expect(page.locator('.list__slot').filter({ hasText: subject }).locator('.msg-actions')).toBeHidden();

    await row.click();

    // The reader takes over the viewport and the BODY text is readable in it.
    await expect(reader).toBeVisible();
    await expect(reader.getByRole('heading', { name: subject })).toBeVisible();
    // The text is in the content frame nested in the reader's wrapper (99f51b81).
    const body = (await readerContent(page)).contentFrame().getByText(marker);
    await expect(body).toBeVisible();
    await expectInsideViewport(page, body, 'the message body text');
    // The body frame is not a sliver under a tall header.
    const frameBox = await page.locator('iframe.reader__frame').boundingBox();
    expect(frameBox!.height).toBeGreaterThan(page.viewportSize()!.height / 3);

    // Focus followed the view change, and the covered list is out of reach. The
    // reader's own Back button went in 2387e7dd (compact reader): focus lands on
    // the subject heading, and the message is a workspace tab that its close
    // control dismisses.
    await expect(reader.getByRole('heading', { name: subject })).toBeFocused();
    const back = page.getByRole('button', { name: `Close ${subject}`, exact: true });
    await expectInsideViewport(page, back, 'the control that closes the message');
    // Out of reach: inert under the reader, or (the message in its own workspace
    // tab) not rendered at all.
    await expect
      .poll(async () => {
        const pane = page.locator('.mail-pane');
        return (await pane.getAttribute('inert')) !== null || !(await pane.isVisible());
      })
      .toBe(true);
    // The row is still mounted underneath, but the reader is what a tap in the
    // middle of the screen lands on.
    const size = page.viewportSize()!;
    expect(
      await page.evaluate(
        ([x, y]) => document.elementFromPoint(x!, y!)?.closest('.reader') !== null,
        [size.width / 2, size.height / 2],
      ),
    ).toBe(true);

    // Closing it: the reader goes, the list returns, and focus is not lost: it
    // lands on the tab that is active again (the ARIA tabs pattern; before the
    // workspace tabs it went back to the row from the reader's Back button).
    await back.click();
    await expect(reader).toBeHidden();
    await expect(row).toBeVisible();
    await expect(page.getByRole('tab', { name: 'Mail', exact: true })).toBeFocused();
    await expect(page.locator('.mail-pane')).not.toHaveAttribute('inert', '');
  });

  test('the sidebar is a drawer: closed by default, opens from the bar, Escape closes it', async ({ page }) => {
    await phoneLogin(page);
    const nav = page.getByRole('navigation', { name: 'Mailboxes' });

    // Precondition: closed — the mailbox buttons are not exposed at all.
    await expect(nav).toBeHidden();
    await expect(menuButton(page)).toHaveAttribute('aria-expanded', 'false');

    await menuButton(page).click();
    await expect(nav).toBeVisible();
    await expect(sidebarInbox(page)).toBeVisible();
    await expectInsideViewport(page, sidebarInbox(page), 'the Inbox entry');
    await expect(menuButton(page)).toHaveAttribute('aria-expanded', 'true');
    // Focus moved into the drawer.
    expect(await page.evaluate(() => document.activeElement?.closest('#shell-nav') !== null)).toBe(true);

    await page.keyboard.press('Escape');
    await expect(nav).toBeHidden();
    await expect(menuButton(page)).toBeFocused();

    // Choosing a destination also closes it and shows that destination.
    await menuButton(page).click();
    await nav.getByRole('button', { name: 'Outbox' }).click();
    await expect(nav).toBeHidden();
    await expect(page.locator('.outbox')).toBeVisible();
    await expect(page.locator('.shell__bar-title')).toHaveText('Outbox');
  });

  test.describe('at 360 px', () => {
    test.use({ viewport: { width: 360, height: 740 } });

    test('Compose is a full-screen sheet with every field and Send reachable', async ({ page }) => {
      await phoneLogin(page);
      await page.locator('.shell__bar').getByRole('button', { name: 'Compose', exact: true }).click();
      const dialog = composeSurface(page);
      await expect(dialog).toBeVisible();

      const to = dialog.getByLabel('To', { exact: true });
      const subject = dialog.getByLabel('Subject', { exact: true });
      await expectInsideViewport(page, to, 'the To field');
      await expectInsideViewport(page, subject, 'the Subject field');
      await to.fill(ENGINE_CREDS.selfAddress);
      await subject.fill('narrow compose');
      await expect(to).toHaveValue(ENGINE_CREDS.selfAddress);

      // Nothing is pushed off to the side: neither the sheet nor the page
      // scrolls horizontally.
      const overflow = await page.evaluate(() => {
        const sheet = document.querySelector('.compose');
        return {
          page: document.documentElement.scrollWidth - window.innerWidth,
          sheet: sheet === null ? null : sheet.scrollWidth - sheet.clientWidth,
        };
      });
      expect(overflow.sheet, 'the compose sheet exists').not.toBeNull();
      expect(overflow.page).toBeLessThanOrEqual(0);
      expect(overflow.sheet).toBeLessThanOrEqual(0);

      // Send may sit below the fold of a long form, but scrolling the sheet
      // brings it fully on screen.
      const send = dialog.getByRole('button', { name: 'Send', exact: true });
      await send.scrollIntoViewIfNeeded();
      await expectInsideViewport(page, send, 'the Send button');
      await expect(send).toBeEnabled();
    });
  });
});

// ── Row actions at phone width (t29-e5) ─────────────────────────────────────

/** The English catalog's value for `id` — the names below are read from the
 *  catalog the app ships, not repeated here. */
function catalog(id: string): string {
  const ftl = readFileSync(new URL('../locales/en/mail.ftl', import.meta.url), 'utf8');
  const line = ftl.split(/\r?\n/).find((l) => l.startsWith(`${id} = `));
  if (line === undefined) throw new Error(`locales/en/mail.ftl has no "${id}"`);
  return line.slice(id.length + 3).trim();
}

const NAMES = {
  more: catalog('mail-more-actions'),
  open: catalog('mail-context-open-message'),
  download: catalog('mail-context-download'),
  // The menu names a snooze preset and a label "<action>: <choice>".
  snoozeTomorrow: `${catalog('mail-snooze')}: ${catalog('mail-snooze-tomorrow')}`,
  labelWork: `${catalog('mail-label')}: Work`,
  flag: catalog('mail-flag'),
  clearFlag: catalog('mail-clear-flag'),
} as const;

/** The row's "More actions" button. */
function moreToggle(slot: Locator): Locator {
  return slot.getByTestId('msg-more-toggle');
}

/** The row menu "More actions" opens (998dcdb9). It is portalled to the end of
 *  the document, so it is found on the page, not inside the row. */
function rowMenu(page: Page): Locator {
  return page.getByRole('menu', { name: NAMES.more, exact: true });
}

/** The open menu's items, in order, whatever their kind (a plain action, or a
 *  label that is on or off). */
function menuItems(page: Page): Locator {
  return rowMenu(page).locator('[role="menuitem"], [role="menuitemcheckbox"]');
}

/** One of the menu's items, by its exact accessible name. */
function menuItem(page: Page, name: string): Locator {
  return rowMenu(page)
    .getByRole('menuitem', { name, exact: true })
    .or(rowMenu(page).getByRole('menuitemcheckbox', { name, exact: true }));
}

/** Whether keyboard focus is on one of the open menu's items. */
function focusInMenu(page: Page): Promise<boolean> {
  return page.evaluate(() => {
    const active = document.activeElement;
    const menu = active?.closest('[role="menu"]') ?? null;
    return menu !== null && active !== menu;
  });
}

/**
 * Assert a finger can land on `target`: it is wholly on screen, the element at
 * its centre is `target` itself (nothing is painted over it), and — given
 * `withinRow` — its box stays inside that row's slot and reaches into no OTHER
 * row's. `withinRow` is left out for a menu item, which floats over the list on
 * purpose; there the hit test is the check.
 */
async function expectTapTarget(
  page: Page,
  target: Locator,
  what: string,
  opts: { withinRow?: Locator } = {},
): Promise<void> {
  await expect(target, `${what} is visible`).toBeVisible();
  await expectInsideViewport(page, target, what);
  // null when the target is on top; otherwise what is (named in the failure).
  // Retried briefly, like the viewport check: while the list refetches after a
  // reload its rows are re-laid out, and a hit test in that frame lands on the
  // list itself. A target that never comes out on top still fails.
  await expect(async () => {
    const covering = await target.evaluate((el) => {
      const r = el.getBoundingClientRect();
      const hit = document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2);
      if (hit !== null && (hit === el || el.contains(hit))) return null;
      return hit === null ? 'nothing' : hit.outerHTML.slice(0, 200);
    });
    expect(covering, `${what} is the element under its own centre`).toBeNull();
  }).toPass({ timeout: 3_000 });

  if (opts.withinRow === undefined) return;
  const box = (await target.boundingBox())!;
  const own = (await opts.withinRow.boundingBox())!;
  expect(box.y, `${what} starts inside its own row`).toBeGreaterThanOrEqual(own.y);
  expect(box.y + box.height, `${what} ends inside its own row`).toBeLessThanOrEqual(own.y + own.height);
  const others = await opts.withinRow.evaluate((slotEl) =>
    [...document.querySelectorAll('.list__slot')]
      .filter((el) => el !== slotEl)
      .map((el) => {
        const r = el.getBoundingClientRect();
        return { top: r.top, bottom: r.bottom };
      }),
  );
  // A row really is drawn directly above or below this one, so the overlap
  // check is not passing only because the rows are far apart.
  expect(
    others.some((o) => Math.abs(o.top - (own.y + own.height)) < 1 || Math.abs(o.bottom - own.y) < 1),
    'a row is drawn directly above or below this one',
  ).toBe(true);
  for (const o of others) {
    const overlaps = box.y < o.bottom - 0.5 && box.y + box.height > o.top + 0.5;
    expect(overlaps, `${what} overlaps the row at ${o.top}–${o.bottom}`).toBe(false);
  }
}

/** Assert a menu item is a usable tap target: brought into the menu's view
 *  (the menu scrolls when it is taller than the screen), then on screen, on
 *  top, and at least 40 px each way. */
async function expectMenuTapTarget(page: Page, item: Locator, what: string): Promise<void> {
  await item.scrollIntoViewIfNeeded();
  await expectTapTarget(page, item, what);
  const box = (await item.boundingBox())!;
  expect(box.width, `${what} width`).toBeGreaterThanOrEqual(40);
  expect(box.height, `${what} height`).toBeGreaterThanOrEqual(40);
}

/**
 * Run `act` (a tap on a menu item) and wait until the engine has answered the
 * `Email/set` it sends, with nothing refused. The list changes at once, before
 * that write lands; a reload straight after the tap would abort the request in
 * flight and test the reload, not the write.
 */
async function tapAndAwaitWrite(page: Page, act: () => Promise<void>): Promise<void> {
  const written = page.waitForResponse(
    (r) =>
      r.request().method() === 'POST' &&
      new URL(r.url()).pathname.endsWith('/jmap/api') &&
      (r.request().postData() ?? '').includes('"Email/set"'),
  );
  await act();
  const res = await written;
  expect(res.status(), 'the Email/set request is answered').toBe(200);
  const body = (await res.json()) as {
    methodResponses: [string, { notUpdated?: Record<string, unknown> | null }][];
  };
  for (const [name, result] of body.methodResponses) {
    expect(name, 'the engine ran Email/set').toBe('Email/set');
    expect(result.notUpdated ?? {}, 'the engine refused none of the update').toEqual({});
  }
}

/** Tap "More actions" on a row whose menu is closed, and wait for it to open. */
async function openActions(page: Page, slot: Locator): Promise<void> {
  const toggle = moreToggle(slot);
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await expect(rowMenu(page)).toHaveCount(0);
  await expectTapTarget(page, toggle, 'the More actions button', {
    withinRow: slot,
  });
  await toggle.tap();
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  await expect(rowMenu(page)).toBeVisible();
}

/** Seed a message to act on and one more so its row has a neighbour, sign in,
 *  and wait for both. Returns the subjects. */
async function seedPair(page: Page, tag: string): Promise<{ target: string; neighbour: string }> {
  const stamp = Date.now();
  const neighbour = `Row ${tag} neighbour ${stamp}`;
  const target = `Row ${tag} target ${stamp}`;
  await seed(neighbour, `neighbour-${stamp}`);
  await seed(target, `target-${stamp}`);
  await phoneLogin(page);
  await waitForRowViaDrawer(page, neighbour);
  await waitForRowViaDrawer(page, target);
  return { target, neighbour };
}

/** A second phone with nothing stored on it: a new browser context (no cookies,
 *  no IndexedDB, no service-worker cache), signed in from scratch. What it lists
 *  can only have come from the server. */
async function secondPhone(browser: Browser): Promise<Page> {
  const baseURL = test.info().project.use.baseURL;
  const context = await browser.newContext({
    ...devices['Pixel 7'],
    ...(baseURL !== undefined ? { baseURL } : {}),
  });
  const page = await context.newPage();
  await phoneLogin(page);
  return page;
}

test.describe('row actions at phone width (phone project)', () => {
  test('More actions opens the row menu; every item is a tap target on top of the list', async ({ page }) => {
    expect(page.viewportSize()!.width).toBeLessThan(BREAKPOINT);
    const { target } = await seedPair(page, 'reach');
    const slot = messageSlot(page, target);

    // Precondition: closed. Neither the menu nor any of its items is exposed,
    // to the eye or to the accessibility tree.
    await expect(rowMenu(page)).toHaveCount(0);
    await expect(page.getByRole('menuitem')).toHaveCount(0);
    // The opener is the one button in the row with the catalog's name, and it
    // says it opens a menu.
    const byName = slot.getByRole('button', { name: NAMES.more, exact: true });
    await expect(byName).toHaveCount(1);
    await expect(byName).toHaveAttribute('data-testid', 'msg-more-toggle');
    await expect(byName).toHaveAttribute('aria-haspopup', 'menu');

    await openActions(page, slot);

    // Open: the menu is wholly on screen and offers this row's actions, among
    // them the three the inline cluster used to hold.
    await expectInsideViewport(page, rowMenu(page), 'the More actions menu');
    for (const name of [NAMES.open, NAMES.snoozeTomorrow, NAMES.labelWork, NAMES.flag, NAMES.download]) {
      await expect(menuItem(page, name), `the menu offers "${name}"`).toHaveCount(1);
    }
    // Every item, not only those three, is a finger-sized target on top of the
    // rows it covers.
    const count = await menuItems(page).count();
    expect(count, 'the menu has items').toBeGreaterThan(5);
    for (let i = 0; i < count; i += 1) {
      const item = menuItems(page).nth(i);
      await expectMenuTapTarget(page, item, `menu item ${i + 1} "${(await item.innerText()).trim()}"`);
    }

    // Opening the menu did not open the message underneath it.
    await expect(page.locator('.reader')).toBeHidden();

    // Escape closes it and gives focus back to the opener.
    await page.keyboard.press('Escape');
    await expect(rowMenu(page)).toHaveCount(0);
    await expect(moreToggle(slot)).toHaveAttribute('aria-expanded', 'false');
    await expect(moreToggle(slot)).toBeFocused();
    await expect(page.locator('.reader')).toBeHidden();
  });

  test('Snooze: a preset takes the message out of the list, and it stays out on a second device', async ({
    page,
    browser,
  }) => {
    test.slow(); // two sign-ins and a reload, each waiting on the engine's sync.
    const { target, neighbour } = await seedPair(page, 'snooze');
    const slot = messageSlot(page, target);

    await openActions(page, slot);
    // The menu floats over the rows beneath: the preset must be on screen and on
    // top of whatever it covers.
    const tomorrow = menuItem(page, NAMES.snoozeTomorrow);
    await expectMenuTapTarget(page, tomorrow, 'the Snooze: Tomorrow item');
    await tapAndAwaitWrite(page, () => tomorrow.tap());

    // The snoozed message leaves the list; its neighbour does not.
    await expect(messageSlot(page, target)).toHaveCount(0);
    await expect(messageRow(page, neighbour)).toBeVisible();
    await expect(rowMenu(page)).toHaveCount(0);
    // The tap went to the menu, not through it to a row.
    await expect(page.locator('.reader')).toBeHidden();

    // Reload: still gone.
    await page.reload();
    await expect(menuButton(page)).toBeVisible();
    await waitForRowViaDrawer(page, neighbour);
    await expect(messageSlot(page, target)).toHaveCount(0);

    // A device with nothing stored on it: the server is what hides it.
    const other = await secondPhone(browser);
    try {
      await waitForRowViaDrawer(other, neighbour);
      await expect(messageSlot(other, target)).toHaveCount(0);
    } finally {
      await other.context().close();
    }
  });

  test('Label: a label chosen from the menu shows on the row, and still does on a second device', async ({
    page,
    browser,
  }) => {
    test.slow(); // two sign-ins and a reload, each waiting on the engine's sync.
    const { target, neighbour } = await seedPair(page, 'label');
    const slot = messageSlot(page, target);
    const chip = (p: Page): Locator => messageSlot(p, target).locator('.tag-chip[data-keyword="work"]');
    const work = (): Locator =>
      rowMenu(page).getByRole('menuitemcheckbox', {
        name: NAMES.labelWork,
        exact: true,
      });

    // Precondition: no chip yet, and the menu says the label is not applied.
    await expect(chip(page)).toHaveCount(0);

    await openActions(page, slot);
    await expect(work()).toHaveAttribute('aria-checked', 'false');
    await expectMenuTapTarget(page, work(), 'the Label: Work item');
    await tapAndAwaitWrite(page, () => work().tap());

    // The chip is on this row only, and the menu has closed behind the choice.
    await expect(chip(page)).toBeVisible();
    await expect(chip(page)).toContainText('Work');
    await expect(messageSlot(page, neighbour).locator('.tag-chip')).toHaveCount(0);
    await expect(rowMenu(page)).toHaveCount(0);
    await expect(page.locator('.reader')).toBeHidden();

    // Reload: the chip is still there, and the menu shows the label as applied.
    await page.reload();
    await expect(menuButton(page)).toBeVisible();
    await waitForRowViaDrawer(page, target);
    await expect(chip(page)).toBeVisible();
    await openActions(page, slot);
    await expect(work()).toHaveAttribute('aria-checked', 'true');
    await page.keyboard.press('Escape');
    await expect(rowMenu(page)).toHaveCount(0);

    // A device with nothing stored on it.
    const other = await secondPhone(browser);
    try {
      await waitForRowViaDrawer(other, target);
      await expect(chip(other)).toBeVisible();
    } finally {
      await other.context().close();
    }
  });

  test('Follow-up: a tap marks the row, the mark is there on a second device, and another tap clears it', async ({
    page,
    browser,
  }) => {
    test.slow(); // two sign-ins and a reload, each waiting on the engine's sync.
    const { target, neighbour } = await seedPair(page, 'followup');
    const slot = messageSlot(page, target);
    const mark = (p: Page): Locator => messageSlot(p, target).getByTestId('msg-followup-mark');

    // Precondition: no mark, and the menu offers to set one, not to clear one.
    await expect(mark(page)).toHaveCount(0);

    await openActions(page, slot);
    await expect(menuItem(page, NAMES.clearFlag)).toHaveCount(0);
    await expectMenuTapTarget(page, menuItem(page, NAMES.flag), 'the Flag for follow-up item');
    await tapAndAwaitWrite(page, () => menuItem(page, NAMES.flag).tap());

    // The menu closes behind the choice and hands focus back to the opener, and
    // the row — this row only — now carries the follow-up mark.
    await expect(rowMenu(page)).toHaveCount(0);
    await expect(moreToggle(slot)).toBeFocused();
    await expect(mark(page)).toBeVisible();
    await expectInsideViewport(page, mark(page), 'the follow-up mark');
    await expect(messageSlot(page, neighbour).getByTestId('msg-followup-mark')).toHaveCount(0);
    await expect(page.locator('.reader')).toBeHidden();
    // The mark sits on the row without taking its taps: the opener beside it
    // is still the element under its own centre.
    await expectTapTarget(page, moreToggle(slot), 'the More actions button beside the mark', { withinRow: slot });

    // Reload: the mark is still there, and the menu offers the opposite action.
    await page.reload();
    await expect(menuButton(page)).toBeVisible();
    await waitForRowViaDrawer(page, target);
    await expect(mark(page)).toBeVisible();
    await openActions(page, slot);
    await expect(menuItem(page, NAMES.clearFlag)).toHaveCount(1);
    await expect(menuItem(page, NAMES.flag)).toHaveCount(0);
    await page.keyboard.press('Escape');
    await expect(rowMenu(page)).toHaveCount(0);

    // A device with nothing stored on it.
    const other = await secondPhone(browser);
    try {
      await waitForRowViaDrawer(other, target);
      await expect(mark(other)).toBeVisible();
    } finally {
      await other.context().close();
    }

    // And off again, from the first phone.
    await openActions(page, slot);
    await expectMenuTapTarget(page, menuItem(page, NAMES.clearFlag), 'the Clear follow-up item');
    await tapAndAwaitWrite(page, () => menuItem(page, NAMES.clearFlag).tap());
    await expect(mark(page)).toHaveCount(0);
    await expect(rowMenu(page)).toHaveCount(0);
    await openActions(page, slot);
    await expect(menuItem(page, NAMES.flag)).toHaveCount(1);
    await expect(menuItem(page, NAMES.clearFlag)).toHaveCount(0);
  });

  test('keyboard: Enter or Space opens the menu onto its first item, arrows walk it, Escape and Tab close it', async ({
    page,
  }) => {
    const { target } = await seedPair(page, 'keys');
    const slot = messageSlot(page, target);
    const toggle = moreToggle(slot);
    const items = menuItems(page);

    // The list is inert while it refetches (and focus inside it drops), so the
    // keyboard starts once the fetch the sign-in began has settled.
    await expect(page.getByRole('region', { name: 'Messages' })).toHaveAttribute('aria-busy', 'false');
    await toggle.focus();
    await expect(toggle).toBeFocused();
    await page.keyboard.press('Enter');
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
    await expect(rowMenu(page)).toBeVisible();

    // Focus went to the first item; the arrows move it through the items in
    // order and wrap at either end, Home and End jump, and none of it closes
    // the menu.
    await expect(items.first()).toBeFocused();
    await expect(items.first()).toHaveAccessibleName(NAMES.open);
    await page.keyboard.press('ArrowDown');
    await expect(items.nth(1)).toBeFocused();
    await page.keyboard.press('ArrowUp');
    await expect(items.first()).toBeFocused();
    await page.keyboard.press('ArrowUp');
    await expect(items.last()).toBeFocused();
    await expect(items.last()).toHaveAccessibleName(NAMES.download);
    await page.keyboard.press('ArrowDown');
    await expect(items.first()).toBeFocused();
    await page.keyboard.press('End');
    await expect(items.last()).toBeFocused();
    await page.keyboard.press('Home');
    await expect(items.first()).toBeFocused();
    await expect(rowMenu(page)).toBeVisible();

    // Escape closes the menu and hands focus back to the opener.
    await page.keyboard.press('Escape');
    await expect(rowMenu(page)).toHaveCount(0);
    await expect(toggle).toHaveAttribute('aria-expanded', 'false');
    await expect(toggle).toBeFocused();

    // Space opens it the same way. Tab leaves the menu, closing it, and focus
    // moves on from the opener instead of being lost with the menu.
    await page.keyboard.press('Space');
    await expect(rowMenu(page)).toBeVisible();
    await expect(items.first()).toBeFocused();
    await page.keyboard.press('Tab');
    await expect(rowMenu(page)).toHaveCount(0);
    await expect(toggle).toHaveAttribute('aria-expanded', 'false');
    expect(
      await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body),
      'focus is on an element after Tab closed the menu',
    ).toBe(true);

    // Nothing above opened the message or the drawer.
    await expect(page.locator('.reader')).toBeHidden();
    await expect(page.getByRole('navigation', { name: 'Mailboxes' })).toBeHidden();
  });

  test('a tap on More actions moves focus into the menu, and closing it hands focus back', async ({ page }) => {
    const { target } = await seedPair(page, 'focus');
    const slot = messageSlot(page, target);

    await openActions(page, slot);
    expect(await focusInMenu(page), 'focus is on one of the menu items once More actions has opened it').toBe(true);
    await expect(menuItems(page).first()).toBeFocused();

    await page.keyboard.press('Escape');
    await expect(rowMenu(page)).toHaveCount(0);
    await expect(moreToggle(slot)).toBeFocused();
  });
});

test.describe('desktop viewport (negative control)', () => {
  // Same project, desktop metrics: the narrow rules must not leak past 760 px.
  test.use({ viewport: { width: 1280, height: 720 }, isMobile: false, hasTouch: false, deviceScaleFactor: 1 });

  test('the three panes sit side by side and stay there while a message is open', async ({ page }) => {
    expect(page.viewportSize()!.width).toBeGreaterThanOrEqual(BREAKPOINT);

    const stamp = Date.now();
    const subject = `Wide read ${stamp}`;
    const marker = `wide-body-${stamp}`;
    await seed(subject, marker);
    await engineLogin(page);
    await expect(async () => {
      await sidebarInbox(page).click();
      await expect(messageRow(page, subject)).toBeVisible({ timeout: 3_000 });
    }).toPass({ timeout: 45_000 });

    const sidebar = page.locator('.sidebar');
    const list = page.locator('.list');
    const reader = page.locator('.reader');

    // No narrow chrome.
    await expect(page.locator('.shell__bar')).toHaveCount(0);
    await expect(page.locator('.shell__scrim')).toHaveCount(0);
    await expect(sidebar).toBeVisible();
    await expect(list).toBeVisible();

    // Opening a message: since the workspace tabs (24fb6149) it opens in a tab
    // whose pane holds the list beside the reader (before that the reader pane
    // was there, empty, before anything was open). The negative control is the
    // same: three panes side by side, the list on screen and reachable.
    await messageRow(page, subject).click();
    await expect((await readerContent(page)).contentFrame().getByText(marker)).toBeVisible();

    const [s, l, r] = await Promise.all([sidebar.boundingBox(), list.boundingBox(), reader.boundingBox()]);
    expect(s!.x, 'the sidebar is at the left edge').toBe(0);
    expect(s!.width, 'the sidebar is not a drawer').toBe(220);
    expect(l!.x, 'the list starts after the sidebar').toBeGreaterThanOrEqual(s!.x + s!.width);
    expect(r!.x, 'the reader is beside the list, not over it').toBeGreaterThanOrEqual(l!.x + l!.width - 1);
    await expect(sidebar).toBeVisible();
    await expect(messageRow(page, subject)).toBeVisible();
    await expect(page.locator('.mail-pane')).not.toHaveAttribute('inert', '');
  });
});
