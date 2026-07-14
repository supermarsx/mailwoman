// Admin panel styles (plan §3 e7). Token-native scoped classes over the FROZEN
// theme contract (`vars.*`), so the panel themes with the rest of the app and
// touches no existing design token. Zero-runtime: vanilla-extract compiles these
// to static CSS at build. Scoped to the lazily-loaded admin screen.

import { style, globalStyle } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const shell = style({
  display: 'grid',
  gridTemplateColumns: '220px 1fr',
  minHeight: '100vh',
  background: vars.color.bg,
  color: vars.color.text,
  fontFamily: vars.font.ui,
});

export const sidebar = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  padding: vars.space[4],
  borderRight: `1px solid ${vars.color.border}`,
  background: vars.color.surface,
});

export const brand = style({
  fontWeight: 700,
  fontSize: '1.05rem',
  marginBottom: vars.space[3],
});

export const navItem = style({
  appearance: 'none',
  textAlign: 'left',
  border: '1px solid transparent',
  background: 'transparent',
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[2]} ${vars.space[3]}`,
  font: 'inherit',
  selectors: {
    '&[aria-current="true"]': {
      background: vars.color.accent,
      color: vars.color.accentText,
    },
  },
});

export const main = style({
  padding: vars.space[6],
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[5],
  overflowY: 'auto',
  maxHeight: '100vh',
});

export const section = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  maxWidth: '760px',
});

export const heading = style({
  fontSize: '1.25rem',
  fontWeight: 700,
  margin: 0,
});

export const card = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: vars.space[4],
  borderRadius: vars.radius.lg,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.surface,
});

export const listRow = style({
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'center',
  gap: vars.space[3],
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderRadius: vars.radius.md,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
});

export const grid = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
  gap: vars.space[3],
});

export const badge = style({
  display: 'inline-block',
  padding: `2px ${vars.space[2]}`,
  borderRadius: vars.radius.pill,
  fontSize: '0.7rem',
  fontWeight: 600,
  background: vars.color.bgAlt,
  color: vars.color.textDim,
  border: `1px solid ${vars.color.border}`,
});

export const badgeDeferred = style({
  background: 'transparent',
  color: vars.color.textDim,
  fontStyle: 'italic',
});

export const table = style({
  width: '100%',
  borderCollapse: 'collapse',
  fontSize: '0.85rem',
});

globalStyle(`${table} th, ${table} td`, {
  textAlign: 'left',
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderBottom: `1px solid ${vars.color.border}`,
  verticalAlign: 'top',
});

globalStyle(`${table} th`, {
  color: vars.color.textDim,
  fontWeight: 600,
  whiteSpace: 'nowrap',
});

export const tableWrap = style({
  overflowX: 'auto',
  width: '100%',
});

export const gate = style({
  display: 'grid',
  placeItems: 'center',
  minHeight: '100vh',
  background: vars.color.bg,
  color: vars.color.text,
  fontFamily: vars.font.ui,
});

export const note = style({
  fontSize: '0.8rem',
  color: vars.color.textDim,
});

export const error = style({
  color: vars.color.danger,
  fontSize: '0.85rem',
});

export const mono = style({
  fontFamily: vars.font.mono,
  fontSize: '0.8rem',
  whiteSpace: 'pre-wrap',
  wordBreak: 'break-all',
});
