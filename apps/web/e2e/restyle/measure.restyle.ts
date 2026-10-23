import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { outPath, prepare, screensFor, settle, test, expect, type Probe, type ScreenContext } from './probes.ts';

/**
 * Geometry of every probe on every screen, for `compare.mjs` (README.md).
 * Writes <T32_OUT>/measure/<label>/<viewport>/<theme>/<screen>.json; the global
 * teardown merges them into <T32_OUT>/measure-<label>.json.
 */

export interface Measured {
  visible: boolean;
  x: number;
  y: number;
  w: number;
  h: number;
  fontSize: number;
  lineHeight: number;
  lineHeightRaw: string;
  fontWeight: number;
  padding: [number, number, number, number];
  border: [number, number, number, number];
  minHeight: string;
  label: string;
}

/** Measure each element; runs in the page. `normal` line heights are resolved by a probe span. */
function measureAll(els: Element[], max: number): Measured[] {
  const px = (v: string) => (v.endsWith('px') ? Math.round(parseFloat(v) * 100) / 100 : NaN);
  const r2 = (n: number) => Math.round(n * 100) / 100;
  return els.slice(0, max).map((el) => {
    const cs = getComputedStyle(el);
    const rect = el.getBoundingClientRect();
    let lineHeight = px(cs.lineHeight);
    if (Number.isNaN(lineHeight)) {
      const span = document.createElement('span');
      span.textContent = 'Hg';
      span.style.cssText = `position:absolute;visibility:hidden;white-space:nowrap;line-height:${cs.lineHeight};font-family:${cs.fontFamily};font-size:${cs.fontSize};font-weight:${cs.fontWeight};font-style:${cs.fontStyle};display:inline-block`;
      document.body.appendChild(span);
      lineHeight = r2(span.getBoundingClientRect().height);
      span.remove();
    }
    const label = (el.getAttribute('aria-label') ?? el.textContent ?? '').replace(/\s+/g, ' ').trim().slice(0, 48);
    return {
      visible: rect.width > 0 && rect.height > 0 && cs.visibility !== 'hidden' && cs.display !== 'none',
      x: r2(rect.x),
      y: r2(rect.y),
      w: r2(rect.width),
      h: r2(rect.height),
      fontSize: px(cs.fontSize),
      lineHeight,
      lineHeightRaw: cs.lineHeight,
      fontWeight: Number(cs.fontWeight),
      padding: [px(cs.paddingTop), px(cs.paddingRight), px(cs.paddingBottom), px(cs.paddingLeft)],
      border: [px(cs.borderTopWidth), px(cs.borderRightWidth), px(cs.borderBottomWidth), px(cs.borderLeftWidth)],
      minHeight: cs.minHeight,
      label,
    };
  });
}

for (const screen of screensFor()) {
  test(`measure ${screen.id} ${screen.tag}`, async ({ page, request, viewportName, theme }) => {
    test.skip(!screen.viewports.includes(viewportName), `${screen.id} is not a ${viewportName} screen`);
    const ctx: ScreenContext = { viewportName, theme, request };
    await prepare(page, ctx);
    await screen.open(page, ctx);
    await settle(page, ctx, screen);

    const probes: Record<string, Measured & { kind: Probe['kind']; interactive: boolean }> = {};
    const missing: string[] = [];
    for (const probe of screen.probes) {
      const max = probe.max ?? 1;
      const items = await probe.locate(page).evaluateAll(measureAll, max);
      if (items.length === 0 || !items.some((m) => m.visible)) missing.push(probe.id);
      items.forEach((m, i) => {
        probes[`${probe.id}#${i}`] = { ...m, kind: probe.kind, interactive: probe.interactive };
      });
    }

    let visibleRows: number | null = null;
    if (screen.rows) {
      const { row, scroller } = screen.rows;
      visibleRows = await page.evaluate(
        ([rowSel, scrollSel]) => {
          const vw = window.innerWidth;
          const vh = window.innerHeight;
          const s = document.querySelector(scrollSel!)?.getBoundingClientRect();
          const top = Math.max(0, s?.top ?? 0);
          const bottom = Math.min(vh, s?.bottom ?? vh);
          return [...document.querySelectorAll(rowSel!)].filter((el) => {
            const r = el.getBoundingClientRect();
            return r.height > 0 && r.top >= top - 0.5 && r.bottom <= bottom + 0.5 && r.left >= -0.5 && r.right <= vw + 0.5;
          }).length;
        },
        [row, scroller],
      );
    }

    const result = {
      screen: screen.id,
      tag: screen.tag,
      viewport: viewportName,
      theme,
      label: process.env['T32_LABEL'],
      url: page.url(),
      dataTheme: await page.locator('html').getAttribute('data-theme'),
      reducedMotion: await page.evaluate(() => matchMedia('(prefers-reduced-motion: reduce)').matches),
      measuredAt: new Date(Date.now()).toISOString(),
      viewportSize: page.viewportSize(),
      visibleRows,
      probeCount: Object.keys(probes).length,
      missing,
      probes,
    };
    const file = outPath('measure', ctx, screen, 'json');
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, JSON.stringify(result, null, 1));

    // Every probe must exist on its own screen at T32-BASE; a missing one here is a harness defect
    // (or, on the after side, a vanished element, which compare.mjs also reports).
    expect(missing, `probes with no visible element on ${screen.id}`).toEqual([]);
    if (screen.close) await screen.close(page);
  });
}
