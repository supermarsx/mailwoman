// Tasks module styles (plan §3 e5). Token-native scoped classes — reuses the V2
// design-token contract so the module themes with the rest of the shell.

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const layout = style({
  display: 'grid',
  gridTemplateColumns: 'minmax(160px, 220px) 1fr',
  gap: vars.space[5],
  height: '100%',
  padding: vars.space[5],
  color: vars.color.text,
  fontFamily: vars.font.ui,
});

export const sidebar = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  borderRight: `1px solid ${vars.color.border}`,
  paddingRight: vars.space[4],
});

export const sidebarHeading = style({
  fontSize: '0.72rem',
  fontWeight: 700,
  textTransform: 'uppercase',
  letterSpacing: '0.04em',
  color: vars.color.textDim,
  margin: `${vars.space[3]} 0 ${vars.space[1]}`,
});

export const navButton = style({
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'center',
  gap: vars.space[2],
  padding: `${vars.space[2]} ${vars.space[3]}`,
  border: '1px solid transparent',
  borderRadius: vars.radius.md,
  background: 'transparent',
  color: vars.color.text,
  font: 'inherit',
  textAlign: 'left',
  cursor: 'pointer',
  selectors: {
    '&[aria-current="true"]': {
      background: vars.color.selection,
      borderColor: vars.color.border,
      fontWeight: 600,
    },
    '&:hover': { background: vars.color.bgAlt },
  },
});

export const colorDot = style({
  width: '0.6rem',
  height: '0.6rem',
  borderRadius: vars.radius.pill,
  flex: '0 0 auto',
});

export const main = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  minWidth: 0,
});

export const addForm = style({
  display: 'flex',
  gap: vars.space[2],
});

export const input = style({
  flex: '1 1 auto',
  padding: `${vars.space[2]} ${vars.space[3]}`,
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  color: vars.color.text,
  font: 'inherit',
});

export const button = style({
  padding: `${vars.space[2]} ${vars.space[4]}`,
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: vars.color.accent,
  color: vars.color.accentText,
  font: 'inherit',
  cursor: 'pointer',
});

export const taskList = style({
  listStyle: 'none',
  margin: 0,
  padding: 0,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
});

export const subtasks = style({
  listStyle: 'none',
  margin: `${vars.space[1]} 0 0 ${vars.space[6]}`,
  padding: 0,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
});

export const row = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[3],
  padding: `${vars.space[2]} ${vars.space[3]}`,
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
});

export const rowDone = style({
  opacity: 0.6,
  textDecoration: 'line-through',
});

export const title = style({ flex: '1 1 auto', minWidth: 0 });

export const meta = style({
  fontSize: '0.75rem',
  color: vars.color.textDim,
  whiteSpace: 'nowrap',
});

export const priorityHigh = style({ color: vars.color.danger, fontWeight: 700 });

export const convert = style({
  display: 'flex',
  flexWrap: 'wrap',
  gap: vars.space[2],
  alignItems: 'center',
  marginTop: 'auto',
  paddingTop: vars.space[3],
  borderTop: `1px solid ${vars.color.border}`,
});

export const empty = style({
  color: vars.color.textDim,
  fontStyle: 'italic',
  padding: vars.space[4],
});
