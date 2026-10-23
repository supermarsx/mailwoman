// Password-change UI styles (plan §3 e7). Token-native — design tokens unchanged.
//
// t32 restyle (plan §2.4): square corners, 2px control and panel borders in
// `borderStrong` with 1px less padding so no box grows (P1), focus is the
// outline ring, headings heavy, field labels small tracked capitals.

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
  gap: vars.space[4],
  fontFamily: vars.font.ui,
  color: vars.color.text,
  maxWidth: '32rem',
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

export const heading = style({ fontSize: '1rem', fontWeight: vars.weight.heavy, margin: 0 });
export const prose = style({ fontSize: '0.9rem', lineHeight: 1.5, margin: 0 });
export const meta = style({ fontSize: '0.8rem', color: vars.color.textDim, margin: 0 });

export const field = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2] });
// Field label: small tracked capitals (the tracking token is 0 for ar/ja/zh).
export const label = style({
  fontSize: '0.8rem',
  fontWeight: vars.weight.bold,
  letterSpacing: vars.tracking.label,
  textTransform: 'uppercase',
  color: vars.color.textDim,
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
  transitionProperty: 'border-color',
  transitionDuration: vars.a11y.motionDuration,
  selectors: {
    // A light page puts fields on the brighter surface instead of the sink.
    ':root[data-appearance="light"] &': { background: vars.color.surface },
    '&:focus-visible': focusOutline,
  },
});

export const policyList = style({
  margin: 0,
  paddingInlineStart: vars.space[5],
  fontSize: '0.82rem',
  color: vars.color.textDim,
  display: 'flex',
  flexDirection: 'column',
  gap: '2px',
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
  transitionProperty: 'box-shadow, background, color',
  transitionDuration: vars.a11y.motionDuration,
  selectors: {
    '&:hover:not(:disabled)': { boxShadow: vars.elevation[1] },
    '&:active:not(:disabled)': { boxShadow: 'none' },
    // Not colour alone: a disabled button also has a dashed border.
    '&:disabled': { opacity: 0.5, cursor: 'not-allowed', borderStyle: 'dashed' },
    '&:focus-visible': focusOutline,
  },
});

// Ghost buttons invert on hover instead of casting a shadow.
export const ghost = style({
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
  transitionProperty: 'box-shadow, background, color',
  transitionDuration: vars.a11y.motionDuration,
  selectors: {
    '&:hover:not(:disabled)': { background: vars.color.text, color: vars.color.bg },
    '&:focus-visible': focusOutline,
  },
});

export const check = style({
  display: 'flex',
  alignItems: 'flex-start',
  gap: vars.space[2],
  fontSize: '0.88rem',
  cursor: 'pointer',
  lineHeight: 1.4,
});

// The checkbox control itself: meet the 24×24 minimum target (WCAG 2.2 §2.5.8)
// and carry the shared focus ring.
export const checkbox = style({
  width: vars.a11y.touchTarget,
  height: vars.a11y.touchTarget,
  minWidth: vars.a11y.touchTarget,
  minHeight: vars.a11y.touchTarget,
  margin: 0,
  cursor: 'pointer',
  accentColor: vars.color.accent,
  selectors: {
    '&:focus-visible': focusOutline,
  },
});

export const phrase = style({
  fontFamily: vars.font.mono,
  fontSize: '0.95rem',
  lineHeight: 1.6,
  padding: vars.space[4],
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  border: `${vars.border.hair} dashed ${vars.color.warning}`,
  color: vars.color.text,
  wordBreak: 'break-word',
  userSelect: 'all',
});

export const warn = style({
  fontSize: '0.88rem',
  lineHeight: 1.5,
  margin: 0,
  padding: vars.space[4],
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  borderInlineStart: `${vars.border.heavy} solid ${vars.color.warning}`,
});

export const banner = style({
  fontSize: '0.88rem',
  lineHeight: 1.5,
  margin: 0,
  padding: vars.space[4],
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  borderInlineStart: `${vars.border.heavy} solid ${vars.color.danger}`,
  fontWeight: 600,
});

export const success = style({ fontSize: '0.9rem', color: vars.color.success, margin: 0, fontWeight: 600 });
export const error = style({ fontSize: '0.85rem', color: vars.color.danger, margin: 0 });

/** One line inside the error region (the region itself carries colour and size). */
export const errorLine = style({ margin: 0, lineHeight: 1.5 });

// -- Forced change screen (t27) ------------------------------------------------
export const forced = style({
  minHeight: '100vh',
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'center',
  justifyContent: 'center',
  gap: vars.space[4],
  padding: vars.space[5],
  fontFamily: vars.font.ui,
  color: vars.color.text,
  background: vars.color.bg,
});

export const forcedBody = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  width: '100%',
  maxWidth: '32rem',
});

export const forcedTitle = style({ fontSize: '1.25rem', fontWeight: vars.weight.heavy, margin: 0 });
