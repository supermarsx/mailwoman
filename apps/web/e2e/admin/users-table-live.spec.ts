// t107: the admin Users table, live against a server built from the tree (engine
// mode over Greenmail with sign-in checks off, so any password signs a new
// mailbox in). No route mocks. Seeds its own users under a unique tag, so the
// search narrows the table to them: two provisioned users who never signed in,
// two who signed in.
//
//   node apps/web/e2e/receipts/linked-live-fixture.mjs   (MW_FIXTURE_ADMIN_PASSWORD=hunter2)
//   PLAYWRIGHT_ADMIN_BASE_URL=... MW_FIXTURE_ADMIN_PASSWORD=hunter2 \
//     npx playwright test -c apps/web/e2e/admin/playwright.admin.config.ts users-table-live
//
// Checks: the Last sign-in column shows relative times and "Never" and sorts;
// search and filter chips narrow the list and live in the URL (survive a
// reload); bulk block and unblock reach the server; the column toggle is
// remembered after a reload; at 390 px the list is cards with no page overflow.
// MW_SCREENSHOT_DIR keeps a copy of each screenshot.
import { test, expect, type Page } from '../csrf.ts';
import type { APIRequestContext, TestInfo } from '@playwright/test';
import { ADMIN } from './admin-path.ts';
import { adminLogin } from './admin-login.ts';

interface Row { accountId: string; loginAddress: string; state: string; status: string; lastSigninAt: number | null; lastSeenAt: number | null }
interface Page_ { rows: Row[]; total: number }

const ADMIN_PASSWORD = process.env['MW_FIXTURE_ADMIN_PASSWORD'] ?? 'hunter2';

async function shot(page: Page, info: TestInfo, name: string): Promise<void> {
  await page.screenshot({ path: info.outputPath(name), fullPage: true });
  const dir = process.env['MW_SCREENSHOT_DIR'];
  if (dir) await page.screenshot({ path: `${dir}/${info.project.name}-${name}`, fullPage: true });
}

async function confirmStepUp(page: Page): Promise<void> {
  const step = page.getByRole('dialog', { name: 'Confirm it is you' });
  if (await step.waitFor({ state: 'visible', timeout: 5000 }).then(() => true, () => false)) {
    await step.getByLabel('Password', { exact: true }).fill(ADMIN_PASSWORD);
    await step.getByRole('button', { name: 'Confirm and continue' }).click();
  }
}

/** Provision `never` users (they never sign in) and sign `signed` users in. */
async function seed(page: Page, info: TestInfo, newContext: () => Promise<APIRequestContext>, tag: string, never: string[], signed: string[]): Promise<void> {
  for (const name of never) {
    const r = await page.request.post(`${ADMIN}/users`, { data: { domain: 'example.org', username: `${tag}${name}`, quota: { bytesLimit: 0, msgLimit: 0 } } });
    expect(r.status(), await r.text()).toBeLessThan(300);
  }
  for (const name of signed) {
    const user = await newContext();
    try {
      const r = await user.post('/api/login', { data: { username: `${tag}${name}@example.org`, password: 'table-pass' } });
      expect(r.status(), await r.text()).toBe(200);
      if(name==='s1'){
        // The ledger uses whole seconds. Foreground activity must advance seen,
        // while retaining the successful authentication timestamp.
        await new Promise(resolve=>setTimeout(resolve,1100));
        const activity=await user.post('/api/account/session/activity');
        expect(activity.ok(),await activity.text()).toBe(true);
      }
    } finally {
      await user.dispose();
    }
  }
  const all = [...never, ...signed].length;
  await expect.poll(async () => ((await (await page.request.get(`${ADMIN}/users/table?q=${tag}`)).json()) as Page_).total, { timeout: 20_000 }).toBe(all);
  void info;
}

const table = (page: Page) => page.getByTestId('users-table');
const bodyRows = (page: Page) => table(page).locator('tbody tr[data-row]');
const noPageOverflow = (page: Page) => page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1);

test('users table: last sign-in column shows and sorts, search and filter chips live in the URL, bulk block/unblock, column toggle persists', async ({ page, playwright }, info) => {
  test.skip(info.project.name === 'phone', 'the phone project runs the @phone card test');
  test.setTimeout(300_000);
  await page.setViewportSize({ width: 1440, height: 900 });
  const tag = `t107${info.project.name.slice(0, 2)}${Date.now().toString(36)}`;
  await adminLogin(page);
  await seed(page, info, () => playwright.request.newContext({ baseURL: String(info.project.use.baseURL) }), tag, ['n1', 'n2'], ['s1', 's2']);

  await page.goto(`${ADMIN}#/users?q=${tag}`);
  await expect(bodyRows(page)).toHaveCount(4);
  await expect(page.getByTestId('users-count')).toContainText(/^4 of \d+ users match$/u);
  await expect(table(page).locator('mark').first()).toHaveText(tag);
  const lastHeader = page.getByRole('columnheader', { name: /Last sign-in/u });
  await expect(lastHeader).toBeVisible();
  const seenHeader = page.getByRole('columnheader', { name: /Last seen/u });
  await expect(seenHeader).toBeVisible();
  await expect(seenHeader).toHaveAttribute('aria-sort','descending');
  await expect(lastHeader).toHaveAttribute('aria-sort','none');
  // Never-signed-in users read "Never" and "Never signed in"; the others a relative time with the exact one in the tooltip.
  const n1 = bodyRows(page).filter({ hasText: `${tag}n1@example.org` });
  await expect(n1.locator('td[data-column="lastSignin"]')).toHaveText('Never');
  await expect(n1.locator('td[data-column="lastSeen"]')).toHaveText('No activity recorded');
  await expect(n1.locator('td[data-column="status"]')).toHaveText('Never signed in');
  const s1 = bodyRows(page).filter({ hasText: `${tag}s1@example.org` });
  const when = s1.locator('td[data-column="lastSignin"] time');
  await expect(when).toHaveText(/now|second|minute/u);
  // The tooltip layer moves `title` into `data-tooltip-title`; the exact time is also in the name.
  expect((await when.getAttribute('title')) ?? (await when.getAttribute('data-tooltip-title')) ?? '').toMatch(/\d{4}/u);
  await expect(when).toHaveAttribute('aria-label', /\d{4}/u);
  const seen=s1.locator('td[data-column="lastSeen"] time');
  await expect(seen).toHaveAttribute('aria-label',/^Last seen /u);
  const apiRow=((await (await page.request.get(`${ADMIN}/users/table?q=${tag}s1`)).json()) as Page_).rows[0]!;
  expect(apiRow.lastSeenAt).toBeGreaterThan(apiRow.lastSigninAt!);
  expect(await seen.getAttribute('datetime')).toBe(new Date(apiRow.lastSeenAt!*1000).toISOString());
  expect(await when.getAttribute('datetime')).toBe(new Date(apiRow.lastSigninAt!*1000).toISOString());
  await shot(page, info, 'users-table-1440.png');
  await seenHeader.getByRole('button').click();
  await expect(seenHeader).toHaveAttribute('aria-sort','ascending');
  await expect(bodyRows(page).nth(0).locator('td[data-column="lastSeen"]')).toHaveText('No activity recorded');
  await page.reload();
  await expect(seenHeader).toHaveAttribute('aria-sort','ascending');

  // Sort by last sign-in: newest first, then oldest (Never) first.
  await lastHeader.getByRole('button').click();
  await expect(lastHeader).toHaveAttribute('aria-sort', 'descending');
  await expect(page).toHaveURL(/sort=lastSignin&dir=desc/u);
  await expect(bodyRows(page).nth(0).locator('td[data-column="lastSignin"] time')).toBeVisible();
  await expect(bodyRows(page).nth(3).locator('td[data-column="lastSignin"]')).toHaveText('Never');
  await lastHeader.getByRole('button').click();
  await expect(lastHeader).toHaveAttribute('aria-sort', 'ascending');
  await expect(bodyRows(page).nth(0).locator('td[data-column="lastSignin"]')).toHaveText('Never');

  // Search narrows as it is typed.
  const search = page.getByRole('searchbox', { name: 'Search users' });
  await search.fill(`${tag}s`);
  await expect(bodyRows(page)).toHaveCount(2);
  await search.fill(tag);
  await expect(bodyRows(page)).toHaveCount(4);

  // A filter chip: last sign-in = never. It is in the URL and survives a reload.
  await page.getByTestId('users-add-filter').click();
  await page.getByRole('menuitemcheckbox', { name: 'Last sign-in' }).click();
  const editor = page.getByTestId('users-filter-editor');
  await editor.getByLabel('Never', { exact: true }).check();
  await editor.getByRole('button', { name: 'Apply' }).click();
  await expect(bodyRows(page)).toHaveCount(2);
  await expect(page).toHaveURL(/signin=never/u);
  await page.reload();
  await expect(page.getByRole('button', { name: 'Last sign-in: Never', exact: true })).toBeVisible();
  await expect(bodyRows(page)).toHaveCount(2);
  await expect(bodyRows(page).filter({ hasText: `${tag}s1@` })).toHaveCount(0);
  await page.getByRole('button', { name: 'Remove filter Last sign-in: Never' }).click();
  await expect(bodyRows(page)).toHaveCount(4);
  // Status: never signed in (provisioned) — the same two.
  await page.getByTestId('users-add-filter').click();
  await page.getByRole('menuitemcheckbox', { name: 'Status' }).click();
  await editor.getByLabel('Never signed in').check();
  await editor.getByRole('button', { name: 'Apply' }).click();
  await expect(bodyRows(page)).toHaveCount(2);
  await page.getByRole('button', { name: 'Clear all' }).click();
  await expect(search).toHaveValue('');
  await search.fill(tag);
  await expect(bodyRows(page)).toHaveCount(4);

  // Bulk block, then unblock, the two who signed in.
  const api = async (): Promise<Row[]> => ((await (await page.request.get(`${ADMIN}/users/table?q=${tag}s`)).json()) as Page_).rows;
  await page.getByRole('checkbox', { name: `Select ${tag}s1@example.org` }).check();
  await page.getByRole('checkbox', { name: `Select ${tag}s2@example.org` }).check();
  const bulk = page.getByTestId('users-bulk');
  await expect(bulk).toContainText('2 selected');
  await bulk.getByRole('button', { name: 'Block login', exact: true }).click();
  let dialog = page.getByRole('alertdialog').filter({ hasText: 'Block login for 2 users?' });
  await dialog.getByRole('button', { name: 'Block login', exact: true }).click();
  await confirmStepUp(page);
  await expect.poll(async () => (await api()).map((r) => r.status).sort().join(), { timeout: 30_000 }).toBe('disabled,disabled');
  await expect(bodyRows(page).filter({ hasText: `${tag}s1@` }).locator('td[data-column="status"]')).toHaveText('Blocked');
  await page.getByRole('checkbox', { name: `Select ${tag}s1@example.org` }).check();
  await page.getByRole('checkbox', { name: `Select ${tag}s2@example.org` }).check();
  await bulk.getByRole('button', { name: 'Unblock login', exact: true }).click();
  dialog = page.getByRole('alertdialog').filter({ hasText: 'Unblock login for 2 users?' });
  await dialog.getByRole('button', { name: 'Unblock login', exact: true }).click();
  await confirmStepUp(page);
  await expect.poll(async () => (await api()).map((r) => r.status).sort().join(), { timeout: 30_000 }).toBe('active,active');
  await expect(bodyRows(page).filter({ hasText: `${tag}s2@` }).locator('td[data-column="status"]')).toHaveText('Active');

  // Column toggle, remembered for this admin after a reload.
  await page.getByRole('button', { name: 'Columns', exact: true }).click();
  await page.getByRole('group', { name: 'Columns' }).getByLabel('Domain').uncheck();
  await expect(page.getByRole('columnheader', { name: /^Domain/u })).toHaveCount(0);
  await expect.poll(async () => JSON.stringify((await (await page.request.get(`${ADMIN}/me/preferences`)).json())['usersTable'] ?? null)).toContain('"domain"');
  await page.reload();
  await expect(bodyRows(page)).toHaveCount(4);
  await expect(page.getByRole('columnheader', { name: /^Domain/u })).toHaveCount(0);
  await page.getByRole('button', { name: 'Columns', exact: true }).click();
  await page.getByRole('group', { name: 'Columns' }).getByLabel('Domain').check();
  await expect(page.getByRole('columnheader', { name: /^Domain/u })).toHaveCount(1);
  await page.keyboard.press('Escape');

  // Keyboard: rows are one Tab stop; arrows move, Space selects.
  await bodyRows(page).nth(0).focus();
  await page.keyboard.press('ArrowDown');
  await expect(bodyRows(page).nth(1)).toBeFocused();
  await page.keyboard.press('Space');
  await expect(bulk).toContainText('1 selected');
  expect(await noPageOverflow(page)).toBe(true);
  await shot(page, info, 'users-table-1440-selected.png');
});

test('users table at 390 px: stacked cards with name, status, last sign-in and actions; no page overflow @phone', async ({ page, playwright }, info) => {
  test.setTimeout(180_000);
  await page.setViewportSize({ width: 390, height: 844 });
  const tag = `t107p${info.project.name.slice(0, 2)}${Date.now().toString(36)}`;
  await adminLogin(page);
  await seed(page, info, () => playwright.request.newContext({ baseURL: String(info.project.use.baseURL) }), tag, ['n1'], ['s1']);
  await page.goto(`${ADMIN}#/users?q=${tag}`);
  const cards = page.getByTestId('users-cards').locator('li');
  await expect(cards).toHaveCount(2);
  await expect(page.getByTestId('users-table')).toHaveCount(0);
  const never = cards.filter({ hasText: `${tag}n1@example.org` });
  await expect(never).toContainText('Never signed in');
  await expect(never).toContainText('Never');
  await expect(never).toContainText('Last seen: No activity recorded');
  await expect(never).toContainText('Last sign-in: Never');
  const signed = cards.filter({ hasText: `${tag}s1@example.org` });
  await expect(signed.locator('time')).toHaveCount(2);
  await expect(signed.locator('time').nth(0)).toHaveAttribute('aria-label',/^Last seen /u);
  await expect(signed.locator('time').nth(0)).toHaveText(/now|second|minute/u);
  await expect(signed.locator('time').nth(1)).toHaveText(/now|second|minute/u);
  await expect(signed.getByRole('link', { name: `Configure ${tag}s1@example.org` })).toBeVisible();
  await expect(signed.getByRole('button', { name: `More actions for ${tag}s1@example.org` })).toBeVisible();
  await expect(page.getByTestId('users-filter-bar')).toBeVisible();
  expect(await noPageOverflow(page)).toBe(true);
  await shot(page, info, 'users-cards-390.png');
});
