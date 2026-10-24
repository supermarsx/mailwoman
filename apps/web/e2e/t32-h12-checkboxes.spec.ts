import AxeBuilder from '@axe-core/playwright';
import { test, expect, type BrowserContext, type Locator, type Page } from '@playwright/test';
import { engineLogin, enginePhoneLogin, drawerButton } from './helpers.ts';

/**
 * Checkboxes and radios across the web client and the admin console (t32-h12).
 *
 * The user report was "tickbox elements are fully borked still". Each control below is
 * checked on the real build, under the shell CSP, for what a person sees and does:
 *
 *   - it is on screen with the 24×24 box every checkbox shares (WCAG 2.2 §2.5.8 target);
 *   - its label text sits on the same line as the box (not under it, not across the row);
 *   - clicking the box toggles it, and so does clicking its label text, when it has one;
 *   - the checked box looks different from the unchecked one (pixels), in the accent colour;
 *   - keyboard focus draws a ring around it (pixels outside the box change);
 *   - axe `target-size` finds no checkbox or radio too small;
 *   - the browser logs no Content Security Policy violation for inline styles on the screen.
 *
 * The appearance block repeats the visual checks on a few controls in other themes, two
 * accents, forced colours, right-to-left and at 390 px.
 *
 * Runs in the `engine` project. The admin console cases need MW_ADMIN_USER and
 * MW_ADMIN_PASSWORD (the restyle stack, `e2e/restyle/stack.sh`, sets both); without them
 * they are skipped and say so.
 */

const BOX = 24;
const ADMIN_USER = process.env['MW_ADMIN_USER'];
const ADMIN_PASSWORD = process.env['MW_ADMIN_PASSWORD'];

// ── page set-up ─────────────────────────────────────────────────────────────

/** Inline-style CSP violations the browser reports, collected per page. */
function watchCsp(page: Page): string[] {
  const seen: string[] = [];
  page.on('console', (m) => {
    const text = m.text();
    // Chromium: "… violates the following Content Security Policy directive 'style-src …'";
    // Firefox: "Content-Security-Policy: The page's settings blocked an inline style (style-src-attr) …".
    if (/Content.Security.Policy/i.test(text) && /style-src/i.test(text)) seen.push(text.slice(0, 200));
  });
  return seen;
}

async function seedTheme(page: Page, theme: string): Promise<void> {
  await page.addInitScript(
    ([th]) => {
      try {
        localStorage.setItem('mw.theme.prefs', JSON.stringify({ mode: 'fixed', theme: th, chosen: true }));
      } catch {
        /* storage unavailable: the data-theme attribute below still applies the pack */
      }
    },
    [theme] as const,
  );
}

/** Apply a theme pack and accent on the open page, as the appearance settings do. */
async function applyLook(page: Page, look: Look): Promise<void> {
  await page.evaluate((l) => {
    const root = document.documentElement;
    root.setAttribute('data-theme', l.theme);
    if (l.accent) root.setAttribute('data-accent', l.accent);
    else root.removeAttribute('data-accent');
    if (l.rtl) {
      root.setAttribute('dir', 'rtl');
      root.setAttribute('lang', 'ar');
    }
  }, look);
  await page.waitForTimeout(100);
}

async function adminSignIn(page: Page): Promise<void> {
  await page.goto('/admin');
  const main = page.getByRole('main');
  await main.getByRole('textbox').nth(0).fill(ADMIN_USER!);
  await main.getByRole('textbox').nth(1).fill(ADMIN_PASSWORD!);
  await page.getByRole('button', { name: /sign in/i }).click();
  await expect(page.getByRole('navigation').first()).toBeVisible();
}

async function adminSection(page: Page, name: string): Promise<void> {
  const link = page.getByRole('navigation').first().getByRole('link', { name, exact: true }).first();
  await link.click();
  await expect(link).toHaveAttribute('aria-current', 'page');
  await page.waitForTimeout(500);
}

// ── pixels ──────────────────────────────────────────────────────────────────

/** Share of pixels that differ between two same-size PNG screenshots (decoded in a blank page). */
async function pixelDiff(context: BrowserContext, a: Buffer, b: Buffer): Promise<number> {
  const blank = await context.newPage();
  try {
    return await blank.evaluate(
      async ([pa, pb]) => {
        const load = (src: string) =>
          new Promise<HTMLImageElement>((resolve, reject) => {
            const img = new Image();
            img.onload = () => resolve(img);
            img.onerror = reject;
            img.src = `data:image/png;base64,${src}`;
          });
        const [ia, ib] = await Promise.all([load(pa), load(pb)]);
        const w = Math.min(ia.width, ib.width);
        const h = Math.min(ia.height, ib.height);
        const read = (img: HTMLImageElement) => {
          const c = document.createElement('canvas');
          c.width = w;
          c.height = h;
          const g = c.getContext('2d')!;
          g.drawImage(img, 0, 0);
          return g.getImageData(0, 0, w, h).data;
        };
        const da = read(ia);
        const db = read(ib);
        let diff = 0;
        for (let i = 0; i < da.length; i += 4) {
          const d = Math.abs(da[i]! - db[i]!) + Math.abs(da[i + 1]! - db[i + 1]!) + Math.abs(da[i + 2]! - db[i + 2]!);
          if (d > 48) diff++;
        }
        return diff / (w * h);
      },
      [a.toString('base64'), b.toString('base64')] as const,
    );
  } finally {
    await blank.close();
  }
}

// ── the per-control check ───────────────────────────────────────────────────

interface Geometry {
  box: { x: number; y: number; w: number; h: number };
  checked: boolean;
  disabled: boolean;
  /** Bounding box of the label's own text (not the input), or null when the label has none. */
  text: { x: number; y: number; w: number; h: number } | null;
  hasLabel: boolean;
  /** The label's box, and whether it draws a frame (border or fill) a person sees as a bar. */
  label: { x: number; w: number; framed: boolean } | null;
  accent: string;
  rootAccent: string;
}

async function geometry(control: Locator): Promise<Geometry> {
  return control.evaluate((el) => {
    const input = el as HTMLInputElement;
    const r = input.getBoundingClientRect();
    const label = input.labels?.[0] ?? null;
    let text: Geometry['text'] = null;
    if (label) {
      // The label's text nodes outside the input itself.
      const walker = document.createTreeWalker(label, NodeFilter.SHOW_TEXT);
      const rects: DOMRect[] = [];
      for (let n = walker.nextNode(); n; n = walker.nextNode()) {
        if (!n.textContent?.trim()) continue;
        const range = document.createRange();
        range.selectNodeContents(n);
        for (const rect of Array.from(range.getClientRects())) if (rect.width > 0) rects.push(rect);
      }
      if (rects.length) {
        // First line of text: the rect nearest the top.
        const first = rects.reduce((a, b) => (b.top < a.top ? b : a));
        // The text's full width on the first line, from its leftmost to its rightmost rect.
        const line = rects.filter((rect) => Math.abs(rect.top - first.top) < 2);
        const left = Math.min(...line.map((rect) => rect.left));
        const right = Math.max(...line.map((rect) => rect.right));
        text = { x: left, y: first.top, w: right - left, h: first.height };
      }
    }
    let labelBox: Geometry['label'] = null;
    if (label) {
      const lr = label.getBoundingClientRect();
      const lcs = getComputedStyle(label);
      // A fill only frames the label when it differs from what is behind it (under forced
      // colours every element reports the same Canvas fill).
      const clear = (c: string) => c === 'rgba(0, 0, 0, 0)' || c === 'transparent';
      let behind = 'rgba(0, 0, 0, 0)';
      for (let p = label.parentElement; p && clear(behind); p = p.parentElement) behind = getComputedStyle(p).backgroundColor;
      const fill = lcs.backgroundColor;
      labelBox = {
        x: lr.left,
        w: lr.width,
        framed: parseFloat(lcs.borderTopWidth) > 0 || (!clear(fill) && fill !== behind),
      };
    }
    // Resolve the theme accent the same way the browser resolves accent-color.
    const probe = document.createElement('span');
    probe.style.color = getComputedStyle(document.documentElement).getPropertyValue('--accent').trim();
    document.body.appendChild(probe);
    const rootAccent = getComputedStyle(probe).color;
    probe.remove();
    return {
      box: { x: r.x, y: r.y, w: r.width, h: r.height },
      checked: input.checked,
      disabled: input.disabled,
      text,
      hasLabel: label !== null,
      label: labelBox,
      accent: getComputedStyle(input).accentColor,
      rootAccent,
    };
  });
}

interface CheckOpts {
  /** Toggle by mouse and keyboard (false for controls whose change would sign someone out). */
  toggle?: boolean;
  /** Expect a label with its own text that toggles the control. */
  labelText?: boolean;
  /** Forced colours: the system draws the box, so the accent is not compared. */
  forced?: boolean;
}

/** Everything a person sees and does with one checkbox; returns problems as text. */
async function checkControl(page: Page, control: Locator, name: string, opts: CheckOpts = {}): Promise<string[]> {
  const problems: string[] = [];
  const say = (p: string) => problems.push(`${name}: ${p}`);
  await control.scrollIntoViewIfNeeded();
  await expect(control, `${name} visible`).toBeVisible();
  const g = await geometry(control);

  if (Math.abs(g.box.w - BOX) > 0.5 || Math.abs(g.box.h - BOX) > 0.5)
    say(`box is ${g.box.w.toFixed(1)}×${g.box.h.toFixed(1)}, expected ${BOX}×${BOX}`);

  // "A single tickbox element uses up a whole line" (user report): the box is square and no
  // wider than the 24px target, its label text starts beside it on the same row (centres
  // within 4px), and a framed label is no wider than its box and text plus 40px.
  if (Math.abs(g.box.w - g.box.h) > 1 || g.box.w > BOX + 0.5)
    say(`whole line: the box is ${g.box.w.toFixed(1)}px wide for ${g.box.h.toFixed(1)}px high`);
  if (opts.labelText !== false && g.text) {
    const centres = Math.abs(g.box.y + g.box.h / 2 - (g.text.y + g.text.h / 2));
    if (centres > 4) say(`whole line: label text centre is ${centres.toFixed(1)}px off the box's centre`);
    const beside = g.text.x >= g.box.x + g.box.w - 0.5 || g.text.x + g.text.w <= g.box.x + 0.5;
    if (!beside) say('whole line: label text is not beside the box');
    if (g.label?.framed) {
      const content = Math.max(g.text.x + g.text.w, g.box.x + g.box.w) - Math.min(g.text.x, g.box.x);
      if (g.label.w - content > 40)
        say(`whole line: the framed label is ${g.label.w.toFixed(0)}px wide for ${content.toFixed(0)}px of box and text`);
    }
  }

  if (opts.labelText !== false && g.text) {
    // The first line of label text must share the box's line: their vertical spans overlap
    // by at least half the text height, and the text starts beside the box, not under it.
    const top = Math.max(g.box.y, g.text.y);
    const bottom = Math.min(g.box.y + g.box.h, g.text.y + g.text.h);
    if (bottom - top < g.text.h / 2)
      say(`label text at y=${g.text.y.toFixed(0)} is not on the box's line (box y=${g.box.y.toFixed(0)}..${(g.box.y + g.box.h).toFixed(0)})`);
  }
  if (opts.labelText && !g.text) say('expected a label with text');

  if (!opts.forced && !g.disabled && g.accent !== g.rootAccent)
    say(`accent-color ${g.accent} is not the theme accent ${g.rootAccent}`);

  if (opts.toggle === false || g.disabled) return problems;

  // Mouse: the box toggles; the label text toggles.
  const before = g.checked;
  const shotOff = await control.screenshot();
  await control.click();
  await page.waitForTimeout(150);
  if ((await control.isChecked()) === before) say('clicking the box did not toggle it');
  const shotOn = await control.screenshot();
  const changed = await pixelDiff(page.context(), shotOff, shotOn);
  if (changed < 0.08) say(`checked and unchecked look the same (${(changed * 100).toFixed(1)}% of pixels differ)`);
  await control.click();
  await page.waitForTimeout(150);
  if ((await control.isChecked()) !== before) say('a second click did not restore the state');

  if (g.text && opts.labelText !== false) {
    const now = await geometry(control);
    if (now.text) {
      await page.mouse.click(now.text.x + Math.min(now.text.w / 2, 20), now.text.y + now.text.h / 2);
      await page.waitForTimeout(150);
      if ((await control.isChecked()) === before) say('clicking the label text did not toggle the box');
      else {
        await page.mouse.click(now.text.x + Math.min(now.text.w / 2, 20), now.text.y + now.text.h / 2);
        await page.waitForTimeout(150);
      }
    }
  }

  // Keyboard: focus, Space toggles, and a ring is drawn outside the box.
  const ringClip = async () => {
    const b = (await control.boundingBox())!;
    const m = 7;
    return page.screenshot({ clip: { x: Math.max(0, b.x - m), y: Math.max(0, b.y - m), width: b.width + 2 * m, height: b.height + 2 * m } });
  };
  // Nothing focused (a click elsewhere could close a dialog).
  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  const unfocused = await ringClip();
  // Reach it from the keyboard (Shift+Tab away, Tab back): Firefox shows :focus-visible only
  // for focus that keyboard navigation moved.
  await control.focus();
  await page.keyboard.press('Shift+Tab');
  await page.keyboard.press('Tab');
  if (!(await control.evaluate((el) => el === document.activeElement))) say('Tab does not come back to the box');
  await page.keyboard.press('Space');
  await page.waitForTimeout(100);
  await page.keyboard.press('Space');
  await page.waitForTimeout(150);
  if ((await control.isChecked()) !== before) say('Space twice did not come back to the same state');
  const focusVisible = await control.evaluate((el) => el.matches(':focus-visible'));
  if (!focusVisible) say('not :focus-visible after keyboard use');
  // Compare only the frame outside the box: the box itself changed with the toggle.
  const focused = await ringClip();
  const ring = await pixelDiff(page.context(), unfocused, focused);
  if (ring < 0.05) say(`no visible focus ring (${(ring * 100).toFixed(1)}% of pixels around the box changed)`);
  await control.blur();
  return problems;
}

/** axe target-size findings on checkboxes and radios only. */
async function targetSize(page: Page): Promise<string[]> {
  const res = await new AxeBuilder({ page }).withRules(['target-size']).analyze();
  return res.violations.flatMap((v) =>
    v.nodes.filter((n) => /type="(checkbox|radio)"/.test(n.html)).map((n) => `target-size: ${n.html.slice(0, 120)}`),
  );
}

async function closeSettings(page: Page): Promise<void> {
  const dialog = page.getByRole('dialog', { name: 'Settings' });
  await dialog.getByRole('button', { name: 'Close settings' }).click();
  await expect(dialog).toBeHidden();
}

async function settingsDialog(page: Page): Promise<Locator> {
  await page.getByRole('button', { name: 'Settings' }).first().click();
  const dialog = page.getByRole('dialog', { name: 'Settings' });
  await expect(dialog).toBeVisible();
  return dialog;
}

// ── mail client ─────────────────────────────────────────────────────────────

test.describe('checkboxes in the mail client', () => {
  test.beforeEach(async ({ page }) => {
    await seedTheme(page, 'dark');
  });

  test('settings: API key scopes and notifications', async ({ page }) => {
    const csp = watchCsp(page);
    await engineLogin(page);
    const dialog = await settingsDialog(page);
    const problems: string[] = [];
    problems.push(...(await checkControl(page, dialog.getByRole('checkbox', { name: 'Search mail' }), 'api key scope', { labelText: true })));
    problems.push(...(await checkControl(page, dialog.getByRole('checkbox', { name: 'Mail', exact: true }), 'api key surface', { labelText: true })));
    problems.push(
      ...(await checkControl(page, dialog.getByRole('checkbox', { name: 'Show notifications for new mail' }), 'notifications', {
        toggle: false,
      })),
    );
    problems.push(...(await targetSize(page)));
    expect(problems).toEqual([]);
    expect(csp, 'inline-style CSP violations').toEqual([]);
  });

  test('compose: message security options', async ({ page }) => {
    const csp = watchCsp(page);
    await engineLogin(page);
    await page.getByRole('button', { name: 'Compose' }).first().click();
    const dialog = page.getByRole('dialog', { name: 'Compose message' });
    await expect(dialog).toBeVisible();
    const problems: string[] = [];
    problems.push(...(await checkControl(page, dialog.getByTestId('sign-toggle'), 'sign', { labelText: true })));
    problems.push(...(await checkControl(page, dialog.getByTestId('encrypt-toggle'), 'encrypt (disabled)', { toggle: false })));
    problems.push(...(await targetSize(page)));
    expect(problems).toEqual([]);
    expect(csp, 'inline-style CSP violations').toEqual([]);
  });

  test('calendar: calendar list and event editor', async ({ page }) => {
    const csp = watchCsp(page);
    await engineLogin(page);
    await page.getByTestId('nav-calendar').click();
    await expect(page.locator('[data-module="calendar"]')).toBeVisible();
    const problems: string[] = [];
    problems.push(...(await checkControl(page, page.getByRole('checkbox', { name: /^Toggle / }).first(), 'calendar visibility')));
    await page.getByRole('button', { name: 'New event' }).click();
    const dialog = page.getByRole('dialog', { name: 'New event' });
    await expect(dialog).toBeVisible();
    problems.push(...(await checkControl(page, dialog.getByRole('checkbox', { name: 'All day' }), 'all day', { labelText: true })));
    problems.push(...(await checkControl(page, dialog.getByRole('checkbox', { name: 'Repeats' }), 'repeats', { labelText: true })));
    problems.push(...(await targetSize(page)));
    expect(problems).toEqual([]);
    expect(csp, 'inline-style CSP violations').toEqual([]);
  });

  test('contacts: favourite in the contact form', async ({ page }) => {
    const csp = watchCsp(page);
    await engineLogin(page);
    await page.getByTestId('nav-contacts').click();
    const region = page.getByRole('region', { name: 'Contacts' });
    await region.getByRole('button', { name: 'New contact' }).click();
    const form = region.getByRole('form', { name: 'New contact' });
    await expect(form).toBeVisible();
    const problems = await checkControl(page, form.getByRole('checkbox', { name: 'Favorite' }), 'favourite', { labelText: true });
    problems.push(...(await targetSize(page)));
    expect(problems).toEqual([]);
    expect(csp, 'inline-style CSP violations').toEqual([]);
  });

  test('keys: key lookup sources', async ({ page }) => {
    const csp = watchCsp(page);
    await engineLogin(page);
    await page.getByTestId('nav-keys').click();
    const group = page.getByRole('group', { name: /sources/i });
    await expect(group).toBeVisible();
    const problems: string[] = [];
    const boxes = group.getByRole('checkbox');
    const n = await boxes.count();
    expect(n).toBeGreaterThan(0);
    for (let i = 0; i < n; i++) problems.push(...(await checkControl(page, boxes.nth(i), `lookup source ${i}`, { labelText: true })));
    problems.push(...(await targetSize(page)));
    expect(problems).toEqual([]);
    expect(csp, 'inline-style CSP violations').toEqual([]);
  });

  test('mailbox: no inline-style CSP violations', async ({ page }) => {
    const csp = watchCsp(page);
    await engineLogin(page);
    await expect(page.locator('.list__row').first()).toBeVisible({ timeout: 45_000 });
    await page.waitForTimeout(500);
    expect(csp, 'inline-style CSP violations').toEqual([]);
  });
});

// ── admin console ───────────────────────────────────────────────────────────

test.describe('checkboxes in the admin console', () => {
  test.skip(!ADMIN_USER || !ADMIN_PASSWORD, 'MW_ADMIN_USER / MW_ADMIN_PASSWORD are not set for this stack');

  test.beforeEach(async ({ page }) => {
    await seedTheme(page, 'dark');
  });

  test('users table: zero-access, disabled and force-change', async ({ page }) => {
    const csp = watchCsp(page);
    await adminSignIn(page);
    await adminSection(page, 'Users');
    const main = page.getByRole('main');
    if ((await main.getByRole('table').count()) === 0) {
      await main.getByRole('textbox', { name: 'Username' }).fill('t32h12user');
      await main.getByRole('textbox', { name: 'Domain' }).fill('example.org');
      await main.getByRole('button', { name: 'Provision user' }).click();
    }
    const table = main.getByRole('table');
    await expect(table).toBeVisible();
    const problems: string[] = [];
    problems.push(...(await checkControl(page, table.getByRole('checkbox', { name: /^Zero-access for/ }).first(), 'zero-access')));
    problems.push(...(await checkControl(page, table.getByRole('checkbox', { name: /^Disable / }).first(), 'disabled')));
    problems.push(...(await checkControl(page, table.getByRole('checkbox', { name: /^Force password change for/ }).first(), 'force change')));
    problems.push(...(await targetSize(page)));
    expect(problems).toEqual([]);
    expect(csp, 'inline-style CSP violations').toEqual([]);
  });

  test('require two-factor: deployment and new domain rule', async ({ page }) => {
    const csp = watchCsp(page);
    await adminSignIn(page);
    await adminSection(page, 'Require two-factor');
    const main = page.getByRole('main');
    const problems: string[] = [];
    // Turning the deployment-wide requirement on would make every sign-in enrol a factor.
    problems.push(...(await checkControl(page, main.getByTestId('admin-2fa-global'), 'whole deployment', { toggle: false })));
    problems.push(...(await checkControl(page, main.getByRole('checkbox', { name: 'Require two-factor for this domain' }), 'domain rule', { labelText: true })));
    problems.push(...(await targetSize(page)));
    expect(problems).toEqual([]);
    expect(csp, 'inline-style CSP violations').toEqual([]);
  });

  test('AI assistant capabilities, UI plugins and egress', async ({ page }) => {
    const csp = watchCsp(page);
    await adminSignIn(page);
    const main = page.getByRole('main');
    const problems: string[] = [];
    await adminSection(page, 'AI assistant');
    problems.push(...(await checkControl(page, main.getByRole('checkbox', { name: 'Summarize' }), 'assist capability', { labelText: true })));
    problems.push(...(await checkControl(page, main.getByRole('checkbox', { name: 'Allow attachments to be sent' }), 'assist ceiling', { labelText: true })));
    problems.push(...(await targetSize(page)));
    await adminSection(page, 'UI plugins');
    problems.push(...(await checkControl(page, main.getByRole('checkbox').first(), 'ui plugin unsigned', { labelText: true })));
    problems.push(...(await targetSize(page)));
    await adminSection(page, 'Egress');
    problems.push(...(await checkControl(page, main.getByTestId('egress-allow-plaintext'), 'egress plaintext', { labelText: true })));
    problems.push(...(await targetSize(page)));
    expect(problems).toEqual([]);
    expect(csp, 'inline-style CSP violations').toEqual([]);
  });
});

// ── appearance matrix ───────────────────────────────────────────────────────

interface Look {
  theme: string;
  accent?: string;
  rtl?: boolean;
}

const LOOKS: Look[] = [
  { theme: 'light' },
  { theme: 'paper', accent: 'green' },
  { theme: 'dark', accent: 'orange' },
  { theme: 'slate-dark' },
  { theme: 'bauhaus-black' },
  { theme: 'hc-light' },
  { theme: 'hc-dark' },
  { theme: 'dark', rtl: true },
];

test.describe('checkboxes in other themes, accents and right-to-left', () => {
  for (const look of LOOKS) {
    test(`${look.theme}${look.accent ? ` + ${look.accent}` : ''}${look.rtl ? ' rtl' : ''}`, async ({ page }) => {
      await seedTheme(page, look.theme);
      await engineLogin(page);
      await applyLook(page, look);
      const problems: string[] = [];
      const dialog = await settingsDialog(page);
      problems.push(...(await checkControl(page, dialog.getByRole('checkbox', { name: 'Search mail' }), `${look.theme} api scope`, { labelText: true })));
      await closeSettings(page);
      await page.getByTestId('nav-calendar').click();
      await page.getByRole('button', { name: 'New event' }).click();
      const ev = page.getByRole('dialog', { name: 'New event' });
      problems.push(...(await checkControl(page, ev.getByRole('checkbox', { name: 'All day' }), `${look.theme} all day`, { labelText: true })));
      expect(problems).toEqual([]);
    });
  }
});

test.describe('checkboxes under forced colours', () => {
  test.use({ contextOptions: { forcedColors: 'active' } });
  test('settings and compose', async ({ page }) => {
    await seedTheme(page, 'dark');
    await engineLogin(page);
    const problems: string[] = [];
    const dialog = await settingsDialog(page);
    problems.push(...(await checkControl(page, dialog.getByRole('checkbox', { name: 'Search mail' }), 'forced api scope', { labelText: true, forced: true })));
    await closeSettings(page);
    await page.getByRole('button', { name: 'Compose' }).first().click();
    const compose = page.getByRole('dialog', { name: 'Compose message' });
    problems.push(...(await checkControl(page, compose.getByTestId('sign-toggle'), 'forced sign', { labelText: true, forced: true })));
    expect(problems).toEqual([]);
  });
});

test.describe('checkboxes at phone width', () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true });
  test('settings and compose at 390 px', async ({ page }) => {
    await seedTheme(page, 'dark');
    await enginePhoneLogin(page);
    const problems: string[] = [];
    await drawerButton(page).click();
    const dialog = await settingsDialog(page);
    problems.push(...(await checkControl(page, dialog.getByRole('checkbox', { name: 'Search mail' }), 'phone api scope', { labelText: true })));
    problems.push(...(await targetSize(page)));
    expect(problems).toEqual([]);
  });
});
