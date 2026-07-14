// API-key / MCP-key UI styles (plan §3 e8). Token-native — design tokens unchanged.

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

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
  padding: vars.space[5],
  borderRadius: vars.radius.lg,
  background: vars.color.surface,
  border: `1px solid ${vars.color.border}`,
});

export const heading = style({ fontSize: '1rem', fontWeight: 600, margin: 0 });
export const subHeading = style({
  fontSize: '0.78rem',
  fontWeight: 600,
  color: vars.color.textDim,
  textTransform: 'uppercase',
  letterSpacing: '0.04em',
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
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderRadius: vars.radius.md,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  font: 'inherit',
  fontSize: '0.9rem',
  transition: `box-shadow ${vars.a11y.motionDuration} ease`,
  selectors: { '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing } },
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
  transition: `box-shadow ${vars.a11y.motionDuration} ease`,
  selectors: { '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing } },
});

export const button = style({
  appearance: 'none',
  border: `1px solid ${vars.color.accent}`,
  background: vars.color.accent,
  color: vars.color.accentText,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[2]} ${vars.space[4]}`,
  font: 'inherit',
  fontSize: '0.9rem',
  fontWeight: 600,
  minHeight: vars.a11y.touchTarget,
  transition: `box-shadow ${vars.a11y.motionDuration} ease`,
  selectors: {
    '&:disabled': { opacity: 0.5, cursor: 'not-allowed' },
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  },
});

export const ghost = style({
  appearance: 'none',
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[2]} ${vars.space[4]}`,
  font: 'inherit',
  fontSize: '0.9rem',
  minHeight: vars.a11y.touchTarget,
  transition: `box-shadow ${vars.a11y.motionDuration} ease`,
  selectors: { '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing } },
});

export const danger = style({
  appearance: 'none',
  border: `1px solid ${vars.color.danger}`,
  background: 'transparent',
  color: vars.color.danger,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[1]} ${vars.space[3]}`,
  font: 'inherit',
  fontSize: '0.82rem',
  fontWeight: 600,
  minHeight: vars.a11y.touchTarget,
  minWidth: vars.a11y.touchTarget,
  transition: `box-shadow ${vars.a11y.motionDuration} ease`,
  selectors: { '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing } },
});

export const token = style({
  fontFamily: vars.font.mono,
  fontSize: '0.9rem',
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  border: `1px dashed ${vars.color.warning}`,
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
  border: `1px solid ${vars.color.border}`,
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
  borderLeft: `3px solid ${vars.color.danger}`,
});

export const error = style({ fontSize: '0.85rem', color: vars.color.danger, margin: 0 });

// Text status pill for a key (never colour-only — carries a text label so it
// survives loss of colour perception, WCAG 1.4.1).
const statusBase = {
  fontSize: '0.72rem',
  fontWeight: 600,
  textTransform: 'uppercase',
  letterSpacing: '0.04em',
  whiteSpace: 'nowrap',
} as const;

export const statusActive = style({ ...statusBase, color: vars.color.success });
export const statusRevoked = style({ ...statusBase, color: vars.color.danger });

// The "copied to clipboard" confirmation (announced via an aria-live region and
// visible — not colour-only).
export const copiedNote = style({ fontSize: '0.8rem', color: vars.color.success, margin: 0 });
