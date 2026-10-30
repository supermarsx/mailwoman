import { test, expect, type APIRequestContext } from '../csrf.ts';
import { ADMIN } from '../admin/admin-path.ts';

const KEY = 'login.showAccountHelpNotice';
async function policy(api: APIRequestContext, value: unknown): Promise<void> {
  const current = await api.get(`${ADMIN}/settings/${KEY}`);
  expect(current.ok()).toBe(true);
  const written = await api.put(`${ADMIN}/settings/${KEY}`, {
    headers: { 'if-match': current.headers()['etag']! }, data: { globalValue: value },
  });
  expect(written.ok(), await written.text()).toBe(true);
}

// Real refused sign-in: Greenmail accepts every password, so use an explicitly
// disabled test account. Rust t27 separately compares wrong-password/disabled bodies.
test('account-help notice is hidden by default and enabled by admin policy', async ({ page, playwright }, info) => {
  const api = await playwright.request.newContext({ baseURL: String(info.project.use.baseURL) });
  const login = await api.post(`${ADMIN}/login`, { data: { username: 'root', password: 'hunter2' } });
  expect(login.ok(), await login.text()).toBe(true);
  expect((await api.post(`${ADMIN}/step-up`, { data: { password: 'hunter2' } })).ok()).toBe(true);
  const previous = await api.get(`${ADMIN}/settings/${KEY}`);
  expect(previous.ok()).toBe(true);
  const previousValue = (await previous.json()).globalValue ?? null;
  const username = `help${info.project.name}${Date.now().toString(36)}`;
  const address = `${username}@example.org`;
  try {
    const created = await api.post(`${ADMIN}/users`, { data: {
      domain: 'example.org', username, quota: { bytesLimit: 0, msgLimit: 0 },
    } });
    expect(created.ok(), await created.text()).toBe(true);
    const users = await (await api.get(`${ADMIN}/users`)).json() as { accountId: string; username: string; loginAddress?: string }[];
    const user = users.find(item => item.username === username || item.loginAddress === address);
    expect(user).toBeDefined();
    const disabled = await api.put(`${ADMIN}/users/${encodeURIComponent(user!.accountId)}/flags`, { data: {
      zeroAccess: false, forcePasswordChange: false, remoteCacheWipe: false, disabled: true,
    } });
    expect(disabled.ok(), await disabled.text()).toBe(true);
    for (const enabled of [null, true, false]) {
      await policy(api, enabled);
      await page.goto('/');
      await expect.poll(async () => (await (await page.request.get('/api/public-config')).json()).showAccountHelpNotice).toBe(enabled === true);
      await page.getByLabel('Email or username', { exact: true }).fill(address);
      await page.getByLabel('Password', { exact: true }).fill('testpass');
      const response = page.waitForResponse(r => r.url().endsWith('/api/login') && r.request().method() === 'POST');
      await page.getByRole('button', { name: 'Sign in', exact: true }).click();
      const refusal = await response;
      expect(refusal.status()).toBe(401);
      expect(await refusal.json()).toEqual({ error: 'invalid credentials' });
      await expect(page.getByRole('alert')).toHaveText('Invalid credentials');
      if (enabled === true) await expect(page.getByTestId('login-refused-note')).toBeVisible();
      else await expect(page.getByTestId('login-refused-note')).toHaveCount(0);
      await page.screenshot({ path: info.outputPath(`account-help-${String(enabled)}.png`) });
    }
  } finally {
    await policy(api, previousValue);
    await api.dispose();
  }
});
