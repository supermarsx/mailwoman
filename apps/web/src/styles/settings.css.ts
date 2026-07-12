// Settings panel styles (plan §3 e4). Token-native scoped classes.

import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const panel = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[6],
  width: '100%',
  maxWidth: '520px',
  maxHeight: '90vh',
  overflowY: 'auto',
  padding: vars.space[6],
  borderRadius: vars.radius.lg,
  background: vars.color.surface,
  border: `1px solid ${vars.color.border}`,
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

export const label = style({
  fontSize: '0.8rem',
  fontWeight: 600,
  color: vars.color.textDim,
});

export const options = style({
  display: 'flex',
  flexWrap: 'wrap',
  gap: vars.space[2],
});

export const option = style({
  appearance: 'none',
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[2]} ${vars.space[4]}`,
  font: 'inherit',
  fontSize: '0.85rem',
  selectors: {
    '&[aria-pressed="true"]': {
      background: vars.color.accent,
      color: vars.color.accentText,
      borderColor: vars.color.accent,
    },
  },
});

export const swatch = style({
  width: '28px',
  height: '28px',
  borderRadius: vars.radius.pill,
  border: `2px solid ${vars.color.border}`,
  cursor: 'pointer',
  padding: 0,
  selectors: {
    '&[aria-pressed="true"]': { borderColor: vars.color.text },
  },
});

export const select = style({
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderRadius: vars.radius.md,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  font: 'inherit',
});
