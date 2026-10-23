/* global process, console */
// Compare two measure runs of the t32 restyle harness (README.md in this folder).
//
//   node compare.mjs <before> <after> [--partial] [--json <file>]
//
// <before>/<after>: a merged measure-<label>.json, or a measure/<label>/ directory.
// Exits 1 when any rule below is broken, 2 on a usage or input error.
//
// For every probe instance present (visible) in both runs, on the same viewport,
// theme and screen:
//   - height grows by more than 1 px                       (every kind)
//   - width grows by more than 1 px                        (kind "control")
//   - font-size grows at all
//   - line-height (px) grows by more than 1 px
//   - any padding side grows by more than 1 px
//   - phone, interactive: width or height differs by more than 1 px either way
//   - desktop, interactive: width or height drops below min(before, 24 px)
// Per screen: fewer fully visible list rows than before.
// A probe instance visible before and absent or invisible after is "vanished".
// A screen measured before and not after fails too, unless --partial is given,
// which limits the comparison to the screens present in <after> (for a run with
// --grep or T32_SCREENS).
import { readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const TOL = 1;
const EPS = 0.01;
const TOUCH_FLOOR = 24;

function usage(msg) {
  if (msg) console.error(`compare.mjs: ${msg}`);
  console.error('usage: node compare.mjs <before> <after> [--partial] [--json <file>]');
  process.exit(2);
}

/** Load a run: { "<viewport>/<theme>/<screen>": screenResult }. */
export function load(p) {
  let st;
  try {
    st = statSync(p);
  } catch {
    usage(`${p} does not exist`);
  }
  if (st.isFile()) {
    const j = JSON.parse(readFileSync(p, 'utf8'));
    return j.screens ?? j;
  }
  const out = {};
  const walk = (dir) => {
    for (const name of readdirSync(dir)) {
      const full = path.join(dir, name);
      if (statSync(full).isDirectory()) walk(full);
      else if (name.endsWith('.json')) {
        const r = JSON.parse(readFileSync(full, 'utf8'));
        out[`${r.viewport}/${r.theme}/${r.screen}`] = r;
      }
    }
  };
  walk(p);
  return out;
}

const r1 = (n) => (Number.isFinite(n) ? Math.round(n * 100) / 100 : n);

/** All violations between two loaded runs, plus the instances that shrank. */
export function compare(before, after, { partial = false } = {}) {
  const violations = [];
  const shrunk = [];
  const add = (key, probe, rule, b, a) => violations.push({ screen: key, probe, rule, before: r1(b), after: r1(a) });

  for (const [key, b] of Object.entries(before).sort()) {
    const a = after[key];
    if (a === undefined) {
      if (!partial) add(key, '*', 'screen not measured after', 'measured', 'absent');
      continue;
    }
    const phone = b.viewport === 'phone';
    if (b.visibleRows !== null && b.visibleRows !== undefined && (a.visibleRows ?? -1) < b.visibleRows) {
      add(key, '*', 'fewer visible list rows', b.visibleRows, a.visibleRows);
    }
    for (const [id, pb] of Object.entries(b.probes)) {
      if (!pb.visible) continue;
      const pa = a.probes[id];
      if (pa === undefined || !pa.visible) {
        add(key, id, 'vanished', `${pb.w}x${pb.h}`, pa === undefined ? 'absent' : 'invisible');
        continue;
      }
      if (pa.h > pb.h + TOL) add(key, id, 'height +>1px', pb.h, pa.h);
      if (pb.kind === 'control' && pa.w > pb.w + TOL) add(key, id, 'control width +>1px', pb.w, pa.w);
      if (pa.fontSize > pb.fontSize + EPS) add(key, id, 'font-size grew', pb.fontSize, pa.fontSize);
      if (pa.lineHeight > pb.lineHeight + TOL) add(key, id, 'line-height +>1px', pb.lineHeight, pa.lineHeight);
      ['top', 'right', 'bottom', 'left'].forEach((side, i) => {
        if (pa.padding[i] > pb.padding[i] + TOL) add(key, id, `padding-${side} +>1px`, pb.padding[i], pa.padding[i]);
      });
      if (pb.interactive && phone) {
        if (Math.abs(pa.w - pb.w) > TOL) add(key, id, 'phone tap target width changed >1px', pb.w, pa.w);
        if (Math.abs(pa.h - pb.h) > TOL) add(key, id, 'phone tap target height changed >1px', pb.h, pa.h);
      }
      if (pb.interactive && !phone) {
        if (pa.w + EPS < Math.min(pb.w, TOUCH_FLOOR)) add(key, id, 'target width below min(before,24)', pb.w, pa.w);
        if (pa.h + EPS < Math.min(pb.h, TOUCH_FLOOR)) add(key, id, 'target height below min(before,24)', pb.h, pa.h);
      }
      if (pa.h < pb.h - TOL || (pb.kind === 'control' && pa.w < pb.w - TOL)) {
        shrunk.push({ screen: key, probe: id, before: `${pb.w}x${pb.h}`, after: `${pa.w}x${pa.h}` });
      }
    }
  }
  return { violations, shrunk };
}

function table(rows, cols) {
  if (rows.length === 0) return '';
  const widths = cols.map((c) => Math.max(c.length, ...rows.map((r) => String(r[c]).length)));
  const line = (vals) => vals.map((v, i) => String(v).padEnd(widths[i])).join('  ');
  return [line(cols), line(widths.map((w) => '-'.repeat(w))), ...rows.map((r) => line(cols.map((c) => r[c])))].join('\n');
}

const isMain = process.argv[1] !== undefined && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href;
if (isMain) {
  const args = process.argv.slice(2);
  const partial = args.includes('--partial');
  const ji = args.indexOf('--json');
  const jsonOut = ji >= 0 ? args[ji + 1] : undefined;
  const pos = args.filter((a, i) => !a.startsWith('--') && (ji < 0 || i !== ji + 1));
  if (pos.length !== 2) usage();
  const before = load(pos[0]);
  const after = load(pos[1]);
  const { violations, shrunk } = compare(before, after, { partial });
  const screens = Object.keys(before).filter((k) => partial ? after[k] !== undefined : true).length;
  const probes = Object.entries(before)
    .filter(([k]) => after[k] !== undefined)
    .reduce((n, [, r]) => n + Object.values(r.probes).filter((p) => p.visible).length, 0);
  console.log(`compare.mjs: ${screens} screen(s), ${probes} probe instance(s) compared${partial ? ' (--partial)' : ''}`);
  if (shrunk.length > 0) {
    console.log(`\n${shrunk.length} instance(s) shrank by more than 1 px (allowed):`);
    console.log(table(shrunk, ['screen', 'probe', 'before', 'after']));
  }
  if (jsonOut) writeFileSync(jsonOut, JSON.stringify({ violations, shrunk }, null, 1));
  if (violations.length > 0) {
    console.log(`\n${violations.length} violation(s):`);
    console.log(table(violations, ['screen', 'probe', 'rule', 'before', 'after']));
    process.exit(1);
  }
  console.log('\n0 violations');
}
