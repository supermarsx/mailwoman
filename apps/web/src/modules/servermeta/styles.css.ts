// Server / mailbox METADATA view styles (t13 e8). Token-native.

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const wrap = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  fontFamily: vars.font.ui,
  color: vars.color.text,
});

export const heading = style({ fontSize: '1rem', fontWeight: 600, margin: 0 });
export const subHeading = style({
  fontSize: '0.74rem',
  fontWeight: 600,
  color: vars.color.textDim,
  textTransform: 'uppercase',
  letterSpacing: '0.04em',
  margin: 0,
});
export const meta = style({ fontSize: '0.8rem', color: vars.color.textDim, margin: 0 });
export const error = style({ fontSize: '0.85rem', color: vars.color.danger, margin: 0 });
export const mono = style({ fontFamily: vars.font.mono, fontSize: '0.85rem', wordBreak: 'break-all' });

export const notice = style({
  fontSize: '0.82rem',
  color: vars.color.textDim,
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderRadius: vars.radius.md,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bgAlt,
  margin: 0,
});

export const list = style({
  listStyle: 'none',
  margin: 0,
  padding: 0,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
});

export const row = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  padding: vars.space[3],
  borderRadius: vars.radius.lg,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.surface,
});

export const rowHeader = style({
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'center',
  gap: vars.space[3],
  flexWrap: 'wrap',
});

export const valueText = style({
  fontFamily: vars.font.mono,
  fontSize: '0.82rem',
  color: vars.color.text,
  whiteSpace: 'pre-wrap',
  wordBreak: 'break-word',
  margin: 0,
});

export const unset = style({ fontSize: '0.8rem', color: vars.color.textDim, fontStyle: 'italic' });

export const input = style({
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderRadius: vars.radius.md,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  font: 'inherit',
  fontSize: '0.9rem',
  minHeight: vars.a11y.touchTarget,
  flex: 1,
  minWidth: '10rem',
});

export const button = style({
  appearance: 'none',
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[1]} ${vars.space[3]}`,
  minHeight: vars.a11y.touchTarget,
  font: 'inherit',
  fontSize: '0.82rem',
  fontWeight: 600,
  transition: `background ${vars.a11y.motionDuration}`,
  selectors: {
    '&:disabled': { opacity: 0.5, cursor: 'not-allowed' },
    '&:hover:not(:disabled)': { background: vars.color.bgAlt },
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  },
});

export const dangerButton = style([button, { color: vars.color.danger, borderColor: vars.color.danger }]);

export const editRow = style({
  display: 'flex',
  gap: vars.space[2],
  alignItems: 'center',
  flexWrap: 'wrap',
});

export const addForm = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: vars.space[4],
  borderRadius: vars.radius.lg,
  border: `1px dashed ${vars.color.border}`,
  background: vars.color.bg,
});
