import { test, expect } from '../csrf.ts';
import { adminLogin } from './admin-login.ts';
import { createServer } from 'node:net';

/** A loopback port nothing listens on, fresh per run so no earlier backoff applies. */
const deadPort = () => new Promise<number>(resolve => { const server = createServer(); server.listen(0, '127.0.0.1', () => { const port = (server.address() as { port: number }).port; server.close(() => resolve(port)); }); });

/**
 * t39 L1 / D3: a domain whose selected mail backend is down signs a user in
 * through the next backend, and the failover is visible in Admin > Logs >
 * Application logs. Needs a live plaintext IMAP server the engine may reach on a
 * private address (the Greenmail of the admin stack):
 *   MW_E2E_FAILOVER_REPLICA   host:port of that IMAP server, e.g. 127.0.0.1:3143
 *   MW_E2E_FAILOVER_USER      a mailbox it accepts, e.g. alice / its password
 *   MW_E2E_FAILOVER_PASSWORD
 * The selected backend is a freshly closed loopback port, where nothing listens.
 */
const replica = process.env['MW_E2E_FAILOVER_REPLICA'];
const user = process.env['MW_E2E_FAILOVER_USER'] ?? 'failover';
const password = process.env['MW_E2E_FAILOVER_PASSWORD'] ?? 'Fo-pass-7139';

test('a dead selected backend fails over and the application log shows it @phone', async ({ page, browser }) => {
  test.skip(!replica, 'set MW_E2E_FAILOVER_REPLICA to a live plaintext IMAP host:port');
  const [host, port] = replica!.split(':');
  const dead = await deadPort();
  await adminLogin(page);
  const name = `failover-${Date.now()}-${test.info().project.name}.test`;
  const socket = (p: number) => ({ host: host!, port: p, tls: 'none', auth: 'password', allowPrivate: true, security: { mode: 'fixed', allowed: ['none'], allowPlaintext: true } });
  const path = `/admin/domains/${name}`;
  const created = await page.request.put(path, { data: { enabled: true, upstream: { version: 2, backends: [
    { id: 'primary', role: 'mail', protocol: 'imap', priority: 1, endpoint: { ...socket(dead), host: '127.0.0.1' } },
    { id: 'replica', role: 'mail', protocol: 'imap', priority: 2, endpoint: socket(Number(port)) },
  ] } } });
  expect(created.ok(), `create fixture: ${created.status()} ${await created.text()}`).toBe(true);
  try {
    await page.goto(`/admin#/domains/${name}/connections`);
    const toggle = page.getByRole('checkbox', { name: /Fail over to the next mail backend/ });
    await expect(toggle).not.toBeChecked();
    const saved = page.waitForResponse(r => r.url().endsWith(path) && r.request().method() === 'PUT');
    await toggle.check();
    expect((await saved).status()).toBe(200);
    await expect(toggle).toBeChecked();
    expect((await (await page.request.get(path)).json()).upstream.failover).toBe(true);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await page.screenshot({ path: test.info().outputPath('failover-toggle.png'), fullPage: true });

    const mailbox = await browser.newContext();
    try {
      const login = await mailbox.request.post('/api/login', { data: { username: `${user}@${name}`, password } });
      expect(login.status(), await login.text()).toBe(200);
    } finally { await mailbox.close(); }

    await page.goto('/admin#/logs/application');
    await page.getByLabel('Component', { exact: true }).fill('backend_failover');
    const rows = page.getByRole('table', { name: 'Application logs' }).locator('tbody tr');
    const failover = rows.filter({ hasText: 'event: "failover"' }).filter({ hasText: name });
    await expect(failover.first()).toBeVisible({ timeout: 15_000 });
    await expect(failover.first()).toContainText('from: "primary"');
    await expect(failover.first()).toContainText('to: "replica"');
    await expect(failover.first()).toContainText('reason: "connect"');
    await expect(rows.first()).not.toContainText(password);
    await page.screenshot({ path: test.info().outputPath('failover-application-log.png'), fullPage: true });
  } finally {
    const current = await page.request.get(path);
    await page.request.delete(`${path}?force=true`, { headers: { 'If-Match': current.headers()['etag'] ?? '*' } });
  }
});
