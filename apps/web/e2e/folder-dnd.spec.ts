import { test, expect } from './csrf.ts';
import { login } from './helpers.ts';
import type { Locator, Page } from '@playwright/test';

/**
 * Folder drag and drop in the sidebar: reorder among siblings, nest under another
 * folder as an arrangement in this app, undo from the toast, and the drag state in
 * light and dark themes.
 *
 * The server answers 429 to a headless browser's default `HeadlessChrome` user
 * agent, so this spec presents itself as an ordinary Chrome.
 */
test.use({
  userAgent:
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36',
  viewport: { width: 1280, height: 1200 },
});

const JMAP = 'urn:ietf:params:jmap:core';
const MAIL = 'urn:ietf:params:jmap:mail';

/** Three top-level folders with names unique to this run (the mock keeps state between specs). */
async function seedFolders(page: Page): Promise<[string, string, string]> {
  const stamp = Date.now().toString(36);
  const names: [string, string, string] = [`Dnd A ${stamp}`, `Dnd B ${stamp}`, `Dnd C ${stamp}`];
  const session = (await (await page.request.get('/jmap/session')).json()) as { primaryAccounts: Record<string, string> };
  const accountId = session.primaryAccounts[MAIL]!;
  const response = await page.request.post('/jmap/api', {
    data: {
      using: [JMAP, MAIL],
      methodCalls: [['Mailbox/set', { accountId, create: Object.fromEntries(names.map((name, i) => [`c${i}`, { name, parentId: null }])) }, 'seed']],
    },
  });
  expect(response.ok()).toBe(true);
  const body = (await response.json()) as { methodResponses: [string, { created?: Record<string, unknown> }, string][] };
  expect(Object.keys(body.methodResponses[0]![1].created ?? {})).toHaveLength(3);
  return names;
}

const nav = (page: Page): Locator => page.getByRole('navigation', { name: 'Mailboxes' });
const folder = (page: Page, name: string): Locator => nav(page).getByRole('button', { name, exact: true });
/** The row element (the draggable one) that holds a folder's button. */
const row = (page: Page, name: string): Locator => folder(page, name).locator('xpath=ancestor::*[@draggable][1]');

async function order(page: Page, names: readonly string[]): Promise<string[]> {
  const all = await nav(page).locator('.sidebar__box-name').allInnerTexts();
  return all.filter((text) => names.includes(text.trim())).map((text) => text.trim());
}

test.describe('folder drag and drop', () => {
  test('reorders, nests as an arrangement, and undoes from the toast', async ({ page }) => {
    await login(page);
    const seeded = await seedFolders(page);
    await page.reload();
    await expect(folder(page, seeded[2])).toBeVisible();
    // Stored arrangements are keyed by folder id, so the starting order is whatever the sidebar shows; name the folders by position.
    const [a, b, c] = await order(page, seeded) as [string, string, string];

    // Reorder: drop the last of the three on the top quarter of the first.
    await row(page, c).dragTo(row(page, a), { targetPosition: { x: 40, y: 2 } });
    await expect.poll(() => order(page, [a, b, c])).toEqual([c, a, b]);
    await expect(page.getByText(/Folder moved:/).filter({ hasText: c })).toBeVisible();

    // Nest: drop B on the middle of A. The default is to ask how.
    await row(page, b).dragTo(row(page, a), { targetPosition: { x: 40, y: 18 } });
    const dialog = page.getByRole('dialog').or(page.getByRole('alertdialog'));
    await expect(dialog.getByText(/Move .* here\?/)).toContainText(b);
    await dialog.getByRole('radio', { name: /Only arrange in this app/ }).check();
    await dialog.getByRole('button', { name: 'Move', exact: true }).click();
    await expect(page.getByText(/Folder moved:/).filter({ hasText: b })).toBeVisible();

    // B is now below A: its row is one level deeper once A is expanded.
    const depthOf = async (name: string): Promise<string> =>
      (await folder(page, name).locator('xpath=ancestor::*[contains(@style,"--folder-depth")][1]').getAttribute('style')) ?? '';
    const expand = page.getByRole('button', { name: `Expand ${a}` });
    if (await expand.isVisible().catch(() => false)) await expand.click();
    await expect.poll(() => depthOf(b)).toMatch(/--folder-depth:\s*1/);

    // Undo the nest from the toast: B is a top-level folder again.
    await page.getByRole('button', { name: 'Undo', exact: true }).last().click();
    await expect.poll(() => depthOf(b)).toMatch(/--folder-depth:\s*0/);
  });

  for (const scheme of ['light', 'dark'] as const) {
    test(`shows the drop indicator while dragging (${scheme})`, async ({ page }, testInfo) => {
      await login(page);
      // Pick the theme pack the way a person does, in Settings, then come back to Mail.
      await page.getByRole('button', { name: 'Settings', exact: true }).click();
      const pack = page.getByRole('button', { name: scheme === 'light' ? 'Light' : 'Dark', exact: true }).first();
      await pack.click();
      await expect(page.locator('html')).toHaveAttribute('data-theme', scheme);
      await page.getByRole('tab', { name: /^Mail/ }).first().click();
      const seeded = await seedFolders(page);
      await page.reload();
      await expect(folder(page, seeded[2])).toBeVisible();
      const [a, b, c] = await order(page, seeded) as [string, string, string];

      // Hold the drag over the middle of A so the "inside" highlight is up.
      await row(page, a).scrollIntoViewIfNeeded();
      await row(page, c).scrollIntoViewIfNeeded();
      const source = (await row(page, c).boundingBox())!;
      const target = (await row(page, a).boundingBox())!;
      await page.mouse.move(source.x + 40, source.y + source.height / 2);
      await page.mouse.down();
      await page.mouse.move(source.x + 40, source.y + source.height / 2 - 4, { steps: 3 });
      await page.mouse.move(target.x + 40, target.y + target.height / 2, { steps: 8 });
      // Chromium sends dragover for the row under the pointer only on a move after it arrives.
      await page.mouse.move(target.x + 41, target.y + target.height / 2 + 1, { steps: 2 });
      await expect(row(page, a)).toHaveAttribute('data-drop', 'inside');
      await expect(row(page, c)).toHaveAttribute('data-dragging', 'true');
      const top = Math.min(source.y, target.y) - 70;
      const shot = await page.screenshot({ path: testInfo.outputPath(`folder-drag-${scheme}.png`), clip: { x: 0, y: Math.max(0, top), width: 260, height: Math.min(420, Math.abs(source.y - target.y) + source.height + 140) } });
      await testInfo.attach(`folder-drag-${scheme}`, { body: shot, contentType: 'image/png' });
      // Escape cancels: nothing moves.
      await page.keyboard.press('Escape');
      await page.mouse.up();
      await expect.poll(() => order(page, [a, b, c])).toEqual([a, b, c]);
    });
  }
});
