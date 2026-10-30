// t39 R3: a provisioned user's lifecycle and the dashboard session count, live.
// The admin provisions a user in the console; the user signs in (the dashboard's
// active webmail session count goes up); Block login revokes that session and
// refuses a new sign-in; Unblock lets the user back in; Delete local access
// (typed confirmation) leaves a deleted record and refuses sign-in again. Runs
// against the receipts stack (Greenmail without sign-in checks, so the new
// mailbox signs in with any password). No route mocks. t107: the Users table
// lists the new user (search, status, last sign-in) before and after its sign-in.
import { test, expect, type Page } from '../csrf.ts';
import { ADMIN } from './admin-path.ts';
import { adminLogin } from './admin-login.ts';

interface User { accountId: string; username: string; domain: string | null; status: string; loginAddress?: string | null }
interface Dashboard { counts: { activeSessions: number } }

async function userSignIn(page: Page, address: string): Promise<number> {
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill(address);
  await page.getByLabel('Password', { exact: true }).fill('lifecycle-pass');
  const response = page.waitForResponse((r) => r.request().method() === 'POST' && /\/(api\/)?(auth\/)?login$/.test(new URL(r.url()).pathname));
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  return (await response).status();
}
async function confirmStepUp(page: Page): Promise<void> {
  const step = page.getByRole('dialog', { name: 'Confirm it is you' });
  // The first sensitive action asks for the password; later ones reuse the five-minute proof.
  if (await step.waitFor({ state: 'visible', timeout: 5000 }).then(() => true, () => false)) {
    await step.getByLabel('Password', { exact: true }).fill('hunter2');
    await step.getByRole('button', { name: 'Confirm and continue' }).click();
  }
}

test('provisioned user: session counted, block revokes it, unblock restores, delete refuses sign-in', async ({ page, browser }, info) => {
  test.setTimeout(240_000);
  const username = `life${info.project.name.slice(0, 2)}${Date.now().toString(36)}`;
  const address = `${username}@example.org`;
  await adminLogin(page);
  const sessions = async (): Promise<number> => ((await (await page.request.get(`${ADMIN}/dashboard`)).json()) as Dashboard).counts.activeSessions;

  await page.goto(`${ADMIN}#/users/%2Bnew`);
  const form = page.getByRole('form', { name: 'Provision user', exact: true });
  await form.getByLabel('Username', { exact: true }).fill(username);
  await form.getByLabel('Domain', { exact: true }).fill('example.org');
  const provisioned = page.waitForResponse((r) => r.request().method() === 'POST' && new URL(r.url()).pathname.endsWith(`${ADMIN}/users`));
  await form.getByRole('button', { name: 'Provision user', exact: true }).click();
  expect((await provisioned).status()).toBeLessThan(300);
  const find = async (): Promise<User | undefined> => ((await (await page.request.get(`${ADMIN}/users`)).json()) as User[]).find((u) => u.username === username || u.loginAddress === address);
  await expect.poll(async () => (await find()) !== undefined).toBe(true);
  // t107: the new user is in the Users table, found by search, never signed in.
  await page.goto(`${ADMIN}#/users?q=${username}`);
  const listed = page.getByTestId('users-table').locator('tbody tr[data-row]');
  await expect(listed).toHaveCount(1);
  await expect(listed.locator('td[data-column="status"]')).toHaveText('Never signed in');
  await expect(listed.locator('td[data-column="lastSignin"]')).toHaveText('Never');

  const device = await browser.newContext({ baseURL: String(info.project.use.baseURL), locale: 'en-US', serviceWorkers: 'block' });
  const user = await device.newPage();
  try {
    const before = await sessions();
    expect(await userSignIn(user, address)).toBe(200);
    await expect(user.getByRole('button', { name: 'Compose', exact: true }).first()).toBeAttached();
    expect((await user.request.get('/api/me')).status()).toBe(200);
    await expect.poll(sessions).toBeGreaterThan(before);
    const counted = await sessions();
    await page.goto(`${ADMIN}#/dashboard`);
    await expect(page.getByText('Active webmail sessions', { exact: true }).first()).toBeVisible();

    // t107: after the sign-in, the table shows when it happened.
    await page.goto(`${ADMIN}#/users?q=${username}&sort=lastSignin&dir=desc`);
    await expect(listed.first().locator('td[data-column="lastSignin"] time')).toHaveText(/now|second|minute/u);
    const record = (await find())!;
    await page.goto(`${ADMIN}#/users/${encodeURIComponent(record.accountId)}`);
    const access = page.getByRole('heading', { name: 'Mailwoman access', exact: true }).locator('..');
    await page.getByRole('button', { name: `Block login for ${address}` }).click();
    let dialog = page.getByRole('dialog').or(page.getByRole('alertdialog')).filter({ hasText: 'Block Mailwoman login' });
    await dialog.getByRole('button', { name: 'Block login', exact: true }).click();
    await confirmStepUp(page);
    await expect.poll(async () => (await find())?.status).toBe('disabled');
    // The user's session is revoked and a new sign-in is refused.
    await expect.poll(async () => (await user.request.get('/api/me')).status()).toBe(401);
    await expect.poll(sessions).toBeLessThan(counted);
    expect(await userSignIn(user, address)).not.toBe(200);

    await page.getByRole('button', { name: `Unblock login for ${address}` }).click();
    dialog = page.getByRole('dialog').or(page.getByRole('alertdialog')).filter({ hasText: 'Unblock Mailwoman login' });
    await dialog.getByRole('button', { name: 'Unblock login', exact: true }).click();
    await confirmStepUp(page);
    await expect.poll(async () => (await find())?.status).toBe('active');
    expect(await userSignIn(user, address)).toBe(200);
    expect((await user.request.get('/api/me')).status()).toBe(200);

    await page.getByRole('button', { name: `Remove Mailwoman access for ${address}` }).click();
    dialog = page.getByRole('dialog').or(page.getByRole('alertdialog')).filter({ hasText: 'Delete local Mailwoman access' });
    const remove = dialog.getByRole('button', { name: 'Delete local access', exact: true });
    await expect(remove).toBeDisabled();
    await dialog.getByRole('textbox').fill(address);
    await remove.click();
    await confirmStepUp(page);
    await expect.poll(async () => (await find())?.status).toBe('deleted');
    await expect.poll(async () => (await user.request.get('/api/me')).status()).toBe(401);
    expect(await userSignIn(user, address)).not.toBe(200);
    await expect(access).toBeVisible();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true);
    await page.screenshot({ path: info.outputPath('user-lifecycle.png'), fullPage: true });
  } finally {
    await device.close();
  }
});
