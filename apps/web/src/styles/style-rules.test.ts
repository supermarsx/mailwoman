// Style rules of the t32 restyle (dark-native, Bauhaus x brutalism), checked on the source.
//
// Every `*.css` and `*.css.ts` file under `src/` is scanned (comments removed) for:
//   - a gradient: `linear-`, `radial-` or `conic-gradient` (also `repeating-`), anywhere;
//   - a `box-shadow` / `text-shadow` with a non-zero blur, outside `theme/` (shadows are hard
//     offsets from `vars.elevation.*`, or inset bars with no blur);
//   - a `border-radius` other than `0`, `50%`, `999px`, `vars.radius.*` or `var(--radius-*)`;
//   - a colour literal (hex, `rgb()`, `hsl()`, `oklch()` …) outside `theme/`, `styles/print.css.ts`
//     and the commented ALLOWLIST below (colours come from `vars.color.*`);
//   - a physical left/right margin, padding, border, `text-align`, `float` or `left`/`right`
//     position, outside `styles/print.css.ts` (paper, out of t32 scope) and the ALLOWLIST: the
//     logical properties (`margin-inline-start`, `inset-inline-end` …) follow the text direction.
//
// Limits: a value built at run time, or held in an identifier that is not a `const` literal in
// the same file, is not resolved; 4-value `margin` / `padding` / `inset` shorthands are not read
// as physical (the safe-area paddings in app.css are physical on purpose).
//
// The self-test block runs each check on a seeded defect, so a check that stops finding
// anything fails here rather than passing silently. `MW_STYLE_RULES_SRC` points the scan at
// another tree (used to show it failing on the T32-BASE sources).

import { describe, expect, it } from 'vitest';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative, resolve, sep } from 'node:path';

import {AVATAR_COLORS} from '../components/senderIcons/palette.ts';
import {contrastRatio} from '../theme/contrast.ts';

type Rule = 'gradient' | 'blurred-shadow' | 'radius' | 'colour' | 'physical';

interface Finding {
  file: string;
  line: number;
  rule: Rule;
  text: string;
}

interface Allowed {
  file: string;
  rule: Rule;
  /** The literal or declaration text as reported. */
  text: string;
  reason: string;
}

// Each entry needs a reason; an entry that matches nothing fails the test (no stale exemptions).
const ALLOWLIST: Allowed[] = [
  ...AVATAR_COLORS.map((text):Allowed=>({file:'components/senderIcons/palette.css',rule:'colour',text,reason:'stable sender identity colour; generated palette paired with white initials at AA contrast'})),
  {
    file: 'modules/calendar/calendar.css.ts',
    rule: 'colour',
    text: "'#fff'",
    reason: 'ink on a user-chosen event colour; the colour is user data with no theme counterpart',
  },
  {
    file: 'modules/keys/keys.css.ts',
    rule: 'colour',
    text: "'#ffffff'",
    reason: 'QR code frame: a QR code needs dark modules on a light field to scan, in every theme',
  },
  {
    file: 'modules/zeroaccess/styles.css.ts',
    rule: 'colour',
    text: "'#ffffff'",
    reason: 'QR code frame: a QR code needs dark modules on a light field to scan, in every theme',
  },
  {
    file: 'components/person-avatar.css',
    rule: 'colour',
    text: '#fff',
    reason: 'white avatar initials pair with the contrast-checked identity palette, not the interface theme',
  },
  // t69: hatching marks tentative and invited events so their status is not shown by colour alone.
  {
    file: 'components/event-card/eventCard.css.ts',
    rule: 'gradient',
    text: 'repeating-linear-gradient(',
    reason: 'hatch on a tentative RSVP dot and an invited agenda block: the status is not shown by colour alone',
  },
  {
    file: 'modules/calendar/calendar.css.ts',
    rule: 'gradient',
    text: 'repeating-linear-gradient(',
    reason: 'hatch on a tentative event chip: the status is not shown by colour alone',
  },
  {
    file: 'modules/calendar/calendar.css.ts',
    rule: 'colour',
    text: "'repeating-linear-gradient(135deg, rgba(255,255,255,0.38) 0 4px, transparent 4px 8px)'",
    reason: 'light hatch over a user-chosen event colour; the colour is user data with no theme counterpart',
  },
  {
    file: 'components/event-card/eventCard.css.ts',
    rule: 'colour',
    text: '`${vars.border.rule} solid rgba(0,0,0,0.15)`',
    reason: 'edge on a user-chosen agenda block colour; the colour is user data with no theme counterpart',
  },
  {
    file: 'screens/Settings/linkedAccounts.css.ts',
    rule: 'gradient',
    text: 'radial-gradient(',
    reason: 'the dot grid of a drag handle, drawn in currentColor; a pattern, not a shaded surface',
  },
  {
    file: 'components/selectionBar.css.ts',
    rule: 'physical',
    text: 'left: 50%',
    reason: 'centring: left 50% with translate(-50%) centres the bar in both directions; a logical start would shift it in RTL',
  },
];

const COLOUR_EXEMPT = ['theme/', 'styles/print.css.ts'];
const SHADOW_EXEMPT = ['theme/'];
const PHYSICAL_EXEMPT = ['styles/print.css.ts'];

// ── source handling ─────────────────────────────────────────────────────────

/** Blank out comments, keeping offsets and newlines. `//` only outside strings and URLs. */
export function stripComments(src: string, ts: boolean): string {
  let out = '';
  let i = 0;
  let quote: string | null = null;
  while (i < src.length) {
    const c = src[i]!;
    const n = src[i + 1];
    if (quote) {
      out += c;
      if (c === '\\') {
        out += n ?? '';
        i += 2;
        continue;
      }
      if (c === quote) quote = null;
      i += 1;
      continue;
    }
    if (c === '/' && n === '*') {
      const end = src.indexOf('*/', i + 2);
      const stop = end < 0 ? src.length : end + 2;
      out += src.slice(i, stop).replace(/[^\n]/g, ' ');
      i = stop;
      continue;
    }
    if (ts && c === '/' && n === '/') {
      const end = src.indexOf('\n', i);
      const stop = end < 0 ? src.length : end;
      out += ' '.repeat(stop - i);
      i = stop;
      continue;
    }
    if (c === '"' || c === "'" || (ts && c === '`')) quote = c;
    out += c;
    i += 1;
  }
  return out;
}

interface Decl {
  prop: string;
  value: string;
  index: number;
}

/** Declarations of a plain stylesheet: `prop: value` followed by `;` or `}`. */
function cssDecls(css: string): Decl[] {
  return [...css.matchAll(/(?<![\w-])(-?[a-z][a-z-]*)\s*:\s*([^;{}]+?)\s*(?=[;}])/g)].map((m) => ({
    prop: m[1]!,
    value: m[2]!,
    index: m.index!,
  }));
}

const TS_VALUE = String.raw`'(?:[^'\\\n]|\\.)*'|"(?:[^"\\\n]|\\.)*"|\`(?:[^\`\\]|\\.)*\`|-?[\w.$]+(?:\[[^\]\n]*\])*`;

/** Properties of a vanilla-extract style object: `prop: value` (value as written). */
function tsDecls(ts: string): Decl[] {
  const re = new RegExp(String.raw`(?<![\w$.'"-])(?:([a-zA-Z]+)|'([-a-z]+)')\s*:\s*(${TS_VALUE})`, 'g');
  return [...ts.matchAll(re)].map((m) => ({ prop: m[1] ?? m[2]!, value: m[3]!, index: m.index! }));
}

/** `const name = <literal>` in the file, to resolve a value given by name. */
function tsConsts(ts: string): Map<string, string> {
  const re = new RegExp(String.raw`\bconst\s+([\w$]+)\s*(?::\s*[\w<>]+\s*)?=\s*('(?:[^'\\\n]|\\.)*'|"(?:[^"\\\n]|\\.)*"|\`(?:[^\`\\]|\\.)*\`)`, 'g');
  return new Map([...ts.matchAll(re)].map((m) => [m[1]!, m[2]!]));
}

/** The text of a value: a literal's contents with `${expr}` replaced by `expr`. */
function valueText(raw: string, consts: Map<string, string>): string {
  let v = raw.trim();
  const named = consts.get(v);
  if (named !== undefined) v = named;
  if (/^['"`]/.test(v)) v = v.slice(1, -1);
  return v.replace(/\$\{\s*([^}]*?)\s*\}/g, '$1');
}

const lineOf = (src: string, index: number) => src.slice(0, index).split('\n').length;

/** Split on commas outside parentheses. */
function splitTop(v: string): string[] {
  const parts: string[] = [];
  let depth = 0;
  let cur = '';
  for (const c of v) {
    if (c === '(') depth += 1;
    if (c === ')') depth -= 1;
    if (c === ',' && depth === 0) {
      parts.push(cur);
      cur = '';
    } else cur += c;
  }
  parts.push(cur);
  return parts.map((p) => p.trim()).filter(Boolean);
}

const LENGTH = /^-?(?:\d*\.)?\d+(?:px|em|rem|vh|vw|%)?$/;

/** True when any shadow in the list has a third length (the blur) that is not zero. */
export function hasBlur(value: string): boolean {
  return splitTop(value).some((shadow) => {
    const lengths = shadow.split(/\s+/).filter((t) => LENGTH.test(t));
    return lengths.length >= 3 && parseFloat(lengths[2]!) !== 0;
  });
}

const RADIUS_OK = /^(?:0|0px|50%|999px|vars\.radius\.(?:sm|md|lg|pill)|var\(--radius-(?:sm|md|lg|pill)\))$/;

/** True when every part of a radius value is an allowed one. */
export function radiusOk(value: string): boolean {
  const parts = value.split(/[\s/]+/).filter(Boolean);
  return parts.length > 0 && parts.every((p) => RADIUS_OK.test(p));
}

const GRADIENT = /\b(?:repeating-)?(?:linear|radial|conic)-gradient\s*\(/g;
const COLOUR_LITERAL = /#[0-9a-fA-F]{3,8}\b|\b(?:rgba?|hsla?|hwb|lab|lch|oklab|oklch|color)\(/;
const RADIUS_PROP = /^border(?:-[a-z]+)*-radius$|^border[A-Za-z]*Radius$/;
const SHADOW_PROP = /^(?:box-shadow|text-shadow|boxShadow|textShadow)$/;
const PHYSICAL_PROP =
  /^(?:(?:margin|padding|border|scroll-margin|scroll-padding)-(?:left|right)(?:-[a-z]+)?|(?:margin|padding|border|scrollMargin|scrollPadding)(?:Left|Right)[A-Za-z]*|left|right)$/;
const ALIGN_PROP = /^(?:text-align|textAlign|float|clear)$/;

/** All findings in one file's source. `rel` is the path under `src/`, with `/`. */
export function scan(rel: string, src: string): Finding[] {
  const ts = rel.endsWith('.ts');
  const code = stripComments(src, ts);
  const out: Finding[] = [];
  const add = (rule: Rule, index: number, text: string) =>
    out.push({ file: rel, line: lineOf(code, index), rule, text: text.replace(/\s+/g, ' ').trim() });
  const under = (prefixes: string[]) => prefixes.some((p) => rel === p || rel.startsWith(p));

  for (const m of code.matchAll(GRADIENT)) add('gradient', m.index!, m[0]);

  const consts = ts ? tsConsts(code) : new Map<string, string>();
  const decls = ts ? tsDecls(code) : cssDecls(code);
  for (const d of decls) {
    const value = valueText(d.value, consts);
    if (SHADOW_PROP.test(d.prop) && !under(SHADOW_EXEMPT) && hasBlur(value)) {
      add('blurred-shadow', d.index, `${d.prop}: ${value}`);
    }
    if (RADIUS_PROP.test(d.prop) && !radiusOk(value)) add('radius', d.index, `${d.prop}: ${value}`);
    if (!under(PHYSICAL_EXEMPT)) {
      if (PHYSICAL_PROP.test(d.prop)) add('physical', d.index, `${d.prop}: ${value}`);
      if (ALIGN_PROP.test(d.prop) && /^(?:left|right)$/.test(value.trim())) add('physical', d.index, `${d.prop}: ${value}`);
    }
  }

  if (!under(COLOUR_EXEMPT)) {
    if (ts) {
      // Colours in a style file are string or template literals.
      const lit = /'(?:[^'\\\n]|\\.)*'|"(?:[^"\\\n]|\\.)*"|`(?:[^`\\]|\\.)*`/g;
      for (const m of code.matchAll(lit)) if (COLOUR_LITERAL.test(m[0])) add('colour', m.index!, m[0]);
    } else {
      // Only declaration values: `#name` in a selector is an id, not a colour.
      for (const d of decls) {
        const hit = d.value.match(new RegExp(COLOUR_LITERAL.source, 'g'));
        for (const h of hit ?? []) add('colour', d.index, h);
      }
    }
  }
  return out;
}

// ── the tree ────────────────────────────────────────────────────────────────

const SRC = resolve(process.env['MW_STYLE_RULES_SRC'] ?? resolve(process.cwd(), 'src'));

function styleFiles(dir: string): string[] {
  const out: string[] = [];
  for (const name of readdirSync(dir)) {
    const full = join(dir, name);
    if (statSync(full).isDirectory()) out.push(...styleFiles(full));
    else if (name.endsWith('.css') || name.endsWith('.css.ts')) out.push(full);
  }
  return out;
}

const fmt = (f: Finding) => `${f.file}:${f.line} [${f.rule}] ${f.text}`;

describe('style rules (t32): flat, square, token colours, logical sides', () => {
  const files = styleFiles(SRC).map((full) => relative(SRC, full).split(sep).join('/'));
  const findings = files.flatMap((rel) => scan(rel, readFileSync(join(SRC, rel), 'utf8')));
  const allowedBy = (f: Finding) => ALLOWLIST.find((a) => a.file === f.file && a.rule === f.rule && a.text === f.text);

  it('scans the whole style tree', () => {
    // A scan that finds no files passes every rule; make sure it saw the tree.
    expect(files.length).toBeGreaterThan(40);
    expect(files).toContain('styles/app.css');
    expect(files).toContain('theme/themes.css.ts');
  });

  for (const rule of ['gradient', 'blurred-shadow', 'radius', 'colour', 'physical'] as Rule[]) {
    it(`no ${rule} findings outside the allowlist`, () => {
      expect(findings.filter((f) => f.rule === rule && !allowedBy(f)).map(fmt)).toEqual([]);
    });
  }

  it('every allowlist entry has a reason and still matches a literal', () => {
    for (const a of ALLOWLIST) {
      expect(a.reason.length, `${a.file} ${a.text}`).toBeGreaterThan(20);
      expect(findings.some((f) => allowedBy(f) === a), `stale allowlist entry ${a.file} ${a.text}`).toBe(true);
    }
  });
});

describe('style rules: each check finds a seeded defect', () => {
  const rules = (rel: string, src: string) => scan(rel, src).map((f) => f.rule);

  it('gradient, in plain CSS and in a style object', () => {
    expect(rules('styles/x.css', '.a { background: linear-gradient(#000, #111); }')).toContain('gradient');
    expect(rules('components/x.css.ts', "export const a = style({ backgroundImage: 'repeating-radial-gradient(red, blue)' });")).toContain('gradient');
    expect(rules('theme/x.css.ts', "const g = 'conic-gradient(red, blue)';")).toContain('gradient');
  });

  it('a blurred shadow, but not a hard offset or an inset bar', () => {
    expect(rules('styles/x.css', '.a { box-shadow: 0 2px 6px var(--shadow); }')).toContain('blurred-shadow');
    expect(rules('components/x.css.ts', "const soft = `0 1px 4px ${vars.color.text}`;\nexport const a = style({ boxShadow: soft });")).toContain('blurred-shadow');
    expect(rules('components/x.css.ts', 'export const a = style({ boxShadow: `inset 4px 0 0 ${vars.color.accent}, 2px 2px 0 0 #000` });')).not.toContain('blurred-shadow');
    expect(rules('theme/x.css.ts', "export const a = style({ boxShadow: '0 2px 8px black' });")).not.toContain('blurred-shadow');
  });

  it('a 6px radius, but not 0, 50%, 999px or a radius token', () => {
    expect(rules('styles/x.css', '.a { border-radius: 6px; }')).toContain('radius');
    expect(rules('components/x.css.ts', "export const a = style({ borderTopLeftRadius: '6px' });")).toContain('radius');
    expect(rules('components/x.css.ts', 'export const a = style({ borderRadius: `${vars.radius.md} 6px` });')).toContain('radius');
    const ok = ".a { border-radius: 0; } .b { border-radius: 50%; } .c { border-radius: var(--radius-pill); }";
    expect(rules('styles/x.css', ok)).toEqual([]);
    expect(rules('components/x.css.ts', "export const a = style({ borderRadius: vars.radius.sm, b: 0 }); const c = style({ borderRadius: '999px' });")).toEqual([]);
  });

  it('a colour literal outside theme/, but not an id selector or a comment', () => {
    expect(rules('styles/x.css', '.a { color: #abc; }')).toContain('colour');
    expect(rules('styles/x.css', '.a { color: rgba(0, 0, 0, 0.4); }')).toContain('colour');
    expect(rules('components/x.css.ts', "export const a = style({ background: 'rgba(0,0,0,0.4)' });")).toContain('colour');
    expect(rules('styles/x.css', '#add { color: var(--text); } /* #ffffff */')).toEqual([]);
    expect(rules('components/x.css.ts', "// background '#ffffff'\nexport const a = style({ color: vars.color.text });")).toEqual([]);
    expect(rules('theme/x.css.ts', "const ink = '#111111';")).toEqual([]);
  });

  it('a physical side, but not a logical one', () => {
    expect(rules('styles/x.css', '.a { margin-left: 4px; }')).toContain('physical');
    expect(rules('styles/x.css', '.a { position: absolute; right: 0; }')).toContain('physical');
    expect(rules('styles/x.css', '.a { text-align: left; }')).toContain('physical');
    expect(rules('components/x.css.ts', 'export const a = style({ paddingRight: vars.space[2] });')).toContain('physical');
    expect(rules('components/x.css.ts', "export const a = style({ borderLeftColor: 'transparent' });")).toContain('physical');
    expect(rules('components/x.css.ts', "export const a = style({ textAlign: 'right' });")).toContain('physical');
    const ok = '.a { margin-inline-start: 4px; inset-inline-end: 0; text-align: start; } .b:hover { color: var(--text); }';
    expect(rules('styles/x.css', ok)).toEqual([]);
    expect(rules('components/x.css.ts', "export const a = style({ paddingInlineEnd: 2, textAlign: 'center', insetInlineStart: 0 });")).toEqual([]);
  });

  it('the blur and radius parsers', () => {
    expect(hasBlur('2px 2px 0 0 #000')).toBe(false);
    expect(hasBlur('0 0 0 2px bg, 0 0 0 5px focus')).toBe(false);
    expect(hasBlur('inset 0 -3px 0 accent, 0 1px 3px rgba(0, 0, 0, 0.3)')).toBe(true);
    expect(radiusOk('vars.radius.md vars.radius.md 0 0')).toBe(true);
    expect(radiusOk('4px')).toBe(false);
  });
});

it('avatar palette exceptions remain exact generated identity colours with readable ink',()=>{
 const css=readFileSync(join(SRC,'components/senderIcons/palette.css'),'utf8');
 expect([...css.matchAll(/background-color:(#[a-f0-9]{6})/g)].map(match=>match[1])).toEqual([...AVATAR_COLORS]);
 for(const color of AVATAR_COLORS) expect(contrastRatio(color,'#ffffff'),color).toBeGreaterThanOrEqual(4.5);
});
