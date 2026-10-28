import { readFileSync } from 'node:fs';
import { test, expect, type Locator, type Page } from '@playwright/test';

/**
 * Sign-in card size on the mail client and the admin console (t32-h14).
 *
 * The user asked: "why is the login box for admin so wide can be make both admin and
 * client be normal sized". Both sign-in screens now show the same card: at most
 * AUTH_CARD_MAX_WIDTH (`src/styles/authCard.css.ts`, 360 px) wide, centred, with a
 * 16 px gutter on each side on a phone. Checked on the real build under the shell CSP, at 1440 and
 * 390 px, in the light and dark packs, right-to-left and with forced colours:
 *
 *   - the mail sign-in card, with single-sign-on buttons, and its second-factor step;
 *   - the admin sign-in card and its TOTP step (the server's challenge is stubbed, so
 *     no admin account with TOTP is needed).
 *
 * Each card must be the full maximum width at 1440 (so both screens are the same size),
 * never wider than it, centred, and the second step must keep the first step's width.
 * Runs in the `engine` project; needs no admin credentials (the form renders without).
 *
 * `T32_H14_SHOT_DIR` saves a screenshot of every case into that directory.
 */

/** AUTH_CARD_MAX_WIDTH, read from the style module (a `.css.ts` file only runs inside the build). */
const MAX = Number(
  /AUTH_CARD_MAX_WIDTH = '(\d+)px'/.exec(
    readFileSync(new URL('../src/styles/authCard.css.ts', import.meta.url), 'utf8'),
  )?.[1] ?? 'NaN',
);
const GUTTER = 16;
const TOL = 0.5;
// No service worker: the mail sign-in test answers `/api/sso/providers` with
// `page.route`, and once the worker (public/sw.js) controls the page it fetches
// that public endpoint itself, where `page.route` never sees it, so the card
// showed the stack's real (empty) provider list instead of the stubbed one.
test.use({ serviceWorkers: 'block' });

const SHOT_DIR = process.env['T32_H14_SHOT_DIR'];

interface Look {
  name: string;
  theme: string;
  rtl?: boolean;
  forced?: boolean;
}

const LOOKS: Look[] = [
  { name: 'light', theme: 'light' },
  { name: 'dark', theme: 'dark' },
  { name: 'rtl', theme: 'dark', rtl: true },
  { name: 'forced', theme: 'light', forced: true },
];

const WIDTHS = [1440, 390];

async function prepare(page: Page, look: Look, width: number): Promise<void> {
  await page.setViewportSize({ width, height: width > 600 ? 900 : 844 });
  if (look.forced) await page.emulateMedia({ forcedColors: 'active' });
  await page.addInitScript(
    ([th]) => {
      try {
        localStorage.setItem('mw.theme.prefs', JSON.stringify({ mode: 'fixed', theme: th, chosen: true }));
      } catch {
        /* storage unavailable: the data-theme attribute set after load still applies the pack */
      }
    },
    [look.theme] as const,
  );
}

async function applyLook(page: Page, look: Look): Promise<void> {
  await page.evaluate((l) => {
    const root = document.documentElement;
    root.setAttribute('data-theme', l.theme);
    if (l.rtl) {
      root.setAttribute('dir', 'rtl');
      root.setAttribute('lang', 'ar');
    }
  }, look);
  await page.waitForTimeout(100);
}

/** The sign-in card: the only element child of the page's main region. */
function card(page: Page): Locator {
  return page.getByRole('main').locator(':scope > *').first();
}

interface Box {
  width: number;
  start: number;
  end: number;
  viewport: number;
  scrollWidth: number;
}

async function measure(page: Page): Promise<Box> {
  const el = card(page);
  await expect(el).toBeVisible();
  return el.evaluate((node) => {
    const r = node.getBoundingClientRect();
    const vw = document.documentElement.clientWidth;
    return {
      width: r.width,
      start: r.left,
      end: vw - r.right,
      viewport: vw,
      scrollWidth: document.documentElement.scrollWidth,
    };
  });
}

function expectSized(box: Box, width: number, what: string): void {
  expect(box.width, `${what}: card width`).toBeLessThanOrEqual(MAX + TOL);
  expect(Math.abs(box.start - box.end), `${what}: centred (left ${box.start}, right ${box.end})`).toBeLessThanOrEqual(1);
  expect(box.scrollWidth, `${what}: no sideways scroll`).toBeLessThanOrEqual(box.viewport);
  if (width >= MAX + 2 * GUTTER + 40) {
    // Room to spare: the card is exactly the shared maximum, the same on both screens.
    expect(box.width, `${what}: card is the shared width`).toBeGreaterThanOrEqual(MAX - TOL);
  } else {
    expect(box.start, `${what}: start gutter`).toBeGreaterThanOrEqual(GUTTER - TOL);
    expect(box.end, `${what}: end gutter`).toBeGreaterThanOrEqual(GUTTER - TOL);
  }
}

async function shot(page: Page, name: string): Promise<void> {
  if (SHOT_DIR) await page.screenshot({ path: `${SHOT_DIR}/${name}.png` });
}

for (const width of WIDTHS) {
  for (const look of LOOKS) {
    const tag = `${width}-${look.name}`;

    test(`mail sign-in card and its second step at ${tag}`, async ({ page }) => {
      await prepare(page, look, width);
      await page.route('**/api/sso/providers', (route) =>
        route.fulfill({
          json: { providers: [{ id: 'corp', kind: 'oidc', displayName: 'Example Corporation Single Sign-On' }] },
        }),
      );
      await page.route('**/api/login', (route) =>
        route.fulfill({ json: { twofaRequired: true, pendingToken: 'pending-h14', factors: ['totp', 'recovery'] } }),
      );
      await page.goto('/');
      const form = page.getByRole('form', { name: 'Sign in' });
      await expect(form).toBeVisible();
      await expect(form.locator('[data-sso-id="corp"]')).toBeVisible();
      await applyLook(page, look);
      const first = await measure(page);
      expectSized(first, width, 'mail sign-in');
      await shot(page, `mail-${tag}`);

      await form.getByRole('textbox', { name: 'Email or username' }).fill('testuser@example.org');
      await form.getByLabel('Password', { exact: true }).fill('not-checked');
      await form.getByRole('button', { name: 'Sign in', exact: true }).click();
      await expect(form).toBeHidden();
      const second = await measure(page);
      expectSized(second, width, 'mail second step');
      expect(Math.abs(second.width - first.width), 'mail second step keeps the width').toBeLessThanOrEqual(1);
      await shot(page, `mail-2fa-${tag}`);
    });

    test(`admin sign-in card and its TOTP step at ${tag}`, async ({ page }) => {
      await prepare(page, look, width);
      await page.route('**/admin/login', (route) =>
        route.request().method() === 'POST'
          ? route.fulfill({ json: { twofaRequired: true, pendingToken: 'pending-h14', factors: ['totp'] } })
          : route.continue(),
      );
      await page.goto('/admin');
      const form = page.getByRole('main').getByRole('form');
      await expect(form.getByRole('textbox').first()).toBeVisible();
      await applyLook(page, look);
      const first = await measure(page);
      expectSized(first, width, 'admin sign-in');
      await shot(page, `admin-${tag}`);

      await form.getByRole('textbox').nth(0).fill('root');
      await form.locator('input[type="password"]').fill('not-checked');
      await page.getByRole('button', { name: 'Sign in', exact: true }).click();
      await expect(page.locator('input[autocomplete="one-time-code"]')).toBeVisible();
      const second = await measure(page);
      expectSized(second, width, 'admin TOTP step');
      expect(Math.abs(second.width - first.width), 'admin TOTP step keeps the width').toBeLessThanOrEqual(1);
      await shot(page, `admin-totp-${tag}`);
    });
  }
}
