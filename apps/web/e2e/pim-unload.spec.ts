import { test, expect, type Page } from '@playwright/test';
import { engineLogin, gotoModule, uid } from './pim-helpers.ts';

/**
 * A note or task save that starts just before the tab reloads or closes still
 * reaches the server (t32-h11, 26.20). Note and task saves are sent with fetch
 * `keepalive`, which the browser finishes after the page is gone; an ordinary
 * fetch is cancelled with the page.
 *
 * To make "the save is still out when the page goes" certain rather than a race
 * with a fast local server, each case adds network latency through the Chrome
 * DevTools Protocol just before the edit, then reloads or closes at once, and
 * checks from a fresh page what the server kept. Chromium only (the `pim`
 * project runs Desktop Chrome).
 */

const notes = (page: Page) => page.locator('[data-module="notes"]');
const tasks = (page: Page) => page.locator('[data-module="tasks"]');

/** Hold every request of `page` back by `ms` before it reaches the server. */
async function slowNetwork(page: Page, ms: number): Promise<void> {
  const cdp = await page.context().newCDPSession(page);
  await cdp.send('Network.enable');
  await cdp.send('Network.emulateNetworkConditions', {
    offline: false,
    latency: ms,
    downloadThroughput: -1,
    uploadThroughput: -1,
  });
}

/** Open the app in `page`, signing in only if the session cookie did not survive. */
async function openShell(page: Page): Promise<void> {
  await page.goto('/');
  const compose = page.getByRole('button', { name: 'Compose' });
  const signIn = page.getByRole('button', { name: 'Sign in', exact: true });
  await expect(compose.or(signIn)).toBeVisible();
  if (await signIn.isVisible().catch(() => false)) await engineLogin(page);
  await expect(compose).toBeVisible();
}

test.describe('saves started just before the page goes (engine mode)', () => {
  test('a note title typed right before a reload is kept', async ({ page, browserName }) => {
    test.skip(browserName !== 'chromium', 'uses the Chrome DevTools Protocol');
    await engineLogin(page);
    await gotoModule(page, 'notes');

    await notes(page).getByRole('button', { name: '+ New note' }).click();
    const titleInput = notes(page).getByLabel('Note editor').getByLabel('Note title');
    await expect(titleInput).toHaveValue('Untitled note');
    // The create has been answered (the note is open); now the title's save is
    // made to be still out when the page reloads.
    const title = `Typed before reload ${uid()}`;
    await slowNetwork(page, 1500);
    await titleInput.fill(title);
    await page.reload({ waitUntil: 'commit' });

    // A fresh page, at normal speed, asks the server.
    const fresh = await page.context().newPage();
    await page.close();
    await openShell(fresh);
    await expect(async () => {
      await gotoModule(fresh, 'calendar');
      await gotoModule(fresh, 'notes');
      await notes(fresh).getByLabel('Search notes').fill(title);
      await expect(notes(fresh).getByRole('option').filter({ hasText: title })).toBeVisible({ timeout: 2000 });
    }).toPass({ timeout: 20_000 });
  });

  test('a task renamed and saved right before the tab closes is kept', async ({ page, context, browserName }) => {
    test.skip(browserName !== 'chromium', 'uses the Chrome DevTools Protocol');
    await engineLogin(page);
    await gotoModule(page, 'tasks');

    const before = `Rename me ${uid()}`;
    const after = `Renamed before close ${uid()}`;
    await tasks(page).getByLabel('New task title').fill(before);
    await tasks(page).getByRole('button', { name: 'Add' }).click();
    await expect(tasks(page).getByRole('list', { name: 'Tasks' }).getByText(before)).toBeVisible();

    await tasks(page).getByRole('button', { name: `Edit ${before}` }).click();
    const form = tasks(page).getByRole('form', { name: `Edit ${before}` });
    await form.getByLabel('Title').fill(after);
    await slowNetwork(page, 1500);
    await form.getByRole('button', { name: 'Save' }).click();
    await page.close();

    const fresh = await context.newPage();
    await openShell(fresh);
    await expect(async () => {
      await gotoModule(fresh, 'notes');
      await gotoModule(fresh, 'tasks');
      await expect(tasks(fresh).getByRole('list', { name: 'Tasks' }).getByText(after)).toBeVisible({ timeout: 2000 });
    }).toPass({ timeout: 20_000 });
  });
});
