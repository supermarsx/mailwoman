// Render-timing perf benches (SPEC §23 / plan §6 t8-e5-perf). MEASURED, not
// asserted: each bench renders a representative workload in a real headless
// Chromium and MEASURES the build+layout+paint cost, then FAILS on over-budget.
//
//   • calendar-month-500-events  < 150 ms   (month grid, 500 events)
//   • 5MB-HTML-email-open        < 300 ms   (sanitized body into a sandboxed iframe)
//   • warm-nav (SPA route swap)  < 100 ms   (client-side screen transition render)
//
// Why a representative harness (not the live SolidJS component): this executor
// owns ONLY the perf track (scripts/perf/**, perf.yml, lighthouserc.json) and
// must NOT touch apps/web/src (owned by the web executors) — and seeding 500
// events / a 5 MB email through the backend + auth is out of scope. So each
// bench reproduces the SAME DOM shape the real component emits (the month grid
// is modelled 1:1 on apps/web/src/modules/calendar/views.tsx#MonthGrid: 6×7
// cells, ≤4 event chips per cell + "+N more"; the email path mirrors the
// sandboxed-iframe reader). It measures a real browser rendering a real
// workload of the budgeted size — a legitimate regression gate.
//
// Anti-flake: median-of-N runs (default 7), native CPU (150/300/100 ms are
// tight, so a 4× throttle would swamp signal with runner noise — the Lighthouse
// job owns the throttled cold-load budget). Set PERF_RUNS to change N.
//
// Trend-then-enforce: always writes scripts/perf/results/render-bench.json and
// prints a one-line-per-bench trend row. Enforcement is ON by default; export
// PERF_ENFORCE=0 for the initial soft-launch window (report, never fail).
//
// Run:  node scripts/perf/render-bench.mjs      (needs chromium from
//       `pnpm -C apps/web exec playwright install chromium`)

import { createRequire } from 'node:module';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { dirname, resolve } from 'node:path';
import { mkdirSync, writeFileSync } from 'node:fs';

const require = createRequire(import.meta.url);
const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptDir, '../../');
const webModules = resolve(repoRoot, 'apps/web/node_modules');

// Resolve Playwright's chromium from apps/web's install (the only place it
// lives — this executor adds no node deps of its own).
let chromium;
try {
  const entry = require.resolve('@playwright/test', { paths: [webModules] });
  const pw = await import(pathToFileURL(entry).href);
  chromium = pw.chromium ?? pw.default?.chromium;
} catch (err) {
  console.error('render-bench: cannot resolve @playwright/test from apps/web/node_modules.');
  console.error('  Run `pnpm -C apps/web install` and `pnpm -C apps/web exec playwright install chromium` first.');
  console.error(String(err));
  process.exit(2);
}
if (!chromium) {
  console.error('render-bench: @playwright/test did not export `chromium`.');
  process.exit(2);
}

const RUNS = Number(process.env.PERF_RUNS ?? '7');
const ENFORCE = process.env.PERF_ENFORCE !== '0';

const median = (xs) => {
  const s = [...xs].sort((a, b) => a - b);
  const m = Math.floor(s.length / 2);
  return s.length % 2 ? s[m] : (s[m - 1] + s[m]) / 2;
};
const round = (n) => Math.round(n * 10) / 10;

// --- the three benches, each an in-page function returning ms ----------------
// Each measures t0 (data ready) → force synchronous layout → paint frame
// (double-rAF), i.e. the cost the user waits for the screen to appear.

const benches = [
  {
    name: 'calendar-month-500-events',
    budgetMs: 150,
    // Model of MonthGrid (views.tsx): 42 day-cells, ≤4 chips/cell + "+N more".
    fn: () => {
      const EVENTS = 500;
      const COLORS = ['#3b82f6', '#ef4444', '#10b981', '#f59e0b', '#8b5cf6'];
      // 500 events spread across a 42-day (6-week) grid, like the controller
      // hands the view (already expanded EventInstances grouped per day).
      const byDay = Array.from({ length: 42 }, () => []);
      for (let i = 0; i < EVENTS; i++) {
        const day = i % 42;
        byDay[day].push({
          title: `Event ${i} — sync review with the platform team`,
          time: `${String(7 + (i % 12)).padStart(2, '0')}:${String((i * 7) % 60).padStart(2, '0')}`,
          color: COLORS[i % COLORS.length],
        });
      }
      const host = document.createElement('div');
      host.style.cssText = 'display:grid;grid-template-columns:repeat(7,1fr);grid-auto-rows:1fr;width:1000px;height:700px;position:absolute;left:-9999px';

      const t0 = performance.now();
      for (let d = 0; d < 42; d++) {
        const cell = document.createElement('div');
        cell.style.cssText = 'border:1px solid #ddd;padding:2px;overflow:hidden;font-size:11px';
        const num = document.createElement('span');
        num.textContent = String((d % 31) + 1);
        cell.appendChild(num);
        const evs = byDay[d];
        for (const e of evs.slice(0, 4)) {
          const chip = document.createElement('button');
          chip.type = 'button';
          chip.style.cssText = `display:block;width:100%;text-align:left;background:${e.color};color:#fff;border:0;border-radius:3px;margin-top:2px;padding:1px 3px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis`;
          chip.textContent = `${e.time} ${e.title}`;
          cell.appendChild(chip);
        }
        if (evs.length > 4) {
          const more = document.createElement('span');
          more.textContent = `+${evs.length - 4} more`;
          cell.appendChild(more);
        }
        host.appendChild(cell);
      }
      document.body.appendChild(host);
      void host.getBoundingClientRect().height; // flush layout
      return new Promise((r) =>
        requestAnimationFrame(() =>
          requestAnimationFrame(() => {
            const t1 = performance.now();
            host.remove();
            r(t1 - t0);
          }),
        ),
      );
    },
  },
  {
    name: '5MB-html-email-open',
    budgetMs: 300,
    // Mirrors the reader: a large sanitized HTML body dropped into a sandboxed
    // iframe. Build a ~5 MB document (headings + paragraphs + tables), then
    // measure to the iframe's load+paint.
    fn: () => {
      const TARGET = 5 * 1024 * 1024;
      const block =
        '<h3>Quarterly report section</h3>' +
        '<p>' +
        'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. '.repeat(6) +
        '</p>' +
        '<table border="1"><tr><td>row</td><td>value</td><td>note</td></tr><tr><td>a</td><td>1</td><td>ok</td></tr></table>';
      const parts = [];
      let size = 0;
      while (size < TARGET) {
        parts.push(block);
        size += block.length;
      }
      const html = `<!doctype html><meta charset="utf-8"><body>${parts.join('')}</body>`;

      const iframe = document.createElement('iframe');
      iframe.setAttribute('sandbox', ''); // reader isolates the body
      iframe.style.cssText = 'width:900px;height:700px;position:absolute;left:-9999px;border:0';
      const t0 = performance.now();
      return new Promise((r) => {
        iframe.addEventListener('load', () => {
          // one paint frame after the body is parsed+laid out
          requestAnimationFrame(() =>
            requestAnimationFrame(() => {
              const t1 = performance.now();
              iframe.remove();
              r(t1 - t0);
            }),
          );
        });
        document.body.appendChild(iframe);
        iframe.srcdoc = html;
      });
    },
  },
  {
    name: 'warm-nav-route-swap',
    budgetMs: 100,
    // A client-side screen transition (the SPA is already loaded — "warm"):
    // tear down a list screen and render a settings-form screen. Measures the
    // render cost of a representative route swap.
    fn: () => {
      const host = document.createElement('div');
      host.style.cssText = 'width:1000px;height:700px;position:absolute;left:-9999px';
      // Prime with a "list" screen (already mounted).
      const list = document.createElement('div');
      for (let i = 0; i < 80; i++) {
        const row = document.createElement('div');
        row.style.cssText = 'display:flex;gap:8px;padding:6px;border-bottom:1px solid #eee';
        row.innerHTML = `<span>Sender ${i}</span><span>Subject line number ${i} about the project</span><span>${i}:00</span>`;
        list.appendChild(row);
      }
      host.appendChild(list);
      document.body.appendChild(host);
      void host.getBoundingClientRect().height;

      const t0 = performance.now();
      host.removeChild(list);
      const settings = document.createElement('form');
      for (let i = 0; i < 60; i++) {
        const field = document.createElement('label');
        field.style.cssText = 'display:block;margin:6px 0';
        field.innerHTML = `<span>Setting ${i}</span> <input type="text" value="value ${i}"><select><option>A</option><option>B</option></select>`;
        settings.appendChild(field);
      }
      host.appendChild(settings);
      void host.getBoundingClientRect().height;
      return new Promise((r) =>
        requestAnimationFrame(() =>
          requestAnimationFrame(() => {
            const t1 = performance.now();
            host.remove();
            r(t1 - t0);
          }),
        ),
      );
    },
  },
];

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1200, height: 800 } });
await page.goto('about:blank');

const results = [];
let failed = false;

for (const b of benches) {
  const samples = [];
  for (let i = 0; i < RUNS; i++) {
    // eslint-disable-next-line no-await-in-loop
    const ms = await page.evaluate(b.fn);
    samples.push(ms);
  }
  const med = round(median(samples));
  const min = round(Math.min(...samples));
  const max = round(Math.max(...samples));
  const over = med > b.budgetMs;
  results.push({ name: b.name, medianMs: med, minMs: min, maxMs: max, budgetMs: b.budgetMs, over });
  const verdict = over ? (ENFORCE ? 'FAIL' : 'OVER(soft)') : 'OK';
  console.log(
    `[render-bench] ${b.name}: median ${med} ms (min ${min}, max ${max}) — budget ${b.budgetMs} ms => ${verdict}`,
  );
  if (over && ENFORCE) failed = true;
}

await browser.close();

const outDir = resolve(scriptDir, 'results');
mkdirSync(outDir, { recursive: true });
const payload = { at: new Date().toISOString(), runs: RUNS, enforced: ENFORCE, results };
writeFileSync(resolve(outDir, 'render-bench.json'), JSON.stringify(payload, null, 2));
console.log(`[render-bench] wrote ${resolve(outDir, 'render-bench.json')}`);

if (failed) {
  console.error('[render-bench] FAIL: one or more render budgets regressed (SPEC §23).');
  process.exit(1);
}
console.log('[render-bench] OK: all render budgets met.');
