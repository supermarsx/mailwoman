import { test, expect, type Page, type BrowserContext, type Browser } from '@playwright/test';

// The admin console's loading placeholders (t72), against a live server built
// from the tree. Each leg delays one request so the placeholder is on screen
// long enough to measure, then checks the hand-over moves no box:
//   - sign-in: the static card skeleton (index.html, styles/boot.css) has the
//     sign-in form's box, on a first visit and with the height remembered from
//     the last one; light pack and reduced motion included;
//   - after sign-in: the console frame mounts at once with placeholders in the
//     navigation and the page, and the dashboard takes over without moving them;
//   - a reload with a session: the static console frame.
// Runs in the `chromium` and `firefox` projects of playwright.admin.config.ts;
// every leg runs at 1440 and at 390. MW_T72_SHOTS saves screenshots there.

const WIDTHS = [1440, 390] as const;
const ADMIN = process.env['MW_E2E_ADMIN_PATH'] ?? '/admin';
const USER = process.env['MW_ADMIN_USER'] ?? 'root';
const PASSWORD = process.env['MW_ADMIN_PASSWORD'] ?? 'hunter2';
const SHOTS = process.env['MW_T72_SHOTS'];
const TOLERANCE = 2;

type Box = { x: number; y: number; width: number; height: number };

function expectSameBox(actual: Box | null, expected: Box | null, what: string): void {
  expect(actual, what).not.toBeNull();
  expect(expected, what).not.toBeNull();
  for (const k of ['x', 'y', 'width', 'height'] as const) {
    expect(Math.abs(actual![k] - expected![k]), `${what}: ${k} ${actual![k]} vs ${expected![k]}`).toBeLessThanOrEqual(TOLERANCE);
  }
}

async function shot(page: Page, name: string): Promise<void> {
  if (SHOTS) await page.screenshot({ path: `${SHOTS}/${name}-${test.info().project.name}.png` });
}

/** A context at `width` that records every CSP violation the page reports. */
async function open(browser: Browser, width: number, opts: { reducedMotion?: 'reduce'; light?: boolean } = {}): Promise<{ context: BrowserContext; page: Page; csp: string[] }> {
  const context = await browser.newContext({
    viewport: { width, height: width > 500 ? 900 : 844 },
    serviceWorkers: 'block',
    ...(opts.reducedMotion ? { reducedMotion: opts.reducedMotion } : {}),
    ...(width < 500 ? { hasTouch: true } : {}),
  });
  if (opts.light) {
    // A light pack on :root from the first frame: what the console has once an
    // admin's light appearance is applied. Skeleton and form read the same tokens.
    await context.addInitScript(() => {
      const light = (): void => { const root = document.documentElement; if (root) { root.setAttribute('data-theme', 'light'); root.setAttribute('data-appearance', 'light'); } };
      light();
      document.addEventListener('readystatechange', light);
    });
  }
  await context.addInitScript(() => {
    (window as unknown as { __csp: string[] }).__csp = [];
    document.addEventListener('securitypolicyviolation', (e) => (window as unknown as { __csp: string[] }).__csp.push(`${e.violatedDirective} ${e.blockedURI}`));
  });
  const page = await context.newPage();
  const csp: string[] = [];
  page.on('console', (m) => { if (/content.security.policy/i.test(m.text())) csp.push(m.text()); });
  return { context, page, csp };
}

async function expectNoCsp(page: Page, csp: string[]): Promise<void> {
  expect(csp, 'CSP console messages').toEqual([]);
  expect(await page.evaluate(() => (window as unknown as { __csp: string[] }).__csp), 'securitypolicyviolation events').toEqual([]);
}

/** Hold one request back until `release()` is called. */
async function hold(page: Page, pattern: string): Promise<() => void> {
  let release!: () => void;
  const gate = new Promise<void>((resolve) => { release = resolve; });
  await page.route(pattern, async (route) => { await gate; await route.continue(); });
  return () => release();
}

async function signIn(page: Page): Promise<void> {
  await page.getByLabel('Admin username').fill(USER);
  await page.getByLabel('Password', { exact: true }).fill(PASSWORD);
  const login = page.waitForResponse((r) => r.url().endsWith('/admin/login') && r.request().method() === 'POST');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  expect((await login).status()).toBe(200);
}

for (const width of WIDTHS) {
  test(`sign-in skeleton has the form's box, first visit and remembered (${width})`, async ({ browser }) => {
    const { context, page, csp } = await open(browser, width);
    try {
      let release = await hold(page, '**/admin/session');
      await page.goto(ADMIN);
      expect(await page.evaluate(() => document.documentElement.getAttribute('data-mw-boot'))).toBe('admin');
      const skeleton = page.locator('#mw-boot .mwb-admin-card');
      await expect(skeleton).toBeVisible();
      await page.waitForTimeout(400); // past the 150 ms appear delay
      const skeletonBox = await skeleton.boundingBox();
      await shot(page, `login-skeleton-${width}`);
      release();
      await expect(page.locator('#mw-boot')).toHaveCount(0);
      const form = page.getByTestId('admin-login-card');
      await expect(form).toBeVisible();
      expectSameBox(await form.boundingBox(), skeletonBox, 'first visit');
      await shot(page, `login-form-${width}`);
      expect(await page.evaluate(() => document.documentElement.hasAttribute('data-mw-boot'))).toBe(false);
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);

      // Second visit: the height measured on the first one.
      await page.unroute('**/admin/session');
      release = await hold(page, '**/admin/session');
      await page.reload();
      await expect(skeleton).toBeVisible();
      expect(await page.evaluate(() => getComputedStyle(document.documentElement).getPropertyValue('--mwb-admin-card'))).not.toBe('');
      const remembered = await skeleton.boundingBox();
      release();
      await expect(page.locator('#mw-boot')).toHaveCount(0);
      expectSameBox(await form.boundingBox(), remembered, 'remembered height');
      await expectNoCsp(page, csp);
    } finally {
      await context.close();
    }
  });

  test(`sign-in skeleton under a light pack and with reduced motion (${width})`, async ({ browser }) => {
    const { context, page, csp } = await open(browser, width, { light: true, reducedMotion: 'reduce' });
    try {
      const release = await hold(page, '**/admin/session');
      await page.goto(ADMIN);
      const skeleton = page.locator('#mw-boot .mwb-admin-card');
      await expect(skeleton).toBeVisible();
      await page.waitForTimeout(400);
      const look = await skeleton.evaluate((card) => {
        const bar = card.querySelector('.mwb-block')!;
        return { bg: getComputedStyle(card).backgroundColor, page: getComputedStyle(document.getElementById('mw-boot')!).backgroundColor, pulse: getComputedStyle(bar).animationName };
      });
      expect(look.pulse, 'no pulse with reduced motion').toBe('none');
      const box = await skeleton.boundingBox();
      await shot(page, `login-skeleton-light-${width}`);
      release();
      // Reduced motion: no fade, the placeholder goes at once.
      await expect(page.locator('#mw-boot')).toHaveCount(0);
      const form = page.getByTestId('admin-login-card');
      expectSameBox(await form.boundingBox(), box, 'light + reduced motion');
      const real = await form.evaluate((f) => ({ bg: getComputedStyle(f).backgroundColor, page: getComputedStyle(document.body).backgroundColor }));
      expect(look.bg, 'card surface').toBe(real.bg);
      expect(look.page, 'page background').toBe(real.page);
      const lightness = (rgb: string): number => (rgb.match(/\d+/g) ?? []).slice(0, 3).reduce((sum, n) => sum + Number(n), 0);
      expect(lightness(real.bg), `a light surface: ${real.bg}`).toBeGreaterThan(600);
      expect(lightness(look.bg), `a light skeleton card: ${look.bg}`).toBeGreaterThan(600);
      await shot(page, `login-form-light-${width}`);
      await expectNoCsp(page, csp);
    } finally {
      await context.close();
    }
  });

  test(`after sign-in the frame stays mounted while the roles and the dashboard load (${width})`, async ({ browser }) => {
    const { context, page, csp } = await open(browser, width);
    try {
      await page.goto(ADMIN);
      await expect(page.getByTestId('admin-login-card')).toBeVisible();
      await expect(page.locator('#mw-boot')).toHaveCount(0);
      const releaseMe = await hold(page, '**/admin/me');
      const releaseDash = await hold(page, '**/admin/dashboard');
      await signIn(page);
      const frame = page.locator('[data-screen="admin"]');
      await expect(frame).toBeVisible();
      const pageSkeleton = page.getByTestId('admin-page-skeleton');
      await expect(pageSkeleton).toBeVisible();
      const bar = width < 761 ? page.locator('[data-screen="admin"] header') : page.locator('#admin-sidebar');
      await expect(bar).toBeVisible();
      if (width >= 761) await expect(page.getByTestId('admin-nav-skeleton')).toBeVisible();
      await expect(page.locator('#admin-sidebar nav a')).toHaveCount(0);
      await page.waitForTimeout(300);
      const barBox = await bar.boundingBox();
      const headBox = await page.locator('#admin-main > div').first().boundingBox();
      const skeletonBox = await pageSkeleton.boundingBox();
      await shot(page, `console-roles-loading-${width}`);

      releaseMe();
      // The roles arrive: the navigation fills in, and the same placeholder page
      // stays while the dashboard's data loads (the section's Suspense fallback).
      if (width >= 761) await expect(page.locator('#admin-sidebar nav a').first()).toBeVisible();
      await expect(page.getByTestId('admin-nav-skeleton')).toHaveCount(0);
      await expect(pageSkeleton).toBeVisible();
      await page.waitForTimeout(200);
      await shot(page, `console-dashboard-loading-${width}`);
      expectSameBox(await bar.boundingBox(), barBox, 'sidebar / top bar');
      expectSameBox(await page.locator('#admin-main > div').first().boundingBox(), headBox, 'main head row');
      expectSameBox(await pageSkeleton.boundingBox(), skeletonBox, 'placeholder page');
      const heading = await pageSkeleton.locator('.mwb-console-page-head').boundingBox();

      releaseDash();
      await expect(page.getByTestId('tile-users')).toBeVisible();
      await expect(pageSkeleton).toHaveCount(0);
      const dash = page.locator('section[aria-labelledby="admin-dash-title"]');
      const dashBox = await dash.boundingBox();
      expect(Math.abs(dashBox!.y - skeletonBox!.y), 'page top').toBeLessThanOrEqual(TOLERANCE);
      expect(Math.abs(dashBox!.x - skeletonBox!.x), 'page start').toBeLessThanOrEqual(TOLERANCE);
      const realHead = await dash.locator('h2').locator('..').boundingBox();
      expectSameBox(realHead && { ...realHead, width: heading!.width }, heading, 'page heading row');
      expectSameBox(await bar.boundingBox(), barBox, 'sidebar / top bar after the dashboard');
      expectSameBox(await page.locator('#admin-main > div').first().boundingBox(), headBox, 'main head row after the dashboard');
      await shot(page, `console-dashboard-${width}`);
      await expectNoCsp(page, csp);
    } finally {
      await context.close();
    }
  });

  test(`a reload with a session draws the console frame, not the sign-in card (${width})`, async ({ browser }) => {
    const { context, page, csp } = await open(browser, width);
    try {
      await page.goto(ADMIN);
      await signIn(page);
      await expect(page.getByTestId('tile-users')).toBeVisible();
      const release = await hold(page, '**/admin/session');
      await page.reload();
      expect(await page.evaluate(() => document.documentElement.getAttribute('data-mw-boot'))).toBe('admin-console');
      await expect(page.locator('#mw-boot .mwb-console')).toBeVisible();
      await expect(page.locator('#mw-boot .mwb-admin')).toBeHidden();
      await page.waitForTimeout(400);
      const staticBar = await page.locator(width < 761 ? '#mw-boot .mwb-console-top' : '#mw-boot .mwb-console-side').boundingBox();
      await shot(page, `console-boot-${width}`);
      release();
      await expect(page.locator('#mw-boot')).toHaveCount(0);
      const bar = page.locator(width < 761 ? '[data-screen="admin"] header' : '#admin-sidebar');
      const realBar = await bar.boundingBox();
      expect(Math.abs(realBar!.width - staticBar!.width), 'bar width').toBeLessThanOrEqual(TOLERANCE);
      expect(Math.abs(realBar!.height - staticBar!.height), 'bar height').toBeLessThanOrEqual(TOLERANCE);
      await expect(page.getByTestId('tile-users')).toBeVisible();

      expect(await page.evaluate(() => localStorage.getItem('mw.boot.admin'))).toBe('1');
      await expectNoCsp(page, csp);

      // Signed out: the sign-in form forgets the session, so the next boot is the card.
      if (width >= 761) {
        await page.locator('#admin-sidebar').getByRole('button', { name: 'Sign out' }).click();
        await expect(page.getByTestId('admin-login-card')).toBeVisible();
        expect(await page.evaluate(() => localStorage.getItem('mw.boot.admin'))).toBeNull();
        await page.reload();
        // 'admin' while the card is up; null once the form has taken over.
        expect(await page.evaluate(() => document.documentElement.getAttribute('data-mw-boot') ?? '')).toMatch(/^(admin)?$/);
      }
    } finally {
      await context.close();
    }
  });
}
