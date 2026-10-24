// Checkbox and radio styles (t32-h12), checked on the source.
//
// Every native checkbox and radio takes its size, accent and margin from one rule in
// `styles/checks.css.ts`. Two things brought back broken boxes before, and are refused
// here in every `*.css` / `*.css.ts` file under `src/`:
//   - a rule whose selector targets a checkbox or radio input (`… input[type="checkbox"]`)
//     anywhere else and sets its size or margin: a higher-specificity rule like that
//     overrides the shared box on one screen;
//   - a rule for a bare `… input` descendant that sets padding, border, background or
//     size without leaving checkboxes and radios out (`:not([type='checkbox'], …)`):
//     `.field input` did that, and the admin console drew checkboxes stretched across
//     the row with their text underneath.
// The self-test block runs both checks on seeded text, so a check that stops finding
// anything fails here.

import { describe, expect, it } from 'vitest';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative, resolve, sep } from 'node:path';

// `MW_CHECKS_SRC` points the scan at another tree (used to show it failing on HEAD before t32-h12).
const SRC = process.env['MW_CHECKS_SRC'] ?? resolve(__dirname, '..');
const OWNER = 'styles/checks.css.ts';

// Each entry needs a reason; an entry that matches nothing fails the test (no stale exemptions).
const ALLOWLIST: { finding: string; reason: string }[] = [
  {
    finding: 'modules/notes/notes.css: `.notes__tag-add input` styles every input, checkboxes and radios included',
    reason: 'the tag-add box holds one text input (modules/notes/index.tsx), no checkbox',
  },
  {
    finding:
      "modules/notes/notes.css: `:root[data-appearance='light'] .notes__tag-add input` styles every input, checkboxes and radios included",
    reason: 'same text input, light-appearance background',
  },
];

/** Comments out: block comments everywhere, `//` line comments in `.css.ts` (not `://`). */
function stripComments(src: string, ts: boolean): string {
  const noBlock = src.replace(/\/\*[\s\S]*?\*\//g, '');
  return ts ? noBlock.replace(/(^|[^:'"`])\/\/.*$/gm, '$1') : noBlock;
}

interface Rule {
  selector: string;
  body: string;
}

/** `selector { body }` pairs of a plain stylesheet (nested at-rule blocks are unwrapped). */
export function cssRules(css: string): Rule[] {
  const out: Rule[] = [];
  const re = /([^{}]+)\{([^{}]*)\}/g;
  for (let m = re.exec(css); m; m = re.exec(css)) {
    out.push({ selector: m[1]!.replace(/^[^;]*@[^{]*$/, '').trim(), body: m[2]! });
  }
  return out;
}

/** `globalStyle(selector, { … })` calls and `'&…': { … }` selector blocks of a `.css.ts` file. */
export function tsRules(ts: string): Rule[] {
  const out: Rule[] = [];
  const global = /globalStyle\(\s*([`'"])([\s\S]*?)\1\s*,\s*\{([^{}]*)\}/g;
  for (let m = global.exec(ts); m; m = global.exec(ts)) out.push({ selector: m[2]!, body: m[3]! });
  const nested = /(['"])(&[^'"]*)\1\s*:\s*\{([^{}]*)\}/g;
  for (let m = nested.exec(ts); m; m = nested.exec(ts)) out.push({ selector: m[2]!, body: m[3]! });
  return out;
}

const CHECK_TARGET = /input\[type=["']?(checkbox|radio)["']?\](?::[\w-]+(?:\([^)]*\))?)*\s*$/;
const BARE_INPUT = /(^|[\s>+~])input(?::(?!not)[\w-]+(?:\([^)]*\))?)*\s*$/;
const SIZE_PROPS = /(^|[\s;{,])(width|height|inline-size|block-size|min-width|min-height|min-inline-size|min-block-size|margin[\w-]*|inlineSize|blockSize|minWidth|minHeight|minInlineSize|minBlockSize)\s*:/;
const BOX_PROPS = /(^|[\s;{,])(padding[\w-]*|border(?!-?radius)[\w-]*|background[\w-]*|width|height|inline-size|block-size|inlineSize|blockSize)\s*:/;

export function findingsIn(file: string, rules: Rule[]): string[] {
  const found: string[] = [];
  for (const { selector, body } of rules) {
    for (const part of selector.split(',').map((s) => s.trim())) {
      if (file !== OWNER && CHECK_TARGET.test(part) && SIZE_PROPS.test(body))
        found.push(`${file}: \`${part}\` sizes a checkbox or radio outside ${OWNER}`);
      if (BARE_INPUT.test(part) && BOX_PROPS.test(body))
        found.push(`${file}: \`${part}\` styles every input, checkboxes and radios included`);
    }
  }
  return found;
}

function walk(dir: string, acc: string[] = []): string[] {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) walk(p, acc);
    else if (/\.css(\.ts)?$/.test(name)) acc.push(p);
  }
  return acc;
}

describe('checkbox and radio styles', () => {
  it('come from styles/checks.css.ts only, and no bare `input` rule restyles them', () => {
    const found: string[] = [];
    for (const path of walk(SRC)) {
      const file = relative(SRC, path).split(sep).join('/');
      const ts = file.endsWith('.ts');
      const text = stripComments(readFileSync(path, 'utf8'), ts);
      found.push(...findingsIn(file, ts ? tsRules(text) : cssRules(text)));
    }
    const allowed = new Set(ALLOWLIST.map((a) => a.finding));
    expect(found.filter((f) => !allowed.has(f))).toEqual([]);
    expect(ALLOWLIST.filter((a) => !found.includes(a.finding)).map((a) => a.finding), 'stale allowlist entries').toEqual([]);
  });

  it('the shared rule sets the 24px target, the accent and no margin', () => {
    const src = readFileSync(join(SRC, OWNER), 'utf8');
    expect(src).toMatch(/const CHECKS = 'input\[type="checkbox"\], input\[type="radio"\]'/);
    const body = /globalStyle\(CHECKS, \{([^{}]*)\}\);/.exec(src)?.[1] ?? '';
    expect(body).toMatch(/accentColor: vars\.color\.accent,/);
    expect(body).toMatch(/inlineSize: vars\.a11y\.touchTarget,/);
    expect(body).toMatch(/blockSize: vars\.a11y\.touchTarget,/);
    expect(body).toMatch(/margin: 0,/);
  });
});

describe('checkbox style checks (self-test)', () => {
  it('finds a checkbox sized outside the shared rule', () => {
    expect(findingsIn('x/a.css.ts', tsRules('globalStyle(`${row} > input[type="checkbox"]`, { inlineSize: "16px" });'))).toHaveLength(1);
    expect(findingsIn('x/a.css', cssRules('.row input[type=radio]:checked { margin: 2px; }'))).toHaveLength(1);
  });
  it('finds a bare input rule with box styles, and not one that leaves checkboxes out', () => {
    expect(findingsIn('x/a.css', cssRules('.field input, .field textarea { padding: 4px; }'))).toHaveLength(1);
    expect(findingsIn('x/a.css', cssRules(".field input:not([type='checkbox'], [type='radio']) { padding: 4px; }"))).toEqual([]);
    expect(findingsIn('x/a.css', cssRules('.field input { font: inherit; }'))).toEqual([]);
  });
  it('leaves a label styled by what it holds alone', () => {
    expect(findingsIn('x/a.css.ts', tsRules("x = style({ selectors: { '&:has(> input[type=\"checkbox\"])': { flexDirection: 'row' } } })"))).toEqual([]);
  });
});
