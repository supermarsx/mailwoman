// Ribbon layout-preset styles (plan §3 e4: optional collapsible Outlook-style
// ribbon). Vanilla-extract scoped classes built entirely on the token contract
// — this is the "new components use vars.* directly" path (vs the legacy
// bridge that keeps app.css working).

import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const ribbon = style({
  display: 'flex',
  flexDirection: 'column',
  background: vars.color.bgAlt,
  borderBottom: `1px solid ${vars.color.border}`,
  backgroundImage: vars.texture.grain,
  fontFamily: vars.font.ui,
});

export const tabs = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[2],
  padding: `${vars.space[2]} ${vars.space[4]} 0`,
});

export const tab = style({
  appearance: 'none',
  border: 'none',
  background: 'transparent',
  color: vars.color.textDim,
  font: 'inherit',
  fontWeight: 600,
  cursor: 'pointer',
  padding: `${vars.space[2]} ${vars.space[4]}`,
  borderRadius: `${vars.radius.sm} ${vars.radius.sm} 0 0`,
});

export const tabActive = style({
  color: vars.color.text,
  background: vars.color.surface,
  boxShadow: vars.elevation[1],
});

export const collapseBtn = style({
  marginLeft: 'auto',
  appearance: 'none',
  border: `1px solid ${vars.color.border}`,
  background: 'transparent',
  color: vars.color.textDim,
  borderRadius: vars.radius.sm,
  cursor: 'pointer',
  padding: `${vars.space[1]} ${vars.space[3]}`,
  font: 'inherit',
});

export const body = style({
  display: 'flex',
  alignItems: 'stretch',
  gap: vars.space[4],
  padding: vars.space[4],
  background: vars.color.surface,
  overflowX: 'auto',
});

export const group = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  paddingRight: vars.space[4],
  borderRight: `1px solid ${vars.color.border}`,
  selectors: {
    '&:last-child': { borderRight: 'none' },
  },
});

export const groupRow = style({
  display: 'flex',
  gap: vars.space[2],
  flexWrap: 'wrap',
});

export const groupLabel = style({
  fontSize: '0.7rem',
  color: vars.color.textDim,
  textTransform: 'uppercase',
  letterSpacing: '0.04em',
});

export const btn = style({
  appearance: 'none',
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[2],
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[2]} ${vars.space[3]}`,
  font: 'inherit',
  fontSize: '0.85rem',
  selectors: {
    '&:hover': { borderColor: vars.color.accent },
    '&[aria-pressed="true"]': {
      background: vars.color.accent,
      color: vars.color.accentText,
      borderColor: vars.color.accent,
    },
  },
});
