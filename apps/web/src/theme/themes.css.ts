// Built-in theme registration (plan §3 e4, §2.3).
//
// Each theme binds the token VALUES from `tokens.ts` to the FROZEN contract
// (`contract.css.ts`) under a `:root[data-theme="…"]` selector via
// `createGlobalTheme`, so runtime switching is a single attribute flip. We also
// emit a legacy custom-property bridge (`--bg`, `--text`, …) from the same
// palette so the existing `app.css` components theme WITHOUT a rewrite — token
// migration is opportunistic (plan §1.6), not a gate, and the existing
// selectors / DOM the Playwright specs rely on stay intact.
//
// Import order matters: the per-theme rules and the gating overrides below are
// emitted in source order, so the gating rules (equal specificity,
// `:root[data-theme]`) come last and win on ties.

import { createGlobalTheme, globalStyle } from '@vanilla-extract/css';
import { vars, type ThemeName } from './contract.css.ts';
import { THEMES, type ThemeTokens } from './tokens.ts';

/** Legacy `app.css` custom properties, mapped from a theme palette. */
function legacyVars(c: ThemeTokens['color']): Record<string, string> {
  return {
    '--bg': c.bg,
    '--bg-alt': c.bgAlt,
    '--bg-sink': c.bgSink,
    '--surface': c.surface,
    '--border': c.border,
    '--text': c.text,
    '--text-dim': c.textDim,
    '--accent': c.accent,
    '--accent-text': c.accentText,
    '--danger': c.danger,
    '--success': c.success,
    '--warning': c.warning,
    '--link': c.link,
    '--selection': c.selection,
    '--border-strong': c.borderStrong,
    '--signal': c.signal,
    '--signal-text': c.signalText,
    '--focus': c.focus,
    '--scrim': c.scrim,
    '--scrim-text': c.scrimText,
  };
}

// Default (pre-JS / no data-theme): bind the contract and the legacy bridge to
// Dark, the default pack (t32), so the first frame is dark and components using
// `vars.*` have values before the theme slice sets the attribute. A user who
// chose a light pack sees this for the first frame only; no inline script may
// run before the bundle under the CSP.
createGlobalTheme(':root', vars, THEMES.dark);
globalStyle(':root', { vars: legacyVars(THEMES.dark.color) });

// Structural tokens for plain-CSS files (`app.css` and the other `.css` files
// cannot read the hashed vanilla-extract names). Each one points at the
// contract variable, so it follows the active pack and the density overrides.
globalStyle(':root', {
  vars: {
    '--radius-sm': vars.radius.sm,
    '--radius-md': vars.radius.md,
    '--radius-lg': vars.radius.lg,
    '--radius-pill': vars.radius.pill,
    '--border-hair': vars.border.hair,
    '--border-rule': vars.border.rule,
    '--border-heavy': vars.border.heavy,
    '--elevation-1': vars.elevation[1],
    '--elevation-2': vars.elevation[2],
    '--elevation-3': vars.elevation[3],
    '--weight-regular': vars.weight.regular,
    '--weight-medium': vars.weight.medium,
    '--weight-bold': vars.weight.bold,
    '--weight-heavy': vars.weight.heavy,
    '--tracking-label': vars.tracking.label,
    '--line-height-ui': vars.lineHeight.ui,
    '--focus-ring': vars.a11y.focusRing,
    '--focus-ring-width': vars.a11y.focusRingWidth,
    '--motion': vars.a11y.motionDuration,
  },
});

// Per-theme: contract vars + the legacy bridge under the same selector.
(Object.keys(THEMES) as ThemeName[]).forEach((name) => {
  const selector = `:root[data-theme="${name}"]`;
  createGlobalTheme(selector, vars, THEMES[name]);
  globalStyle(selector, { vars: legacyVars(THEMES[name].color) });
});

// ── Density (plan §2.3 `data-density`) ───────────────────────────────────────
// Overrides row height + base font size; cozy is the default.
globalStyle(':root[data-density="compact"]', {
  vars: { [vars.density.rowH]: '40px', [vars.fontSize.base]: '13px' },
});
globalStyle(':root[data-density="cozy"]', {
  vars: { [vars.density.rowH]: '56px', [vars.fontSize.base]: '14px' },
});
globalStyle(':root[data-density="relaxed"]', {
  vars: { [vars.density.rowH]: '68px', [vars.fontSize.base]: '15px' },
});

// ── Token application over existing chrome (opportunistic, additive) ──────────
// These style existing selectors without changing the DOM. They are emitted
// BEFORE `app.css` in the built stylesheet, so an `app.css` rule of equal
// specificity wins: `app.css` must not restate a property set here.
globalStyle('body', {
  // `--mw-ui-font` is an optional inline override set by the font setting;
  // falls back to the theme's ui font stack.
  fontFamily: `var(--mw-ui-font, ${vars.font.ui})`,
  fontSize: vars.fontSize.base,
  lineHeight: vars.lineHeight.ui,
  background: vars.color.bg,
  color: vars.color.text,
  backgroundImage: vars.texture.paper,
  backgroundAttachment: 'fixed',
});
// Headings are heavy; their sizes are left to the components.
globalStyle('h1, h2, h3', { fontWeight: vars.weight.heavy });
// One focus indicator for everything: an outline, which forced-colours mode
// keeps (a box-shadow ring is dropped there). Components that draw their own
// ring with `vars.a11y.focusRing` set `outline: none` on themselves.
globalStyle(':focus-visible', {
  outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`,
  outlineOffset: '2px',
});
globalStyle('.sidebar', { backgroundImage: vars.texture.grain });
globalStyle('.list__row', { minHeight: vars.density.rowH });
globalStyle('::selection', { background: vars.color.selection });

// Letter-spacing breaks Arabic joining and is not used in Japanese or Chinese
// text, so the label tracking is 0 for those languages. `:root[data-theme]`
// keeps the specificity above the per-pack assignment.
globalStyle(':root[data-theme]:lang(ar), :root[data-theme]:lang(ja), :root[data-theme]:lang(zh)', {
  vars: { [vars.tracking.label]: '0' },
});
globalStyle(':root:lang(ar), :root:lang(ja), :root:lang(zh)', {
  vars: { [vars.tracking.label]: '0' },
});

// ── Texture gating (plan §3 e4) ──────────────────────────────────────────────
// Grove is the only theme with non-`none` textures; force them off under
// reduced-transparency, high-contrast (forced-colors), and data-saver. Uses
// `:root[data-theme]` (same specificity as the theme selectors) placed LAST so
// it overrides on ties — a lower-specificity `:root` would lose to the grove
// theme's own `:root[data-theme="grove-…"]` texture assignment.
const NONE = { [vars.texture.grain]: 'none', [vars.texture.paper]: 'none' } as const;
globalStyle(':root[data-theme]', {
  '@media': {
    '(prefers-reduced-transparency: reduce)': { vars: NONE },
    '(prefers-reduced-data: reduce)': { vars: NONE },
    '(forced-colors: active)': { vars: NONE },
  },
});

// ── Reduced motion (plan §6 e0; SPEC §24 — real gap: was ZERO handling) ───────
// Collapse the shared motion-duration tokens to ~0 so every transition/animation
// authored against `vars.a11y.motionDuration(Slow)` becomes instant. Driven by
// BOTH the media query AND the `<html data-reduced-motion>` flag the shell sets
// (rootAttributes.ts), so a user toggle and the OS setting both take effect.
const NO_MOTION = {
  [vars.a11y.motionDuration]: '0.01ms',
  [vars.a11y.motionDurationSlow]: '0.01ms',
} as const;
globalStyle(':root', {
  '@media': { '(prefers-reduced-motion: reduce)': { vars: NO_MOTION } },
});
globalStyle(':root[data-reduced-motion]', { vars: NO_MOTION });
// Belt-and-braces: also neutralise raw CSS animations/transitions under reduced
// motion for elements that don't yet use the tokens (existing chrome).
globalStyle(':root[data-reduced-motion] *', {
  animationDuration: '0.01ms !important',
  animationIterationCount: '1 !important',
  transitionDuration: '0.01ms !important',
});

// ── High contrast (plan §6 e0; SPEC §24) ──────────────────────────────────────
// Thicken the focus ring when the user asks for more contrast
// (`prefers-contrast: more`) or the platform forces colors. Uses the same low
// specificity as the token defaults, placed last so it wins on ties.
const HC_FOCUS = {
  [vars.a11y.focusRingWidth]: '4px',
  [vars.a11y.focusRing]: `0 0 0 2px ${vars.color.bg}, 0 0 0 6px ${vars.color.focus}`,
} as const;
globalStyle(':root', {
  '@media': {
    '(prefers-contrast: more)': { vars: HC_FOCUS },
    '(forced-colors: active)': { vars: HC_FOCUS },
  },
});
