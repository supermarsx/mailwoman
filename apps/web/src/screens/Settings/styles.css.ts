// Settings account-surface styles (t16 e15). Token-native — no new design tokens.
// Layout uses logical properties (inline-start/end, margin-inline) so the surface
// mirrors correctly under `dir="rtl"` (W20) with no per-direction overrides.

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const stack = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
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
export const subheading = style({ fontSize: '0.9rem', fontWeight: 600, margin: 0 });
export const prose = style({ fontSize: '0.9rem', lineHeight: 1.5, margin: 0 });
export const meta = style({ fontSize: '0.8rem', color: vars.color.textDim, margin: 0 });

export const field = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2] });
export const label = style({ fontSize: '0.85rem', fontWeight: 600 });

export const input = style({
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderRadius: vars.radius.md,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  font: 'inherit',
  fontSize: '0.9rem',
  minHeight: vars.a11y.touchTarget,
  transitionProperty: 'box-shadow, border-color',
  transitionDuration: vars.a11y.motionDuration,
  selectors: {
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  },
});

export const textarea = style([input, { minHeight: '5rem', resize: 'vertical', lineHeight: 1.5 }]);

export const select = style([input, { cursor: 'pointer' }]);

export const row = style({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  gap: vars.space[3],
  flexWrap: 'wrap',
});

export const grow = style({ flex: 1, minWidth: '12rem' });

export const actions = style({ display: 'flex', gap: vars.space[2], flexWrap: 'wrap' });

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
  transitionProperty: 'box-shadow, background, border-color',
  transitionDuration: vars.a11y.motionDuration,
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
  transitionProperty: 'box-shadow, background, border-color',
  transitionDuration: vars.a11y.motionDuration,
  selectors: {
    '&:disabled': { opacity: 0.5, cursor: 'not-allowed' },
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  },
});

export const danger = style([
  ghost,
  {
    borderColor: vars.color.danger,
    color: vars.color.danger,
  },
]);

// A pressable option in a segmented control (presets / direction).
export const option = style({
  appearance: 'none',
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[2]} ${vars.space[3]}`,
  font: 'inherit',
  fontSize: '0.85rem',
  minHeight: vars.a11y.touchTarget,
  selectors: {
    '&[aria-pressed="true"]': {
      borderColor: vars.color.accent,
      background: vars.color.bgAlt,
      fontWeight: 600,
    },
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  },
});

export const options = style({ display: 'flex', gap: vars.space[2], flexWrap: 'wrap' });

export const list = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  margin: 0,
  padding: 0,
  listStyle: 'none',
});

export const item = style({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  gap: vars.space[3],
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  flexWrap: 'wrap',
});

export const itemMain = style({ display: 'flex', flexDirection: 'column', gap: '2px', minWidth: '10rem' });
export const itemName = style({ fontSize: '0.9rem', fontWeight: 600 });

export const badge = style({
  fontSize: '0.72rem',
  fontWeight: 600,
  padding: `1px ${vars.space[2]}`,
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  color: vars.color.textDim,
});

export const check = style({
  display: 'flex',
  alignItems: 'flex-start',
  gap: vars.space[2],
  fontSize: '0.88rem',
  cursor: 'pointer',
  lineHeight: 1.4,
});

export const checkbox = style({
  width: vars.a11y.touchTarget,
  height: vars.a11y.touchTarget,
  minWidth: vars.a11y.touchTarget,
  minHeight: vars.a11y.touchTarget,
  margin: 0,
  cursor: 'pointer',
  selectors: {
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  },
});

// The one-time recovery-code panel — dashed warning border, monospace grid.
export const codesPanel = style({
  padding: vars.space[4],
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  border: `1px dashed ${vars.color.warning}`,
});

export const codesGrid = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fill, minmax(9rem, 1fr))',
  gap: vars.space[2],
  fontFamily: vars.font.mono,
  fontSize: '0.95rem',
  margin: `${vars.space[3]} 0`,
  userSelect: 'all',
});

export const secret = style({
  fontFamily: vars.font.mono,
  fontSize: '1rem',
  letterSpacing: '0.05em',
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  border: `1px solid ${vars.color.border}`,
  wordBreak: 'break-all',
  userSelect: 'all',
});

export const warn = style({
  fontSize: '0.88rem',
  lineHeight: 1.5,
  margin: 0,
  padding: vars.space[4],
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  borderInlineStart: `3px solid ${vars.color.warning}`,
});

export const success = style({ fontSize: '0.9rem', color: vars.color.success, margin: 0, fontWeight: 600 });
export const error = style({ fontSize: '0.85rem', color: vars.color.danger, margin: 0 });

// Direction-preview frame (W20) — respects its own `dir` attribute.
export const previewFrame = style({
  padding: vars.space[4],
  borderRadius: vars.radius.md,
  border: `1px dashed ${vars.color.border}`,
  background: vars.color.bg,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
});
