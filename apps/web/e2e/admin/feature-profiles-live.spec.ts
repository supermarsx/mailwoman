import { test, expect } from '../csrf.ts';
import { ADMIN } from './admin-path.ts';
import { adminLogin } from './admin-login.ts';

/**
 * t39 N9 + L5 against a live engine-mode stack:
 * - a feature profile made in Admin > Feature profiles and assigned to a domain
 *   changes what `/api/features` reports to a user of that domain;
 * - the authentication history links that user's sign-in to their user page;
 * - Observability > Audit log retention saves a default and prunes.
 *
 * Needs a plaintext IMAP server the engine may reach on a private address and a
 * mailbox on it whose login is an address (its domain is created here):
 *   MW_E2E_PROFILE_IMAP       host:port, e.g. 127.0.0.1:3143
 *   MW_E2E_PROFILE_USER       e.g. ana@example.test
 *   MW_E2E_PROFILE_PASSWORD
 *   MW_E2E_OLD_AUDIT_ROWS     optional: how many audit rows older than 30 days
 *                             (category NULL) were seeded before the run; the
 *                             prune must then remove at least that many.
 */
const imap = process.env['MW_E2E_PROFILE_IMAP'];
const user = process.env['MW_E2E_PROFILE_USER'] ?? '';
const password = process.env['MW_E2E_PROFILE_PASSWORD'] ?? '';
const seeded = Number(process.env['MW_E2E_OLD_AUDIT_ROWS'] ?? '0');
const cooldown = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 3100));

interface Features { features: Record<string, { enabled: boolean }> }

test('a domain profile changes a user\'s features; the auth log links the user; retention prunes', async ({ page, browser }) => {
  test.skip(!imap || !user.includes('@'), 'set MW_E2E_PROFILE_IMAP and MW_E2E_PROFILE_USER (an address)');
  const [host, port] = imap!.split(':');
  const domain = user.split('@')[1]!;
  const project = test.info().project.name;

  await adminLogin(page);
  const created = await page.request.put(`${ADMIN}/domains/${domain}`, { data: { enabled: true, upstream: { version: 2, backends: [
    { id: 'mail', role: 'mail', protocol: 'imap', priority: 1, endpoint: { host: host!, port: Number(port), tls: 'none', auth: 'password', allowPrivate: true, security: { mode: 'fixed', allowed: ['none'], allowPlaintext: true } } },
  ] } } });
  expect(created.ok(), `domain fixture: ${created.status()} ${await created.text()}`).toBe(true);

  // The user signs in (a native account of that domain) and has export.
  const mail = await browser.newContext();
  await cooldown();
  const signin = await mail.request.post('/api/login', { data: { username: user, password } });
  expect(signin.status(), await signin.text()).toBe(200);
  const before = (await (await mail.request.get('/api/features')).json()) as Features;
  expect(before.features['export']!.enabled).toBe(true);

  // Make a profile that turns export off, locked, and assign it to the domain.
  const name = `No export ${project} ${Date.now()}`;
  await page.goto(`${ADMIN}#/features`);
  await expect(page.getByRole('heading', { name: 'Feature profiles', exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'New profile' }).first().click();
  await page.getByLabel('Profile name').fill(name);
  await page.getByLabel('Value for Export').selectOption('off');
  await page.getByRole('checkbox', { name: /Locked\s+Export/ }).check();
  const saved = page.waitForResponse((r) => r.url().includes(`${ADMIN}/features/profiles/`) && r.request().method() === 'PUT');
  await page.getByRole('button', { name: 'Create profile' }).click();
  expect((await saved).status()).toBe(200);
  await expect(page.getByRole('heading', { name, exact: true })).toBeVisible();
  const form = page.getByRole('form', { name: 'Assign this profile' });
  await form.getByRole('combobox', { name: 'Applies to' }).selectOption('domain');
  await form.getByRole('combobox', { name: 'Name' }).selectOption(domain);
  const assigned = page.waitForResponse((r) => r.url().endsWith(`${ADMIN}/features/assignments`) && r.request().method() === 'PUT');
  await form.getByRole('button', { name: 'Assign' }).click();
  expect((await assigned).status()).toBe(200);
  await expect(page.getByRole('rowheader', { name: domain })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: test.info().outputPath('feature-profile-assigned.png'), fullPage: true });

  const after = (await (await mail.request.get('/api/features')).json()) as Features;
  expect(after.features['export']!.enabled, 'the domain profile turns export off for the user').toBe(false);

  // Remove the assignment again: the user's export comes back.
  const removed = page.waitForResponse((r) => r.url().endsWith(`${ADMIN}/features/assignments`) && r.request().method() === 'DELETE');
  await page.getByRole('button', { name: `Remove ${domain}` }).click();
  expect((await removed).status()).toBe(200);
  const restored = (await (await mail.request.get('/api/features')).json()) as Features;
  expect(restored.features['export']!.enabled).toBe(true);

  // The authentication history links the user's sign-in to the user page.
  await page.goto(`${ADMIN}#/logs`);
  await page.getByLabel('Account, attempted username or IP').fill(user);
  await page.getByLabel('Sign-in area').selectOption('webmail');
  await page.getByRole('button', { name: 'Apply filters' }).click();
  const link = page.getByRole('link', { name: new RegExp(user.replace(/[.@]/g, '\\$&')) }).first();
  await expect(link).toBeVisible();
  expect(await link.getAttribute('href')).toMatch(/#\/users\/[^/]+$/);
  await page.screenshot({ path: test.info().outputPath('auth-log-actor-link.png'), fullPage: true });
  await link.click();
  await expect(page).toHaveURL(/#\/users\//);
  await expect(page.getByText(user).first()).toBeVisible();

  // Retention: keep 30 days by default, then prune now.
  await page.goto(`${ADMIN}#/observability`);
  const retention = page.getByTestId('audit-retention');
  await expect(retention.getByRole('heading', { name: 'Audit log retention' })).toBeVisible();
  const row = retention.locator('[data-category="*"]');
  await row.getByLabel('Keep', { exact: true }).selectOption('days');
  await row.getByLabel('Days', { exact: true }).fill('30');
  const put = page.waitForResponse((r) => r.url().endsWith(`${ADMIN}/audit/retention/*`) && r.request().method() === 'PUT');
  await row.getByRole('button', { name: /Save/ }).click();
  expect((await put).status()).toBe(200);
  await expect(row.locator('[data-kept]')).toHaveText('30 days');
  await expect(retention.getByText(/Next scheduled prune|Pruned daily/)).toBeVisible();
  await retention.getByRole('button', { name: 'Prune now' }).click();
  const prune = page.waitForResponse((r) => r.url().endsWith(`${ADMIN}/audit/retention/prune`));
  await page.getByRole('alertdialog').getByRole('button', { name: 'Prune now' }).click();
  const report = (await (await prune).json()) as { report: { total: number } };
  expect(report.report.total).toBeGreaterThanOrEqual(seeded);
  await expect(page.getByText(/entr(y|ies) removed|No entries were old enough/).first()).toBeVisible();
  if (seeded > 0) await expect(retention.getByText(/Last prune that removed entries/)).toBeVisible();
  const audit = (await (await page.request.get(`${ADMIN}/audit?limit=100`)).json()) as { action: string }[];
  expect(audit.some((e) => e.action === 'audit-retention-changed')).toBe(true);
  if (seeded > 0) expect(audit.some((e) => e.action === 'audit-pruned')).toBe(true);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: test.info().outputPath('audit-retention-pruned.png'), fullPage: true });
  await mail.close();
});
