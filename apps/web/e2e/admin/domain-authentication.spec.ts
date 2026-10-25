import { test, expect } from '../csrf.ts';

test('domain password mechanism priority persists through the dedicated editor @phone', async ({ page }) => {
  await page.goto('/admin');
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password', { exact: true }).fill('hunter2');
  const authenticated = page.waitForResponse(response => response.url().endsWith('/admin/login') && response.request().method() === 'POST');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  expect((await authenticated).status()).toBe(200);
  await expect(page.locator('#admin-main')).toBeVisible();
  const status = await page.request.get('/admin/domains/status');
  expect(status.ok()).toBe(true);
  expect((await status.json()).authentication?.runtimeReady, 'fresh server with authentication policy runtime is required').toBe(true);
  const name = `auth-${Date.now()}-${test.info().workerIndex}.example.test`;
  const path = `/admin/domains/${name}`;
  const create = await page.request.put(path, { data: { enabled: false, upstream: { version: 2, backends: [{ id: 'mail', role: 'mail', protocol: 'imap', endpoint: { host: 'mail.example.test', port: 993, tls: 'implicit', auth: 'password' } }] } } });
  expect(create.ok(), `create fixture: ${create.status()}`).toBe(true);
  try {
    await page.goto(`/admin#/domains/${name}/backends/mail`);
    await page.getByLabel('Customize authentication methods', { exact: true }).check();
    await page.getByLabel(/^Authentication selection/).selectOption('auto');
    await page.getByLabel(/^Authentication priority 1/).selectOption('plain');
    await page.getByLabel(/^Authentication priority 2/).selectOption('login');
    await expect(page.getByRole('option', { name: 'POP3 USER/PASS commands', exact: true })).toHaveCount(0);
    await page.getByRole('button', { name: 'Save backend', exact: true }).click();
    await expect(page).toHaveURL(new RegExp(`#/domains/${name.replaceAll('.', '\\.')}$`));
    const saved = await (await page.request.get(path)).json();
    expect(saved.upstream.backends[0].endpoint.authentication).toEqual({ mode: 'auto', mechanisms: ['plain', 'login'] });
    await page.goto(`/admin#/domains/${name}/backends/mail`);
    await expect(page.getByLabel(/^Authentication priority 1/)).toHaveValue('plain');
    await expect(page.getByLabel(/^Authentication priority 2/)).toHaveValue('login');
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await page.screenshot({ path: test.info().outputPath('domain-authentication-editor.png'), fullPage: true });
  } finally {
    const current = await page.request.get(path);
    if (current.ok()) expect((await page.request.delete(path, { headers: { 'if-match': current.headers()['etag']! } })).ok()).toBe(true);
  }
});
