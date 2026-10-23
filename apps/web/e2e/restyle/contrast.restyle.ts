import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { check } from './checks.ts';
import { outPath, prepare, screensFor, settle, test, expect, type ScreenContext } from './probes.ts';

/**
 * Rendered contrast and "not colour alone" checks per screen × theme × viewport
 * (README.md). Fails on any of:
 *   - an axe `color-contrast` violation (WCAG 2 AA), message bodies excluded;
 *   - a control boundary below 3:1: inputs, selects, textareas, and buttons bordered on
 *     all four sides, where neither the border nor the control's own fill reaches
 *     3:1 against the background next to it (the nearest opaque ancestor);
 *   - a keyboard focus stop (Tab, up to FOCUS_STOPS) with no outline or box-shadow
 *     ring, or one below 3:1 against the background next to it;
 *   - an unread list row with neither weight >= 600 nor a marker (pseudo-element or
 *     an element whose class names "unread");
 *   - a selected list row or option with no inset bar, outline, 3 px start border or
 *     marker pseudo-element;
 *   - a link inside running text without an underline;
 *   - a selected tab with no bar (2 px bottom border, inset shadow or pseudo-element).
 * T32_CONTRAST_REPORT_ONLY=1 writes the findings without failing (used for the
 * T32-BASE baseline, which predates the rules).
 * Writes <T32_OUT>/contrast/<label>/<viewport>/<theme>/<screen>.json.
 */

for (const screen of screensFor()) {
  test(`contrast ${screen.id} ${screen.tag}`, async ({ page, request, viewportName, theme }) => {
    test.skip(!screen.viewports.includes(viewportName), `${screen.id} is not a ${viewportName} screen`);
    const ctx: ScreenContext = { viewportName, theme, request };
    await prepare(page, ctx);
    await screen.open(page, ctx);
    await settle(page, ctx, screen);
    const result = await check(page, screen.exclude ?? []);
    const file = outPath('contrast', ctx, screen, 'json');
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(
      file,
      JSON.stringify(
        { screen: screen.id, tag: screen.tag, viewport: viewportName, theme, label: process.env['T32_LABEL'], ...result },
        null,
        1,
      ),
    );
    const lines = result.findings.map((f) => `${f.check}: ${f.element} — ${f.detail}`);
    if (lines.length > 0) console.log(`\n[${viewportName}/${theme}/${screen.id}] ${lines.length} finding(s):\n  ${lines.join('\n  ')}`);
    if (process.env['T32_CONTRAST_REPORT_ONLY'] !== '1') {
      expect(lines, `contrast / signal findings on ${viewportName}/${theme}/${screen.id}`).toEqual([]);
    }
    if (screen.close) await screen.close(page);
  });
}

