import AxeBuilder from '@axe-core/playwright';
import { test, expect, type BrowserContext, type Locator, type Page } from '@playwright/test';
import { composeSurface, engineLogin, settingsSurface, enginePhoneLogin, drawerButton } from './helpers.ts';

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
 * Each screen also checks every visible checkbox and radio on it, whatever row holds it
 * (a `.field` column, a flex or grid row, a table cell): square, at most 24px, label text
 * beside it on its row. That is the user's "a single tickbox element uses up a whole line".
 *
 * Runs in the `engine` project. The admin console cases sign in with MW_ADMIN_USER /
 * MW_ADMIN_PASSWORD (default `root` / `hunter2`, the dev and CI stacks' values); on a stack
 * whose admin panel refuses them they are skipped and say so.
 */

const BOX = 24;
const ADMIN_USER = process.env['MW_ADMIN_USER'] ?? 'root';
const ADMIN_PASSWORD = process.env['MW_ADMIN_PASSWORD'] ?? 'hunter2';

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
  const nav = page.getByRole('navigation').first();
  const form = page.getByRole('main').getByRole('textbox').first();
  // The beforeEach sign-in check leaves an admin session in this context; sign in through
  // the form only when the console asks for it.
  await expect(nav.or(form)).toBeVisible();
  if (await form.isVisible()) {
    const main = page.getByRole('main');
    await main.getByRole('textbox').nth(0).fill(ADMIN_USER);
    await main.getByRole('textbox').nth(1).fill(ADMIN_PASSWORD);
    await page.getByRole('button', { name: /sign in/i }).click();
  }
  await expect(nav).toBeVisible();
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

/**
 * Every visible checkbox and radio on the page, whatever holds it: the box is square and
 * at most 24px, and the first line of its label text (when it has one) sits beside it on
 * the same row. Elements inside an `inert` subtree (the page behind a dialog) are skipped.
 */
async function everyBoxOnItsRow(page: Page, screen: string): Promise<string[]> {
  const found = await page.evaluate((max) => {
    const out: string[] = [];
    for (const el of Array.from(document.querySelectorAll<HTMLInputElement>('input[type="checkbox"], input[type="radio"]'))) {
      if (el.closest('[inert]') || el.getClientRects().length === 0) continue;
      const b = el.getBoundingClientRect();
      if (b.width === 0 && b.height === 0) continue;
      const label = el.labels?.[0] ?? null;
      const name = el.getAttribute('aria-label') || label?.textContent?.trim().slice(0, 40) || el.id || el.type;
      const holder = (label ?? el.parentElement)!;
      const where = `${getComputedStyle(holder).display}/${getComputedStyle(holder.parentElement ?? holder).display}`;
      if (Math.abs(b.width - b.height) > 1 || b.width > max + 0.5)
        out.push(`${name} [${where}]: box ${b.width.toFixed(1)}×${b.height.toFixed(1)}`);
      if (!label) continue;
      const walker = document.createTreeWalker(label, NodeFilter.SHOW_TEXT);
      const rects: DOMRect[] = [];
      for (let n = walker.nextNode(); n; n = walker.nextNode()) {
        if (!n.textContent?.trim()) continue;
        const range = document.createRange();
        range.selectNodeContents(n);
        for (const r of Array.from(range.getClientRects())) if (r.width > 0) rects.push(r);
      }
      if (rects.length === 0) continue;
      const first = rects.reduce((a, c) => (c.top < a.top ? c : a));
      const line = rects.filter((r) => Math.abs(r.top - first.top) < 2);
      const left = Math.min(...line.map((r) => r.left));
      const right = Math.max(...line.map((r) => r.right));
      const overlap = Math.min(b.bottom, first.bottom) - Math.max(b.top, first.top);
      if (overlap < first.height / 2) out.push(`${name} [${where}]: label text not on the box's row`);
      if (!(left >= b.right - 0.5 || right <= b.left + 0.5)) out.push(`${name} [${where}]: label text not beside the box`);
    }
    return out;
  }, BOX);
  return found.map((f) => `${screen}: every box: ${f}`);
}

/** axe target-size findings on checkboxes and radios only. */
async function targetSize(page: Page): Promise<string[]> {
  const res = await new AxeBuilder({ page }).withRules(['target-size']).analyze();
  return res.violations.flatMap((v) =>
    v.nodes.filter((n) => /type="(checkbox|radio)"/.test(n.html)).map((n) => `target-size: ${n.html.slice(0, 120)}`),
  );
}

async function closeSettings(page: Page): Promise<void> {
  const dialog = settingsSurface(page);
  await dialog.getByRole('button', { name: 'Close settings' }).click();
  await expect(dialog).toBeHidden();
}

async function settingsDialog(page: Page): Promise<Locator> {
  await page.getByRole('button', { name: 'Settings', exact: true }).first().click();
  const dialog = settingsSurface(page);
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
    problems.push(...(await targetSize(page)), ...(await everyBoxOnItsRow(page, 'settings')));
    // The rule builder: condition and action checkboxes in flex and grid rows.
    await dialog.getByRole('button', { name: 'New rule' }).click();
    await page.waitForTimeout(300);
    problems.push(...(await everyBoxOnItsRow(page, 'settings › new rule')));
    expect(problems).toEqual([]);
    expect(csp, 'inline-style CSP violations').toEqual([]);
  });

  test('compose: message security options', async ({ page }) => {
    const csp = watchCsp(page);
    await engineLogin(page);
    await page.getByRole('button', { name: 'Compose', exact: true }).first().click();
    const dialog = composeSurface(page);
    await expect(dialog).toBeVisible();
    const problems: string[] = [];
    problems.push(...(await checkControl(page, dialog.getByTestId('sign-toggle'), 'sign', { labelText: true })));
    problems.push(...(await checkControl(page, dialog.getByTestId('encrypt-toggle'), 'encrypt (disabled)', { toggle: false })));
    // t29-e12 send options: the receipt request (engine), and open tracking where the stack has MW_PUBLIC_URL.
    problems.push(...(await checkControl(page, dialog.getByTestId('compose-request-receipt'), 'request receipt', { labelText: true })));
    if ((await dialog.getByTestId('compose-track-opens').count()) > 0) {
      problems.push(...(await checkControl(page, dialog.getByTestId('compose-track-opens'), 'record opens', { labelText: true })));
    }
    problems.push(...(await targetSize(page)), ...(await everyBoxOnItsRow(page, new URL(page.url()).hash || new URL(page.url()).pathname)));
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
    // Weekly repeats: the weekday options, a wrapping flex row of framed chips.
    await dialog.getByRole('checkbox', { name: 'Repeats' }).check();
    await dialog.getByRole('combobox', { name: 'Frequency' }).selectOption('weekly');
    problems.push(...(await checkControl(page, dialog.getByRole('checkbox', { name: 'Mon', exact: true }), 'weekday', { labelText: true })));
    problems.push(...(await targetSize(page)), ...(await everyBoxOnItsRow(page, 'calendar › new event')));
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
    problems.push(...(await targetSize(page)), ...(await everyBoxOnItsRow(page, new URL(page.url()).hash || new URL(page.url()).pathname)));
    expect(problems).toEqual([]);
    expect(csp, 'inline-style CSP violations').toEqual([]);
  });

  test('tasks: the complete box beside each task', async ({ page }) => {
    const csp = watchCsp(page);
    await engineLogin(page);
    await page.getByTestId('nav-tasks').click();
    const list = page.getByRole('list', { name: 'Tasks' });
    await expect(list).toBeVisible();
    await page.waitForTimeout(500);
    if ((await list.getByText('t32-h12 task', { exact: true }).count()) === 0) {
      await page.getByRole('textbox', { name: 'New task title' }).fill('t32-h12 task');
      await page.getByRole('button', { name: 'Add', exact: true }).click();
    }
    await expect(list.getByText('t32-h12 task', { exact: true })).toBeVisible();
    // Completing a task moves it out of the open list, so the box is not toggled here.
    const problems = await checkControl(page, page.getByRole('checkbox', { name: /^(Complete|Reopen) t32-h12 task$/ }), 'complete task', {
      toggle: false,
    });
    problems.push(...(await targetSize(page)), ...(await everyBoxOnItsRow(page, 'tasks')));
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
    problems.push(...(await targetSize(page)), ...(await everyBoxOnItsRow(page, new URL(page.url()).hash || new URL(page.url()).pathname)));
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
  test.beforeEach(async ({ page }) => {
    // A stack without admin credentials answers 401 to the admin sign-in.
    const signIn = await page.request.post('/admin/login', { data: { username: ADMIN_USER, password: ADMIN_PASSWORD } });
    test.skip(
      signIn.status() === 401 && process.env['MW_ADMIN_USER'] === undefined,
      'the admin panel refuses root/hunter2 on this stack (set MW_ADMIN_USER/MW_ADMIN_PASSWORD to run this)',
    );
    expect(signIn.status(), 'admin login').toBe(200);
  });

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
    problems.push(...(await targetSize(page)), ...(await everyBoxOnItsRow(page, new URL(page.url()).hash || new URL(page.url()).pathname)));
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
    problems.push(...(await targetSize(page)), ...(await everyBoxOnItsRow(page, new URL(page.url()).hash || new URL(page.url()).pathname)));
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
    problems.push(...(await targetSize(page)), ...(await everyBoxOnItsRow(page, new URL(page.url()).hash || new URL(page.url()).pathname)));
    await adminSection(page, 'UI plugins');
    problems.push(...(await checkControl(page, main.getByRole('checkbox').first(), 'ui plugin unsigned', { labelText: true })));
    problems.push(...(await targetSize(page)), ...(await everyBoxOnItsRow(page, new URL(page.url()).hash || new URL(page.url()).pathname)));
    await adminSection(page, 'Egress');
    problems.push(...(await checkControl(page, main.getByTestId('egress-allow-plaintext'), 'egress plaintext', { labelText: true })));
    problems.push(...(await targetSize(page)), ...(await everyBoxOnItsRow(page, new URL(page.url()).hash || new URL(page.url()).pathname)));
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
    await page.getByRole('button', { name: 'Compose', exact: true }).first().click();
    const compose = composeSurface(page);
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
    problems.push(...(await targetSize(page)), ...(await everyBoxOnItsRow(page, new URL(page.url()).hash || new URL(page.url()).pathname)));
    expect(problems).toEqual([]);
  });
});
