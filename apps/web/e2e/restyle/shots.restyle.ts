import { mkdirSync } from 'node:fs';
import path from 'node:path';
import { outPath, prepare, screensFor, settle, test, type ScreenContext } from './probes.ts';

/**
 * Full-page screenshot per screen × theme × viewport (README.md), to
 * <T32_OUT>/screens/<label>/<viewport>/<theme>/<screen>.png. The global teardown
 * writes <T32_OUT>/screens/<label>/INDEX.md.
 */
for (const screen of screensFor()) {
  test(`shot ${screen.id} ${screen.tag}`, async ({ page, request, viewportName, theme }) => {
    test.skip(!screen.viewports.includes(viewportName), `${screen.id} is not a ${viewportName} screen`);
    const ctx: ScreenContext = { viewportName, theme, request };
    await prepare(page, ctx);
    await screen.open(page, ctx);
    await settle(page, ctx, screen);
    // outPath's first segment is the kind; screenshots live under screens/.
    const file = outPath('screens', ctx, screen, 'png');
    mkdirSync(path.dirname(file), { recursive: true });
    await page.screenshot({ path: file, fullPage: true, animations: 'disabled', caret: 'hide' });
    if (screen.close) await screen.close(page);
  });
}
