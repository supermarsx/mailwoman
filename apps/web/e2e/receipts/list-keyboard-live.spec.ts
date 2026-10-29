// t44: arrow keys in the message list against a live server, no route mocks.
// Opening a message marks it read after a short delay, and that update used to
// remount every row: focus fell to <body> and no further arrow key reached the
// list. This drives a real open + mark-read and keeps pressing keys after it.
//   - Up/Down/Home/End/PageUp/PageDown move focus row by row, the row stays
//     visible and is the one tab stop (tabindex 0);
//   - a click opens a message, and arrows still work after it has been marked read;
//   - Enter opens the focused row;
//   - Shift+Down extends a selection;
//   - typing in the search box never moves the list;
//   - t51: opening a message (click, Enter, j) leaves focus on the list row in
//     reading panes right/bottom and in folder tabs (Tabs, Side by side); where
//     the reader covers the list (Off, phone) focus goes to its subject.
// Runs in the receipts config (chromium, firefox, phone) against
// MW_RECEIPTS_BASE_URL. The mailbox needs at least 30 messages
// (MW_LIST_KEYS_USER / MW_LIST_KEYS_PASS, default testuser / testpass).
import { test, expect, type Page } from '../csrf.ts';

const USER = { name: process.env['MW_LIST_KEYS_USER'] ?? 'testuser', pass: process.env['MW_LIST_KEYS_PASS'] ?? 'testpass' };
const touch = (page: Page): boolean => (page.viewportSize()?.width ?? 1280) < 600;
const rows = (page: Page) => page.locator('.list__scroll .list__row');

async function signIn(page: Page): Promise<void> {
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill(USER.name);
  await page.getByLabel('Password', { exact: true }).fill(USER.pass);
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(rows(page).nth(5)).toBeAttached({ timeout: 60000 });
  await page.waitForLoadState('networkidle');
}

/** The focused row's position in the list (data-index), or null when focus left the list. */
async function focusedIndex(page: Page): Promise<number | null> {
  return page.evaluate(() => {
    const el = document.activeElement;
    return el instanceof HTMLElement && el.classList.contains('list__row') ? Number(el.dataset['index']) : null;
  });
}
async function expectFocused(page: Page, index: number): Promise<void> {
  await expect.poll(() => focusedIndex(page)).toBe(index);
  const row = page.locator(`.list__scroll .list__row[data-index="${index}"]`);
  await expect(row).toHaveAttribute('tabindex', '0');
  await expect(row).toBeInViewport();
}
async function backToList(page: Page, subject: string): Promise<void> {
  // Phone: opening a message replaces the list with the reader.
  if (!touch(page)) return;
  await page.getByRole('button', { name: `Close ${subject}`, exact: true }).click();
  await expect(rows(page).first()).toBeVisible();
}

test.describe('message list keyboard', () => {
  // Desktop projects at 1440 wide; the phone project keeps its 390.
  test.beforeEach(async ({ page }, info) => {
    if (info.project.name !== 'phone') await page.setViewportSize({ width: 1440, height: 900 });
  });

  test('arrows keep working after an opened message is marked read', async ({ page }) => {
    await signIn(page);
    // The first unread row among the first few (each run reads one).
    const unread = page.locator('.list__scroll .list__row--unread').first();
    await expect(unread, 'the mailbox needs unread messages near the top').toBeAttached();
    const at = Number(await unread.getAttribute('data-index'));
    expect(at).toBeLessThan(20);
    const opened = page.locator(`.list__scroll .list__row[data-index="${at}"]`);
    await opened.scrollIntoViewIfNeeded();
    const subject = (await opened.locator('.list__subject').textContent()) ?? '';
    await opened.click();
    // Wait for the delayed mark-read to land in the list (the update that remounted every row).
    await expect(opened).not.toHaveClass(/list__row--unread/, { timeout: 15000 });
    await page.waitForTimeout(500);
    await backToList(page, subject);
    if (touch(page)) await opened.focus();
    await expectFocused(page, at);
    await page.keyboard.press('ArrowDown');
    await expectFocused(page, at + 1);
    await page.keyboard.press('ArrowDown');
    await expectFocused(page, at + 2);
    await page.keyboard.press('ArrowUp');
    await expectFocused(page, at + 1);
  });

  test('Up/Down, Home/End, PageUp/PageDown move a visible focus; Enter opens', async ({ page }) => {
    await signIn(page);
    await rows(page).nth(0).focus();
    await expectFocused(page, 0);
    await page.keyboard.press('ArrowUp');
    await expectFocused(page, 0);
    for (let i = 1; i <= 4; i += 1) {
      await page.keyboard.press('ArrowDown');
      await expectFocused(page, i);
    }
    await page.keyboard.press('PageDown');
    const afterPage = (await focusedIndex(page)) ?? -1;
    expect(afterPage).toBeGreaterThan(5);
    await expectFocused(page, afterPage);
    await page.keyboard.press('PageUp');
    await expectFocused(page, 4);
    await page.keyboard.press('End');
    const last = (await focusedIndex(page)) ?? -1;
    expect(last).toBeGreaterThanOrEqual(29);
    await expectFocused(page, last);
    await page.keyboard.press('Home');
    await expectFocused(page, 0);
    // A visible focus indicator on the focused row.
    const outline = await page.evaluate(() => {
      const s = getComputedStyle(document.activeElement as Element);
      return { style: s.outlineStyle, width: parseFloat(s.outlineWidth), shadow: s.boxShadow };
    });
    expect(outline.style !== 'none' && outline.width > 0 || outline.shadow !== 'none').toBe(true);

    await page.keyboard.press('ArrowDown');
    await expectFocused(page, 1);
    const subject = (await rows(page).nth(1).locator('.list__subject').textContent()) ?? '';
    await page.keyboard.press('Enter');
    await expect(page.locator('.list__row--active[data-index="1"]')).toBeAttached();
    await expect(page.getByRole('heading', { name: subject }).first()).toBeVisible();
  });

  test('Shift+Down extends the selection; typing in search does not move the list', async ({ page }) => {
    await signIn(page);
    await rows(page).nth(0).focus();
    await page.keyboard.press('Shift+ArrowDown');
    await page.keyboard.press('Shift+ArrowDown');
    await expectFocused(page, 2);
    await expect(page.locator('.list__row--selected')).toHaveCount(3);
    await page.keyboard.press('Escape');
    await expect(page.locator('.list__row--selected')).toHaveCount(0);

    if (touch(page)) return; // no search box in the phone top bar without opening it
    const search = page.getByRole('searchbox').first();
    if (!(await search.isVisible())) return;
    await search.click();
    await search.press('ArrowDown');
    await search.press('End');
    await expect(search).toBeFocused();
    expect(await focusedIndex(page)).toBeNull();
  });

  // t51: opening a message never takes focus off the row that opened it while
  // that list is on screen; the message shows in its tab and a polite status
  // names it. Where the reader covers the list (phone, reading pane Off) focus
  // goes to the reader's subject rather than falling to <body>.
  const activeTab = (page: Page) => page.locator('[role=tab][aria-selected="true"]');
  const focusedFolderRow = (page: Page) => page.evaluate(() => (document.activeElement as HTMLElement | null)?.dataset['folderMessage'] ?? null);
  async function pane(page: Page, mode: 'right' | 'bottom' | 'off'): Promise<void> {
    await page.getByTestId(`reading-pane-${mode}`).click();
    await expect(page.getByTestId(`reading-pane-${mode}`)).toHaveAttribute('aria-pressed', 'true');
  }
  async function subjectAt(page: Page, index: number): Promise<string> {
    return ((await page.locator(`.list__scroll .list__row[data-index="${index}"] .list__subject`).textContent()) ?? '').trim();
  }

  for (const mode of ['right', 'bottom'] as const) {
    test(`reading pane ${mode}: click, Enter and j open messages while focus stays in the list`, async ({ page }) => {
      test.skip(touch(page), 'phone: the reader replaces the list (covered below)');
      await signIn(page);
      await pane(page, mode);
      try {
        const first = await subjectAt(page, 2);
        await rows(page).nth(2).click();
        await expect(page.getByRole('heading', { name: first, exact: true }).first()).toBeVisible();
        await expect(activeTab(page)).toHaveText(first);
        await page.waitForTimeout(1500); // past the delayed mark-read
        await expectFocused(page, 2);
        await expect(page.getByTestId('message-opened-status')).toHaveText(`Opened: ${first}`);

        await page.keyboard.press('ArrowDown');
        await expectFocused(page, 3);
        const second = await subjectAt(page, 3);
        await page.keyboard.press('Enter');
        await expect(page.getByRole('heading', { name: second, exact: true }).first()).toBeVisible();
        await expect(activeTab(page)).toHaveText(second);
        await page.waitForTimeout(1500);
        await expectFocused(page, 3);

        const third = await subjectAt(page, 4);
        await page.keyboard.press('j');
        await expect(page.getByRole('heading', { name: third, exact: true }).first()).toBeVisible();
        await page.waitForTimeout(1500);
        await expectFocused(page, 4);
        await page.keyboard.press('ArrowDown');
        await expectFocused(page, 5);
      } finally { await pane(page, 'right'); }
    });
  }

  test('reading pane off (and phone): the reader covers the list, so focus goes to its subject, not <body>', async ({ page }) => {
    await signIn(page);
    if (!touch(page)) await pane(page, 'off');
    try {
      const subject = await subjectAt(page, 1);
      await rows(page).nth(1).click();
      const heading = page.getByRole('heading', { name: subject, exact: true }).first();
      await expect(heading).toBeVisible();
      await expect(heading).toBeFocused();
      await page.keyboard.press('j');
      const next = await subjectAt(page, 2);
      const nextHeading = page.getByRole('heading', { name: next, exact: true }).first();
      await expect(nextHeading).toBeVisible();
      await expect.poll(() => page.evaluate(() => document.activeElement?.tagName)).not.toBe('BODY');
    } finally {
      if (!touch(page)) { await page.getByRole('tab', { name: 'Mail', exact: true }).click(); await pane(page, 'right'); }
    }
  });

  for (const layout of ['Tabs', 'Side by side'] as const) {
    test(`folder tab, ${layout} layout: the folder list stays on screen and its row keeps focus`, async ({ page }) => {
      test.skip(touch(page), 'phone: one pane at a time');
      await signIn(page);
      await pane(page, 'right'); // the folder companion follows the reading pane (Off: the reader covers it)
      await page.getByRole('button', { name: layout, exact: true }).click();
      try {
        await page.locator('#shell-nav .sidebar__box').first().click({ button: 'right' });
        await page.getByRole('menuitem', { name: 'Open in new tab', exact: true }).click();
        const folderRows = page.locator('.workspace-pane [data-folder-message]');
        await expect(folderRows.nth(3)).toBeVisible();
        const opened = folderRows.nth(1);
        const id = await opened.getAttribute('data-folder-message');
        const subject = ((await opened.locator('strong').textContent()) ?? '').trim();
        await opened.click();
        await expect(page.getByRole('heading', { name: subject, exact: true }).first()).toBeVisible();
        await expect(activeTab(page)).toHaveText(subject);
        await page.waitForTimeout(1500);
        await expect(opened).toBeVisible();
        expect(await focusedFolderRow(page)).toBe(id);
        // Arrow to the next folder row and open it with Enter: still in the folder list.
        await page.keyboard.press('ArrowDown');
        const nextId = await folderRows.nth(2).getAttribute('data-folder-message');
        expect(await focusedFolderRow(page)).toBe(nextId);
        const nextSubject = ((await folderRows.nth(2).locator('strong').textContent()) ?? '').trim();
        await page.keyboard.press('Enter');
        await expect(page.getByRole('heading', { name: nextSubject, exact: true }).first()).toBeVisible();
        await page.waitForTimeout(1500);
        expect(await focusedFolderRow(page)).toBe(nextId);
      } finally { await page.getByRole('button', { name: 'Tabs', exact: true }).click(); }
    });
  }
});
