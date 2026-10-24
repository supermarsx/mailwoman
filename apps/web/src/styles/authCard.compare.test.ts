// @vitest-environment node
// The restyle compare's sign-in card exemption (AUTH_CARD, t32-h14) on seeded runs: the
// admin sign-in card narrowing to the shared width is not counted, and any other growth,
// on those probes or elsewhere, still is. compare.mjs is the harness script in
// e2e/restyle/; it is plain JavaScript, so it is loaded by URL.

import { describe, expect, it } from 'vitest';

interface Probe {
  visible: boolean;
  kind: string;
  interactive: boolean;
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
interface Result {
  violations: { screen: string; probe: string; rule: string }[];
  cards: { screen: string; probe: string; rule: string }[];
}
interface CompareModule {
  compare: (before: Record<string, Screen>, after: Record<string, Screen>) => Result;
  AUTH_CARD_MAX_WIDTH: number;
}

const mod = (await import(/* @vite-ignore */ new URL('../../e2e/restyle/compare.mjs', import.meta.url).href)) as CompareModule;

function probe(kind: string, w: number, h: number, extra: Partial<Probe> = {}): Probe {
  return {
    visible: true,
    kind,
    interactive: kind === 'control',
    x: 0,
    y: 0,
    w,
    h,
    fontSize: 14,
    lineHeight: 20,
    padding: [0, 0, 0, 0],
    border: [2, 2, 2, 2],
    ...extra,
  };
}

function screen(viewport: string, probes: Record<string, Probe>): Record<string, Screen> {
  return { [`${viewport}/dark/admin-login`]: { viewport, theme: 'dark', screen: 'admin-login', visibleRows: null, probes } };
}

/** The admin sign-in card as T32-BASE measured it, and as the shared-width card measures. */
const desktopBefore = (): Record<string, Probe> => ({
  'admin.login-card#0': probe('container', 778, 307.78),
  'admin.login-help#0': probe('text', 778, 17),
  'admin.login-field#0': probe('control', 778, 37),
});
const desktopAfter = (): Record<string, Probe> => ({
  'admin.login-card#0': probe('container', 360, 323.89),
  'admin.login-help#0': probe('text', 334, 46.03),
  'admin.login-field#0': probe('control', 334, 34.8),
});
const phoneBefore = (): Record<string, Probe> => ({
  'admin.login-card#0': probe('container', 412, 330),
  'admin.login-field#0': probe('control', 386, 37),
  'admin.login-submit#0': probe('control', 386, 37),
});
const phoneAfter = (): Record<string, Probe> => ({
  'admin.login-card#0': probe('container', 360, 330),
  'admin.login-field#0': probe('control', 334, 37),
  'admin.login-submit#0': probe('control', 334, 37),
});

function run(viewport: string, before: Record<string, Probe>, after: Record<string, Probe>): Result {
  return mod.compare(screen(viewport, before), screen(viewport, after));
}

describe('restyle compare: AUTH_CARD sign-in card exemption', () => {
  it('reads the shared card width from styles/authCard.css.ts', () => {
    expect(mod.AUTH_CARD_MAX_WIDTH).toBe(360);
  });

  it('does not count the narrowing: the card and its note wrapping, the phone field and button inside the gutter', () => {
    const desk = run('desktop', desktopBefore(), desktopAfter());
    expect(desk.violations).toEqual([]);
    expect(desk.cards.map((c) => c.probe).sort()).toEqual(['admin.login-card#0', 'admin.login-help#0']);
    const phone = run('phone', phoneBefore(), phoneAfter());
    expect(phone.violations).toEqual([]);
    expect(phone.cards.map((c) => c.probe).sort()).toEqual(['admin.login-field#0', 'admin.login-submit#0']);
  });

  it('still fails a seeded unrelated growth on the same screen', () => {
    const before = { ...desktopBefore(), 'admin.login-title#0': probe('text', 778, 27) };
    const after = { ...desktopAfter(), 'admin.login-title#0': probe('text', 334, 40) };
    const r = run('desktop', before, after);
    expect(r.violations).toEqual([expect.objectContaining({ probe: 'admin.login-title#0', rule: 'height +>1px' })]);
  });

  it('still fails a card wider than the shared maximum', () => {
    const after = desktopAfter();
    after['admin.login-card#0'] = probe('container', 400, 323.89);
    const r = run('desktop', desktopBefore(), after);
    expect(r.violations.map((v) => v.probe).sort()).toEqual(['admin.login-card#0', 'admin.login-help#0']);
  });

  it('still fails a card that grew by more than its wrapped text', () => {
    const after = desktopAfter();
    after['admin.login-card#0'] = probe('container', 360, 360);
    const r = run('desktop', desktopBefore(), after);
    expect(r.violations).toEqual([expect.objectContaining({ probe: 'admin.login-card#0', rule: 'height +>1px' })]);
  });

  it('still fails note growth when the card did not narrow', () => {
    const before = desktopBefore();
    before['admin.login-card#0'] = probe('container', 360, 307.78);
    const r = run('desktop', before, desktopAfter());
    expect(r.violations.map((v) => v.probe).sort()).toEqual(['admin.login-card#0', 'admin.login-help#0']);
  });

  it('still fails a phone field that got wider or changed height, and a grown padding', () => {
    const after = phoneAfter();
    after['admin.login-field#0'] = probe('control', 400, 37);
    after['admin.login-submit#0'] = probe('control', 334, 44, { padding: [4, 0, 4, 0] });
    const r = run('phone', phoneBefore(), after);
    const rules = r.violations.map((v) => `${v.probe} ${v.rule}`).sort();
    expect(rules).toContain('admin.login-field#0 phone tap target width changed >1px');
    expect(rules).toContain('admin.login-submit#0 phone tap target height changed >1px');
    expect(rules).toContain('admin.login-submit#0 padding-top +>1px');
    expect(r.cards).toEqual([]);
  });
});
