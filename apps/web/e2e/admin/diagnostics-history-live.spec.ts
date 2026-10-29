// t39 N5: the diagnostics history table ("Previous runs") against a live server.
// Two real runs are saved; after a reload the table lists both, newest first,
// with the suite, status and check counts the server stored, and opening an
// older run shows that run's results. No route mocks.
import { test, expect } from '../csrf.ts';
import { ADMIN } from './admin-path.ts';
import { adminLogin } from './admin-login.ts';

interface Run { id: string; suite: string; status: string; startedAt: number; results: { status: string }[] }

test('saved diagnostic runs are listed after a reload and an older run opens @phone', async ({ page }) => {
  await adminLogin(page);
  await page.goto(`${ADMIN}#/diagnostics`);
  const run = page.getByRole('button', { name: 'Run server checks', exact: true });
  const ids: string[] = [];
  for (let n = 0; n < 2; n++) {
    await expect(run).toBeEnabled();
    const response = page.waitForResponse((r) => r.url().endsWith(`${ADMIN}/diag/run`) && r.request().method() === 'POST');
    await run.click();
    const report = (await (await response).json()) as { run: Run; historySaved: boolean };
    expect(report.historySaved).toBe(true);
    ids.push(report.run.id);
    // Runs are keyed by their start second; keep the two apart.
    if (n === 0) await page.waitForTimeout(1100);
  }

  const saved = (await (await page.request.get(`${ADMIN}/diag/runs`)).json()) as { runs: Run[] };
  const newest = saved.runs.find((r) => r.id === ids[1])!;
  const older = saved.runs.find((r) => r.id === ids[0])!;
  expect(newest).toBeTruthy();
  expect(older).toBeTruthy();
  expect(saved.runs.indexOf(newest)).toBeLessThan(saved.runs.indexOf(older));

  await page.reload();
  const history = page.getByRole('table', { name: 'Previous runs', exact: true });
  await expect(history).toBeVisible();
  const rows = history.locator('tbody tr');
  await expect.poll(async () => rows.count()).toBeGreaterThanOrEqual(2);
  // Row cells: started, suite, status, checks, failed (then the open action).
  const first = rows.nth(0).locator('td');
  await expect(first.nth(1)).toHaveText(newest.suite);
  await expect(first.nth(2)).toHaveText(newest.status);
  await expect(first.nth(3)).toHaveText(String(newest.results.length));
  await expect(first.nth(4)).toHaveText(String(newest.results.filter((c) => c.status === 'fail').length));
  await expect(first.nth(0).locator('time')).toHaveAttribute('datetime', new Date(newest.startedAt * 1000).toISOString());
  const second = rows.nth(1).locator('td');
  await expect(second.nth(0).locator('time')).toHaveAttribute('datetime', new Date(older.startedAt * 1000).toISOString());

  const open = rows.nth(1).getByRole('button', { name: /^View run from / });
  await open.click();
  await expect(open).toHaveAttribute('aria-pressed', 'true');
  await expect(rows.nth(0).getByRole('button', { name: /^View run from / })).toHaveAttribute('aria-pressed', 'false');
  const results = page.getByRole('table', { name: 'Check results', exact: true });
  await expect(results.locator('tbody tr')).toHaveCount(older.results.length);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true);
  await page.screenshot({ path: test.info().outputPath('diagnostics-history.png'), fullPage: true });
});
