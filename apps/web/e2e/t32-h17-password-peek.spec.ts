import AxeBuilder from '@axe-core/playwright';
import { test, expect, type Locator, type Page } from '@playwright/test';

/**
 * Show/hide button on the password field of both sign-in screens (t32-h17).
 *
 * The user asked: "add capability to peek password on the login too". The mail
 * client's sign-in and the admin console's sign-in each get a button at the end of
 * the password field (components/PasswordInput.tsx). On the production build under
 * the shell CSP, for each screen:
 *
 *   - the button exists, named "Show password", `aria-pressed`, `aria-controls` = the field;
 *   - pressing it shows the typed text, pressing again hides it; a pointer press leaves
 *     focus and the caret in the field;
 *   - Enter in the field still submits the form, and the button does not submit;
 *   - after a refused sign-in the field is hidden again;
 *   - Tab goes field -> button -> the next control;
 *   - axe finds nothing on the form (including target-size);
 *   - forced colours: the button keeps a visible glyph; right-to-left: it sits at the
 *     inline end (the left) and the text never runs under it; at 390 px it stays inside
 *     the field and the field is no taller than the input;
 *   - no Content-Security-Policy violation.
 *
 * The sign-in answers are stubbed (a 401 for each submit), so the cases need no
 * account and no admin credentials and run against any stack. With MW_ADMIN_USER set,
 * one admin case also signs in with a wrong password against the real server.
 * Runs in the `engine` project.
 */

interface Screen {
  name: string;
  path: string;
  form: string;
  loginRoute: string;
  /** Fill whatever else the form needs before it can be submitted. */
  prefill: (form: Locator) => Promise<void>;
  /** The control after the toggle in Tab order. */
  next: (form: Locator) => Locator;
}

const SCREENS: Screen[] = [
  {
    name: 'mail',
    path: '/',
    form: 'Sign in',
    loginRoute: '**/api/login',
    prefill: (form) => form.getByLabel('Email address').fill('ada@example.org'),
    next: (form) => form.getByRole('button', { name: 'Sign in', exact: true }),
  },
  {
    name: 'admin',
    path: '/admin',
    form: 'Admin sign in',
    loginRoute: '**/admin/login',
    prefill: (form) => form.getByLabel('Username').fill('root'),
    next: (form) => form.getByRole('button', { name: 'Sign in', exact: true }),
  },
];

/** CSP violations reported by the page (event) or the console. */
function watchCsp(page: Page): string[] {
  const seen: string[] = [];
  page.on('console', (m) => {
    const text = m.text();
    if (/Content.Security.Policy/i.test(text)) seen.push(text.slice(0, 200));
  });
  return seen;
}

async function cspEvents(page: Page): Promise<string[]> {
  return page.evaluate(() => (window as unknown as { __csp?: string[] }).__csp ?? []);
}

/** Stub the sign-in endpoint with a 401; returns the count of submits seen. */
async function refuseSignIn(page: Page, route: string): Promise<{ count: () => number }> {
  let n = 0;
  await page.route(route, async (r) => {
    if (r.request().method() !== 'POST') return r.fallback();
    n++;
    await r.fulfill({ status: 401, contentType: 'application/json', body: '{"error":"unauthorized"}' });
  });
  return { count: () => n };
}

async function open(page: Page, s: Screen): Promise<{ form: Locator; field: Locator; toggle: Locator }> {
  await page.addInitScript(() => {
    const w = window as unknown as { __csp: string[] };
    w.__csp = [];
    document.addEventListener('securitypolicyviolation', (e) => {
      w.__csp.push(`${e.violatedDirective} ${e.blockedURI}`);
    });
  });
  await page.goto(s.path);
  const form = page.getByRole('form', { name: s.form });
  await expect(form).toBeVisible();
  const field = form.getByLabel('Password', { exact: true });
  const toggle = form.getByRole('button', { name: 'Show password' });
  return { form, field, toggle };
}

for (const s of SCREENS) {
  test.describe(`${s.name} sign-in password field`, () => {
    test('has a show/hide toggle that names and controls the field', async ({ page }) => {
      const csp = watchCsp(page);
      const { field, toggle } = await open(page, s);
      await expect(toggle).toHaveCount(1);
      await expect(toggle).toHaveAttribute('type', 'button');
      await expect(toggle).toHaveAttribute('aria-pressed', 'false');
      const id = await field.getAttribute('id');
      expect(id, 'the field has an id').toBeTruthy();
      await expect(toggle).toHaveAttribute('aria-controls', id!);
      await expect(field).toHaveAttribute('type', 'password');
      await expect(field).toHaveAttribute('autocomplete', 'current-password');
      expect(csp).toEqual([]);
      expect(await cspEvents(page)).toEqual([]);
    });

    test('shows the typed text and hides it again; focus and caret stay in the field', async ({ page }) => {
      const { field, toggle } = await open(page, s);
      await field.fill('abcdef');
      await field.press('Home');
      await field.press('ArrowRight');
      await field.press('ArrowRight');
      await toggle.click();
      await expect(field).toHaveAttribute('type', 'text');
      await expect(toggle).toHaveAttribute('aria-pressed', 'true');
      await expect(field).toHaveValue('abcdef');
      await expect(field).toBeFocused();
      // The caret is still after "ab".
      await page.keyboard.type('X');
      await expect(field).toHaveValue('abXcdef');
      await expect(field).toHaveAttribute('autocapitalize', 'off');
      await expect(field).toHaveAttribute('spellcheck', 'false');
      await toggle.click();
      await expect(field).toHaveAttribute('type', 'password');
      await expect(toggle).toHaveAttribute('aria-pressed', 'false');
      await expect(field).toBeFocused();
    });

    test('Enter in the field submits, the toggle does not, and a refused sign-in hides the field again', async ({
      page,
    }) => {
      const csp = watchCsp(page);
      const { form, field, toggle } = await open(page, s);
      const stub = await refuseSignIn(page, s.loginRoute);
      await s.prefill(form);
      await field.fill('not-the-password');

      // Pointer, Enter and Space: three presses, so the field ends up shown.
      await toggle.click();
      await toggle.press('Enter');
      await toggle.press('Space');
      await page.waitForTimeout(300);
      expect(stub.count(), 'pressing the toggle sends no sign-in').toBe(0);
      await expect(field).toHaveAttribute('type', 'text');

      await field.press('Enter');
      await expect.poll(stub.count, { message: 'Enter in the field submits' }).toBe(1);
      await expect(form.getByRole('alert')).toBeVisible();
      await expect(field).toHaveAttribute('type', 'password');
      await expect(toggle).toHaveAttribute('aria-pressed', 'false');

      // Shown again while the error is on screen, then refused a second time.
      await toggle.click();
      await expect(field).toHaveAttribute('type', 'text');
      await field.press('Enter');
      await expect.poll(stub.count).toBe(2);
      await expect(field).toHaveAttribute('type', 'password');
      expect(csp).toEqual([]);
      expect(await cspEvents(page)).toEqual([]);
    });

    test('Tab goes field, toggle, next control', async ({ page }) => {
      const { form, field, toggle } = await open(page, s);
      await field.focus();
      await page.keyboard.press('Tab');
      await expect(toggle).toBeFocused();
      // The global :focus-visible outline marks it.
      const outline = await toggle.evaluate((el) => getComputedStyle(el).outlineStyle);
      expect(outline).not.toBe('none');
      await page.keyboard.press('Tab');
      await expect(s.next(form)).toBeFocused();
      await page.keyboard.press('Shift+Tab');
      await expect(toggle).toBeFocused();
      await page.keyboard.press('Shift+Tab');
      await expect(field).toBeFocused();
    });

    test('axe finds nothing on the form, including target-size', async ({ page }) => {
      const { form, toggle } = await open(page, s);
      await form.evaluate((el) => el.setAttribute('data-h17', ''));
      for (const pressed of [false, true]) {
        if (pressed) await toggle.click();
        const res = await new AxeBuilder({ page })
          .include('[data-h17]')
          .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'])
          .analyze();
        expect(res.violations.map((v) => `${v.id}: ${v.nodes.map((n) => n.html.slice(0, 100)).join(' | ')}`)).toEqual(
          [],
        );
      }
      const box = (await toggle.boundingBox())!;
      expect(box.width).toBeGreaterThanOrEqual(24);
      expect(box.height).toBeGreaterThanOrEqual(24);
    });

    for (const look of [
      { tag: 'forced colours', forced: true, rtl: false, width: 1280 },
      { tag: 'right-to-left', forced: false, rtl: true, width: 1280 },
      { tag: '390 px', forced: false, rtl: false, width: 390 },
      { tag: '390 px right-to-left', forced: false, rtl: true, width: 390 },
    ]) {
      test(`the toggle sits inside the field at the inline end (${look.tag})`, async ({ page }) => {
        const csp = watchCsp(page);
        await page.setViewportSize({ width: look.width, height: 844 });
        if (look.forced) await page.emulateMedia({ forcedColors: 'active' });
        const { field, toggle } = await open(page, s);
        if (look.rtl) {
          await page.evaluate(() => {
            document.documentElement.setAttribute('dir', 'rtl');
            document.documentElement.setAttribute('lang', 'ar');
          });
        }
        await field.fill('a long password typed all the way to the end of the field');
        const f = (await field.boundingBox())!;
        const b = (await toggle.boundingBox())!;
        // Inside the field's box, vertically and horizontally.
        expect(b.y).toBeGreaterThanOrEqual(f.y);
        expect(b.y + b.height).toBeLessThanOrEqual(f.y + f.height + 0.5);
        expect(b.x).toBeGreaterThanOrEqual(f.x);
        expect(b.x + b.width).toBeLessThanOrEqual(f.x + f.width + 0.5);
        // At the inline end: the right in LTR, the left in RTL.
        const centre = b.x + b.width / 2;
        if (look.rtl) expect(centre).toBeLessThan(f.x + f.width / 2);
        else expect(centre).toBeGreaterThan(f.x + f.width / 2);
        // The text stops before the button: the end padding covers the button and its inset.
        const pad = await field.evaluate((el) => {
          const cs = getComputedStyle(el);
          return parseFloat(cs.direction === 'rtl' ? cs.paddingLeft : cs.paddingRight);
        });
        const inset = look.rtl ? b.x - f.x : f.x + f.width - (b.x + b.width);
        expect(pad).toBeGreaterThanOrEqual(b.width + inset);
        // No box grows: the wrapper is exactly as tall as the input.
        const wrapH = await field.evaluate((el) => el.parentElement!.getBoundingClientRect().height);
        expect(Math.abs(wrapH - f.height)).toBeLessThan(0.5);
        // The card stays within the 360 px sign-in card and the page does not scroll sideways.
        expect(f.width).toBeLessThanOrEqual(360);
        const scrollW = await page.evaluate(() => document.documentElement.scrollWidth);
        expect(scrollW).toBeLessThanOrEqual(look.width);
        // The glyph is painted in a colour other than the field behind it.
        const paint = await toggle.evaluate((el) => {
          const input = el.previousElementSibling as HTMLElement;
          return { fg: getComputedStyle(el).color, bg: getComputedStyle(input).backgroundColor };
        });
        expect(paint.fg).not.toBe(paint.bg);
        expect(paint.fg).not.toBe('rgba(0, 0, 0, 0)');
        if (look.forced) {
          // Forced colours paint the transparent border too: the button has a visible edge.
          const border = await toggle.evaluate((el) => getComputedStyle(el).borderTopColor);
          expect(border).not.toBe('rgba(0, 0, 0, 0)');
        }
        await toggle.click();
        await expect(field).toHaveAttribute('type', 'text');
        expect(csp).toEqual([]);
        expect(await cspEvents(page)).toEqual([]);
      });
    }
  });
}

test('admin: a wrong password against the real server leaves the field hidden', async ({ page }) => {
  test.skip(!process.env['MW_ADMIN_USER'], 'needs the stack admin user (MW_ADMIN_USER)');
  const s = SCREENS[1]!;
  const { form, field, toggle } = await open(page, s);
  await form.getByLabel('Username').fill(process.env['MW_ADMIN_USER']!);
  await field.fill('definitely-not-the-password');
  await toggle.click();
  await expect(field).toHaveAttribute('type', 'text');
  const answer = page.waitForResponse((r) => r.url().endsWith('/admin/login') && r.request().method() === 'POST');
  await field.press('Enter');
  expect([401, 429]).toContain((await answer).status());
  await expect(form.getByRole('alert')).toBeVisible();
  await expect(field).toHaveAttribute('type', 'password');
  await expect(toggle).toHaveAttribute('aria-pressed', 'false');
});
