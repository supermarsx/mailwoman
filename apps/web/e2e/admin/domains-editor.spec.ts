import { test, expect } from '../csrf.ts';
import { adminLogin } from './admin-login.ts';

// Real admin API and SPA. Every browser uses a fresh domain and removes its data.
test('domain connection editor, policies, mappings and protected deletion @phone', async ({ page }) => {
  const domain = `ui-${Date.now()}-${test.info().project.name}.test`;
  const alias = `alias-${domain}`;
  const host = `login-${domain}`;
  const admin = process.env['MW_E2E_ADMIN_PATH'] ?? '/admin';
  const tab = (name: string) => page.getByRole('navigation', { name: 'Domain configuration pages' }).getByRole('link', { name, exact: true });
  await adminLogin(page);
  // An unconfigured domain's page is the add form with its name filled in (3d3bf859).
  await page.goto(`${admin}#/domains/${domain}`);
  const add = page.getByRole('form', { name: 'Add domain' });
  await expect(add.getByRole('textbox', { name: 'Domain name', exact: true })).toHaveValue(domain);
  await add.getByRole('button', { name: 'Create and configure' }).click();
  await expect(page.getByRole('heading', { name: domain, exact: true })).toBeVisible();
  try {
    await expect(page.getByText('Server mode: engine (set for the whole server)')).toBeVisible();
    // Connections and sign-in policy are separate pages since 618be244; each is reviewed and applied.
    await tab('Connections and discovery').click();
    await page.getByRole('combobox', { name: 'Incoming mail protocol' }).selectOption('imap');
    await page.getByRole('textbox', { name: 'IMAP Host', exact: true }).fill('mail.example.com');
    await page.getByRole('checkbox', { name: 'Configure SMTP' }).check();
    await page.getByRole('textbox', { name: 'SMTP Host', exact: true }).fill('smtp.example.com');
    await page.getByRole('button', { name: 'Review changes', exact: true }).click();
    let review = page.getByRole('alertdialog');
    await expect(review.getByText(/mail.example.com:993/).first()).toBeVisible();
    await review.getByRole('button', { name: 'Apply changes', exact: true }).click();
    await expect(review).toHaveCount(0);
    await expect(page.getByRole('button', { name: 'Review changes', exact: true })).toBeDisabled();
    await tab('Sign-in policy').click();
    await page.getByRole('textbox', { name: 'Allowlist', exact: true }).fill(`alice@${domain}`);
    await page.getByRole('button', { name: 'Review changes', exact: true }).click();
    review = page.getByRole('alertdialog');
    await expect(review.getByText(`alice@${domain}`)).toBeVisible();
    await review.getByRole('button', { name: 'Apply changes', exact: true }).click();
    await expect(review).toHaveCount(0);
    await page.goto(`${admin}#/domains/${domain}/connections`);
    await page.reload();
    await expect(page.getByRole('textbox', { name: 'IMAP Host', exact: true })).toHaveValue('mail.example.com');
    await expect(page.getByRole('textbox', { name: 'SMTP Host', exact: true })).toHaveValue('smtp.example.com');
    await tab('Sign-in policy').click();
    await expect(page.getByRole('textbox', { name: 'Allowlist', exact: true })).toHaveValue(`alice@${domain}`);
    // Aliases are added on their own editor page (3a61dfff) and listed in a table (e5360292).
    await tab('Domain aliases').click();
    await page.getByRole('button', { name: 'Add alias', exact: true }).click();
    await page.getByRole('textbox', { name: 'Alias domain' }).fill(alias);
    await page.getByRole('button', { name: 'Save', exact: true }).click();
    await expect(page.getByRole('table', { name: 'Domain aliases' }).getByRole('rowheader', { name: alias, exact: true })).toBeVisible();
    await tab('Login hostnames').click();
    const hosts = page.getByRole('region', { name: 'Login host mappings', exact: true });
    await hosts.getByRole('textbox', { name: 'New login host' }).fill(host);
    await hosts.getByRole('button', { name: 'Add', exact: true }).click();
    await expect(hosts.getByText(host, { exact: true })).toBeVisible();
    await tab('Overview').click();
    await page.getByRole('button', { name: 'Set preferred domain' }).click();
    await page.getByRole('button', { name: 'Clear preferred domain' }).click();
    await expect(page.getByRole('button', { name: 'Set preferred domain' })).toBeVisible();
    await page.getByRole('button', { name: 'Disable domain' }).click();
    await page.getByRole('alertdialog').getByRole('button', { name: 'Continue' }).click();
    await page.getByRole('button', { name: 'Enable domain' }).click();
    await expect(page.getByRole('button', { name: 'Disable domain' })).toBeVisible();
    await page.screenshot({ path: test.info().outputPath('domain-editor.png'), fullPage: true });
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
    await page.getByRole('button', { name: 'Delete', exact: true }).click();
    const deletion = page.getByRole('alertdialog');
    await expect(deletion.getByRole('button', { name: 'Delete', exact: true })).toBeDisabled();
    await deletion.getByRole('textbox').fill(domain);
    await deletion.getByRole('button', { name: 'Delete', exact: true }).click();
    await expect(deletion).toHaveCount(0);
    expect((await page.request.get(`${admin}/domains/${domain}`)).status()).toBe(404);
  } finally {
    const left = await page.request.get(`${admin}/domains/${domain}`);
    if (left.ok()) await page.request.delete(`${admin}/domains/${domain}?force=true`, { headers: { 'if-match': left.headers()['etag']! } });
  }
});

test('saved upstream probe reports a real unauthenticated IMAP capability check @phone', async ({ page }) => {
  // The probe dials from the server, so the target is the stack's own IMAP upstream (Greenmail in CI);
  // MW_DOMAIN_PROBE_PORT points it at a scripted fixture on the server host instead.
  const upstream = new URL(process.env['MW_E2E_ENGINE_IMAP_URL'] ?? 'imap://greenmail:3143');
  const probeHost = process.env['MW_DOMAIN_PROBE_PORT'] ? '127.0.0.1' : upstream.hostname;
  const port = process.env['MW_DOMAIN_PROBE_PORT'] ?? upstream.port;
  // A name under example.com (RFC 2606) gets a real NXDOMAIN; some resolvers never answer for .test.
  const domain = `probe-${Date.now()}-${test.info().project.name}.example.com`;
  const admin = process.env['MW_E2E_ADMIN_PATH'] ?? '/admin';
  const tab = (name: string) => page.getByRole('navigation', { name: 'Domain configuration pages' }).getByRole('link', { name, exact: true });
  await adminLogin(page);
  await page.goto(`${admin}#/domains/new`);
  await page.getByRole('textbox', { name: 'Domain name', exact: true }).fill(domain);
  await page.getByRole('button', { name: 'Create and configure' }).click();
  await expect(page.getByRole('heading', { name: domain, exact: true })).toBeVisible();
  try {
    await tab('Connections and discovery').click();
    await page.getByRole('combobox', { name: 'Incoming mail protocol' }).selectOption('imap');
    await page.getByRole('textbox', { name: 'IMAP Host', exact: true }).fill(probeHost);
    await page.getByRole('spinbutton', { name: 'IMAP Port', exact: true }).fill(port);
    await page.getByRole('combobox', { name: 'IMAP Connection security' }).selectOption('none');
    await page.getByRole('checkbox', { name: 'Allow private network addresses for IMAP' }).check();
    await page.getByRole('button', { name: 'Review changes', exact: true }).click();
    await page.getByRole('alertdialog').getByRole('button', { name: 'Apply changes' }).click();
    await expect(page.getByRole('alertdialog')).toHaveCount(0);
    await tab('Diagnostics').click();
    await page.getByRole('button', { name: 'Test saved connections' }).click();
    await expect(page.getByText(/Connection check passed/)).toBeVisible();
    await page.getByText('Advertised capabilities', { exact: true }).click();
    await expect(page.getByText(/IMAP4REV1/i).last()).toBeVisible();
    // The server bounds the DNS checks at 30 s (domain_probe.rs); lookups run one after another.
    const dnsResponse = page.waitForResponse(response => response.url().endsWith('/diag/dns'), { timeout: 35_000 });
    await page.getByRole('button', { name: 'Check DNS' }).click();
    const response = await dnsResponse;
    expect(response.status()).toBe(200);
    const report = await response.json() as { domain: string; mx: { records: unknown[] }; spf: { present: boolean }; srv: unknown[] };
    expect(report.domain).toBe(domain); expect(Array.isArray(report.mx.records)).toBe(true); expect(typeof report.spf.present).toBe('boolean'); expect(report.srv).toHaveLength(3);
    await page.screenshot({ path: test.info().outputPath('domain-probe.png'), fullPage: true });
    await tab('Overview').click();
    await page.getByRole('button', { name: 'Delete', exact: true }).click();
    await page.getByRole('alertdialog').getByRole('textbox').fill(domain);
    await page.getByRole('alertdialog').getByRole('button', { name: 'Delete', exact: true }).click();
    await expect(page.getByRole('alertdialog')).toHaveCount(0);
    expect((await page.request.get(`${admin}/domains/${domain}`)).status()).toBe(404);
  } finally {
    const left = await page.request.get(`${admin}/domains/${domain}`);
    if (left.ok()) await page.request.delete(`${admin}/domains/${domain}?force=true`, { headers: { 'if-match': left.headers()['etag']! } });
  }
});
