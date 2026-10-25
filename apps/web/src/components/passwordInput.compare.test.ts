// @vitest-environment node
// The restyle compare's show/hide button exemption (PASSWORD_TOGGLE, t32-h17) on seeded
// runs: a password field's end padding growing to make room for the button inside it is
// not counted while the field's box does not grow, and any other growth still is. The
// admin password field also keeps the sign-in card exemption (AUTH_CARD) on a phone.
// compare.mjs is the harness script in e2e/restyle/; it is plain JavaScript, so it is
// loaded by URL.

import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

interface Probe {
  visible: boolean;
  kind: string;
  interactive: boolean;
  label: string;
  x: number;
  y: number;
  w: number;
  h: number;
  fontSize: number;
  lineHeight: number;
  padding: number[];
  border: number[];
}
interface Screen {
  viewport: string;
  theme: string;
  screen: string;
  visibleRows: null;
  probes: Record<string, Probe>;
}
interface Row {
  screen: string;
  probe: string;
  rule: string;
}
interface Result {
  violations: Row[];
  cards: Row[];
  toggles?: Row[];
}
interface CompareModule {
  compare: (before: Record<string, Screen>, after: Record<string, Screen>) => Result;
  PASSWORD_TOGGLE_END_PADDING?: number;
}

const mod = (await import(/* @vite-ignore */ new URL('../../e2e/restyle/compare.mjs', import.meta.url).href)) as CompareModule;

function field(label: string, w: number, h: number, padding: number[], extra: Partial<Probe> = {}): Probe {
  return {
    visible: true,
    kind: 'control',
    interactive: true,
    label,
    x: 0,
    y: 0,
    w,
    h,
    fontSize: 16,
    lineHeight: 19.2,
    padding,
    border: [2, 2, 2, 2],
    ...extra,
  };
}

function screen(viewport: string, name: string, probes: Record<string, Probe>): Record<string, Screen> {
  return { [`${viewport}/dark/${name}`]: { viewport, theme: 'dark', screen: name, visibleRows: null, probes } };
}

const BEFORE_PAD = [7, 9.6, 7, 9.6];
const AFTER_PAD = [7, 40, 7, 8.6];

function run(viewport: string, name: string, before: Record<string, Probe>, after: Record<string, Probe>): Result {
  return mod.compare(screen(viewport, name, before), screen(viewport, name, after));
}

const rules = (r: Result): string[] => r.violations.map((v) => `${v.probe} ${v.rule}`).sort();

describe('restyle compare: PASSWORD_TOGGLE show/hide button exemption', () => {
  it('allows exactly the end padding the button needs: touch target + twice space[3] (components/passwordInput.css.ts)', () => {
    const css = readFileSync(new URL('./passwordInput.css.ts', import.meta.url), 'utf8');
    expect(css).toContain('paddingInlineEnd: `calc(${vars.a11y.touchTarget} + ${vars.space[3]} + ${vars.space[3]})`');
    const tokens = readFileSync(new URL('../theme/tokens.ts', import.meta.url), 'utf8');
    const target = Number(/touchTarget: '(\d+)px'/.exec(tokens)?.[1]);
    const space3 = Number(/\n\s*3: '(\d+)px'/.exec(tokens)?.[1]);
    expect(mod.PASSWORD_TOGGLE_END_PADDING).toBe(target + 2 * space3);
  });

  it('does not count the end padding of the mail, admin and settings password fields when the box did not grow', () => {
    const login = run(
      'desktop',
      'login',
      { 'login.password#0': field('Password', 306.81, 37, BEFORE_PAD) },
      { 'login.password#0': field('Password', 306.81, 34.8, AFTER_PAD) },
    );
    expect(login.violations).toEqual([]);
    expect(login.toggles?.map((t) => t.probe)).toEqual(['login.password#0']);

    const settings = run(
      'desktop',
      'settings',
      { 'settings.password#0': field('Current password', 436, 28, [3, 8, 3, 8]) },
      { 'settings.password#0': field('Current password', 436, 27.28, [3, 40, 3, 7]) },
    );
    expect(settings.violations).toEqual([]);

    const admin = run(
      'desktop',
      'admin-login',
      { 'admin.login-field#1': field('', 334, 37, BEFORE_PAD) },
      { 'admin.login-field#1': field('Password', 334, 34.8, AFTER_PAD) },
    );
    expect(admin.violations).toEqual([]);
  });

  it('keeps the sign-in card exemption for the admin password field on a phone', () => {
    const before = {
      'admin.login-card#0': field('', 412, 330, [0, 0, 0, 0], { kind: 'container', interactive: false }),
      'admin.login-field#0': field('', 386, 37, BEFORE_PAD),
      'admin.login-field#1': field('', 386, 37, BEFORE_PAD),
    };
    const after = {
      'admin.login-card#0': field('', 360, 330, [0, 0, 0, 0], { kind: 'container', interactive: false }),
      'admin.login-field#0': field('', 334, 37, BEFORE_PAD),
      'admin.login-field#1': field('Password', 334, 37, AFTER_PAD),
    };
    const r = run('phone', 'admin-login', before, after);
    expect(r.violations).toEqual([]);
    expect(r.cards.map((c) => c.probe).sort()).toEqual(['admin.login-field#0', 'admin.login-field#1']);
  });

  it('still fails an end padding past the button, a grown box, another padding side, a grown border or font', () => {
    const before = { 'login.password#0': field('Password', 306.81, 37, BEFORE_PAD) };
    const cases: [string, Probe, string[]][] = [
      ['end padding past 40 px', field('Password', 306.81, 34.8, [7, 48, 7, 8.6]), ['login.password#0 padding-right +>1px']],
      ['taller box', field('Password', 306.81, 40, AFTER_PAD), ['login.password#0 height +>1px', 'login.password#0 padding-right +>1px']],
      ['wider box', field('Password', 320, 34.8, AFTER_PAD), ['login.password#0 control width +>1px', 'login.password#0 padding-right +>1px']],
      ['start padding too', field('Password', 306.81, 34.8, [7, 40, 7, 20]), ['login.password#0 padding-left +>1px', 'login.password#0 padding-right +>1px']],
      ['block padding', field('Password', 306.81, 34.8, [9, 40, 7, 8.6]), ['login.password#0 padding-right +>1px', 'login.password#0 padding-top +>1px']],
      ['thicker border', field('Password', 306.81, 34.8, AFTER_PAD, { border: [2, 4, 2, 2] }), ['login.password#0 padding-right +>1px']],
      ['larger font', field('Password', 306.81, 34.8, AFTER_PAD, { fontSize: 18 }), ['login.password#0 font-size grew', 'login.password#0 padding-right +>1px']],
    ];
    for (const [name, after, expected] of cases) {
      expect(rules(run('desktop', 'login', before, { 'login.password#0': after })), name).toEqual(expected);
    }
  });

  it('still fails the same padding growth on a field that has no button', () => {
    const r = run(
      'desktop',
      'admin-login',
      { 'admin.login-field#0': field('', 334, 37, BEFORE_PAD), 'login.email#0': field('', 306.81, 37, BEFORE_PAD) },
      { 'admin.login-field#0': field('', 334, 34.8, AFTER_PAD), 'login.email#0': field('', 306.81, 34.8, AFTER_PAD) },
    );
    expect(rules(r)).toEqual(['admin.login-field#0 padding-right +>1px', 'login.email#0 padding-right +>1px']);
  });
});
