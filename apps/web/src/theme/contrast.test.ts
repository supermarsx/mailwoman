// The automated per-theme × per-pair WCAG contrast matrix (SPEC §17.1, §24).
//
// `contrast.ts` (t19-e6) owns the maths and the requirement table; this file is
// the gate that runs that table across every entry in `THEME_REGISTRY` and
// reports the measured ratio for each pair by name. The maths is NOT re-derived
// here — the one independent check below is a hand-computed cross-check against
// WCAG reference values, which is a different thing from a second
// implementation.
//
// Division of labour with `registry.test.ts`: that file pins the registry SHAPE
// and carries a per-theme contrast SMOKE assertion (`failingPairs` is empty).
// This file is the exhaustive matrix — per tier, per theme, with every ratio
// named — plus the properties the smoke check cannot see: that every pair is
// actually MEASURABLE, that the advisory tier stays confined to `border`, that
// a pack's declared appearance matches its palette, and that `themeCssVars`
// emits what it claims to.
//
// A CORRECTION TO AN EARLIER READING OF THIS FILE: `themeCssVars` was described
// here as "the second painting surface". It is not — it is UNWIRED. It has zero
// runtime callers (see the `themeCssVars is not wired up` block at the bottom),
// so its output currently paints nothing and its contrast shortfalls are
// latent, not shipped. The tests over it are still worth having: they pin a
// function the app intends to use and describe the state it must be in when
// somebody wires it. They are not evidence about rendered output.

import { describe, it, expect } from 'vitest';
import { THEME_LIST, type ThemeEntry } from './registry.ts';
import {
  CONTRAST_PAIRS,
  advisoryFailures,
  checkPalette,
  contrastRatio,
  failingPairs,
  parseColor,
  relativeLuminance,
  worstScrimRatio,
  type ContrastLevel,
  type ContrastResult,
} from './contrast.ts';
import { BRAND_SLOTS, THEMES } from './tokens.ts';
import { themeCssVars } from './themeCssVars.ts';
import { bodyFrameDoc } from '../viewers/sandbox.ts';

/** `fg on bg = 4.83 (min 4.5)` — the shape every failure message uses. */
function name(r: ContrastResult): string {
  return `${r.pair.fg} on ${r.pair.bg} = ${r.ratio.toFixed(2)} (min ${r.pair.min})`;
}

function atLevel(entry: ThemeEntry, level: ContrastLevel): ContrastResult[] {
  return checkPalette(entry.palette).filter((r) => r.pair.level === level);
}

const ENFORCED: readonly ContrastLevel[] = ['text', 'non-text'];

// ─────────────────────────────────────────────────────────────────────────────

describe('WCAG maths — independent cross-check', () => {
  // Deliberately small. `contrast.ts` is the implementation and `registry.test.ts`
  // already pins its reference extremes; the point here is that this suite does
  // not take the module's word for the numbers it then enforces.
  it('reproduces the WCAG reference extremes', () => {
    expect(contrastRatio('#000000', '#ffffff')).toBe(21);
    expect(contrastRatio('#ffffff', '#000000')).toBe(21); // order-independent
    expect(contrastRatio('#808080', '#808080')).toBe(1);
  });

  it('matches a pair computed by hand from the shipped light palette', () => {
    // light.text #111111 on light.bg #f4f1ea, worked through WCAG 2.2 step by
    // step (each power taken on a calculator to 6 places):
    //   text: 0x11=17 → s = 0.066667 > 0.04045 → ((s + 0.055) / 1.055)^2.4
    //         = (0.115324)^2.4 = 0.0056054; all three channels are equal, so
    //         L(text) = 0.0056054
    //   bg:   0xf4=244 → s = 0.956863 → (0.959112)^2.4 = 0.904661
    //         0xf1=241 → s = 0.945098 → (0.947960)^2.4 = 0.879622
    //         0xea=234 → s = 0.917647 → (0.921940)^2.4 = 0.822786
    //         L(bg) = 0.2126·0.904661 + 0.7152·0.879622 + 0.0722·0.822786
    //               = 0.880842
    //   ratio = (0.880842 + 0.05) / (0.0056054 + 0.05) = 16.74
    const light = THEMES.light.color;
    expect(light.text).toBe('#111111');
    expect(light.bg).toBe('#f4f1ea');
    expect(relativeLuminance([17, 17, 17])).toBeCloseTo(0.0056054, 6);
    expect(relativeLuminance([244, 241, 234])).toBeCloseTo(0.880842, 5);
    expect(contrastRatio(light.text, light.bg)).toBeCloseTo(16.74, 2);
  });

  it('holds every measured ratio inside the 1..21 range the formula allows', () => {
    for (const entry of THEME_LIST) {
      for (const r of checkPalette(entry.palette)) {
        expect(r.ratio, `${entry.id}: ${name(r)}`).toBeGreaterThanOrEqual(1);
        expect(r.ratio, `${entry.id}: ${name(r)}`).toBeLessThanOrEqual(21);
      }
    }
  });
});

// ─────────────────────────────────────────────────────────────────────────────

describe('the requirement table itself', () => {
  it('measures all 18 built-in themes', () => {
    // Guards against a pack being added to the token table but never reaching
    // the registry — the matrix below iterates the registry, so a theme missing
    // from it would be silently unmeasured rather than failing.
    // 13 packs before t32 V, plus Black, Paper, Blueprint, Terminal and
    // Primary blocks.
    expect(THEME_LIST).toHaveLength(18);
    expect(THEME_LIST.filter((e) => e.appearance === 'light')).toHaveLength(8);
    expect(THEME_LIST.filter((e) => e.appearance === 'dark')).toHaveLength(10);
  });

  it('covers every surface × foreground combination it claims to', () => {
    // 4 surfaces × (6 text colours + accent + borderStrong + focus + border)
    // + 3 filled-control labels + text-on-selection + signalText-on-signal.
    expect(CONTRAST_PAIRS).toHaveLength(4 * 10 + 3 + 1 + 1);
    const enforced = CONTRAST_PAIRS.filter((p) => p.level !== 'advisory');
    expect(enforced).toHaveLength(CONTRAST_PAIRS.length - 4);
    // No pair is declared twice — a duplicate would inflate the matrix without
    // testing anything new.
    const keys = CONTRAST_PAIRS.map((p) => `${p.fg}/${p.bg}`);
    expect(new Set(keys).size).toBe(keys.length);
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// The matrix: one describe per theme, one assertion per enforced tier, and the
// advisory tier reported alongside. Failure messages carry the measured ratio
// for every pair in the tier, so a red run names the exact token pair and by
// how much it misses.

describe.each(THEME_LIST.map((e) => [e.id, e] as const))('%s', (_id, entry) => {
  for (const level of ENFORCED) {
    it(`meets the ${level} tier on every declared pair`, () => {
      const results = atLevel(entry, level);
      expect(results.length, 'tier is present in the table').toBeGreaterThan(0);
      expect(results.filter((r) => !r.ok).map(name)).toEqual([]);
    });
  }

  it('measures every pair — no token is unmeasurable or translucent', () => {
    // `contrastRatio` returns 0 for an unparseable or translucent colour, which
    // fails any threshold — but ONLY on the enforced tiers. A `border` token
    // that stopped being measurable would slip through as "advisory", so the
    // measurability of the whole palette is asserted separately.
    const unmeasurable = checkPalette(entry.palette).filter((r) => r.ratio === 0);
    expect(unmeasurable.map(name)).toEqual([]);
    for (const pair of CONTRAST_PAIRS) {
      expect(parseColor(entry.palette[pair.fg]), `${entry.id}.${pair.fg}`).not.toBeNull();
      expect(parseColor(entry.palette[pair.bg]), `${entry.id}.${pair.bg}`).not.toBeNull();
    }
  });

  it('reports its advisory (border) shortfalls without failing on them', () => {
    // The `border` 3:1 tier is measured and NOT enforced — see the rationale in
    // `contrast.ts`: one token draws both decorative dividers (exempt from
    // §1.4.11) and real control boundaries (3:1 required), and enforcing it
    // would darken every divider app-wide. Closing it needs a component audit
    // that splits the control boundary onto its own token, not a token change.
    //
    // What IS pinned: the shortfall never spreads past `border`. If a future
    // palette edit made, say, `accent` advisory to dodge the gate, this fails.
    const advisory = advisoryFailures(entry.palette);
    expect(new Set(advisory.map((r) => r.pair.fg)).size).toBeLessThanOrEqual(1);
    for (const r of advisory) expect(r.pair.fg).toBe('border');

    const ratios = atLevel(entry, 'advisory').map(name);
    expect(ratios).toHaveLength(4);
  });

  it('paints an appearance consistent with the one it declares', () => {
    // A pack whose palette contradicts its `appearance` still passes every
    // ratio (contrast is symmetric) but breaks `color-scheme`, the UA-painted
    // scrollbars, and the light/dark pairing the system+schedule modes resolve
    // through. Ratios alone cannot catch an inverted palette; luminance can.
    const bg = relativeLuminance(parseColor(entry.palette.bg) ?? [0, 0, 0]);
    const text = relativeLuminance(parseColor(entry.palette.text) ?? [0, 0, 0]);
    if (entry.appearance === 'light') {
      expect(bg, `${entry.id}.bg luminance`).toBeGreaterThan(text);
      expect(bg).toBeGreaterThan(0.5);
    } else {
      expect(bg, `${entry.id}.bg luminance`).toBeLessThan(text);
      expect(bg).toBeLessThan(0.5);
    }
  });
});

// ─────────────────────────────────────────────────────────────────────────────

describe('high-contrast packs', () => {
  const hc = THEME_LIST.filter((e) => e.highContrast);

  it('ships one for each appearance', () => {
    expect(hc.map((e) => e.id).sort()).toEqual(['hc-dark', 'hc-light']);
  });

  it('clears the ADVISORY border tier too, unlike the ordinary packs', () => {
    // The HC packs are the answer to `prefers-contrast: more`, so the advisory
    // gap the other packs carry must not exist here. This is the one place the
    // border 3:1 IS enforced.
    for (const entry of hc) expect(advisoryFailures(entry.palette).map(name)).toEqual([]);
  });

  it('clears AAA (7:1) on body text, not just the AA floor', () => {
    for (const entry of hc) {
      for (const bg of ['bg', 'bgAlt', 'bgSink', 'surface'] as const) {
        const ratio = contrastRatio(entry.palette.text, entry.palette[bg]);
        expect(ratio, `${entry.id}: text on ${bg} = ${ratio.toFixed(2)}`).toBeGreaterThanOrEqual(7);
      }
    }
  });
});

describe('t32 token additions', () => {
  // Each new pair is enforced, so a weak value in any pack fails the matrix
  // above. These seed one weak value at a time to show the table catches it.
  const base = THEMES.dark.color;

  it.each([
    ['borderStrong', '#2a2a2a', 'borderStrong on bg'],
    ['focus', '#222222', 'focus on surface'],
    ['signalText', '#c9a000', 'signalText on signal'],
  ] as const)('fails a palette with a weak %s', (key, weak, expected) => {
    expect(failingPairs(base)).toEqual([]);
    const seeded = { ...base, [key]: weak };
    const failed = failingPairs(seeded).map((r) => `${r.pair.fg} on ${r.pair.bg}`);
    expect(failed).toContain(expected);
  });

  it('keeps the focus ring colour equal to the focus token in every pack', () => {
    for (const entry of THEME_LIST) {
      expect(entry.tokens.a11y.focusRingColor, entry.id).toBe(entry.tokens.color.focus);
      expect(entry.tokens.a11y.focusRing, entry.id).toContain(entry.tokens.color.focus);
    }
  });

  it.each(THEME_LIST.map((e) => [e.id, e] as const))(
    '%s: scrim text clears 4.5:1 over a white and a black image',
    (_id, entry) => {
      const ratio = worstScrimRatio(entry.palette.scrim, entry.palette.scrimText);
      expect(ratio, `${entry.id}: ${entry.palette.scrim}`).toBeGreaterThanOrEqual(4.5);
      // A scrim is translucent by design; the ordinary pair table cannot
      // measure it, which is why it has its own check.
      expect(parseColor(entry.palette.scrim)).toBeNull();
    },
  );

  it('fails a scrim that is too thin to hold text over an image', () => {
    expect(worstScrimRatio('rgba(17, 17, 17, 0.3)', THEMES.dark.color.text)).toBeLessThan(4.5);
    expect(worstScrimRatio('rgba(17, 17, 17, 0.8)', THEMES.dark.color.text)).toBeGreaterThan(4.5);
    // Unreadable input is a failure, never a pass.
    expect(worstScrimRatio('rgba(17, 17, 17, 1.5)', '#ffffff')).toBe(0);
    expect(worstScrimRatio('papayawhip', '#ffffff')).toBe(0);
  });

  const OWN_SIGNAL: Partial<Record<ThemeEntry['id'], string>> = {
    'bauhaus-black': '#ffd000',
    paper: '#d8a227',
    blueprint: '#ffd84d',
    terminal: '#ffb000',
    primary: '#ffd100',
  };

  it('puts the brand slots in front of each pack value', () => {
    for (const entry of THEME_LIST) {
      const c = entry.tokens.color;
      expect(c.accent).toContain(`var(${BRAND_SLOTS.accent},`);
      expect(c.danger).toContain(`var(${BRAND_SLOTS.danger},`);
      expect(c.signal).toContain(`var(${BRAND_SLOTS.signal},`);
      expect(c.signalText).toContain(`var(${BRAND_SLOTS.signalText},`);
      // Measured on the pack's own value, not on the wrapper.
      expect(entry.palette.danger).toMatch(/^#[0-9a-f]{6}$/);
      // Bauhaus yellow, except the Bauhaus packs, which name their own marker.
      expect(entry.palette.signal).toBe(OWN_SIGNAL[entry.id] ?? '#f5c400');
    }
  });

  it('gives every pack the same square, hard-edged structure', () => {
    for (const entry of THEME_LIST) {
      const t = entry.tokens;
      expect(t.radius, entry.id).toEqual({ sm: '0', md: '0', lg: '0', pill: '999px' });
      expect(t.border).toEqual({ hair: '1px', rule: '2px', heavy: '3px' });
      expect(t.elevation[0]).toBe('none');
      for (const level of [1, 2, 3] as const) {
        // `Xpx Ypx 0 0 colour`: offset only, zero blur, zero spread.
        expect(t.elevation[level], `${entry.id} elevation ${level}`).toMatch(
          /^(\d)px \1px 0 0 #(111111|000000)$/,
        );
      }
      expect(t.font.ui.startsWith('"Jost"')).toBe(true);
      expect(t.lineHeight.ui).toBe('1.2');
      expect(t.weight).toEqual({ regular: '400', medium: '500', bold: '700', heavy: '800' });
      expect(t.tracking).toEqual({ none: '0', label: '0.06em' });
      expect(t.a11y.focusRingWidth).toBe('3px');
      // No size token moved.
      expect(t.fontSize.base).toBe('14px');
      expect(t.density.rowH).toBe('56px');
      expect(t.a11y.touchTarget).toBe('24px');
    }
  });
});

// The user's accent used to be a free colour (`ACCENT_PRESETS`) laid over every
// pack unmeasured, and this file recorded the shortfalls as a KNOWN GAP. Since
// t32 V the accent is one of ten named accents shaded per theme, and
// `accents.test.ts` holds every theme × accent cell to the same pairs as the
// packs themselves.

// ─────────────────────────────────────────────────────────────────────────────

describe('themeCssVars — the sandboxed message iframe', () => {
  // The reader iframe is a separate opaque-origin document that cannot inherit
  // the parent's CSS custom properties, so `themeCssVars()` re-emits concrete
  // values under stable `--mw-color-*` names. That is what it is FOR; whether
  // anything calls it is a separate question, answered at the bottom of this
  // file. Until this suite it had no test at all.

  /** Pull `--mw-color-x: value` out of the emitted CSS. */
  function cssVar(css: string, key: string): string | undefined {
    return new RegExp(`--mw-color-${key}:\\s*([^;}]+)`).exec(css)?.[1]?.trim();
  }

  it.each(THEME_LIST.map((e) => [e.id, e] as const))(
    '%s emits the pack’s own palette, so mail matches the chrome',
    (id, entry) => {
      const css = themeCssVars(id);
      expect(cssVar(css, 'bg')).toBe(entry.palette.bg);
      expect(cssVar(css, 'surface')).toBe(entry.palette.surface);
      expect(cssVar(css, 'text')).toBe(entry.palette.text);
      expect(cssVar(css, 'text-dim')).toBe(entry.palette.textDim);
      expect(cssVar(css, 'border')).toBe(entry.palette.border);
      expect(cssVar(css, 'link')).toBe(entry.palette.link);
      expect(cssVar(css, 'selection')).toBe(entry.palette.selection);
      // The accent arrives unwrapped — an iframe cannot resolve the parent's
      // `var(--mw-accent, …)`, so a leaked `var()` would paint nothing.
      expect(cssVar(css, 'accent')).toBe(entry.palette.accent);
      expect(css).not.toContain('var(--mw-accent');
    },
  );

  it('carries the same contrast as the chrome, because it carries the same values', () => {
    // The iframe pairs (text/textDim/link on bg and surface) are already in
    // CONTRAST_PAIRS; asserting equality above is what makes the matrix apply
    // here. This test states the consequence explicitly so the link is not lost.
    for (const entry of THEME_LIST) {
      const css = themeCssVars(entry.id);
      for (const fg of ['text', 'textDim', 'link'] as const) {
        const emitted = cssVar(css, fg === 'textDim' ? 'text-dim' : fg) ?? '';
        const ratio = contrastRatio(emitted, cssVar(css, 'bg') ?? '');
        expect(ratio, `${entry.id}: ${fg} on iframe bg = ${ratio.toFixed(2)}`).toBeGreaterThanOrEqual(4.5);
      }
    }
  });

  it('honours an inline accent override and the density font sizes', () => {
    expect(cssVar(themeCssVars('light', { accent: '#be123c' }), 'accent')).toBe('#be123c');
    // Empty string means "theme default", not "no accent".
    expect(cssVar(themeCssVars('light', { accent: '' }), 'accent')).toBe(
      THEME_LIST.find((e) => e.id === 'light')?.palette.accent,
    );
    expect(themeCssVars('light', { density: 'compact' })).toContain('font-size: 13px');
    expect(themeCssVars('light', { density: 'cozy' })).toContain('font-size: 14px');
    expect(themeCssVars('light', { density: 'relaxed' })).toContain('font-size: 15px');
    // Default density is cozy.
    expect(themeCssVars('light')).toContain('font-size: 14px');
  });

  it('emits the body, link, selection and blockquote rules the reader depends on', () => {
    const css = themeCssVars('dark');
    expect(css).toContain('html, body {');
    expect(css).toContain('a { color: var(--mw-color-link); }');
    expect(css).toContain('::selection');
    expect(css).toContain('blockquote');
    expect(css).toContain('img { max-width: 100%');
    // Not wrapped in @media print unless asked.
    expect(css).not.toContain('@media print');
  });
});

describe('themeCssVars — the print stylesheet', () => {
  it('forces paper white and ink black regardless of the chrome theme', () => {
    for (const entry of THEME_LIST) {
      const css = themeCssVars(entry.id, { forPrint: true });
      expect(css.startsWith('@media print {')).toBe(true);
      expect(css.trimEnd().endsWith('}')).toBe(true);
      expect(/--mw-color-bg:\s*#ffffff/.test(css)).toBe(true);
      expect(/--mw-color-surface:\s*#ffffff/.test(css)).toBe(true);
      expect(/--mw-color-text:\s*#000000/.test(css)).toBe(true);
    }
  });

  it('KNOWN GAP: dark packs print theme-coloured links and dim text on white paper', () => {
    // `forPrint` overrides bg/surface/text but passes `link`, `textDim`,
    // `border` and `selection` through from the theme. For a DARK pack those
    // are tuned for a near-black page, so on white paper they land well under
    // 4.5:1 — a real WCAG §1.4.3 shortfall on the printed output.
    //
    // Not fixed here: `themeCssVars.ts` is outside this lane's locks (theme
    // sources belong to t19-e6). This test RECORDS the exact set so the gap is
    // visible and so a fix — most likely pinning link/textDim to print-safe
    // values alongside the bg/text overrides — turns it red and gets the
    // characterisation deleted rather than silently drifting.
    const shortfalls: string[] = [];
    for (const entry of THEME_LIST) {
      const css = themeCssVars(entry.id, { forPrint: true });
      for (const [key, token] of [
        ['link', 'link'],
        ['text-dim', 'textDim'],
      ] as const) {
        const value = new RegExp(`--mw-color-${key}:\\s*([^;}]+)`).exec(css)?.[1]?.trim() ?? '';
        expect(value, `${entry.id} ${key} passthrough`).toBe(entry.palette[token]);
        const ratio = contrastRatio(value, '#ffffff');
        if (ratio < 4.5) shortfalls.push(`${entry.id}/${key}=${ratio.toFixed(2)}`);
      }
    }
    // Every dark pack, both tokens; no light pack. That asymmetry is the
    // diagnosis — the override list is incomplete, it is not a palette bug.
    const affected = [...new Set(shortfalls.map((s) => s.split('/')[0]))].sort();
    expect(affected).toEqual(
      THEME_LIST.filter((e) => e.appearance === 'dark')
        .map((e) => e.id)
        .sort(),
    );
    expect(shortfalls).toHaveLength(affected.length * 2);
  });
});

// ─────────────────────────────────────────────────────────────────────────────

describe('message theme bridge', () => {
  it('uses the same text variable as the isolated theme producer', () => {
    const framed = bodyFrameDoc('full-sanitized', { html: '<p>x</p>' }, { themeVars: themeCssVars('dark') });
    expect(framed).toContain('--mw-color-text:');
    expect(framed).toContain('color:var(--mw-color-text,#1c1e21)');
    expect(framed).not.toContain('var(--mw-text,');
  });
  it('applies theme rules after the baseline body stylesheet', () => {
    const doc = bodyFrameDoc('full-sanitized', { html: '<p>x</p>' }, { themeVars: themeCssVars('dark') });
    expect(doc.indexOf('background: var(--mw-color-bg)')).toBeGreaterThan(doc.indexOf('background:transparent'));
    expect(doc.indexOf('color: var(--mw-color-text)')).toBeGreaterThan(doc.indexOf('color:var(--mw-color-text,#1c1e21)'));
  });
  it('supplies concrete theme colors only when requested', () => {
    expect(bodyFrameDoc('full-sanitized', { html: '<p>x</p>' }, { themeVars: themeCssVars('dark') })).toContain(THEMES.dark.color.text);
    expect(bodyFrameDoc('full-sanitized', { html: '<p>x</p>' })).not.toContain(THEMES.dark.color.text);
  });
});
