// Settings panel styles (plan §3 e4). Token-native scoped classes.
//
// t32 restyle (plan §2.4): the panel is a square dialog with a 2px `borderStrong`
// rule and the hard `elevation[3]` shadow, its padding 1px less so the box does
// not grow (P1); group labels are small tracked capitals; a pressed option has
// an accent rule and a 4px inset accent bar. Options keep a 1px rule, so the
// theme gallery cards (built on `option`) stay the same size.

import { style, globalStyle } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

/** The outline ring (forced colours keep it; they drop a box-shadow ring). */
const focusOutline = {
  outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`,
  outlineOffset: '2px',
} as const;

export const panel = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[6],
  width: '100%',
  maxWidth: '520px',
  maxHeight: '90vh',
  overflowY: 'auto',
  padding: `calc(${vars.space[6]} - 1px)`,
  borderRadius: vars.radius.lg,
  background: vars.color.surface,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  color: vars.color.text,
  boxShadow: vars.elevation[3],
  fontFamily: vars.font.ui,
});

export const header = style({
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'center',
});

export const row = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
});

// Group label: small tracked capitals (the tracking token is 0 for ar/ja/zh).
export const label = style({
  fontSize: '0.8rem',
  fontWeight: vars.weight.bold,
  letterSpacing: vars.tracking.label,
  textTransform: 'uppercase',
  color: vars.color.textDim,
});

export const options = style({
  display: 'flex',
  flexWrap: 'wrap',
  gap: vars.space[2],
});

export const option = style({
  appearance: 'none',
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  background: 'transparent',
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[2]} ${vars.space[4]}`,
  font: 'inherit',
  fontSize: '0.85rem',
  selectors: {
    '&:hover:not([aria-pressed="true"])': { background: vars.color.bgAlt },
    // A 4px inset bar and an accent rule, not an accent fill: the theme cards
    // carry faded description lines (inline opacity) that a fill would sink
    // below 4.5:1. The bar is a shadow, so the box does not change.
    '&[aria-pressed="true"]': {
      background: vars.color.bgAlt,
      borderColor: vars.color.accent,
      boxShadow: `inset 4px 0 0 ${vars.color.accent}`,
    },
    '&[aria-pressed="true"]:dir(rtl)': { boxShadow: `inset -4px 0 0 ${vars.color.accent}` },
    '&:focus-visible': focusOutline,
  },
});

// Accent swatches are the one round control (the Bauhaus circle). The chosen one
// also carries a ring outside its border, so the choice is not colour alone.
export const swatch = style({
  width: '28px',
  height: '28px',
  borderRadius: vars.radius.pill,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  cursor: 'pointer',
  padding: 0,
  selectors: {
    '&[aria-pressed="true"]': {
      borderColor: vars.color.text,
      boxShadow: `0 0 0 2px ${vars.color.surface}, 0 0 0 4px ${vars.color.text}`,
    },
    '&:focus-visible': focusOutline,
  },
});

export const select = style({
  padding: `calc(${vars.space[2]} - 1px) calc(${vars.space[3]} - 1px)`,
  borderRadius: vars.radius.md,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: vars.color.bgSink,
  color: vars.color.text,
  font: 'inherit',
  selectors: {
    // A light page puts fields on the brighter surface instead of the sink.
    ':root[data-appearance="light"] &': { background: vars.color.surface },
    '&:focus-visible': focusOutline,
  },
});

// ── former inline styles (t32-h12) ──────────────────────────────────────────
// Static `style={{…}}` objects compile into the template as a literal `style=`
// attribute, which the shell CSP (`style-src 'self'`) drops.
/** Small dim explanatory line under a setting. */
export const hint = style({ margin: 0, fontSize: '0.78rem', color: vars.color.textDim });

// Compact choices stay intrinsic; galleries and text editors retain their width.
globalStyle(`${row} > select, ${row} > button`, { alignSelf: 'flex-start', maxInlineSize: '100%' });
