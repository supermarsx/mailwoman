// t39 N4: two thin browser proofs against a live server.
//   - Settings search: typing filters the section list by name and by each
//     section's search terms; a query that matches nothing leaves no section;
//     clearing it brings every section back; a match opens its section.
//   - The calendar's mini month (Date navigator): next/previous month move it,
//     and choosing a day moves the main calendar to that day.
import { test, expect, type Page } from '../csrf.ts';

const touch = (page: Page): boolean => (page.viewportSize()?.width ?? 1280) < 600;
async function signIn(page: Page): Promise<void> {
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill('testuser');
  await page.getByLabel('Password', { exact: true }).fill('testpass');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.locator('.list__row').first()).toBeVisible();
}
async function openNav(page: Page): Promise<void> {
  if (!touch(page)) return;
  const menu = page.locator('header.shell__bar button[aria-controls="shell-nav"]').first();
  if ((await menu.getAttribute('aria-expanded')) !== 'true') await menu.click();
}

test('settings search filters sections by name and terms, and a match opens', async ({ page }) => {
  page.setDefaultTimeout(15000);
  await signIn(page);
  await openNav(page);
  await page.locator('.sidebar, #shell-nav').getByRole('button', { name: 'Settings', exact: true }).first().click();
  const nav = page.getByRole('navigation', { name: 'Settings sections' });
  const sections = nav.getByRole('button');
  await expect(sections.first()).toBeVisible();
  const all = await sections.count();
  expect(all).toBeGreaterThan(5);
  const search = page.getByRole('searchbox', { name: 'Search settings' });

  await search.fill('signature');
  await expect(nav.getByRole('button', { name: 'Signatures', exact: true })).toBeVisible();
  await expect.poll(() => sections.count()).toBeLessThan(all);
  // A search term, not a section name: "dark mode" belongs to Message display.
  await search.fill('dark mode');
  await expect(nav.getByRole('button', { name: 'Message display', exact: true })).toBeVisible();
  await expect.poll(() => sections.count()).toBeLessThan(all);
  await search.fill('qqqzzz-no-such-setting');
  await expect(sections).toHaveCount(0);
  await search.fill('');
  await expect(sections).toHaveCount(all);

  await search.fill('signature');
  await nav.getByRole('button', { name: 'Signatures', exact: true }).click();
  await expect(page.getByRole('button', { name: 'New signature', exact: true })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true);
  await page.screenshot({ path: test.info().outputPath('settings-search.png'), fullPage: true });
});

test('the mini calendar moves by month and a chosen day moves the main calendar', async ({ page }) => {
  page.setDefaultTimeout(15000);
  await signIn(page);
  await openNav(page);
  await page.locator('.sidebar, #shell-nav').getByRole('button', { name: 'Calendar', exact: true }).first().click();
  const mini = page.getByRole('region', { name: 'Date navigator', exact: true });
  await expect(mini).toBeVisible();
  const month = (d: Date): string => d.toLocaleDateString('en-US', { month: 'long', year: 'numeric' });
  const now = new Date();
  const next = new Date(now.getFullYear(), now.getMonth() + 1, 1);
  await expect(mini.getByRole('table')).toHaveAccessibleName(month(now));
  await mini.getByRole('button', { name: 'Next month', exact: true }).click();
  await expect(mini.getByRole('table')).toHaveAccessibleName(month(next));
  await mini.getByRole('button', { name: 'Previous month', exact: true }).click();
  await expect(mini.getByRole('table')).toHaveAccessibleName(month(now));
  await mini.getByRole('button', { name: 'Next month', exact: true }).click();

  // The 15th of next month, by its full-date label.
  const target = new Date(next.getFullYear(), next.getMonth(), 15);
  const day = mini.locator('td button:not([data-outside])').filter({ hasText: /^15$/ });
  await day.click();
  await expect(day).toHaveAttribute('aria-pressed', 'true');
  // The main view follows: its title names the chosen month.
  await expect(page.locator('main, [role=main], .calendar').getByText(new RegExp(target.toLocaleDateString('en-US', { month: 'long' }))).first()).toBeVisible();
  await mini.getByRole('button', { name: 'Today', exact: true }).click();
  await expect(mini.getByRole('table')).toHaveAccessibleName(month(now));
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true);
  await page.screenshot({ path: test.info().outputPath('mini-calendar.png'), fullPage: true });
});
