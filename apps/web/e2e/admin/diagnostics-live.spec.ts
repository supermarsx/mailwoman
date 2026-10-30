// t105: the Diagnostics sub-tabs and check cards against a live server.
// Every sub-tab opens from its own link (deep link) and from the tab bar; Run all
// runs the server suite and then the saved-backends job, with progress, and each
// card shows the status the server stored; one card reruns in place (the same
// element, no page swap); the copied report is redacted (no admin password, no
// domain names) while "Replace domain names" is on (the default); and at 390 px
// no page scrolls sideways. A fixture domain with one IMAP backend on a closed
// loopback port gives the backends job a real failure to show. No route mocks.
import { test, expect } from '../csrf.ts';
import { ADMIN } from './admin-path.ts';
import { adminLogin } from './admin-login.ts';
import { createServer } from 'node:net';
import { readFile } from 'node:fs/promises';
import type { Page } from '@playwright/test';

const TABS: [string, string, (page: Page) => ReturnType<Page['getByRole']>][] = [
  ['', 'Overview', (p) => p.getByRole('button', { name: 'Run all checks' })],
  ['server', 'Server', (p) => p.getByRole('article', { name: 'Database connection' })],
  ['connectivity', 'Connectivity', (p) => p.getByRole('article', { name: 'DNS records' })],
  ['backends', 'Mail backends', (p) => p.getByRole('heading', { name: 'Failover' })],
  ['posture', 'Security posture', (p) => p.getByRole('article', { name: 'fail2ban client address' })],
  ['tools', 'Tools', (p) => p.getByRole('heading', { name: 'Service health' })],
  ['history', 'History', (p) => p.getByRole('table', { name: 'Previous runs' })],
  ['report', 'Report', (p) => p.getByRole('heading', { name: 'Troubleshooting report' })],
];
const STATUS: Record<string, string> = { pass: 'OK', warn: 'Warning', fail: 'Failed', skipped: 'Skipped', error: 'Failed' };
const closedPort = () => new Promise<number>((resolve) => { const s = createServer(); s.listen(0, '127.0.0.1', () => { const port = (s.address() as { port: number }).port; s.close(() => resolve(port)); }); });
const noSideScroll = (page: Page) => page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1);
interface Result { id: string; status: string; summary: string; domain?: string }
interface RunReply { run: { id: string; status: string; results: Result[] }; historySaved: boolean; auditSaved: boolean }

test('diagnostics sub-tabs, Run all, a card rerun in place and the redacted report @phone', async ({ page }) => {
  test.setTimeout(240_000);
  await adminLogin(page);
  const name = `diag-${Date.now()}-${test.info().project.name}.test`;
  const path = `${ADMIN}/domains/${name}`;
  const port = await closedPort();
  const created = await page.request.put(path, { data: { enabled: true, upstream: { version: 2, backends: [
    { id: 'primary', role: 'mail', protocol: 'imap', priority: 1, endpoint: { host: '127.0.0.1', port, tls: 'none', auth: 'password', allowPrivate: true, security: { mode: 'fixed', allowed: ['none'], allowPlaintext: true } } },
  ] } } });
  expect(created.ok(), `fixture domain: ${created.status()} ${await created.text()}`).toBe(true);
  try {
    // ── every sub-tab: by deep link, then by the tab bar ──
    for (const [id, label, landmark] of TABS) {
      await page.goto(`${ADMIN}#/diagnostics${id ? `/${id}` : ''}`);
      const nav = page.getByRole('navigation', { name: 'Diagnostics pages' });
      await expect(nav.getByRole('link', { name: label, exact: true })).toHaveAttribute('aria-current', 'page');
      await expect(landmark(page).first()).toBeVisible();
      expect(await noSideScroll(page), `${label}: no sideways scroll`).toBe(true);
    }
    const nav = page.getByRole('navigation', { name: 'Diagnostics pages' });
    await nav.getByRole('link', { name: 'Server', exact: true }).click();
    await expect(page).toHaveURL(/#\/diagnostics\/server$/);
    await nav.getByRole('link', { name: 'Overview', exact: true }).click();
    await expect(page).toHaveURL(/#\/diagnostics$/);

    // ── Run all: server suite, then the backends job, with progress ──
    const runAll = page.getByRole('button', { name: 'Run all checks' });
    await expect(runAll).toBeEnabled();
    const server = page.waitForResponse((r) => r.url().endsWith(`${ADMIN}/diag/run`) && r.request().method() === 'POST');
    const job = page.waitForResponse((r) => r.url().endsWith(`${ADMIN}/diag/jobs`) && r.request().method() === 'POST', { timeout: 60_000 });
    await runAll.click();
    await expect(page.getByTestId('diag-run-all-progress')).toContainText(/Step [12] of 2/);
    const reply = await server;
    expect(reply.status()).toBe(200);
    const report = (await reply.json()) as RunReply;
    expect(report.historySaved).toBe(true);
    expect(report.auditSaved).toBe(true);
    const byId = new Map(report.run.results.map((r) => [r.id, r]));
    for (const id of ['database-connectivity', 'database-write', 'database-migrations', 'server-key-roundtrip', 'egress-private-policy']) expect(byId.get(id)?.status, id).toBe('pass');
    expect(byId.get('fail2ban-client-address')?.status).toBe('warn');
    expect((await job).status()).toBe(202);
    await expect(page.getByText('All checks finished. The results are saved in History.')).toBeVisible({ timeout: 70_000 });
    await expect(page.getByTestId('diag-run-all-progress')).toHaveCount(0);
    await expect(page.locator('[data-group="server"]')).toContainText('4 OK');
    await page.screenshot({ path: test.info().outputPath('diagnostics-overview.png'), fullPage: true });

    // Each card shows the status the server stored.
    await page.goto(`${ADMIN}#/diagnostics/posture`);
    await expect(page.getByRole('article', { name: 'fail2ban client address' }).getByTestId('diag-status')).toHaveText('Warning');
    await expect(page.getByRole('article', { name: 'fail2ban client address' }).getByText('How to fix')).toBeVisible();
    await page.goto(`${ADMIN}#/diagnostics/server`);
    const serverCards: [string, string][] = [['database-connectivity', 'Database connection'], ['database-write', 'Database writes'], ['database-migrations', 'Database migrations'], ['server-key-roundtrip', 'Server key']];
    for (const [id, title] of serverCards) {
      await expect(page.getByRole('article', { name: title }).getByTestId('diag-status')).toHaveText(STATUS[byId.get(id)!.status]!);
    }
    // The fixture backend's closed port is a Failed card with guidance.
    await page.goto(`${ADMIN}#/diagnostics/backends`);
    const backend = page.getByRole('article', { name: `${name} · primary` });
    await expect(backend.getByTestId('diag-status')).toHaveText('Failed');
    await expect(backend.getByText('How to fix')).toBeVisible();
    await expect(page.getByTestId('diag-failover')).toContainText(name);

    // ── one card reruns in place ──
    await page.goto(`${ADMIN}#/diagnostics/server`);
    const card = page.getByRole('article', { name: 'Database connection' });
    await expect(card.getByTestId('diag-status')).toHaveText('OK');
    await card.evaluate((el) => { (el as HTMLElement).dataset['t105Mark'] = 'same'; });
    const rerun = page.waitForResponse((r) => r.url().endsWith(`${ADMIN}/diag/run`) && r.request().method() === 'POST');
    await card.getByRole('button', { name: 'Run again' }).click();
    let answer = await rerun;
    if (answer.status() === 429) {
      // Run all spent this session's two runs a minute: the corner toast says how
      // long to wait, and its Retry waits that out and sends the run again.
      const toast = page.getByTestId('admin-request-error');
      await expect(toast).toContainText(/try again in \d+ s/i);
      const retried = page.waitForResponse((r) => r.url().endsWith(`${ADMIN}/diag/run`) && r.request().method() === 'POST', { timeout: 90_000 });
      await toast.getByRole('button', { name: 'Retry' }).click();
      answer = await retried;
    }
    expect(answer.status()).toBe(200);
    await expect(card.getByTestId('diag-status')).toHaveText('OK');
    // The same element: updated in place, not re-rendered or swapped for a page.
    await expect(page.locator('article[data-t105-mark="same"]')).toHaveCount(1);

    // ── the report: redaction on by default ──
    await page.goto(`${ADMIN}#/diagnostics/report`);
    await page.evaluate(() => { navigator.clipboard.writeText = async (text: string) => { (window as unknown as { copied?: string }).copied = text; }; });
    const redaction = page.getByRole('checkbox', { name: 'Replace domain names and backend IDs in the report' });
    await expect(redaction).toBeChecked();
    const reportRequest = page.waitForRequest((r) => r.url().includes(`${ADMIN}/diag/report?`));
    await page.getByRole('button', { name: 'Copy troubleshooting report' }).click();
    expect((await reportRequest).url()).toContain('omitDomains=true');
    await expect.poll(() => page.evaluate(() => (window as unknown as { copied?: string }).copied ?? '')).not.toBe('');
    const copiedText = await page.evaluate(() => (window as unknown as { copied?: string }).copied!);
    const copied = JSON.parse(copiedText);
    expect(copied.redaction.domainsOmitted).toBe(true);
    expect(copiedText).not.toContain('hunter2');
    expect(copiedText).not.toContain(name);
    const domains = await (await page.request.get(`${ADMIN}/domains`)).json() as { name: string }[];
    expect(domains.some((d) => d.name === name)).toBe(true);
    for (const domain of domains) expect(copiedText).not.toContain(domain.name);
    // With redaction off the names are there, so the check above means something.
    await redaction.uncheck();
    const downloaded = page.waitForEvent('download');
    await page.getByRole('button', { name: 'Export troubleshooting report' }).click();
    const support = JSON.parse(await readFile((await (await downloaded).path())!, 'utf8'));
    expect(support.schemaVersion).toBe(1);
    expect(JSON.stringify(support)).toContain(name);
    expect(JSON.stringify(support)).not.toContain('hunter2');
    expect(await noSideScroll(page)).toBe(true);
    await page.screenshot({ path: test.info().outputPath('diagnostics-report.png'), fullPage: true });
  } finally {
    const current = await page.request.get(path);
    await page.request.delete(`${path}?force=true`, { headers: { 'If-Match': current.headers()['etag'] ?? '*' } });
  }
});
