// t88: search filter chips against a server built from the tree, over Greenmail,
// no route mocks (scripts/e2e-receipts-stack.mjs).
//
// Three messages arrive by SMTP, all carrying one token unique to the leg: two
// from alice<token> (subjects "Alpha" and "Beta"), one from bob<token>. Then:
//   - typing `from:alice<token> subject:alpha` and pressing the magnifier icon
//     shows a From chip and a Subject chip, and only the Alpha message;
//   - editing the Subject chip to "beta" rewrites the input and refilters;
//   - removing the Subject chip leaves `from:alice<token>` (both alice messages);
//   - the × icon clears the input and the chips and restores the Inbox;
//   - "Search messages from this sender" in the reader starts a from: search;
//   - Enter searches, Esc clears; at 390 px the chip row scrolls sideways and the
//     page does not; editing a chip does not move the list.
//
// Runs in chromium and firefox, each at 1440 and 390 px wide (the `phone`
// project is skipped: the widths are set here). MW_SEARCH_SHOTS: a directory for
// screenshots.
import type { Locator, Page } from '@playwright/test';
import { test, expect } from '../csrf.ts';
import { drawerButton, injectViaSmtp, messageRow } from '../helpers.ts';

const shots = process.env['MW_SEARCH_SHOTS'];

async function signIn(page: Page): Promise<void> {
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill('testuser');
  await page.getByLabel('Password', { exact: true }).fill('testpass');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Compose', exact: true }).first()).toBeVisible();
}

async function listSettled(page: Page): Promise<void> {
  await expect(page.getByRole('region', { name: 'Messages', exact: true })).toHaveAttribute('aria-busy', 'false');
}

const searchBox = (page: Page): Locator => page.getByRole('searchbox', { name: 'Search mail' });
const filterBar = (page: Page): Locator => page.getByRole('toolbar', { name: 'Search filters' });
/** The tooltip text (TooltipLayer moves `title` into `data-tooltip-title`). */
const tooltip = (el: Locator): Promise<string | null> => el.evaluate((node) => node.getAttribute('title') ?? node.getAttribute('data-tooltip-title'));
const chip = (page: Page, name: string): Locator => filterBar(page).getByRole('button', { name, exact: true });

for (const width of [1440, 390]) {
  test(`search filters: chips, two-way sync, refine from the reader, icon buttons at ${width}px`, async ({ page }, info) => {
    test.skip(info.project.name === 'phone', 'the widths are set by the test');
    const narrow = width < 761;
    await page.setViewportSize({ width, height: narrow ? 844 : 900 });
    const browser = info.project.name;
    const token = `t${Date.now()}${browser === 'firefox' ? 'f' : 'c'}${width}`;
    const alice = `alice${token}@example.org`;
    const bob = `bob${token}@example.org`;
    const alpha = `Alpha ${token}`;
    const beta = `Beta ${token}`;
    const third = `Gamma ${token}`;
    const shot = async (name: string): Promise<void> => {
      if (shots) await page.screenshot({ path: `${shots}/${browser}-${width}-${name}.png`, fullPage: false });
    };

    await injectViaSmtp({ from: `Alice <${alice}>`, subject: alpha, text: 'First message.' });
    await injectViaSmtp({ from: `Alice <${alice}>`, subject: beta, text: 'Second message.' });
    await injectViaSmtp({ from: `Bob <${bob}>`, subject: third, text: 'Third message.' });

    await signIn(page);
    await expect(async () => {
      await page.getByRole('button', { name: 'Refresh', exact: true }).first().click({ timeout: 2_000 }).catch(() => undefined);
      await listSettled(page);
      await expect(messageRow(page, third)).toBeVisible({ timeout: 3_000 });
      await expect(messageRow(page, alpha)).toBeVisible({ timeout: 1_000 });
      await expect(messageRow(page, beta)).toBeVisible({ timeout: 1_000 });
    }).toPass({ timeout: 90_000 });

    // The icon buttons: a magnifier with a name and a tooltip, no text label.
    const go = page.getByRole('button', { name: 'Search', exact: true });
    await expect(go.locator('svg[data-icon="search"]')).toHaveCount(1);
    await expect(go).toHaveText('');
    expect(await tooltip(go)).toBe('Search (Enter)');
    await expect(page.getByTestId('search-clear')).toHaveCount(0);
    await expect(filterBar(page)).toHaveCount(0);
    await shot('0-folder');

    // 1. Type operators, press the magnifier: chips appear, the list refilters.
    await searchBox(page).fill(`from:${alice} subject:alpha`);
    await go.click();
    await expect(chip(page, `From: ${alice}`)).toBeVisible();
    await expect(chip(page, 'Subject: alpha')).toBeVisible();
    await expect(chip(page, 'Folder: All folders')).toBeVisible();
    await listSettled(page);
    await expect(messageRow(page, alpha)).toBeVisible();
    await expect(messageRow(page, beta)).toHaveCount(0);
    await expect(messageRow(page, third)).toHaveCount(0);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1)).toBe(true);
    await shot('1-chips');

    // 2. Edit the Subject chip: the input text changes and the list refilters.
    const list = page.getByRole('region', { name: 'Messages', exact: true });
    const listTop = (await list.boundingBox())!.y;
    const barHeight = (await filterBar(page).boundingBox())!.height;
    await chip(page, 'Subject: alpha').click();
    const editor = page.getByRole('dialog', { name: 'Edit filter: Subject' });
    await expect(editor).toBeVisible();
    await shot('2-editor');
    await editor.getByRole('textbox', { name: 'Subject' }).fill('beta');
    await editor.getByRole('button', { name: 'Apply', exact: true }).click();
    await expect(editor).toHaveCount(0);
    await expect(searchBox(page)).toHaveValue(`from:${alice} subject:beta`);
    await expect(chip(page, 'Subject: beta')).toBeVisible();
    await listSettled(page);
    await expect(messageRow(page, beta)).toBeVisible();
    await expect(messageRow(page, alpha)).toHaveCount(0);
    // A chip edit does not move the list or resize the bar.
    expect(Math.abs((await list.boundingBox())!.y - listTop)).toBeLessThanOrEqual(0.5);
    expect(Math.abs((await filterBar(page).boundingBox())!.height - barHeight)).toBeLessThanOrEqual(0.5);
    await shot('3-edited');

    // 3. Remove the Subject chip: both of Alice's messages, not Bob's.
    await filterBar(page).getByRole('button', { name: 'Remove filter: Subject: beta', exact: true }).click();
    await expect(searchBox(page)).toHaveValue(`from:${alice}`);
    await expect(chip(page, 'Subject: beta')).toHaveCount(0);
    await listSettled(page);
    await expect(messageRow(page, alpha)).toBeVisible();
    await expect(messageRow(page, beta)).toBeVisible();
    await expect(messageRow(page, third)).toHaveCount(0);
    await shot('4-removed');

    // 4. The × icon clears the query and the chips and restores the Inbox.
    const clear = page.getByTestId('search-clear');
    await expect(clear.locator('svg[data-icon="close"]')).toHaveCount(1);
    await expect(clear).toHaveAccessibleName('Clear');
    expect(await tooltip(clear)).toBe('Clear search (Esc)');
    if (!narrow) {
      await clear.hover();
      await expect(page.getByRole('tooltip').filter({ hasText: 'Clear search (Esc)' })).toBeVisible();
      await shot('4b-clear-tooltip');
    }
    await clear.click();
    await expect(searchBox(page)).toHaveValue('');
    await expect(filterBar(page)).toHaveCount(0);
    await listSettled(page);
    await expect(messageRow(page, third)).toBeVisible();
    await expect(messageRow(page, alpha)).toBeVisible();
    await shot('5-cleared');

    // 5. From the reader: "Search messages from this sender".
    await messageRow(page, third).click();
    const related = page.getByTestId('reader-search-related').filter({ visible: true }).first();
    await expect(related).toBeVisible();
    await related.click();
    await shot('6-reader-menu');
    await page.getByRole('menuitem', { name: 'Search messages from this sender', exact: true }).click();
    await expect(searchBox(page)).toHaveValue(`from:"${bob}"`);
    await expect(searchBox(page)).toBeVisible();
    await expect(chip(page, `From: ${bob}`)).toBeVisible();
    await listSettled(page);
    await expect(messageRow(page, third)).toBeVisible();
    await expect(messageRow(page, alpha)).toHaveCount(0);
    await shot('7-from-sender');

    // 6. Typing operators updates the chips; Enter searches; Esc clears.
    await searchBox(page).fill(`from:${alice} has:attachment is:unread after:2020-01-01 smaller:5m -subject:"no such words"`);
    await expect(chip(page, 'Has attachment')).toBeVisible();
    await searchBox(page).press('Enter');
    await listSettled(page);
    await expect(chip(page, 'Unread')).toBeVisible();
    await expect(chip(page, 'Date: since 2020-01-01')).toBeVisible();
    await expect(chip(page, 'Size: under 5m')).toBeVisible();
    await expect(chip(page, 'Subject: not no such words')).toBeVisible();
    // Toggle Has attachment off: the input follows (canonical syntax).
    await chip(page, 'Has attachment').click();
    await expect(searchBox(page)).toHaveValue(`from:${alice} -has:attachment is:unread after:2020-01-01 smaller:5m -subject:"no such words"`);
    await listSettled(page);
    await expect(messageRow(page, alpha)).toBeVisible();
    // The chip row scrolls sideways inside its own box; the page never does.
    const overflow = await filterBar(page).evaluate((el) => ({ scroll: el.scrollWidth, client: el.clientWidth, overflowX: getComputedStyle(el).overflowX }));
    expect(overflow.overflowX).toBe('auto');
    if (narrow) expect(overflow.scroll).toBeGreaterThan(overflow.client);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1)).toBe(true);
    await shot('8-many-chips');
    if (narrow) {
      await filterBar(page).evaluate((el) => { el.scrollLeft = el.scrollWidth; });
      await shot('9-scrolled');
    }
    await searchBox(page).focus();
    await searchBox(page).press('Escape');
    await expect(searchBox(page)).toHaveValue('');
    await expect(filterBar(page)).toHaveCount(0);
    await listSettled(page);
    await expect(messageRow(page, third)).toBeVisible();
    await shot('10-esc-cleared');

    // The search shortcut still focuses the input.
    await page.locator('body').click({ position: { x: 1, y: 1 } }).catch(() => undefined);
    if (narrow && await drawerButton(page).isVisible()) await expect(drawerButton(page)).toBeVisible();
    await page.keyboard.press('/');
    await expect(searchBox(page)).toBeFocused();
  });
}
