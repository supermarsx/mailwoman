// Zero-access UI styles (plan §3 e8). Token-native scoped classes — design tokens
// unchanged (regression gate), light+dark via the frozen `vars` contract.
//
// t32 restyle (plan §2.4): square corners, 2px control and panel borders in
// `borderStrong` with 1px less padding so no box grows (P1), focus is the
// outline ring, headings heavy, labels and status badges small tracked capitals.

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

/** The outline ring (forced colours keep it; they drop a box-shadow ring). */
const focusOutline = {
  outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`,
  outlineOffset: '2px',
} as const;

/** `base` less the 1px a 2px rule adds over the old 1px border (P1). */
const less1 = (base: string): string => `calc(${base} - 1px)`;

export const panel = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[5],
  fontFamily: vars.font.ui,
  color: vars.color.text,
});

export const section = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: less1(vars.space[5]),
  borderRadius: vars.radius.lg,
  background: vars.color.surface,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
});

export const heading = style({
  fontSize: '1rem',
  fontWeight: vars.weight.heavy,
  margin: 0,
});

export const subHeading = style({
  fontSize: '0.8rem',
  fontWeight: vars.weight.bold,
  color: vars.color.textDim,
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
});

export const prose = style({
  fontSize: '0.9rem',
  lineHeight: 1.5,
  color: vars.color.text,
  margin: 0,
});

export const caveat = style({
  fontSize: '0.9rem',
  lineHeight: 1.5,
  margin: 0,
  padding: vars.space[4],
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  borderInlineStart: `${vars.border.heavy} solid ${vars.color.warning}`,
  color: vars.color.text,
});

export const list = style({
  margin: 0,
  paddingInlineStart: vars.space[5],
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  fontSize: '0.88rem',
  lineHeight: 1.45,
  color: vars.color.textDim,
});

export const row = style({
  display: 'flex',
  gap: vars.space[3],
  alignItems: 'center',
  flexWrap: 'wrap',
});

export const field = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
});

export const input = style({
  padding: `${less1(vars.space[2])} ${less1(vars.space[3])}`,
  borderRadius: vars.radius.md,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: vars.color.bgSink,
  color: vars.color.text,
  font: 'inherit',
  fontSize: '0.9rem',
  minHeight: vars.a11y.touchTarget,
  selectors: {
    // A light page puts fields on the brighter surface instead of the sink.
    ':root[data-appearance="light"] &': { background: vars.color.surface },
    '&:focus-visible': focusOutline,
  },
});

export const button = style({
  appearance: 'none',
  border: `${vars.border.rule} solid ${vars.color.accent}`,
  background: vars.color.accent,
  color: vars.color.accentText,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${less1(vars.space[2])} ${less1(vars.space[4])}`,
  font: 'inherit',
  fontSize: '0.9rem',
  fontWeight: vars.weight.medium,
  minHeight: vars.a11y.touchTarget,
  selectors: {
    '&:hover:not(:disabled)': { boxShadow: vars.elevation[1] },
    '&:active:not(:disabled)': { boxShadow: 'none' },
    // Not colour alone: a disabled button also has a dashed border.
    '&:disabled': { opacity: 0.5, cursor: 'not-allowed', borderStyle: 'dashed' },
    '&:focus-visible': focusOutline,
  },
});

// Ghost buttons invert on hover instead of casting a shadow.
export const buttonGhost = style({
  appearance: 'none',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: 'transparent',
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${less1(vars.space[2])} ${less1(vars.space[4])}`,
  font: 'inherit',
  fontSize: '0.9rem',
  minHeight: vars.a11y.touchTarget,
  selectors: {
    '&:hover:not(:disabled)': { background: vars.color.text, color: vars.color.bg },
    '&:focus-visible': focusOutline,
  },
});

export const danger = style({
  appearance: 'none',
  border: `${vars.border.rule} solid ${vars.color.danger}`,
  background: vars.color.danger,
  color: vars.color.accentText,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${less1(vars.space[2])} ${less1(vars.space[4])}`,
  font: 'inherit',
  fontSize: '0.9rem',
  fontWeight: vars.weight.medium,
  minHeight: vars.a11y.touchTarget,
  selectors: {
    '&:hover:not(:disabled)': { boxShadow: vars.elevation[1] },
    '&:active:not(:disabled)': { boxShadow: 'none' },
    '&:focus-visible': focusOutline,
  },
});

export const phrase = style({
  fontFamily: vars.font.mono,
  fontSize: '0.95rem',
  lineHeight: 1.7,
  padding: vars.space[4],
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  border: `${vars.border.hair} dashed ${vars.color.border}`,
  wordSpacing: '0.3em',
  userSelect: 'all',
});

export const sasGrid = style({
  display: 'flex',
  flexWrap: 'wrap',
  gap: vars.space[2],
});

export const sasWord = style({
  fontFamily: vars.font.mono,
  fontSize: '0.95rem',
  fontWeight: 600,
  padding: `${vars.space[1]} ${vars.space[3]}`,
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  border: `${vars.border.hair} solid ${vars.color.border}`,
});

// A QR code needs dark modules on a light field to scan, whatever the theme, so
// the frame stays white (the one colour literal here, on purpose).
export const qrFrame = style({
  width: '220px',
  height: '220px',
  padding: vars.space[3],
  background: '#ffffff',
  borderRadius: vars.radius.md,
  border: `${vars.border.hair} solid ${vars.color.border}`,
});

export const badge = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  fontSize: '0.75rem',
  fontWeight: vars.weight.bold,
  letterSpacing: vars.tracking.label,
  textTransform: 'uppercase',
  padding: `${vars.space[1]} ${vars.space[3]}`,
  borderRadius: vars.radius.md,
});

export const badgeOn = style([
  badge,
  {
    background: vars.color.success,
    color: vars.color.accentText,
  },
]);

export const badgeOff = style([
  badge,
  {
    background: vars.color.bgAlt,
    color: vars.color.textDim,
    border: `${vars.border.hair} solid ${vars.color.border}`,
  },
]);

export const error = style({
  fontSize: '0.85rem',
  color: vars.color.danger,
  margin: 0,
});
