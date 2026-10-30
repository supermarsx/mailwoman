// t112: no screen shows a raw Fluent message id. Against a live server built
// from the tree, walks every admin nav section (and the tabs inside each), then
// every section of the mail app's Settings, and asserts that no visible text,
// and no aria-label / title / placeholder of a visible element, is a message id:
// lowercase words joined by hyphens, two or more hyphens, no spaces, whose first
// word starts an id in the en catalogs (so a header name such as
// strict-transport-security shown as data does not count). Each token that only
// matches the shape is printed for review.
// The first visit of a section may show ids for a moment while its catalog
// loads; the check polls, so only an id that stays fails.
// Screenshots go to MW_T112_SHOTS (default: the test output dir).
import { test, expect, type Page } from '../csrf.ts';
import type { TestInfo } from '@playwright/test';
import { ADMIN } from '../admin/admin-path.ts';
import { adminLogin } from '../admin/admin-login.ts';
import { readdirSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

test.describe.configure({ timeout: 900_000 });

const enDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../locales/en');
const enIds = new Set<string>();
for (const f of readdirSync(enDir).filter((n) => n.endsWith('.ftl'))) {
  for (const line of readFileSync(path.join(enDir, f), 'utf8').split(/\r?\n/)) {
    const m = /^([a-z][a-z0-9-]*)\s*=/.exec(line);
    if (m) enIds.add(m[1]!);
  }
}
const idPrefixes = new Set([...enIds].map((id) => id.split('-')[0]!));
const SHAPE = /^[a-z][a-z0-9]*(?:-[a-z0-9]+){2,}$/;

const shot = (info: TestInfo, name: string): string => {
  const dir = process.env['MW_T112_SHOTS'];
  const file = `${info.project.name}-${name.replace(/[^a-z0-9-]+/gi, '_')}.png`;
  return dir ? path.join(dir, file) : info.outputPath(file);
};

/** Every id-shaped token on screen: visible text plus visible elements' labels. */
async function shapedTokens(page: Page): Promise<string[]> {
  const texts = await page.evaluate(() => {
    const out: string[] = [document.body.innerText];
    for (const el of Array.from(document.querySelectorAll<HTMLElement>('[aria-label],[title],[placeholder]'))) {
      const r = el.getBoundingClientRect();
      if (r.width === 0 && r.height === 0) continue;
      for (const a of ['aria-label', 'title', 'placeholder']) { const v = el.getAttribute(a); if (v) out.push(v); }
    }
    return out;
  });
  const found = new Set<string>();
  for (const text of texts) {
    for (const raw of text.split(/[\s"'“”‘’«»()[\]{},;:!?…·|]+/)) {
      const token = raw.replace(/^[.]+|[.]+$/g, '');
      if (SHAPE.test(token)) found.add(token);
    }
  }
  return [...found];
}
const rawIds = (tokens: string[]): string[] => tokens.filter((t) => enIds.has(t) || idPrefixes.has(t.split('-')[0]!));

const shapeOnly = new Map<string, Set<string>>();
async function expectNoRawIds(page: Page, where: string, info: TestInfo): Promise<void> {
  await expect.poll(async () => {
    // Observability deliberately shows machine audit action identifiers from
    // the API. A shared prefix (e.g. linked-credential-stored) is not evidence
    // of an untranslated label. Exact catalog IDs remain forbidden everywhere.
    const actions = where.startsWith('admin-observability')
      ? new Set(await page.locator('#admin-main td[data-label="Action"]').allTextContents())
      : new Set<string>();
    return rawIds(await shapedTokens(page)).filter(token => enIds.has(token) || !actions.has(token));
  }, { message: `raw message ids on ${where}`, timeout: 10_000 }).toEqual([]);
  for (const t of await shapedTokens(page)) { if (!shapeOnly.has(t)) shapeOnly.set(t, new Set()); shapeOnly.get(t)!.add(where); }
  await page.screenshot({ path: shot(info, where) });
}

test('admin console: no section or tab shows a raw message id', async ({ page }, info) => {
  test.skip(info.project.name === 'phone', 'the desktop projects walk the nav');
  await page.setViewportSize({ width: 1440, height: 900 });
  await adminLogin(page);
  expect(new URL(page.url()).pathname.startsWith(ADMIN)).toBe(true);
  const nav = page.getByRole('navigation', { name: 'Admin sections' }).first();
  await expect(nav).toBeVisible();
  await expectNoRawIds(page, 'admin-landing', info);
  const sections = await nav.locator('a[data-section]').evaluateAll((as) => as.map((a) => ({ id: a.getAttribute('data-section')!, href: a.getAttribute('href')! })));
  expect(sections.length).toBeGreaterThan(20);
  expect(sections.map((s) => s.id)).toContain('features');
  // The t98 regression itself: the Feature profiles nav entry has its name.
  await expect(nav.locator('a[data-section="features"]')).toHaveText('Feature profiles');
  for (const s of sections) {
    await nav.locator(`a[data-section="${s.id}"]`).click();
    await expect(nav.locator(`a[data-section="${s.id}"]`)).toHaveAttribute('aria-current', 'page');
    await page.waitForLoadState('networkidle', { timeout: 5_000 }).catch(() => undefined);
    await expectNoRawIds(page, `admin-${s.id}`, info);
    const tabs = page.locator('#admin-main [role="tablist"] [role="tab"]');
    const count = await tabs.count();
    for (let i = 1; i < count; i++) {
      const tab = tabs.nth(i);
      if (!(await tab.isVisible()) || (await tab.isDisabled())) continue;
      const name = ((await tab.textContent()) ?? `${i}`).trim().slice(0, 30);
      await tab.click();
      await page.waitForLoadState('networkidle', { timeout: 5_000 }).catch(() => undefined);
      await expectNoRawIds(page, `admin-${s.id}-tab-${name}`, info);
    }
  }
  console.log(`t112 admin: ${sections.length} sections walked; shape-only tokens (not ids): ${JSON.stringify([...shapeOnly].map(([t, w]) => [t, [...w].slice(0, 2)]))}`);
});

test('mail Settings: no section shows a raw message id', async ({ page }, info) => {
  test.skip(info.project.name === 'phone', 'the desktop projects walk the nav');
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill('testuser');
  await page.getByLabel('Password', { exact: true }).fill('testpass');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.locator('.list__row').first()).toBeVisible({ timeout: 60_000 });
  await expectNoRawIds(page, 'mail-inbox', info);
  await page.locator('.sidebar, #shell-nav').getByRole('button', { name: 'Settings', exact: true }).first().click();
  const nav = page.getByRole('navigation', { name: 'Settings sections' });
  const buttons = nav.getByRole('button');
  await expect(buttons.first()).toBeVisible();
  const names = (await buttons.allTextContents()).map((n) => n.trim()).filter(Boolean);
  expect(names.length).toBeGreaterThan(5);
  for (const name of names) {
    await nav.getByRole('button', { name, exact: true }).click();
    await page.waitForLoadState('networkidle', { timeout: 5_000 }).catch(() => undefined);
    await expectNoRawIds(page, `settings-${name}`, info);
  }
  console.log(`t112 settings: ${names.length} sections walked; shape-only tokens (not ids): ${JSON.stringify([...shapeOnly].map(([t, w]) => [t, [...w].slice(0, 2)]))}`);
});
