import { readFileSync } from 'node:fs';
import { test, expect, devices, type Browser, type Locator, type Page } from '@playwright/test';
import {
  ENGINE_CREDS,
  drawerButton as menuButton,
  engineLogin,
  enginePhoneLogin as phoneLogin,
  injectViaSmtp,
  messageRow,
  messageSlot,
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
 * Row actions (t29-e5): the per-row cluster (Snooze, Label, Follow-up, …) is
 * closed at this width until the row's "More actions" button opens it. The
 * second group of cases opens it with real taps and checks each of the three
 * does what it says, on screen and again after the list is fetched afresh.
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
    const body = page.frameLocator('iframe.reader__frame').getByText(marker);
    await expect(body).toBeVisible();
    await expectInsideViewport(page, body, 'the message body text');
    // The body frame is not a sliver under a tall header.
    const frameBox = await page.locator('iframe.reader__frame').boundingBox();
    expect(frameBox!.height).toBeGreaterThan(page.viewportSize()!.height / 3);

    // Focus followed the view change, and the covered list is out of reach.
    const back = reader.getByRole('button', { name: 'Back' });
    await expect(back).toBeFocused();
    await expectInsideViewport(page, back, 'the Back control');
    await expect(page.locator('.mail-pane')).toHaveAttribute('inert', '');
    // The row is still mounted underneath, but the reader is what a tap in the
    // middle of the screen lands on.
    const size = page.viewportSize()!;
    expect(
      await page.evaluate(
        ([x, y]) => document.elementFromPoint(x!, y!)?.closest('.reader') !== null,
        [size.width / 2, size.height / 2],
      ),
    ).toBe(true);

    // Back: the reader goes, the list returns, focus lands on the row just read.
    await back.click();
    await expect(reader).toBeHidden();
    await expect(row).toBeVisible();
    await expect(row).toBeFocused();
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
      await page.locator('.shell__bar').getByRole('button', { name: 'Compose' }).click();
      const dialog = page.getByRole('dialog', { name: 'Compose message' });
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
  snooze: catalog('mail-snooze'),
  snoozeMenu: catalog('mail-snooze-menu'),
  tomorrow: catalog('mail-snooze-tomorrow'),
  label: catalog('mail-label'),
  labelsMenu: catalog('mail-labels-menu'),
  flag: catalog('mail-flag'),
  clearFlag: catalog('mail-clear-flag'),
} as const;

/** The row's "More actions" button. */
function moreToggle(slot: Locator): Locator {
  return slot.getByTestId('msg-more-toggle');
}

/** The row's action cluster (closed at phone width until the toggle opens it). */
function actionCluster(slot: Locator): Locator {
  return slot.locator('.msg-actions');
}

/** One of the cluster's buttons, by its exact accessible name. */
function action(slot: Locator, name: string): Locator {
  return actionCluster(slot).getByRole('button', { name, exact: true });
}

/** Whether keyboard focus is on something inside the row's cluster. */
function focusInCluster(slot: Locator): Promise<boolean> {
  return slot.evaluate((el) => el.querySelector('.msg-actions')?.contains(document.activeElement) === true);
}

/**
 * Assert a finger can land on `target`: it is wholly on screen, the element at
 * its centre is `target` itself (nothing is painted over it), and — given
 * `withinRow` — its box stays inside that row's slot and reaches into no OTHER
 * row's. `withinRow` is left out for a menu item, which drops below its row on
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
  const onTop = await target.evaluate((el) => {
    const r = el.getBoundingClientRect();
    const hit = document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2);
    return hit !== null && (hit === el || el.contains(hit));
  });
  expect(onTop, `${what} is the element under its own centre`).toBe(true);

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

/** Tap "More actions" on a row whose cluster is closed, and wait for it to open. */
async function openActions(page: Page, slot: Locator): Promise<void> {
  const toggle = moreToggle(slot);
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await expect(actionCluster(slot)).toBeHidden();
  await expectTapTarget(page, toggle, 'the More actions button', {
    withinRow: slot,
  });
  await toggle.tap();
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  await expect(actionCluster(slot)).toBeVisible();
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
  test('More actions opens the cluster; Snooze, Label and Follow-up are tap targets inside their own row', async ({
    page,
  }) => {
    expect(page.viewportSize()!.width).toBeLessThan(BREAKPOINT);
    const { target } = await seedPair(page, 'reach');
    const slot = messageSlot(page, target);

    // Precondition: closed. None of the three is exposed, to the eye or to the
    // accessibility tree.
    await expect(actionCluster(slot)).toBeHidden();
    for (const name of [NAMES.snooze, NAMES.label, NAMES.flag]) {
      await expect(slot.getByRole('button', { name, exact: true })).toHaveCount(0);
    }
    // The opener is the one button in the row with the catalog's name.
    const byName = slot.getByRole('button', { name: NAMES.more, exact: true });
    await expect(byName).toHaveCount(1);
    await expect(byName).toHaveAttribute('data-testid', 'msg-more-toggle');

    await openActions(page, slot);

    // Open: a named group holding the three, each a tap target of usable size
    // that stays inside this row.
    await expect(slot.getByRole('group', { name: NAMES.more })).toBeVisible();
    for (const name of [NAMES.snooze, NAMES.label, NAMES.flag]) {
      const button = action(slot, name);
      await expectTapTarget(page, button, `the ${name} button`, {
        withinRow: slot,
      });
      const box = (await button.boundingBox())!;
      expect(box.width, `${name} width`).toBeGreaterThanOrEqual(40);
      expect(box.height, `${name} height`).toBeGreaterThanOrEqual(40);
    }
    // The opener is still reachable while the cluster is open (it closes it).
    await expectTapTarget(page, moreToggle(slot), 'the More actions button, open', { withinRow: slot });

    // Opening the cluster did not open the message underneath it.
    await expect(page.locator('.reader')).toBeHidden();

    // Tapping the opener again closes the cluster.
    await moreToggle(slot).tap();
    await expect(actionCluster(slot)).toBeHidden();
    await expect(moreToggle(slot)).toHaveAttribute('aria-expanded', 'false');
    await expect(moreToggle(slot)).toBeFocused();
  });

  test('Snooze: a preset takes the message out of the list, and it stays out on a second device', async ({
    page,
    browser,
  }) => {
    test.slow(); // two sign-ins and a reload, each waiting on the engine's sync.
    const { target, neighbour } = await seedPair(page, 'snooze');
    const slot = messageSlot(page, target);

    await openActions(page, slot);
    await action(slot, NAMES.snooze).tap();

    // The presets menu drops below the row, over the rows beneath: the item
    // must be on screen and on top of whatever it covers.
    const menu = slot.getByRole('menu', { name: NAMES.snoozeMenu });
    await expect(menu).toBeVisible();
    await expect(action(slot, NAMES.snooze)).toHaveAttribute('aria-expanded', 'true');
    const tomorrow = menu.getByRole('menuitem', {
      name: NAMES.tomorrow,
      exact: true,
    });
    await expectTapTarget(page, tomorrow, 'the Tomorrow preset');
    await tomorrow.tap();

    // The snoozed message leaves the list; its neighbour does not.
    await expect(messageSlot(page, target)).toHaveCount(0);
    await expect(messageRow(page, neighbour)).toBeVisible();
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

    // Precondition: no chip yet.
    await expect(chip(page)).toHaveCount(0);

    await openActions(page, slot);
    await action(slot, NAMES.label).tap();

    const menu = slot.getByRole('menu', { name: NAMES.labelsMenu });
    await expect(menu).toBeVisible();
    const work = menu.getByRole('menuitemcheckbox', { name: /Work/ });
    await expect(work).toHaveAttribute('aria-checked', 'false');
    await expectTapTarget(page, work, 'the Work label');
    await work.tap();

    // The chip is on this row only, and the cluster has closed behind the choice.
    await expect(chip(page)).toBeVisible();
    await expect(chip(page)).toContainText('Work');
    await expect(messageSlot(page, neighbour).locator('.tag-chip')).toHaveCount(0);
    await expect(actionCluster(slot)).toBeHidden();
    await expect(page.locator('.reader')).toBeHidden();

    // Reload: the chip is still there, and the menu shows the label as applied.
    await page.reload();
    await expect(menuButton(page)).toBeVisible();
    await waitForRowViaDrawer(page, target);
    await expect(chip(page)).toBeVisible();
    await openActions(page, slot);
    await action(slot, NAMES.label).tap();
    await expect(slot.getByRole('menuitemcheckbox', { name: /Work/ })).toHaveAttribute('aria-checked', 'true');

    // A device with nothing stored on it.
    const other = await secondPhone(browser);
    try {
      await waitForRowViaDrawer(other, target);
      await expect(chip(other)).toBeVisible();
    } finally {
      await other.context().close();
    }
  });

  test('Follow-up: the flag is set by a tap, is still set on a second device, and is cleared by another tap', async ({
    page,
    browser,
  }) => {
    test.slow(); // two sign-ins and a reload, each waiting on the engine's sync.
    const { target } = await seedPair(page, 'followup');
    const slot = messageSlot(page, target);

    await openActions(page, slot);
    await expect(action(slot, NAMES.flag)).toHaveAttribute('aria-pressed', 'false');
    await action(slot, NAMES.flag).tap();

    // The control now offers the opposite action and reports itself as on. The
    // row has no follow-up mark of its own, so the cluster is where it shows;
    // setting the flag rewrites the list, which may have closed the cluster.
    const expectFlagged = async (p: Page): Promise<void> => {
      const s = messageSlot(p, target);
      await expect(async () => {
        if ((await moreToggle(s).getAttribute('aria-expanded')) !== 'true') await moreToggle(s).tap();
        await expect(action(s, NAMES.clearFlag)).toHaveAttribute('aria-pressed', 'true', { timeout: 2_000 });
      }).toPass({ timeout: 10_000 });
      await expect(action(s, NAMES.flag)).toHaveCount(0);
    };
    await expectFlagged(page);
    await expect(page.locator('.reader')).toBeHidden();

    // Reload.
    await page.reload();
    await expect(menuButton(page)).toBeVisible();
    await waitForRowViaDrawer(page, target);
    await expectFlagged(page);

    // A device with nothing stored on it.
    const other = await secondPhone(browser);
    try {
      await waitForRowViaDrawer(other, target);
      await expectFlagged(other);
    } finally {
      await other.context().close();
    }

    // And off again, from the first phone.
    await expectTapTarget(page, action(slot, NAMES.clearFlag), 'the Clear follow-up button', { withinRow: slot });
    await action(slot, NAMES.clearFlag).tap();
    await expect(async () => {
      if ((await moreToggle(slot).getAttribute('aria-expanded')) !== 'true') await moreToggle(slot).tap();
      await expect(action(slot, NAMES.flag)).toHaveAttribute('aria-pressed', 'false', { timeout: 2_000 });
    }).toPass({ timeout: 10_000 });
  });

  test('keyboard: Enter opens the cluster, Tab walks it, Escape closes a menu and then the cluster', async ({
    page,
  }) => {
    const { target } = await seedPair(page, 'keys');
    const slot = messageSlot(page, target);
    const toggle = moreToggle(slot);

    await toggle.focus();
    await expect(toggle).toBeFocused();
    await page.keyboard.press('Enter');
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
    await expect(actionCluster(slot)).toBeVisible();

    // The cluster's buttons are the next tab stops, in reading order; walking
    // them does not close the cluster.
    await page.keyboard.press('Tab');
    expect(await focusInCluster(slot), 'Tab from the opener lands in the cluster').toBe(true);
    await page.keyboard.press('Tab');
    await expect(action(slot, NAMES.snooze)).toBeFocused();
    await expect(actionCluster(slot)).toBeVisible();

    // Enter on Snooze opens its menu; Escape closes the menu only and hands
    // focus back to Snooze.
    await page.keyboard.press('Enter');
    await expect(slot.getByRole('menu', { name: NAMES.snoozeMenu })).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(slot.getByRole('menu')).toHaveCount(0);
    await expect(actionCluster(slot)).toBeVisible();
    await expect(action(slot, NAMES.snooze)).toBeFocused();

    // Escape again closes the cluster and hands focus back to the opener.
    await page.keyboard.press('Escape');
    await expect(actionCluster(slot)).toBeHidden();
    await expect(toggle).toHaveAttribute('aria-expanded', 'false');
    await expect(toggle).toBeFocused();

    // Nothing above opened the message or the drawer.
    await expect(page.locator('.reader')).toBeHidden();
    await expect(page.getByRole('navigation', { name: 'Mailboxes' })).toBeHidden();
  });

  // DEFECT (components/MessageActions.tsx:101): the opener's handler is
  // `open() ? closeAll() : setOpen(true)` and nothing else, so after "More
  // actions" opens the cluster, focus is still on the opener. The cluster is the
  // next element in the DOM, so Tab (and a screen reader's next-item gesture)
  // reaches it, which the keyboard case above pins; what does not happen is
  // focus moving INTO the cluster on open. Seen failing on 2026-10-05 against
  // the current tree: `focusInCluster` is false after the tap. Remove `fixme`
  // when the handler focuses the first cluster button after opening.
  test.fixme('opening the cluster moves focus into it', async ({ page }) => {
    const { target } = await seedPair(page, 'focus');
    const slot = messageSlot(page, target);

    await openActions(page, slot);
    expect(await focusInCluster(slot), 'focus is on one of the cluster buttons once More actions has opened it').toBe(
      true,
    );
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

    // No narrow chrome, and the reader pane is present before anything is open.
    await expect(page.locator('.shell__bar')).toHaveCount(0);
    await expect(page.locator('.shell__scrim')).toHaveCount(0);
    await expect(sidebar).toBeVisible();
    await expect(list).toBeVisible();
    await expect(reader).toBeVisible();

    const columns = async (): Promise<{ sidebar: number; list: number; reader: number; sidebarWidth: number }> => {
      const [s, l, r] = await Promise.all([sidebar.boundingBox(), list.boundingBox(), reader.boundingBox()]);
      return { sidebar: s!.x, list: l!.x, reader: r!.x, sidebarWidth: s!.width };
    };
    const before = await columns();
    expect(before.sidebar).toBe(0);
    expect(before.sidebarWidth).toBe(220);
    expect(before.list).toBe(220);
    expect(before.reader).toBeGreaterThan(before.list);

    await messageRow(page, subject).click();
    await expect(page.frameLocator('iframe.reader__frame').getByText(marker)).toBeVisible();

    // Still three panes: the list is on screen and reachable beside the reader,
    // and nothing moved.
    await expect(sidebar).toBeVisible();
    await expect(messageRow(page, subject)).toBeVisible();
    await expect(page.locator('.mail-pane')).not.toHaveAttribute('inert', '');
    expect(await columns()).toEqual(before);
  });
});
