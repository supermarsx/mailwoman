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
  parseColor,
  relativeLuminance,
  type ContrastLevel,
  type ContrastResult,
} from './contrast.ts';
import { ACCENT_PRESETS, THEMES } from './tokens.ts';
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
    // light.text #1c1e21 on light.bg #ffffff, worked through WCAG 2.2 by hand:
    //   channels 0x1c=28, 0x1e=30, 0x21=33 → s = 0.109804, 0.117647, 0.129412
    //   each > 0.04045 → ((s + 0.055) / 1.055) ^ 2.4
    //     R = (0.156212)^2.4 = 0.0116141
    //     G = (0.163647)^2.4 = 0.0129827
    //     B = (0.174807)^2.4 = 0.0152087
    //   L = 0.2126R + 0.7152G + 0.0722B = 0.0128523
    //   ratio = (1.0 + 0.05) / (0.0128523 + 0.05) = 16.71
    // White is L = 1 exactly, so the denominator is the only interesting half.
    const light = THEMES.light.color;
    expect(light.text).toBe('#1c1e21');
    expect(light.bg).toBe('#ffffff');
    expect(relativeLuminance([255, 255, 255])).toBe(1);
    expect(relativeLuminance([28, 30, 33])).toBeCloseTo(0.0128523, 6);
    expect(contrastRatio(light.text, light.bg)).toBeCloseTo(16.71, 2);
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
  it('measures all 13 built-in themes', () => {
    // Guards against a pack being added to the token table but never reaching
    // the registry — the matrix below iterates the registry, so a theme missing
    // from it would be silently unmeasured rather than failing.
    expect(THEME_LIST).toHaveLength(13);
    expect(THEME_LIST.filter((e) => e.appearance === 'light')).toHaveLength(6);
    expect(THEME_LIST.filter((e) => e.appearance === 'dark')).toHaveLength(7);
  });

  it('covers every surface × foreground combination it claims to', () => {
    // 4 surfaces × (6 text colours + accent + border) + 3 filled-control labels
    // + text-on-selection.
    expect(CONTRAST_PAIRS).toHaveLength(4 * 8 + 3 + 1);
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

describe('accent overrides', () => {
  // `ACCENT_PRESETS` is offered in Settings and lands as an inline `--mw-accent`
  // that REPLACES the theme's own accent — so a preset is painted on every
  // theme's surfaces, in combinations no single theme palette describes. The
  // per-theme matrix above measures the pack's OWN accent and cannot see this.
  const presets = ACCENT_PRESETS.filter((p) => p.value !== '');

  it('offers only measurable opaque colours', () => {
    expect(presets.length).toBeGreaterThan(0);
    for (const p of presets) expect(parseColor(p.value), p.label).not.toBeNull();
  });

  /** Every (theme, preset, surface) triple where the preset misses 3:1. */
  function surfaceShortfalls(): string[] {
    const out: string[] = [];
    for (const entry of THEME_LIST) {
      for (const preset of presets) {
        for (const bg of ['bg', 'bgAlt', 'bgSink', 'surface'] as const) {
          const ratio = contrastRatio(preset.value, entry.palette[bg]);
          if (ratio < 3) out.push(`${entry.id}/${preset.label}/${bg}=${ratio.toFixed(2)}`);
        }
      }
    }
    return out;
  }

  /** Every (theme, preset) pair where the filled-control label misses 4.5:1. */
  function labelShortfalls(): string[] {
    const out: string[] = [];
    for (const entry of THEME_LIST) {
      for (const preset of presets) {
        const ratio = contrastRatio(entry.palette.accentText, preset.value);
        if (ratio < 4.5) out.push(`${entry.id}/${preset.label}=${ratio.toFixed(2)}`);
      }
    }
    return out;
  }

  // ── KNOWN GAP ───────────────────────────────────────────────────────────
  // The accent override is not contrast-checked against anything. Every pack's
  // OWN accent clears its tiers (the matrix above proves it), but a preset
  // replaces that accent wholesale and is then painted on surfaces its own
  // palette never described, and labelled with an `accentText` chosen for a
  // different colour. Both halves below are the SAME defect.
  //
  // Not fixed here: `ACCENT_PRESETS` lives in `tokens.ts`, outside this lane's
  // locks. The fix is per-appearance preset values, or a contrast-aware clamp
  // where the override is applied — a token/UI change, not a test change.
  // Recording the exact failing SETS makes the gap reviewable, and turns this
  // red if a later palette edit widens it.

  it('KNOWN GAP: every preset misses 3:1 on at least one theme surface', () => {
    const shortfalls = surfaceShortfalls();
    const offenders = [...new Set(shortfalls.map((s) => s.split('/')[1]))].sort();
    expect(offenders).toEqual(presets.map((p) => p.label).sort());
    expect(shortfalls).toHaveLength(40);
    // Out of 13 themes × 6 presets × 4 surfaces = 312 combinations, so ~13%.
    expect(shortfalls.length / (THEME_LIST.length * presets.length * 4)).toBeLessThan(0.15);
  });

  it('KNOWN GAP: the filled-control label misses 4.5:1 under most presets', () => {
    const shortfalls = labelShortfalls();
    expect(shortfalls).toHaveLength(40);
    // Dark packs put dark ink on a bright accent and light packs the reverse,
    // so a preset that suits one appearance fails the other. Both appearances
    // are affected — this is not a dark-pack-only problem.
    const affected = new Set(
      shortfalls.map((s) => THEME_LIST.find((e) => e.id === s.split('/')[0])?.appearance),
    );
    expect(affected).toEqual(new Set(['light', 'dark']));
  });

  it('holds both recorded shortfalls to a near miss, never an invisible control', () => {
    // The gap is a tuning shortfall, not a disappearance: even the worst case
    // stays clearly visible. A combination landing near 1:1 would be an outright
    // defect rather than a WCAG miss, so THAT boundary is enforced even while
    // the 3:1 / 4.5:1 floors are only recorded.
    for (const s of [...surfaceShortfalls(), ...labelShortfalls()]) {
      const ratio = Number(s.split('=')[1]);
      expect(ratio, s).toBeGreaterThan(2.2);
    }
  });
});

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

describe('themeCssVars is not wired up — KNOWN GAP', () => {
  // Found while checking which file owns the print-contrast defect above. The
  // answer turned out to be "none of them, yet".
  //
  // `bodyFrameDoc(mode, content, { themeVars })` in `viewers/sandbox.ts` is the
  // designed injection point, and `styles/print.css.ts:7` states as fact that
  // the message body's print theming comes from
  // `themeCssVars(theme, { forPrint: true })`, "injected into the srcdoc by e7".
  // It is not: no call site anywhere passes `themeVars`, and `themeCssVars` has
  // no runtime caller at all. Its 0% line coverage before this suite was a
  // SYMPTOM, not an oversight.
  //
  // Consequence: the message body is never themed. `BODY_STYLE` falls back to
  // `var(--mw-text, #1c1e21)` over a transparent (⇒ white) iframe, so a dark
  // theme shows a dark reader pane wrapped around a white message block. The
  // print shortfalls recorded above are therefore LATENT — real properties of
  // the function, not currently rendered by anything.
  //
  // Nothing here is fixed: `themeCssVars.ts`, `sandbox.ts`, `Reader.tsx` and
  // `print.css.ts` are all outside this lane's locks, and wiring it is a
  // product decision (mail authored for white backgrounds is a defensible
  // default) rather than a mechanical repair.

  it('emits a variable name that the consumer does not read', () => {
    // The sharper half of the defect, and the reason it would survive being
    // wired: producer and consumer disagree on the name. `themeCssVars` emits
    // `--mw-color-text`; `BODY_STYLE` reads `var(--mw-text, …)`. Passing the
    // output through today would still leave the fallback in force.
    const css = themeCssVars('dark');
    expect(css).toContain('--mw-color-text:');
    expect(css).not.toContain('--mw-text:');

    const framed = bodyFrameDoc('full-sanitized', { html: '<p>x</p>' }, { themeVars: css });
    expect(framed).toContain('var(--mw-text,#1c1e21)');
    // Both names present, neither resolving to the other: the emitted block
    // defines `--mw-color-text`, the stylesheet reads `--mw-text`.
    expect(framed).toContain('--mw-color-text:');
  });

  it('renders identically with and without the theme block, for the text colour', () => {
    // The behavioural consequence of the name mismatch, stated as behaviour
    // rather than as string matching.
    const withTheme = bodyFrameDoc('full-sanitized', { html: '<p>x</p>' }, {
      themeVars: themeCssVars('dark'),
    });
    const without = bodyFrameDoc('full-sanitized', { html: '<p>x</p>' });
    for (const doc of [withTheme, without]) {
      expect(doc).toContain('color:var(--mw-text,#1c1e21)');
    }
    // The dark palette's text colour never reaches the document as the value
    // the body actually resolves.
    expect(without).not.toContain(THEMES.dark.color.text);
  });
});
