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
//
// Flex-fill exemption (t32 coordinator decision): a probe listed in FLEX_FILL is
// sized by its row or column, not by its content (a `flex: 1` field beside buttons,
// a checkbox stretched to its table cell, the message frame filling the reader).
// When the buttons beside it get narrower it takes the space they freed. Its growth
// along the listed axis is not counted when ALL of these hold:
//   - its font-size did not grow and no padding side grew by more than 1 px;
//   - per side, padding + border did not grow by more than 1 px (for a `container`
//     entry the border is inside the space it fills and is not counted);
//   - its band (the visible probes on the same screen that overlap it across the
//     axis, minus those that enclose it) did not grow along the axis by more than 1 px;
//     or, when it did (the instance took space from an element that is not a probe),
//     the smallest probe enclosing it kept its size and the whole band lies inside it;
//   - the other probes in the band shrank along the axis by at least its growth
//     minus 1 px (the space came from its neighbours).
// Every exempted instance is printed. Anything else (another rule, a growth that
// widens the band, a padding increase) is still a violation.
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

/**
 * Probes sized by their row or column. `label` narrows an entry to the matching
 * instances; `axis` is the size that may follow the neighbours.
 */
export const FLEX_FILL = [
  { probe: 'contacts.search', axis: 'w', why: 'search field, flex 1 1 auto beside New contact (contacts.css.ts input)' },
  { probe: 'tasks.convert-field', axis: 'w', why: 'message id field, flex fill beside the Mail/Event → task buttons' },
  {
    probe: 'admin.row-control',
    label: /^(Zero-access|Disable|Force password change) for /,
    axis: 'w',
    why: 'checkboxes stretched to their cell; the auto table layout shares the fixed table width between columns',
  },
  { probe: 'reader.frame', axis: 'h', container: true, why: 'message frame, flex 1 in the reader column' },
];

const probeName = (id) => id.replace(/#\d+$/, '');

function flexFillEntry(id, pb) {
  return FLEX_FILL.find((f) => f.probe === probeName(id) && (f.label === undefined || f.label.test(pb.label ?? '')));
}

const encloses = (o, p) => o.x <= p.x + EPS && o.y <= p.y + EPS && o.x + o.w >= p.x + p.w - EPS && o.y + o.h >= p.y + p.h - EPS;

/** Probes of a run that overlap `p` across `axis` (the same row for 'w', column for 'h'). */
function band(probes, id, axis) {
  const p = probes[id];
  return Object.entries(probes).filter(([oid, o]) => {
    if (oid === id || !o.visible) return false;
    if (encloses(o, p)) return false;
    return axis === 'w' ? o.y < p.y + p.h - EPS && o.y + o.h > p.y + EPS : o.x < p.x + p.w - EPS && o.x + o.w > p.x + EPS;
  });
}

function extent(items, axis) {
  const lo = Math.min(...items.map((o) => (axis === 'w' ? o.x : o.y)));
  const hi = Math.max(...items.map((o) => (axis === 'w' ? o.x + o.w : o.y + o.h)));
  return hi - lo;
}

/**
 * Why a growth of instance `id` along `axis` is a flex-fill growth, or null when it is not.
 * `b`/`a` are the screen results before and after.
 */
export function flexFill(b, a, id, axis) {
  const pb = b.probes[id];
  const pa = a.probes[id];
  const entry = flexFillEntry(id, pb);
  if (!entry || entry.axis !== axis) return null;
  if (pa.fontSize > pb.fontSize + EPS) return null;
  for (let i = 0; i < 4; i++) {
    if (pa.padding[i] > pb.padding[i] + TOL) return null;
    const border = entry.container ? 0 : 1;
    if (pa.padding[i] + border * pa.border[i] > pb.padding[i] + border * pb.border[i] + TOL) return null;
  }
  const size = (p) => (axis === 'w' ? p.w : p.h);
  const growth = size(pa) - size(pb);
  // The band as it was before; the same instances after (an instance missing after is "vanished" anyway).
  const others = band(b.probes, id, axis).filter(([oid]) => a.probes[oid]?.visible);
  if (others.length === 0) return null;
  const before = [pb, ...others.map(([, o]) => o)];
  const after = [pa, ...others.map(([oid]) => a.probes[oid])];
  const eb = extent(before, axis);
  const ea = extent(after, axis);
  if (ea > eb + TOL) {
    // The band can also widen into space freed by an element that is not a probe (a label
    // before the field). Then the smallest probe enclosing it must keep its size and the
    // whole band must still lie inside it.
    const box = Object.entries(b.probes)
      .filter(([oid, o]) => oid !== id && o.visible && encloses(o, pb) && a.probes[oid]?.visible)
      .sort(([, x], [, y]) => x.w * x.h - y.w * y.h)[0];
    if (!box || size(a.probes[box[0]]) > size(box[1]) + TOL) return null;
    if (!after.every((o) => encloses(a.probes[box[0]], o))) return null;
  }
  const freed = others.reduce((n, [oid, o]) => n + Math.max(0, size(o) - size(a.probes[oid])), 0);
  if (freed < growth - TOL) return null;
  return { why: entry.why, growth: r1(growth), band: `${r1(eb)}->${r1(ea)}`, freed: r1(freed) };
}

const r1 = (n) => (Number.isFinite(n) ? Math.round(n * 100) / 100 : n);

/** All violations between two loaded runs, plus the instances that shrank. */
export function compare(before, after, { partial = false } = {}) {
  const violations = [];
  const shrunk = [];
  const exempt = [];
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
      const grew = (axis, rule) => {
        const ff = flexFill(b, a, id, axis);
        if (ff) exempt.push({ screen: key, probe: id, axis, before: axis === 'w' ? pb.w : pb.h, after: axis === 'w' ? pa.w : pa.h, ...ff });
        else add(key, id, rule, axis === 'w' ? pb.w : pb.h, axis === 'w' ? pa.w : pa.h);
      };
      if (pa.h > pb.h + TOL) grew('h', 'height +>1px');
      if (pb.kind === 'control' && pa.w > pb.w + TOL) grew('w', 'control width +>1px');
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
  return { violations, shrunk, exempt };
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
  const { violations, shrunk, exempt } = compare(before, after, { partial });
  const screens = Object.keys(before).filter((k) => partial ? after[k] !== undefined : true).length;
  const probes = Object.entries(before)
    .filter(([k]) => after[k] !== undefined)
    .reduce((n, [, r]) => n + Object.values(r.probes).filter((p) => p.visible).length, 0);
  console.log(`compare.mjs: ${screens} screen(s), ${probes} probe instance(s) compared${partial ? ' (--partial)' : ''}`);
  if (shrunk.length > 0) {
    console.log(`\n${shrunk.length} instance(s) shrank by more than 1 px (allowed):`);
    console.log(table(shrunk, ['screen', 'probe', 'before', 'after']));
  }
  if (exempt.length > 0) {
    console.log(`
${exempt.length} flex-fill growth(s) not counted (FLEX_FILL):`);
    console.log(table(exempt, ['screen', 'probe', 'axis', 'before', 'after', 'band', 'freed', 'why']));
  }
  if (jsonOut) writeFileSync(jsonOut, JSON.stringify({ violations, shrunk, exempt }, null, 1));
  if (violations.length > 0) {
    console.log(`\n${violations.length} violation(s):`);
    console.log(table(violations, ['screen', 'probe', 'rule', 'before', 'after']));
    process.exit(1);
  }
  console.log('\n0 violations');
}
