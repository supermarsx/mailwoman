// UX1 static gate for the admin console (t32-cfg §3.1 UX1, t32-e71a).
//
// Every file under `screens/Admin/**` is scanned (comments removed):
//   *.css.ts  - colour   a colour literal (hex, `rgb()`, `hsl()`, `oklch()` …) or a named colour
//                        (`red`, `white` …) in a style value: colours come from `vars.color.*`;
//                        the system colours of forced-colours mode (`Canvas`, `Highlight` …),
//                        `transparent` and `currentColor` are allowed;
//             - physical a physical left/right margin, padding, border, inset, `text-align`,
//                        `float` or `left`/`right` position: logical properties follow the
//                        text direction (RTL);
//             - focus    an interactive class (`cursor: pointer`, or a `:hover` / `:active`
//                        rule) with no `:focus-visible` (or `:focus-within`) rule of its own,
//                        in the style object or a `globalStyle` on that class;
//             - width    a fixed `width` / `min-width` / `inline-size` over 480 px (rem and em
//                        at 16 px) in a style object that has no `max-width`: it overflows a
//                        390 px phone.
//   *.tsx     - inline-style  a `style=` attribute: the shell CSP (`style-src 'self'`) drops a
//                        literal one in Firefox and check-size gate 3 rejects it in the build;
//                        a style belongs in a class;
//             - colour   a colour literal in the markup (`fill="#…"`, `rgb(…)`).
//
// Existing files that break a rule are listed in ALLOWLIST with the lane that owns the fix
// (t32-cfg §3.3: e77 for the screens it rebuilds, e76 for what its walk assigns); an entry
// that no longer matches fails, so the list only shrinks. The self-test block shows each
// check finding a seeded defect. `MW_ADMIN_STYLE_SRC` points the scan at another
// `screens/Admin` tree (used to show the gate failing on a seeded copy).

import { describe, expect, it } from 'vitest';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative, resolve, sep } from 'node:path';

export type AdminRule = 'colour' | 'physical' | 'focus' | 'width' | 'inline-style';

export interface AdminFinding {
  file: string;
  line: number;
  rule: AdminRule;
  text: string;
}

interface Allowed {
  file: string;
  rule: AdminRule;
  text: string;
  /** The lane that owns the fix. */
  owner: 'e77' | 'e76';
  reason: string;
}

const SHELL_REASON =
  'shell control at HEAD before the UX1 gate; it relies on the global :focus-visible ring in theme/themes.css.ts and needs its own rule (shell.css.ts is outside the e71a file set; REQUEST in state.md)';

// Each entry: the finding as reported, the owner, and why it is here. A stale entry fails.
const ALLOWLIST: Allowed[] = [
  {
    file: 'Plugins/styles.css.ts',
    rule: 'focus',
    text: 'ghost: interactive without :focus-visible',
    owner: 'e77',
    reason: 'Plugins screen button at HEAD; e77 rebuilds Plugins/** on the templates (t32-cfg §4.12) and drops this class',
  },
  {
    file: 'Plugins/styles.css.ts',
    rule: 'focus',
    text: 'check: interactive without :focus-visible',
    owner: 'e77',
    reason: 'Plugins checkbox label at HEAD; e77 rebuilds Plugins/** on the templates (t32-cfg §4.12) and drops this class',
  },
  ...(['navItem', 'paletteOption', 'chip', 'sortButton', 'lock'] as const).map(
    (name): Allowed => ({
      file: 'shell/shell.css.ts',
      rule: 'focus',
      text: `${name}: interactive without :focus-visible`,
      owner: 'e76',
      reason: SHELL_REASON,
    }),
  ),
  {
    file: 'shell/BarChart.tsx',
    rule: 'inline-style',
    text: 'style= attribute',
    owner: 'e76',
    reason: 'bar width set through the CSSOM (dynamic, so CSP and check-size gate 3 allow it); replace with a <meter> or a class scale when e76 walks the charts',
  },
];

// ── source handling ─────────────────────────────────────────────────────────

/** Blank out comments, keeping offsets and newlines. */
export function stripComments(src: string): string {
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
    if (c === '/' && n === '/') {
      const end = src.indexOf('\n', i);
      const stop = end < 0 ? src.length : end;
      out += ' '.repeat(stop - i);
      i = stop;
      continue;
    }
    if (c === '"' || c === "'" || c === '`') quote = c;
    out += c;
    i += 1;
  }
  return out;
}

const lineOf = (src: string, index: number): number => src.slice(0, index).split('\n').length;

/** The text between the bracket at `open` and its partner (exclusive), skipping strings. */
function balanced(src: string, open: number): string {
  const pairs: Record<string, string> = { '(': ')', '{': '}', '[': ']' };
  const stack: string[] = [];
  let quote: string | null = null;
  for (let i = open; i < src.length; i += 1) {
    const c = src[i]!;
    if (quote) {
      if (c === '\\') i += 1;
      else if (c === quote) quote = null;
      continue;
    }
    if (c === '"' || c === "'" || c === '`') quote = c;
    else if (pairs[c]) stack.push(pairs[c]!);
    else if (c === stack[stack.length - 1]) {
      stack.pop();
      if (stack.length === 0) return src.slice(open + 1, i);
    }
  }
  return src.slice(open + 1);
}

const TS_VALUE = String.raw`'(?:[^'\\\n]|\\.)*'|"(?:[^"\\\n]|\\.)*"|\`(?:[^\`\\]|\\.)*\`|-?[\w.$]+(?:\[[^\]\n]*\])*`;

interface Decl {
  prop: string;
  value: string;
  index: number;
}

/** `prop: value` pairs of style objects (value as written). */
function decls(ts: string): Decl[] {
  const re = new RegExp(String.raw`(?<![\w$.'"-])(?:([a-zA-Z]+)|'([-a-z]+)')\s*:\s*(${TS_VALUE})`, 'g');
  return [...ts.matchAll(re)].map((m) => ({ prop: m[1] ?? m[2]!, value: m[3]!, index: m.index! }));
}

/** `const name = '<literal>'` in the file, to resolve a value given by name. */
function consts(ts: string): Map<string, string> {
  const re = /\bconst\s+([\w$]+)\s*(?::\s*[\w<>]+\s*)?=\s*('(?:[^'\\\n]|\\.)*'|"(?:[^"\\\n]|\\.)*"|`(?:[^`\\]|\\.)*`)/g;
  return new Map([...ts.matchAll(re)].map((m) => [m[1]!, m[2]!]));
}

function valueText(raw: string, named: Map<string, string>): string {
  let v = raw.trim();
  const c = named.get(v);
  if (c !== undefined) v = c;
  if (/^['"`]/.test(v)) v = v.slice(1, -1);
  return v.replace(/\$\{\s*([^}]*?)\s*\}/g, '$1');
}

const COLOUR_LITERAL = /#[0-9a-fA-F]{3,8}\b|\b(?:rgba?|hsla?|hwb|lab|lch|oklab|oklch|color)\(/;
const NAMED_COLOURS =
  /\b(?:black|white|red|green|blue|yellow|orange|purple|gray|grey|silver|maroon|navy|teal|olive|lime|aqua|fuchsia|pink|brown|gold|cyan|magenta|indigo|violet|crimson|tomato|coral|salmon|khaki|beige|ivory|tan|plum|orchid)\b/i;
const COLOUR_PROP =
  /^(?:color|background|backgroundColor|border|borderColor|border[A-Z][A-Za-z]*Color|borderBlock[A-Za-z]*|borderInline[A-Za-z]*|outline|outlineColor|fill|stroke|boxShadow|textShadow|caretColor|accentColor|textDecorationColor|columnRuleColor)$/;
const PHYSICAL_PROP =
  /^(?:(?:margin|padding|border|scroll-margin|scroll-padding)-(?:left|right)(?:-[a-z]+)?|(?:margin|padding|border|scrollMargin|scrollPadding)(?:Left|Right)[A-Za-z]*|left|right)$/;
const ALIGN_PROP = /^(?:textAlign|float|clear)$/;
const WIDTH_PROP = /^(?:width|minWidth|inlineSize|minInlineSize)$/;
const MAX_WIDTH = /\b(?:maxWidth|maxInlineSize)\s*:/;
const MAX_FIXED_PX = 480;

/** A plain length in px (rem/em at 16 px), or null when it is not a fixed length. */
export function fixedPx(value: string): number | null {
  const m = /^(-?\d*\.?\d+)(px|rem|em)$/.exec(value.trim());
  if (!m) return null;
  const n = parseFloat(m[1]!);
  return m[2] === 'px' ? n : n * 16;
}

/** Style objects: `const name = style(...)` / `style([...])`, with the name when exported as a const. */
function styleObjects(code: string): { name: string | null; body: string; index: number }[] {
  const out: { name: string | null; body: string; index: number }[] = [];
  for (const m of code.matchAll(/(?:\bconst\s+([\w$]+)\s*=\s*)?\bstyle\s*\(/g)) {
    const open = m.index! + m[0].length - 1;
    out.push({ name: m[1] ?? null, body: balanced(code, open), index: m.index! });
  }
  return out;
}

/** `globalStyle(selector, {...})` calls: the selector text and the object. */
function globalStyles(code: string): { selector: string; body: string; index: number }[] {
  const out: { selector: string; body: string; index: number }[] = [];
  for (const m of code.matchAll(/\bglobalStyle\s*\(/g)) {
    const open = m.index! + m[0].length - 1;
    const args = balanced(code, open);
    const comma = args.search(/,\s*\{/);
    out.push({ selector: comma < 0 ? args : args.slice(0, comma), body: comma < 0 ? '' : args.slice(comma + 1), index: m.index! });
  }
  return out;
}

const INTERACTIVE = /cursor\s*:\s*'pointer'|['"`][^'"`]*:(?:hover|active)\b[^'"`]*['"`]\s*:/;
const FOCUS_RULE = /:focus-visible|:focus-within/;

/** All findings in one file. `rel` is the path under `screens/Admin/`, with `/`. */
export function scanAdmin(rel: string, src: string): AdminFinding[] {
  const code = stripComments(src);
  const out: AdminFinding[] = [];
  const add = (rule: AdminRule, index: number, text: string): void => {
    out.push({ file: rel, line: lineOf(code, index), rule, text: text.replace(/\s+/g, ' ').trim() });
  };

  if (rel.endsWith('.css.ts')) {
    const named = consts(code);
    for (const d of decls(code)) {
      const value = valueText(d.value, named);
      if (PHYSICAL_PROP.test(d.prop)) add('physical', d.index, `${d.prop}: ${value}`);
      if (ALIGN_PROP.test(d.prop) && /^(?:left|right)$/.test(value.trim())) add('physical', d.index, `${d.prop}: ${value}`);
      if (COLOUR_PROP.test(d.prop) && /^['"`]/.test(d.value.trim()) && NAMED_COLOURS.test(value)) add('colour', d.index, `${d.prop}: ${value}`);
    }
    const lit = /'(?:[^'\\\n]|\\.)*'|"(?:[^"\\\n]|\\.)*"|`(?:[^`\\]|\\.)*`/g;
    for (const m of code.matchAll(lit)) if (COLOUR_LITERAL.test(m[0])) add('colour', m.index!, m[0]);

    const globals = globalStyles(code);
    for (const s of styleObjects(code)) {
      if (INTERACTIVE.test(s.body) && !FOCUS_RULE.test(s.body)) {
        const name = s.name;
        const covered =
          name !== null && globals.some((g) => g.selector.includes(`\${${name}}`) && FOCUS_RULE.test(g.selector));
        if (!covered) add('focus', s.index, `${name ?? 'style'}: interactive without :focus-visible`);
      }
      if (!MAX_WIDTH.test(s.body)) {
        for (const d of decls(s.body)) {
          if (!WIDTH_PROP.test(d.prop)) continue;
          const px = fixedPx(valueText(d.value, named));
          if (px !== null && px > MAX_FIXED_PX) add('width', s.index, `${s.name ?? 'style'}: ${d.prop} ${valueText(d.value, named)} without max-width`);
        }
      }
    }
    for (const g of globals) {
      if (MAX_WIDTH.test(g.body)) continue;
      for (const d of decls(g.body)) {
        if (!WIDTH_PROP.test(d.prop)) continue;
        const px = fixedPx(valueText(d.value, named));
        if (px !== null && px > MAX_FIXED_PX) add('width', g.index, `globalStyle: ${d.prop} ${valueText(d.value, named)} without max-width`);
      }
    }
  } else if (rel.endsWith('.tsx')) {
    for (const m of code.matchAll(/<[A-Za-z][^<>]*?\sstyle\s*=\s*(\{|"|')/g)) add('inline-style', m.index!, 'style= attribute');
    for (const m of code.matchAll(/\b(?:fill|stroke|color|stop-color|background)\s*=\s*["'{]\s*['"]?(#[0-9a-fA-F]{3,8}\b|(?:rgba?|hsla?|oklch)\([^)'"]*\))/g)) {
      add('colour', m.index!, m[1]!);
    }
  }
  return out;
}

// ── the tree ────────────────────────────────────────────────────────────────

const ADMIN = resolve(process.env['MW_ADMIN_STYLE_SRC'] ?? resolve(process.cwd(), 'src/screens/Admin'));

function sourceFiles(dir: string): string[] {
  const out: string[] = [];
  for (const name of readdirSync(dir)) {
    const full = join(dir, name);
    if (statSync(full).isDirectory()) out.push(...sourceFiles(full));
    else if ((name.endsWith('.css.ts') || name.endsWith('.tsx')) && !/\.test\.tsx?$/.test(name)) out.push(full);
  }
  return out;
}

const fmt = (f: AdminFinding): string => `${f.file}:${f.line} [${f.rule}] ${f.text}`;

describe('admin style gate (UX1): tokens, logical sides, focus, phone width, no inline style', () => {
  const files = sourceFiles(ADMIN).map((full) => relative(ADMIN, full).split(sep).join('/'));
  const findings = files.flatMap((rel) => scanAdmin(rel, readFileSync(join(ADMIN, rel), 'utf8')));
  const allowedBy = (f: AdminFinding): Allowed | undefined =>
    ALLOWLIST.find((a) => a.file === f.file && a.rule === f.rule && a.text === f.text);

  it('scans the whole admin tree', () => {
    expect(files.length).toBeGreaterThan(30);
    expect(files).toContain('shell/shell.css.ts');
    expect(files).toContain('shell/ux.css.ts');
    expect(files).toContain('index.tsx');
  });

  for (const rule of ['colour', 'physical', 'focus', 'width', 'inline-style'] as AdminRule[]) {
    it(`no ${rule} findings outside the allowlist`, () => {
      expect(findings.filter((f) => f.rule === rule && !allowedBy(f)).map(fmt)).toEqual([]);
    });
  }

  it('every allowlist entry names an owner and a reason, and still matches', () => {
    for (const a of ALLOWLIST) {
      expect(a.reason.length, `${a.file} ${a.text}`).toBeGreaterThan(20);
      expect(findings.some((f) => allowedBy(f) === a), `stale allowlist entry ${a.file} ${a.text}`).toBe(true);
    }
  });
});

describe('admin style gate: each check finds a seeded defect', () => {
  const rules = (rel: string, src: string): AdminRule[] => scanAdmin(rel, src).map((f) => f.rule);

  it('a literal colour, a colour function and a named colour; not a token, a system colour or transparent', () => {
    expect(rules('x.css.ts', "export const a = style({ color: '#ff0000' });")).toContain('colour');
    expect(rules('x.css.ts', "export const a = style({ background: 'rgba(0,0,0,0.4)' });")).toContain('colour');
    expect(rules('x.css.ts', "export const a = style({ borderColor: 'white' });")).toContain('colour');
    expect(rules('x.css.ts', 'export const a = style({ boxShadow: `inset 3px 0 0 red` });')).toContain('colour');
    const ok =
      "export const a = style({ color: vars.color.text, background: 'transparent', borderColor: 'currentColor', '@media': { '(forced-colors: active)': { border: `1px solid CanvasText`, outline: '2px solid Highlight' } } });";
    expect(rules('x.css.ts', ok)).toEqual([]);
  });

  it('a physical side: margin-left, paddingRight, left, text-align right', () => {
    expect(rules('x.css.ts', "export const a = style({ marginLeft: '4px' });")).toContain('physical');
    expect(rules('x.css.ts', "export const a = style({ 'margin-left': '4px' });")).toContain('physical');
    expect(rules('x.css.ts', 'export const a = style({ paddingRight: vars.space[2] });')).toContain('physical');
    expect(rules('x.css.ts', 'export const a = style({ position: "absolute", left: 0 });')).toContain('physical');
    expect(rules('x.css.ts', "export const a = style({ textAlign: 'right' });")).toContain('physical');
    expect(rules('x.css.ts', "export const a = style({ marginInlineStart: '4px', insetInlineEnd: 0, textAlign: 'start' });")).toEqual([]);
  });

  it('an interactive class without :focus-visible; not one with its own rule or a globalStyle on it', () => {
    expect(rules('x.css.ts', "export const a = style({ cursor: 'pointer' });")).toContain('focus');
    expect(rules('x.css.ts', "export const a = style({ selectors: { '&:hover': { color: vars.color.text } } });")).toContain('focus');
    expect(rules('x.css.ts', "export const a = style({ cursor: 'pointer', selectors: { '&:focus-visible': { outline: 'none' } } });")).toEqual([]);
    expect(
      rules('x.css.ts', "export const a = style({ selectors: { '&:hover': {} } });\nglobalStyle(`${a}:focus-visible`, { outline: 'none' });"),
    ).toEqual([]);
    expect(rules('x.css.ts', "export const a = style({ display: 'flex' });")).toEqual([]);
  });

  it('a fixed width over 480 px without max-width; not with max-width, not min() or a percentage', () => {
    expect(rules('x.css.ts', "export const a = style({ width: '600px' });")).toContain('width');
    expect(rules('x.css.ts', "export const a = style({ minWidth: '40rem' });")).toContain('width');
    expect(rules('x.css.ts', "globalStyle('.x', { width: '500px' });")).toContain('width');
    expect(rules('x.css.ts', "export const a = style({ width: '600px', maxWidth: '100%' });")).toEqual([]);
    expect(rules('x.css.ts', "export const a = style({ width: 'min(calc(100vw - 32px), 720px)' });")).toEqual([]);
    expect(rules('x.css.ts', "export const a = style({ width: '480px' });")).toEqual([]);
    expect(fixedPx('31rem')).toBe(496);
  });

  it('a style attribute or a literal colour in markup', () => {
    expect(rules('x.tsx', 'const v = <div style={{ color: x }} />;')).toContain('inline-style');
    expect(rules('x.tsx', 'const v = <span class="a" style="display:none" />;')).toContain('inline-style');
    expect(rules('x.tsx', 'const v = <path fill="#123456" />;')).toContain('colour');
    expect(rules('x.tsx', "const v = <path stroke={'rgb(1,2,3)'} />;")).toContain('colour');
    expect(rules('x.tsx', 'const v = <a href="#/users" class={css.x}>Users</a>; // style={{}}')).toEqual([]);
    expect(rules('x.tsx', 'const v = <path fill="currentColor" />;')).toEqual([]);
  });
});
