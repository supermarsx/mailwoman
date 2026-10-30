// t111 (user request: "loading messages toast should be a neat toast on the
// bottom right like the other neat toast"), live on the receipts stack
// (scripts/e2e-receipts-stack.mjs), Chromium and Firefox at 1440 and 390 px:
//   - a folder load under rows already on screen, held back 1.5 s at the
//     network, shows ONE busy "Loading messages…" toast in the shared toast
//     region (bottom inline-end at 1440, the region's top placement at 390),
//     with the same box as the app's other toasts ("Pinned message …"), in the
//     polite live region, and it goes away when the load ends;
//   - the list draws no notice of its own any more (.list__loading is gone);
//   - a fast load (no delay) never shows the toast, not even for one frame.
// Screenshots: MW_T111_SHOTS.
import { test, expect, type Page } from '../csrf.ts';
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { drawerButton, sidebarInbox, signInManually } from '../helpers.ts';

const SHOTS = process.env['MW_T111_SHOTS'] ?? 'test-results/t111';
const STYLE_KEYS = ['backgroundColor', 'color', 'borderTopColor', 'borderTopWidth', 'borderRadius', 'boxShadow', 'fontSize', 'fontFamily', 'lineHeight', 'paddingTop', 'paddingBottom', 'paddingLeft', 'paddingRight'] as const;

async function boxStyle(page: Page, testId: string): Promise<Record<string, string>> {
  return page.getByTestId(testId).evaluate((el, keys) => {
    const cs = getComputedStyle(el);
    return Object.fromEntries(keys.map((k) => [k, cs[k as keyof CSSStyleDeclaration] as string]));
  }, [...STYLE_KEYS]);
}

/** Records every loading toast that enters the DOM, so a one-frame flash is caught too. */
async function watchToasts(page: Page): Promise<void> {
  await page.evaluate(() => {
    const w = window as unknown as { __t111Seen: number };
    w.__t111Seen = 0;
    new MutationObserver((records) => {
      for (const r of records) for (const n of r.addedNodes) {
        if (n instanceof HTMLElement && (n.matches('[data-testid="mail-loading-toast"]') || n.querySelector('[data-testid="mail-loading-toast"]'))) w.__t111Seen += 1;
      }
    }).observe(document.body, { childList: true, subtree: true });
  });
}
const seen = (page: Page): Promise<number> => page.evaluate(() => (window as unknown as { __t111Seen: number }).__t111Seen);

async function openInbox(page: Page, phone: boolean): Promise<void> {
  if (phone) await drawerButton(page).click();
  await sidebarInbox(page).click();
}

for (const width of [1440, 390] as const) {
  test(`background folder load shows the corner toast at ${width} px`, async ({ page }, info) => {
    test.skip(info.project.name === 'phone', 'the chromium and firefox projects cover 390 px');
    const phone = width < 761;
    const shots = join(SHOTS, `${info.project.name}-${width}`);
    mkdirSync(shots, { recursive: true });
    await page.setViewportSize({ width, height: phone ? 844 : 900 });
    await page.goto('/');
    await signInManually(page, { username: 'testuser', password: 'testpass' });
    await expect(phone ? drawerButton(page) : sidebarInbox(page)).toBeVisible();
    const list = page.getByRole('region', { name: 'Messages', exact: true });
    await expect(list.locator('.list__row').first()).toBeVisible({ timeout: 45_000 });
    await expect(list).toHaveAttribute('aria-busy', 'false');

    // A reference toast from the app itself: pin the first message ("Pinned message …"), read its box, Undo.
    const first = list.locator('.list__row').first();
    await first.click({ button: 'right' });
    await page.getByRole('menuitem', { name: 'Pin', exact: true }).click();
    const reference = page.getByTestId('toast').filter({ hasText: 'Pinned message' });
    await expect(reference).toBeVisible();
    const referenceStyle = await boxStyle(page, 'toast');
    await page.screenshot({ path: join(shots, 'reference-toast.png') });
    await reference.getByRole('button', { name: 'Undo' }).click();
    await expect(page.getByTestId('toast').filter({ hasText: 'Pinned message' })).toHaveCount(0);
    // Let every other toast leave so the region holds only the one under test.
    await expect(page.getByTestId('toast-region').locator('[data-toast-id]')).toHaveCount(0, { timeout: 15_000 });

    await watchToasts(page);
    let hold = 1_500;
    let held = 0;
    await page.route('**/jmap/api', async (route) => {
      const body = route.request().postData() ?? '';
      if (hold > 0 && body.includes('Email/query') && body.includes('"calculateTotal":true')) {
        held += 1;
        await new Promise((done) => setTimeout(done, hold));
      }
      await route.continue();
    });

    // Slow: the toast appears after the delay, in the region, and leaves when the load ends.
    const started = Date.now();
    await openInbox(page, phone);
    const toastEl = page.getByTestId('mail-loading-toast');
    await expect(toastEl).toBeVisible({ timeout: 3_000 });
    expect(Date.now() - started).toBeGreaterThanOrEqual(350);
    await expect(toastEl).toHaveText(/Loading.*Inbox.*…/);
    await expect(toastEl).toHaveAttribute('data-busy', 'true');
    await expect(toastEl).toHaveAttribute('role', 'status');
    expect(await toastEl.evaluate((el) => el.closest('[aria-live="polite"]') !== null && el.closest('[data-testid="toast-region"]') !== null)).toBe(true);
    await expect(toastEl.locator('[data-toast-spinner]')).toBeVisible();
    await expect(page.locator('.list__loading')).toHaveCount(0);
    await expect(page.getByTestId('mail-loading-toast')).toHaveCount(1);
    await expect(list.locator('.list__row').first()).toBeVisible();

    const style = await boxStyle(page, 'mail-loading-toast');
    expect(style).toEqual(referenceStyle);

    const box = (await toastEl.boundingBox())!;
    const view = page.viewportSize()!;
    // One control row (the reference wraps its longer text, so heights are not compared to it).
    expect(box.height).toBeLessThanOrEqual(40);
    if (phone) {
      // The region's phone placement: full width (12 px gutters) under the top bar.
      expect(box.x).toBeLessThanOrEqual(13);
      expect(box.x + box.width).toBeGreaterThanOrEqual(view.width - 13);
      expect(box.y).toBeLessThan(view.height / 3);
    } else {
      // Bottom inline-end corner, 16-24 px from the edges.
      expect(view.width - (box.x + box.width)).toBeLessThanOrEqual(32);
      expect(view.height - (box.y + box.height)).toBeLessThanOrEqual(32);
    }
    await page.screenshot({ path: join(shots, 'loading-toast.png') });
    await page.getByTestId('toast-region').screenshot({ path: join(shots, 'loading-toast-region.png') });

    await expect(toastEl).toHaveCount(0, { timeout: 10_000 });
    await expect(list).toHaveAttribute('aria-busy', 'false');
    expect(held).toBeGreaterThan(0);
    expect(await seen(page)).toBe(1);
    await page.screenshot({ path: join(shots, 'after-load.png') });

    // Fast: the same load without a delay never shows the toast.
    hold = 0;
    for (let i = 0; i < 3; i += 1) {
      await openInbox(page, phone);
      await expect(list).toHaveAttribute('aria-busy', 'false');
      await page.waitForTimeout(600);
    }
    expect(await seen(page)).toBe(1);
    await expect(page.getByTestId('mail-loading-toast')).toHaveCount(0);
  });
}
