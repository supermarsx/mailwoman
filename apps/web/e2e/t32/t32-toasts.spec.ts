import { test, expect, type Locator, type Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { composeSelf, drawerButton, engineLogin, enginePhoneLogin, sidebarInbox } from '../helpers.ts';

/**
 * The shared toasts in a real browser (t32 f39, user request 2026-10-25: "saved
 * alerts and all other messages should be popped up in a neat toast message that
 * auto dismisses after a while but is also user dismissable").
 *
 *  - A toast closes by itself (success after 5 s), and not while the pointer
 *    rests on it or focus is in it; it closes from its button and on Escape.
 *  - Desktop (1440x900): bottom inline-end corner, so bottom left under RTL.
 *    Phone (390x844): top, full width, below the top bar.
 *  - Forced colours keep its border and icon; reduced motion drops the slide.
 *  - No style attribute is refused by the CSP; axe finds nothing (WCAG 2.2 AA).
 *
 * The toasts come from the app's own paths: going offline and back raises
 * "Connection lost — retrying…" (stays while offline) and then "Back online"
 * (success, replaces it by id); a send raises "Message sent" with Cancel and the
 * seconds left. Runs in `engine` and `engine-firefox` against an engine-mode
 * stack (MW_UPSTREAM_DEFAULT set).
 */

test.describe.configure({ mode: 'serial' });

const toastWith = (page: Page, text: string | RegExp): Locator => page.locator('.toast', { hasText: text });

/** Fails the test on a style attribute the shell CSP refused. */
function watchCsp(page: Page): string[] {
  const refused: string[] = [];
  page.on('console', (m) => {
    if (/Content.Security.Policy/i.test(m.text())) refused.push(m.text());
  });
  return refused;
}

/** Re-select the Inbox, which sends a request (through the drawer on a phone). */
async function requestInbox(page: Page, phone: boolean): Promise<void> {
  if (phone) {
    // A retry must not click the drawer button while the drawer is open: that shuts it.
    if ((await drawerButton(page).getAttribute('aria-expanded')) !== 'true') await drawerButton(page).click();
    // Short, so a surrounding toPass can retry instead of one click eating its budget.
    await page.getByRole('navigation', { name: 'Mailboxes' }).getByRole('button', { name: /^Inbox/ }).click({ timeout: 3_000 });
  } else {
    await sidebarInbox(page).click();
  }
}

/** Offline and back: leaves the "Back online" success toast on screen. */
async function backOnline(page: Page, phone = false): Promise<Locator> {
  // Load the mailbox list while still online: there is no Inbox to click until it arrives
  // (Firefox at 390px lost that race in CI and spent the whole budget waiting offline).
  await expect(async () => {
    await requestInbox(page, phone);
  }).toPass({ timeout: 30_000 });
  await page.context().setOffline(true);
  await expect(async () => {
    await requestInbox(page, phone);
    await expect(toastWith(page, 'Connection lost')).toBeVisible({ timeout: 2_000 });
  }).toPass({ timeout: 20_000 });
  await page.context().setOffline(false);
  const back = toastWith(page, 'Back online');
  await expect(async () => {
    // The first request back online may already have raised it; another click then only
    // spends time while the 5 s toast closes by itself.
    if (!(await back.isVisible())) await requestInbox(page, phone);
    await expect(back).toBeVisible({ timeout: 2_000 });
  }).toPass({ timeout: 30_000 });
  // Same id: "Back online" replaced "Connection lost", it did not stack on it.
  await expect(toastWith(page, 'Connection lost')).toHaveCount(0);
  return back;
}

async function box(l: Locator): Promise<{ x: number; y: number; width: number; height: number }> {
  const b = await l.boundingBox();
  if (b === null) throw new Error('toast has no box');
  return b;
}

test.describe('toasts at 1440x900', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('auto-dismiss, pause on hover and focus, close button and Escape', async ({ page }) => {
    test.slow();
    const refused = watchCsp(page);
    await engineLogin(page);

    const back = await backOnline(page);
    await expect(back).toHaveAttribute('role', 'status');
    // Bottom inline-end corner (LTR: bottom right). A "Reconnected" toast from
    // the realtime connection may stack under it, so the bottom edge is checked
    // on the region's newest toast.
    const b = await box(back);
    expect(1440 - (b.x + b.width)).toBeLessThan(40);
    const newest = await box(page.locator('.toast').last());
    expect(900 - (newest.y + newest.height)).toBeLessThan(40);

    // The pointer on the toast stops its 5 s clock.
    await back.hover();
    await expect(back).toHaveAttribute('data-paused', 'true');
    await page.waitForTimeout(7_000);
    await expect(back).toBeVisible();
    // Off the toast, the time left runs out.
    await page.mouse.move(400, 300);
    await expect(back).not.toHaveAttribute('data-paused', 'true');
    await expect(back).toBeHidden({ timeout: 7_000 });

    // Closes from its own button.
    const again = await backOnline(page);
    await again.getByRole('button', { name: 'Close notification' }).click();
    await expect(again).toBeHidden();

    // Keyboard: focus in the toast stops the clock, Enter on close closes it.
    const third = await backOnline(page);
    await page.mouse.move(400, 300);
    await third.getByRole('button', { name: 'Close notification' }).focus();
    await expect(third).toHaveAttribute('data-paused', 'true');
    await page.waitForTimeout(6_000);
    await expect(third).toBeVisible();
    await page.keyboard.press('Enter');
    await expect(third).toBeHidden();

    // Escape closes the newest toast when nothing else takes it. Offline, the
    // one toast is "Connection lost", which stays until closed.
    await page.mouse.move(400, 300);
    await page.context().setOffline(true);
    const lost = toastWith(page, 'Connection lost');
    await expect(async () => {
      await requestInbox(page, false);
      await expect(lost).toBeVisible({ timeout: 2_000 });
    }).toPass({ timeout: 20_000 });
    await expect(lost).toHaveAttribute('role', 'alert');
    await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
    await page.keyboard.press('Escape');
    await expect(lost).toBeHidden();
    await page.context().setOffline(false);

    expect(refused).toEqual([]);
  });

  test('a send shows Message sent with Cancel and the seconds left; axe finds nothing', async ({ page }) => {
    test.slow();
    const refused = watchCsp(page);
    await engineLogin(page);
    await composeSelf(page, `Toast ${Date.now()}`, 'toast check');
    const sent = toastWith(page, 'Message sent');
    await expect(sent).toBeVisible();
    await expect(sent.getByRole('button', { name: 'Cancel' })).toBeVisible();
    await expect(sent).toContainText(/\d+ s/);
    const axe = await new AxeBuilder({ page })
      .include('[data-testid="toast-region"]')
      .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'])
      .analyze();
    expect(axe.violations.map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(' ')).join(', ')}`)).toEqual([]);
    expect(refused).toEqual([]);
  });

  test('right to left: the toast sits at the bottom left', async ({ page }) => {
    await engineLogin(page);
    await page.evaluate(() => document.documentElement.setAttribute('dir', 'rtl'));
    const back = await backOnline(page);
    const b = await box(back);
    expect(b.x).toBeLessThan(40);
    expect(b.y).toBeGreaterThan(450);
  });

  test('forced colours keep the border and the icon', async ({ page }) => {
    await page.emulateMedia({ forcedColors: 'active' });
    await engineLogin(page);
    const back = await backOnline(page);
    const border = await back.evaluate((el) => {
      const cs = getComputedStyle(el);
      return { style: cs.borderTopStyle, width: parseFloat(cs.borderTopWidth), start: parseFloat(cs.borderInlineStartWidth) };
    });
    expect(border.style).toBe('solid');
    expect(border.width).toBeGreaterThanOrEqual(1);
    expect(border.start).toBeGreaterThan(border.width);
    await expect(back.locator('svg').first()).toBeVisible();
    await expect(back.getByRole('button', { name: 'Close notification' })).toBeVisible();
  });

  test('reduced motion drops the slide; full motion keeps it', async ({ page }) => {
    await engineLogin(page);
    const moving = await backOnline(page);
    expect(await moving.evaluate((el) => getComputedStyle(el).animationName)).not.toBe('none');
    await moving.getByRole('button', { name: 'Close notification' }).click();
    await page.emulateMedia({ reducedMotion: 'reduce' });
    const still = await backOnline(page);
    expect(await still.evaluate((el) => getComputedStyle(el).animationName)).toBe('none');
  });
});

test.describe('toasts at 390x844', () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true });

  test('top, full width, closes from its button, and no page scroll', async ({ page }) => {
    test.slow();
    const refused = watchCsp(page);
    await enginePhoneLogin(page);
    const back = await backOnline(page, true);
    const b = await box(back);
    // At the top, under the top bar: the menu button and Compose stay reachable.
    expect(b.y).toBeLessThan(120);
    const menu = await box(drawerButton(page));
    expect(b.y).toBeGreaterThanOrEqual(menu.y + menu.height);
    expect(b.width).toBeGreaterThanOrEqual(390 - 2 * 8 - 2);
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390);
    await back.getByRole('button', { name: 'Close notification' }).click();
    await expect(back).toBeHidden();
    expect(refused).toEqual([]);
  });

  test('right to left at phone width stays full width', async ({ page }) => {
    await enginePhoneLogin(page);
    await page.evaluate(() => document.documentElement.setAttribute('dir', 'rtl'));
    const back = await backOnline(page, true);
    const b = await box(back);
    expect(b.x).toBeLessThan(12);
    expect(b.width).toBeGreaterThanOrEqual(390 - 2 * 8 - 2);
  });
});
