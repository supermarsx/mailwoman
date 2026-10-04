import { test, expect, type Locator, type Page } from '@playwright/test';
import { ENGINE_CREDS, engineLogin, injectViaSmtp, messageRow, sidebarInbox } from './helpers.ts';

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
 * `engineLogin` and `waitForInboxMessage` from helpers.ts wait on the sidebar's
 * Inbox button, which at this width lives in the closed drawer, so the phone
 * cases sign in and poll through the helpers below instead.
 */

// One shared Greenmail account: serial, like the other engine-mode specs.
test.describe.configure({ mode: 'serial' });

const BREAKPOINT = 761;

function menuButton(page: Page): Locator {
  return page.getByRole('button', { name: 'Open folders and apps' });
}

/** Sign in at phone width; the shell is ready once its top bar is up. */
async function phoneLogin(page: Page): Promise<void> {
  await page.goto('/');
  await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
  await page.getByLabel('JMAP server URL').fill(ENGINE_CREDS.imapUrl);
  await page.getByLabel('Username', { exact: true }).fill(ENGINE_CREDS.username);
  await page.getByLabel('Password', { exact: true }).fill(ENGINE_CREDS.password);
  await page.getByRole('button', { name: 'Sign in' }).click();
  await expect(menuButton(page)).toBeVisible();
}

/** Re-select the Inbox through the drawer (which re-queries the engine) until
 *  the injected message is listed. */
async function waitForRowViaDrawer(page: Page, subject: string, timeout = 45_000): Promise<void> {
  await expect(async () => {
    await menuButton(page).click();
    await sidebarInbox(page).click();
    await expect(messageRow(page, subject)).toBeVisible({ timeout: 3_000 });
  }).toPass({ timeout });
}

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
