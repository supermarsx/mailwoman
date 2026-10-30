// Reader notice bars, live (t110). Runs on the receipts stack
// (scripts/e2e-receipts-stack.mjs): a real server and Greenmail, no route mocks.
//
// One seeded message carries all four notices: List-* headers (mailing list),
// remote images (blocked in ask mode), links to files (linked files) and two
// attachments. For Chromium and Firefox, at 1440 and 390 px, in light and dark:
//   - the four bars show in the documented order: list, images, files, attachments;
//   - they share one computed height, padding, radius, border, font family/size;
//   - they stack without overlap, with one equal gap, on one line each, with no
//     horizontal page scroll;
//   - nothing moves after the bars and the body appear (sampled every frame);
//   - Show files opens a menu listing the file with its host; Unsubscribe opens
//     the review dialog and nothing is requested from the list host;
//   - attachments collapse and expand in place (aria-expanded, hidden panel), the
//     default survives reload while expansion stays local to the message.
//
// MW_T110_PHASE=before only takes the screenshots (against an older build).
import { test, expect, type Page } from '../csrf.ts';
import {withUserSetting} from '../helpers.ts';
import { connect } from 'node:net';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const PHASE = process.env['MW_T110_PHASE'] === 'before' ? 'before' : 'after';
const SHOTS = process.env['MW_T110_SHOTS'] ?? 'test-results/t110';
const BARS = ['mailing-list-bar', 'remote-content-bar', 'linked-files-bar', 'reader-attachments-bar'] as const;

async function smtp(from: string, message: string): Promise<void> {
  const port = Number(process.env['MW_E2E_SMTP_PORT'] ?? 42225);
  const steps = ['EHLO t110', `MAIL FROM:<${from}>`, 'RCPT TO:<testuser@example.org>', 'DATA', `${message}\r\n.`, 'QUIT'];
  await new Promise<void>((resolve, reject) => {
    const socket = connect(port, '127.0.0.1'); let i = 0; let buffer = '';
    socket.on('data', (chunk) => { buffer += chunk.toString(); if (!/\r\n$/.test(buffer)) return; const code = Number(buffer.slice(0, 3)); buffer = ''; if (code >= 400) { reject(new Error(`SMTP ${code}`)); socket.destroy(); return; } if (i < steps.length) socket.write(`${steps[i++]}\r\n`); else socket.end(); });
    socket.on('close', () => resolve()); socket.on('error', reject);
  });
}

const b64 = (text: string): string => Buffer.from(text).toString('base64').replace(/.{76}/g, '$&\r\n');
async function seed(subject: string, from: string, tag: string): Promise<void> {
  const html = `<p>Quarterly newsletter ${tag}</p>` +
    Array.from({ length: 4 }, (_, n) => `<img alt="remote ${n}" width="24" height="24" src="http://1.2.3.4/img${n}.png?${tag}">`).join('') +
    `<p>Read <a href="https://files.example.test/reports/q3-report.pdf">the Q3 report</a> and <a href="https://share.example.test/s/budget-2027.xlsx">the budget</a>, or visit <a href="https://www.example.test/news">our site</a>.</p>` +
    `<p>${'Body text. '.repeat(60)}</p>`;
  const boundary = `t110-${Date.now()}`;
  const message = [
    `From: Example News <${from}>`, 'To: testuser@example.org', `Date: ${new Date().toUTCString()}`,
    `Message-ID: <${Date.now()}.${Math.random()}@example.test>`, `Subject: ${subject}`, 'MIME-Version: 1.0',
    'List-Id: "Example News" <news.example.test>',
    'List-Unsubscribe: <https://list.example.test/leave?token=t110>, <mailto:leave@example.test?subject=unsubscribe>',
    'List-Unsubscribe-Post: List-Unsubscribe=One-Click',
    'List-Archive: <https://list.example.test/archive>', 'List-Help: <mailto:help@example.test>',
    `Content-Type: multipart/mixed; boundary="${boundary}"`, '',
    `--${boundary}`, 'Content-Type: text/html; charset=utf-8', 'Content-Transfer-Encoding: base64', '', b64(html),
    `--${boundary}`, 'Content-Type: text/plain; name="agenda.txt"', 'Content-Disposition: attachment; filename="agenda.txt"', 'Content-Transfer-Encoding: base64', '', b64('Agenda\n'.repeat(400)),
    `--${boundary}`, 'Content-Type: text/csv; name="figures.csv"', 'Content-Disposition: attachment; filename="figures.csv"', 'Content-Transfer-Encoding: base64', '', b64('a,b,c\n1,2,3\n'.repeat(900)),
    `--${boundary}--`, '',
  ].join('\r\n');
  await smtp(from, message);
}

async function authority(page: Page): Promise<Record<string, string>> {
  const me = await (await page.request.get('/api/me')).json() as { storageAuthority: string };
  return { 'x-mailwoman-storage-authority': me.storageAuthority };
}
async function userSetting(page: Page, key: string, value: unknown): Promise<void> {
  const r = await page.request.put(`/api/settings/${key}`, { data: { value }, headers: await authority(page) }); expect(r.ok(), await r.text()).toBe(true);
}
async function signIn(page: Page): Promise<void> {
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill('testuser');
  await page.getByLabel('Password', { exact: true }).fill('testpass');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.locator('.list__row').first()).toBeVisible({ timeout: 30000 });
}
async function open(page: Page, subject: string): Promise<void> {
  const row = page.locator('.list__row').filter({ hasText: subject });
  await expect(async () => {
    if (!(await row.isVisible())) {
      if ((page.viewportSize()?.width ?? 1440) < 761) { await page.getByRole('button', { name: 'Open folders and apps' }).click(); }
      await page.getByRole('navigation', { name: 'Mailboxes' }).getByRole('button', { name: 'Inbox' }).click();
    }
    await expect(row).toBeVisible({ timeout: 3000 });
  }).toPass({ timeout: 60000 });
  await row.click();
}

/** Samples every bar's top and the body frame's top on every animation frame. */
async function startProbe(page: Page, subject: string): Promise<void> {
  await page.evaluate(([bars, subject]) => {
    const w = window as unknown as { __t110: { rows: number[][]; stop: boolean } };
    w.__t110 = { rows: [], stop: false };
    const visible = (sel: string): HTMLElement | undefined => [...document.querySelectorAll<HTMLElement>(sel)].find((e) => e.offsetParent !== null);
    const tick = (): void => {
      if (w.__t110.stop) return;
      const els = [...bars.map((b) => visible(`[data-testid="${b}"]`)), visible('iframe.reader__frame')];
      // Only rows that show the message just opened (the previous one may still be up).
      if (![...document.querySelectorAll('h1,h2')].some((h) => (h as HTMLElement).offsetParent !== null && h.textContent?.includes(subject))) { requestAnimationFrame(tick); return; }
      const panel = visible('[data-testid="reader-attachments"]');
      w.__t110.rows.push([...els.map((e) => (e ? Math.round(e.getBoundingClientRect().top) : -1)), panel ? Math.round(panel.getBoundingClientRect().height) : -1, Math.round(visible('[data-testid="reader-notice-stack"]')?.getBoundingClientRect().bottom ?? -1)]);
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  }, [[...BARS], subject] as const);
}
async function stopProbe(page: Page): Promise<number[][]> {
  return page.evaluate(() => { const w = window as unknown as { __t110: { rows: number[][]; stop: boolean } }; w.__t110.stop = true; return w.__t110.rows; });
}

interface BarGeometry { id: string; top: number; bottom: number; height: number; rowHeight: number; paddingInline: string; radius: string; border: string; fontFamily: string; fontSize: string; rowOverflow: number; summaryLines: number }
async function geometry(page: Page): Promise<{ bars: BarGeometry[]; overflow: number }> {
  return page.evaluate((ids) => {
    const visible = (sel: string): HTMLElement | undefined => [...document.querySelectorAll<HTMLElement>(sel)].find((e) => e.offsetParent !== null);
    const bars = ids.map((id) => {
      const node = visible(`[data-testid="${id}"]`)!; const row = node.firstElementChild as HTMLElement; const summary = row.querySelector('[data-tooltip]') as HTMLElement;
      const cs = getComputedStyle(node); const rs = getComputedStyle(row); const r = row.getBoundingClientRect();
      const lh = parseFloat(getComputedStyle(summary).lineHeight) || 16;
      return { id, top: node.getBoundingClientRect().top, bottom: r.bottom, height: node.getBoundingClientRect().height, rowHeight: r.height, paddingInline: `${rs.paddingInlineStart} ${rs.paddingInlineEnd}`, radius: cs.borderTopLeftRadius, border: `${cs.borderTopWidth} ${cs.borderTopStyle}`, fontFamily: cs.fontFamily, fontSize: cs.fontSize, rowOverflow: row.scrollHeight - row.clientHeight, summaryLines: Math.round(summary.getBoundingClientRect().height / lh) };
    });
    return { bars, overflow: document.documentElement.scrollWidth - innerWidth };
  }, [...BARS]);
}

test.describe.configure({ mode: 'serial' });

test('t110 reader notice bars: one style, fixed order, no jump, files menu, list review, attachments collapse', async ({ page,playwright,baseURL }, info) => {
  test.skip(info.project.name === 'phone', 'widths are looped inside the chromium/firefox projects');
  test.setTimeout(15 * 60_000);
  const shots = join(SHOTS, PHASE); mkdirSync(shots, { recursive: true });
  const listRequests: string[] = []; page.on('request', (r) => { if (/list\.example\.test|files\.example\.test|share\.example\.test/.test(r.url())) listRequests.push(r.url()); });
  const report: Record<string, unknown> = { phase: PHASE, project: info.project.name };
  const run = `${info.project.name}-${Date.now()}`;
  const combos = [[1440, 'light'], [1440, 'dark'], [390, 'light'], [390, 'dark']] as const;
  const subjects = combos.map(([w, theme]) => ({ w, theme, subject: `T110 notices ${w} ${theme} ${run}` }));
  for (const s of subjects) await seed(s.subject, `news-${s.w}-${s.theme}-${run}@example.test`, `${s.w}${s.theme}`);

  await page.waitForTimeout(3100); // sign-in cooldown
  await signIn(page);
  const restoreAttachments=await withUserSetting(playwright.request,baseURL!,'reader.attachmentsCollapsed',true);
  const restoreDensity=await withUserSetting(playwright.request,baseURL!,'reader.attachmentsDensity','compact');
  await userSetting(page, 'privacy.remoteContent', 'ask');
  await page.reload(); await expect(page.locator('.list__row').first()).toBeVisible({ timeout: 30000 });

  try {
    for (const s of subjects) {
      const tag = `${info.project.name}-${s.w}-${s.theme}`;
      await page.setViewportSize({ width: s.w, height: s.w < 761 ? 844 : 900 });
      await page.emulateMedia({ colorScheme: s.theme, reducedMotion: 'reduce' });
      await page.evaluate((t) => { document.documentElement.setAttribute('data-theme', t); document.documentElement.setAttribute('data-appearance', t); }, s.theme);

      if (PHASE === 'after') await startProbe(page, s.subject);
      await open(page, s.subject);
      await expect(page.locator('iframe.reader__frame').filter({ visible: true }).first()).toBeVisible({ timeout: 20000 });
      if (PHASE === 'before') {
        await page.waitForTimeout(1500);
        await page.screenshot({ path: join(shots, `${tag}-reader.png`) });
        const files = page.getByText('Files linked in this message', { exact: true }).filter({ visible: true });
        if (await files.count()) { await files.first().click(); await page.screenshot({ path: join(shots, `${tag}-files-open.png`) }); }
        continue;
      }
      for (const id of BARS) await expect(page.getByTestId(id).filter({ visible: true })).toBeVisible({ timeout: 20000 });
      await page.waitForTimeout(2000);
      const rows = await stopProbe(page);
      // From the first frame that shows the body, every bar and the body keep their top.
      const settled = rows.filter((r) => r[BARS.length]! >= 0);
      const columns = BARS.map((_, i) => [...new Set(settled.map((r) => r[i]!))]);
      const frameTops = [...new Set(settled.map((r) => r[BARS.length]!))];
      report[`${tag}-extra`] = { panel: [...new Set(settled.map((r) => r[BARS.length + 1]))], stackBottom: [...new Set(settled.map((r) => r[BARS.length + 2]))] };
      report[`${tag}-probe`] = { columns, frameTops, frames: settled.length };
      // Sub-pixel rounding may differ by 1 px; any real shift is larger.
      const spread = (xs: number[]): number => Math.max(...xs) - Math.min(...xs);
      expect(spread(frameTops), 'the body does not move once shown').toBeLessThanOrEqual(1);
      columns.forEach((c, i) => expect(spread(c), `${BARS[i]} appears with the body and stays put`).toBeLessThanOrEqual(1));

      const g = await geometry(page); report[`${tag}-geometry`] = g;
      expect(g.bars.map((b) => b.id)).toEqual([...BARS]);
      for (let i = 1; i < g.bars.length; i++) expect(g.bars[i]!.top, `${g.bars[i]!.id} below ${g.bars[i - 1]!.id}`).toBeGreaterThan(g.bars[i - 1]!.top);
      const first = g.bars[0]!;
      for (const b of g.bars) {
        expect(b.rowHeight, `${b.id} row height`).toBeCloseTo(first.rowHeight, 0);
        expect(b.paddingInline, `${b.id} padding`).toBe(first.paddingInline);
        expect(b.radius, `${b.id} radius`).toBe(first.radius);
        expect(b.border, `${b.id} border`).toBe(first.border);
        expect(b.fontFamily, `${b.id} font`).toBe(first.fontFamily);
        expect(b.fontSize, `${b.id} font size`).toBe(first.fontSize);
        expect(b.summaryLines, `${b.id} one line`).toBe(1);
        expect(b.rowOverflow, `${b.id} no wrap`).toBeLessThanOrEqual(1);
      }
      // The three single-row bars have the same outer height; no overlap, one equal gap.
      expect(new Set(g.bars.slice(0, 3).map((b) => Math.round(b.height))).size).toBe(1);
      const gaps = g.bars.slice(1).map((b, i) => Math.round(b.top - (g.bars[i]!.top + g.bars[i]!.height)));
      report[`${tag}-gaps`] = gaps;
      gaps.forEach((gap) => expect(gap, 'no overlap').toBeGreaterThanOrEqual(0));
      expect(new Set(gaps).size, 'one gap between bars').toBe(1);
      expect(g.overflow, 'no horizontal page scroll').toBeLessThanOrEqual(1);
      await page.getByTestId('reader-notice-stack').filter({ visible: true }).screenshot({ path: join(shots, `${tag}-bars.png`) });
      await page.screenshot({ path: join(shots, `${tag}-reader.png`) });

      const scope = (id: string) => page.getByTestId(id).filter({ visible: true });
      await expect(scope('mailing-list-bar')).toContainText(/Mailing list · .?Example News/);
      await expect(scope('remote-content-bar').getByRole('status')).toHaveText(/^Remote images blocked/);
      await expect(scope('linked-files-bar')).toContainText('2 files linked in this message');
      await expect(scope('reader-attachments-bar')).toContainText(/^2 attachments · [\d.,]+\s?kB/);
      if (s.w < 761) {
        // Phone: the action labels fold away; each action is an icon button with a name and tooltip.
        for (const name of ['Unsubscribe', 'Show images', 'Show files']) {
          const button = page.getByRole('button', { name, exact: true }).filter({ visible: true }).first();
          await expect(button).toHaveAttribute('data-tooltip', /.+/);
          expect(await button.evaluate((b) => (b.querySelector('span:last-child') as HTMLElement).getBoundingClientRect().width), `${name} label hidden`).toBeLessThanOrEqual(1);
        }
      }

      // Show files: a menu with each file's name and host; nothing fetched until a choice.
      await scope('linked-files-bar').getByRole('button', { name: 'Show files' }).click();
      await expect(page.getByRole('menuitem', { name: /Download .q3-report\.pdf. from .files\.example\.test./ })).toBeVisible();
      await expect(page.getByRole('menuitem', { name: /Download .budget-2027\.xlsx. from .share\.example\.test./ })).toBeVisible();
      await expect(page.getByRole('menuitem', { name: /Download .www\.example\.test\/news./ })).toBeVisible();
      await page.screenshot({ path: join(shots, `${tag}-files-menu.png`) });
      const menuBox = await page.getByRole('menu').boundingBox(); expect(menuBox!.x).toBeGreaterThanOrEqual(0); expect(menuBox!.x + menuBox!.width).toBeLessThanOrEqual(s.w + 1);
      await page.keyboard.press('Escape'); await expect(page.getByRole('menu')).toHaveCount(0);

      // Unsubscribe: the review dialog, nothing requested from the list host.
      await scope('mailing-list-bar').getByRole('button', { name: 'Unsubscribe' }).click();
      const dialog = page.getByRole('dialog', { name: 'Unsubscribe' });
      await expect(dialog).toBeVisible(); await expect(dialog.getByRole('button', { name: 'list.example.test' })).toBeVisible();
      await page.screenshot({ path: join(shots, `${tag}-unsubscribe-review.png`) });
      await dialog.getByRole('button', { name: 'Close' }).click(); await expect(dialog).toHaveCount(0);
      await scope('mailing-list-bar').getByRole('button', { name: 'More list actions' }).click();
      await expect(page.getByRole('menuitem')).toHaveText(['Help', 'List archive']);
      await page.keyboard.press('Escape');
      expect(listRequests, 'no request to a list or file host').toEqual([]);

      // Attachments collapse and expand in place.
      const toggle = scope('reader-attachments-bar').getByTestId('reader-attachments-toggle');
      await expect(toggle).toHaveAttribute('aria-expanded', 'false');
      await toggle.focus();await page.keyboard.press('Enter');
      await expect(toggle).toHaveAttribute('aria-expanded', 'true');
      await expect(page.locator('.mw-thumbs').filter({visible:true})).toHaveAttribute('data-density','compact');
      const frameTop = async (): Promise<number> => (await page.locator('iframe.reader__frame').filter({ visible: true }).first().boundingBox())!.y;
      const openTop = await frameTop(); const barTops = (await geometry(page)).bars.map((b) => Math.round(b.top));
      await toggle.focus(); await page.keyboard.press('Enter');
      await expect(toggle).toHaveAttribute('aria-expanded', 'false'); await expect(toggle).toHaveAccessibleName('Show attachments');
      await expect(page.getByTestId('reader-attachments').filter({ visible: true })).toHaveCount(0);
      expect((await geometry(page)).bars.map((b) => Math.round(b.top)), 'collapsing moves no bar').toEqual(barTops);
      const closedTop = await frameTop(); expect(closedTop).toBeLessThan(openTop);
      await page.screenshot({ path: join(shots, `${tag}-attachments-collapsed.png`) });
      if (s === subjects[0]) {
        // Opening one message does not change the persistent collapsed default.
        await page.waitForTimeout(500);
        await page.reload(); await expect(page.locator('.list__row').first()).toBeVisible({ timeout: 30000 });
        await open(page, s.subject);
        await expect(page.getByTestId('reader-attachments-toggle').filter({ visible: true })).toHaveAttribute('aria-expanded', 'false', { timeout: 20000 });
      }
      await page.getByTestId('reader-attachments-toggle').filter({ visible: true }).click();
      await expect(page.getByTestId('reader-attachments').filter({ visible: true })).toBeVisible();
    }
  } finally {
    await restoreDensity();await restoreAttachments();
    writeFileSync(join(shots, `report-${info.project.name}.json`), JSON.stringify(report, null, 2));
  }
});
