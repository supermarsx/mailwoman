// API-key / MCP-key UI styles (plan §3 e8). Token-native — design tokens unchanged.
//
// t32 restyle (plan §2.4): square corners, 2px control and panel borders in
// `borderStrong` with 1px less padding so no box grows (P1), focus is the
// outline ring, headings heavy, labels small tracked capitals.

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

export const heading = style({ fontSize: '1rem', fontWeight: vars.weight.heavy, margin: 0 });
export const subHeading = style({
  fontSize: '0.78rem',
  fontWeight: vars.weight.bold,
  color: vars.color.textDim,
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
});
export const prose = style({ fontSize: '0.9rem', lineHeight: 1.5, margin: 0 });

export const row = style({ display: 'flex', gap: vars.space[3], alignItems: 'center', flexWrap: 'wrap' });
export const field = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2] });

export const grid = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fill, minmax(220px, 1fr))',
  gap: vars.space[3],
});

export const input = style({
  padding: `${less1(vars.space[2])} ${less1(vars.space[3])}`,
  borderRadius: vars.radius.md,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: vars.color.bgSink,
  color: vars.color.text,
  font: 'inherit',
  fontSize: '0.9rem',
  selectors: {
    // A light page puts fields on the brighter surface instead of the sink.
    ':root[data-appearance="light"] &': { background: vars.color.surface },
    '&:focus-visible': focusOutline,
  },
});

export const check = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[2],
  fontSize: '0.9rem',
  cursor: 'pointer',
});

// The native checkbox itself: 24×24 CSS-px minimum target (WCAG 2.2 §2.5.8) and
// a visible focus ring (§2.4.11). Applied to every `<input type="checkbox">`.
export const checkbox = style({
  minWidth: vars.a11y.touchTarget,
  minHeight: vars.a11y.touchTarget,
  cursor: 'pointer',
  accentColor: vars.color.accent,
  selectors: { '&:focus-visible': focusOutline },
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
  transition: `box-shadow ${vars.a11y.motionDuration} ease`,
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
  transition: `background ${vars.a11y.motionDuration} ease, color ${vars.a11y.motionDuration} ease`,
  selectors: {
    '&:hover:not(:disabled)': { background: vars.color.text, color: vars.color.bg },
    '&:disabled': { opacity: 0.5, cursor: 'not-allowed', borderStyle: 'dashed' },
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
  padding: `${less1(vars.space[1])} ${less1(vars.space[3])}`,
  font: 'inherit',
  fontSize: '0.82rem',
  fontWeight: vars.weight.medium,
  minHeight: vars.a11y.touchTarget,
  minWidth: vars.a11y.touchTarget,
  transition: `box-shadow ${vars.a11y.motionDuration} ease`,
  selectors: {
    '&:hover:not(:disabled)': { boxShadow: vars.elevation[1] },
    '&:active:not(:disabled)': { boxShadow: 'none' },
    '&:focus-visible': focusOutline,
  },
});

export const token = style({
  fontFamily: vars.font.mono,
  fontSize: '0.9rem',
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  border: `${vars.border.hair} dashed ${vars.color.warning}`,
  color: vars.color.text,
  wordBreak: 'break-all',
  userSelect: 'all',
});

export const keyList = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2], listStyle: 'none', margin: 0, padding: 0 });

export const keyItem = style({
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'center',
  gap: vars.space[3],
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  background: vars.color.bg,
  flexWrap: 'wrap',
});

export const meta = style({ fontSize: '0.78rem', color: vars.color.textDim });

export const revoked = style({ opacity: 0.55 });

export const warn = style({
  fontSize: '0.88rem',
  lineHeight: 1.5,
  margin: 0,
  padding: vars.space[4],
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  borderInlineStart: `${vars.border.heavy} solid ${vars.color.danger}`,
});

export const error = style({ fontSize: '0.85rem', color: vars.color.danger, margin: 0 });

// Text status pill for a key (never colour-only — carries a text label so it
// survives loss of colour perception, WCAG 1.4.1).
const statusBase = {
  fontSize: '0.72rem',
  fontWeight: vars.weight.bold,
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
  whiteSpace: 'nowrap',
} as const;

export const statusActive = style({ ...statusBase, color: vars.color.success });
export const statusRevoked = style({ ...statusBase, color: vars.color.danger });

// The "copied to clipboard" confirmation (announced via an aria-live region and
// visible — not colour-only).
export const copiedNote = style({ fontSize: '0.8rem', color: vars.color.success, margin: 0 });
