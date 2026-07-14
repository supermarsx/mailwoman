// Nextcloud UI styles (plan §3 e7). Token-native — design tokens unchanged.

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const panel = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  fontFamily: vars.font.ui,
  color: vars.color.text,
});

export const bar = style({ display: 'flex', alignItems: 'center', gap: vars.space[2], flexWrap: 'wrap' });
export const crumb = style({ fontSize: '0.82rem', color: vars.color.textDim, fontFamily: vars.font.mono });

export const list = style({
  listStyle: 'none',
  margin: 0,
  padding: 0,
  display: 'flex',
  flexDirection: 'column',
  gap: '2px',
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  maxHeight: '22rem',
  overflowY: 'auto',
});

// A single list row (the `<li>`) is a plain layout wrapper; the interactive
// affordance lives on the inner `row` button (keyboard-operable + focus ring).
export const item = style({ display: 'block' });

// Interactive file/folder row: a real <button> so it is natively focusable and
// operable with Enter/Space, with a visible focus ring (WCAG 2.4.11/2.4.13) and
// a 24px minimum target (WCAG 2.5.8). Selection is shown via aria-pressed (not
// colour alone — WCAG 1.4.1) plus the ✓ glyph.
export const row = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[3],
  width: '100%',
  minHeight: vars.a11y.touchTarget,
  padding: `${vars.space[2]} ${vars.space[3]}`,
  appearance: 'none',
  border: 'none',
  background: 'transparent',
  color: vars.color.text,
  font: 'inherit',
  textAlign: 'left',
  cursor: 'pointer',
  borderRadius: vars.radius.sm,
  transition: `background ${vars.a11y.motionDuration}, box-shadow ${vars.a11y.motionDuration}`,
  selectors: {
    '&:hover': { background: vars.color.bgAlt },
    '&[aria-pressed="true"]': { background: vars.color.bgAlt },
    '&:focus-visible': { boxShadow: vars.a11y.focusRing },
  },
});

// Non-interactive row (e.g. a file shown while picking a destination folder).
export const rowStatic = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[3],
  width: '100%',
  minHeight: vars.a11y.touchTarget,
  padding: `${vars.space[2]} ${vars.space[3]}`,
  color: vars.color.textDim,
});

export const name = style({ fontSize: '0.9rem', flex: 1 });
export const size = style({ fontSize: '0.76rem', color: vars.color.textDim });
export const dirIcon = style({ fontSize: '0.9rem' });

export const field = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2] });
export const label = style({ fontSize: '0.85rem', fontWeight: 600 });

export const input = style({
  padding: `${vars.space[2]} ${vars.space[3]}`,
  minHeight: vars.a11y.touchTarget,
  borderRadius: vars.radius.md,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  font: 'inherit',
  fontSize: '0.9rem',
  transition: `box-shadow ${vars.a11y.motionDuration}`,
  selectors: { '&:focus-visible': { boxShadow: vars.a11y.focusRing } },
});

export const check = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[2],
  minHeight: vars.a11y.touchTarget,
  fontSize: '0.88rem',
  cursor: 'pointer',
  selectors: { '&:focus-within': { boxShadow: vars.a11y.focusRing, borderRadius: vars.radius.sm } },
});

export const button = style({
  appearance: 'none',
  border: `1px solid ${vars.color.accent}`,
  background: vars.color.accent,
  color: vars.color.accentText,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  minHeight: vars.a11y.touchTarget,
  padding: `${vars.space[2]} ${vars.space[4]}`,
  font: 'inherit',
  fontSize: '0.9rem',
  fontWeight: 600,
  transition: `box-shadow ${vars.a11y.motionDuration}`,
  selectors: {
    '&:disabled': { opacity: 0.5, cursor: 'not-allowed' },
    '&:focus-visible': { boxShadow: vars.a11y.focusRing },
  },
});

export const ghost = style({
  appearance: 'none',
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  minWidth: vars.a11y.touchTarget,
  minHeight: vars.a11y.touchTarget,
  padding: `${vars.space[1]} ${vars.space[3]}`,
  font: 'inherit',
  fontSize: '0.82rem',
  transition: `box-shadow ${vars.a11y.motionDuration}`,
  selectors: {
    '&:disabled': { opacity: 0.5, cursor: 'not-allowed' },
    '&:focus-visible': { boxShadow: vars.a11y.focusRing },
  },
});

export const linkBox = style({
  fontFamily: vars.font.mono,
  fontSize: '0.85rem',
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  border: `1px solid ${vars.color.border}`,
  color: vars.color.text,
  wordBreak: 'break-all',
  userSelect: 'all',
});

export const meta = style({ fontSize: '0.8rem', color: vars.color.textDim, margin: 0 });
export const error = style({ fontSize: '0.85rem', color: vars.color.danger, margin: 0 });
export const heading = style({ fontSize: '0.95rem', fontWeight: 600, margin: 0 });
