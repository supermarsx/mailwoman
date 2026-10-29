// t74: where a message opened from the Mail list goes, against a live server
// (no route mocks).
//   - reading pane Off: a click opens the message in its own workspace tab; a
//     second click on the same row reuses that tab;
//   - reading pane on (Right): a click shows the message in the reading pane and
//     adds no tab; the next click replaces it;
//   - reading pane on, double-click: exactly one tab opens and shows the message,
//     with no second full Email/get for it and no frame without the message's
//     subject on screen; a second double-click reuses the tab;
//   - Ctrl/Cmd+click selects without opening (no tab, the pane keeps its message);
//   - Shift+Enter on a focused row opens its tab and focus stays on the row (t51);
//   - phone (390): there is no pane beside the list, so a tap opens the message
//     the way the phone layout always did: the reader covers the list.
// Runs in the receipts config (chromium, firefox at 1440; phone at 390) against
// MW_RECEIPTS_BASE_URL with the seeded testuser mailbox. Screenshots go to
// MW_T74_SHOTS when set.
import path from 'node:path';
import { test, expect, type Page, type TestInfo } from '../csrf.ts';

const USER = { name: process.env['MW_LIST_KEYS_USER'] ?? 'testuser', pass: process.env['MW_LIST_KEYS_PASS'] ?? 'testpass' };
const rows = (page: Page) => page.locator('.list__scroll .list__row');
const shot = (info: TestInfo, name: string): string => {
  const dir = process.env['MW_T74_SHOTS'];
  return dir ? path.join(dir, `${info.project.name}-${name}.png`) : info.outputPath(`${name}.png`);
};
const messageTabs = (page: Page) => page.locator('[role=tab][id^="subtab-tab-message:"]');
const mailTab = (page: Page) => page.getByRole('tab', { name: 'Mail', exact: true });
const paneSubject = (page: Page) => page.getByTestId('reading-pane').locator('.reader__subject');

async function signIn(page: Page): Promise<void> {
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill(USER.name);
  await page.getByLabel('Password', { exact: true }).fill(USER.pass);
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(rows(page).nth(8)).toBeAttached({ timeout: 60000 });
  await page.waitForLoadState('networkidle');
}
async function setPane(page: Page, mode: 'right' | 'bottom' | 'off'): Promise<void> {
  if (await mailTab(page).getAttribute('aria-selected') !== 'true') await mailTab(page).click();
  await page.getByTestId(`reading-pane-${mode}`).click();
  await expect(page.getByTestId(`reading-pane-${mode}`)).toHaveAttribute('aria-pressed', 'true');
}
async function subjectAt(page: Page, index: number): Promise<string> {
  return ((await rows(page).nth(index).locator('.list__subject').textContent()) ?? '').trim();
}
/** Close every message tab a previous leg left open (resumed tabs come back on sign-in). */
async function closeMessageTabs(page: Page): Promise<void> {
  for (let i = 0; i < 20 && (await messageTabs(page).count()) > 0; i += 1) {
    const count = await messageTabs(page).count();
    await messageTabs(page).first().click();
    await page.locator('.subtab--active .subtab__close').click();
    await expect(messageTabs(page)).toHaveCount(count - 1);
  }
  if (await mailTab(page).getAttribute('aria-selected') !== 'true') await mailTab(page).click();
}
/** Full-message loads (Email/get with bodies) for one id, from this page and its workers. */
function countFullLoads(page: Page): (id: string) => number {
  const seen: string[] = [];
  page.context().on('request', (request) => {
    const body = request.postData() ?? '';
    if (body.includes('"Email/get"') && body.includes('fetchHTMLBodyValues')) seen.push(body);
  });
  return (id) => seen.filter((body) => body.includes(`"ids":["${id}"]`)).length;
}

test.describe('reading pane routing (t74)', () => {
  test.beforeEach(async ({ page }, info) => {
    if (info.project.name !== 'phone') await page.setViewportSize({ width: 1440, height: 900 });
  });

  test('reading pane off: a click opens the message tab, and the same row again reuses it', async ({ page }, info) => {
    test.skip(info.project.name === 'phone', 'phone: covered by the phone leg');
    await signIn(page);
    await closeMessageTabs(page);
    await setPane(page, 'off');
    try {
      const subject = await subjectAt(page, 1);
      const before = await messageTabs(page).count();
      await rows(page).nth(1).click();
      const tab = page.getByRole('tab', { name: subject, exact: true });
      await expect(tab).toHaveAttribute('aria-selected', 'true');
      await expect(page.locator('.reader__subject').filter({ visible: true }).first()).toHaveText(subject);
      await expect(page.getByTestId('reading-pane')).toBeHidden();
      expect(await messageTabs(page).count()).toBe(before + 1);
      await page.screenshot({ path: shot(info, 'off-click-tab') });
      await mailTab(page).click();
      await rows(page).nth(1).click();
      await expect(tab).toHaveAttribute('aria-selected', 'true');
      expect(await messageTabs(page).count()).toBe(before + 1);
    } finally {
      await closeMessageTabs(page);
      await setPane(page, 'right');
    }
  });

  test('reading pane on: a click shows the message in the pane and adds no tab', async ({ page }, info) => {
    test.skip(info.project.name === 'phone', 'phone: no pane beside the list');
    await signIn(page);
    await closeMessageTabs(page);
    for (const mode of ['right', 'bottom'] as const) {
      await setPane(page, mode);
      const before = await messageTabs(page).count();
      const first = await subjectAt(page, 2);
      await rows(page).nth(2).click();
      await expect(paneSubject(page)).toHaveText(first);
      await expect(mailTab(page)).toHaveAttribute('aria-selected', 'true');
      await expect(rows(page).nth(2)).toBeVisible();
      const second = await subjectAt(page, 3);
      await rows(page).nth(3).click();
      await expect(paneSubject(page)).toHaveText(second);
      await page.waitForTimeout(400);
      expect(await messageTabs(page).count()).toBe(before);
      await expect(page.getByTestId('reading-pane')).toHaveCount(1);
      // t51: the row that opened it keeps focus.
      await expect(rows(page).nth(3)).toBeFocused();
      await page.screenshot({ path: shot(info, `pane-${mode}-click`) });
    }
    await setPane(page, 'right');
  });

  test('reading pane on, double-click: exactly one tab, no second load, no frame without the message', async ({ page }, info) => {
    test.skip(info.project.name === 'phone', 'phone: no pane beside the list');
    const loads = countFullLoads(page);
    await signIn(page);
    await closeMessageTabs(page);
    await setPane(page, 'right');
    const row = rows(page).nth(4);
    const subject = await subjectAt(page, 4);
    const id = (await row.getAttribute('data-email-id'))!;
    const before = await messageTabs(page).count();
    // Sample every frame from the first click until the tab shows the message.
    await page.evaluate((want) => {
      const w = window as unknown as { __t74: { frames: number; missing: number; listGone: number; stop: boolean } };
      w.__t74 = { frames: 0, missing: 0, listGone: 0, stop: false };
      const tick = (): void => {
        const s = w.__t74;
        if (s.stop) return;
        s.frames += 1;
        const visible = [...document.querySelectorAll<HTMLElement>('.reader__subject')].some((el) => el.textContent?.trim() === want && el.checkVisibility());
        // Counted only once the first click has put the message on screen.
        if (s.frames > 0 && (window as unknown as { __t74seen?: boolean }).__t74seen && !visible) s.missing += 1;
        if (visible) (window as unknown as { __t74seen?: boolean }).__t74seen = true;
        const list = document.querySelector<HTMLElement>('.mail-pane');
        if (!list || !list.checkVisibility()) s.listGone += 1;
        requestAnimationFrame(tick);
      };
      requestAnimationFrame(tick);
    }, subject);
    await row.dblclick();
    const tab = page.getByRole('tab', { name: subject, exact: true });
    await expect(tab).toHaveAttribute('aria-selected', 'true');
    await expect(page.locator('.workspace-pane[role=tabpanel] .reader__subject').filter({ visible: true }).first()).toHaveText(subject);
    await page.waitForTimeout(1500);
    const sample = await page.evaluate(() => { const w = window as unknown as { __t74: { frames: number; missing: number; listGone: number; stop: boolean } }; w.__t74.stop = true; return w.__t74; });
    test.info().annotations.push({ type: 't74-frames', description: JSON.stringify(sample) });
    await page.screenshot({ path: shot(info, 'pane-dblclick-tab') });
    expect(await messageTabs(page).count()).toBe(before + 1);
    expect(loads(id)).toBe(1);
    expect(sample.missing).toBe(0);
    // The list stays beside the tab (reading pane on).
    expect(sample.listGone).toBe(0);
    await expect(row).toBeVisible();
    // Again: the same tab, not a second one.
    await row.dblclick();
    await expect(tab).toHaveAttribute('aria-selected', 'true');
    await page.waitForTimeout(400);
    expect(await messageTabs(page).count()).toBe(before + 1);
    expect(loads(id)).toBe(1);
    await closeMessageTabs(page);
  });

  test('Ctrl/Cmd+click selects without opening; Shift+Enter opens the focused row in a tab', async ({ page }, info) => {
    test.skip(info.project.name === 'phone', 'phone: no modifier clicks or pane');
    await signIn(page);
    await closeMessageTabs(page);
    await setPane(page, 'right');
    const shown = await subjectAt(page, 1);
    await rows(page).nth(1).click();
    await expect(paneSubject(page)).toHaveText(shown);
    const before = await messageTabs(page).count();
    await rows(page).nth(5).click({ modifiers: ['ControlOrMeta'] });
    await rows(page).nth(6).click({ modifiers: ['ControlOrMeta'] });
    await expect(page.locator('.list__row--selected')).toHaveCount(2);
    await page.waitForTimeout(400);
    expect(await messageTabs(page).count()).toBe(before);
    await expect(paneSubject(page)).toHaveText(shown);
    await page.screenshot({ path: shot(info, 'ctrl-click-select') });
    await page.keyboard.press('Escape');
    await expect(page.locator('.list__row--selected')).toHaveCount(0);

    const target = rows(page).nth(7);
    const subject = await subjectAt(page, 7);
    await target.focus();
    await page.keyboard.press('Shift+Enter');
    const tab = page.getByRole('tab', { name: subject, exact: true });
    await expect(tab).toHaveAttribute('aria-selected', 'true');
    expect(await messageTabs(page).count()).toBe(before + 1);
    await page.waitForTimeout(600);
    await expect(target).toBeFocused();
    await expect(page.locator('.list__row--selected')).toHaveCount(0);
    await page.screenshot({ path: shot(info, 'shift-enter-tab') });
    await closeMessageTabs(page);
  });

  test('phone: a tap opens the message over the list, as the phone layout did before', async ({ page }, info) => {
    test.skip(info.project.name !== 'phone', 'phone only');
    await signIn(page);
    const subject = await subjectAt(page, 1);
    await rows(page).nth(1).tap();
    await expect(page.locator('.reader__subject').filter({ visible: true }).first()).toHaveText(subject);
    await expect(page.locator('.mail-pane')).toBeHidden();
    await expect(page.getByTestId('reading-pane')).toBeHidden();
    await page.screenshot({ path: shot(info, 'tap-open') });
    await page.getByRole('button', { name: `Close ${subject}`, exact: true }).click();
    await expect(rows(page).first()).toBeVisible();
  });
});
