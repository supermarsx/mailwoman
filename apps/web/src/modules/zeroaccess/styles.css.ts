// Zero-access UI styles (plan §3 e8). Token-native scoped classes — design tokens
// unchanged (regression gate), light+dark via the frozen `vars` contract.

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

export const heading = style({
  fontSize: '1rem',
  fontWeight: 600,
  margin: 0,
});

export const subHeading = style({
  fontSize: '0.8rem',
  fontWeight: 600,
  color: vars.color.textDim,
  textTransform: 'uppercase',
  letterSpacing: '0.04em',
});

export const prose = style({
  fontSize: '0.9rem',
  lineHeight: 1.5,
  color: vars.color.text,
  margin: 0,
});

export const caveat = style({
  fontSize: '0.9rem',
  lineHeight: 1.5,
  margin: 0,
  padding: vars.space[4],
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  borderLeft: `3px solid ${vars.color.warning}`,
  color: vars.color.text,
});

export const list = style({
  margin: 0,
  paddingLeft: vars.space[5],
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  fontSize: '0.88rem',
  lineHeight: 1.45,
  color: vars.color.textDim,
});

export const row = style({
  display: 'flex',
  gap: vars.space[3],
  alignItems: 'center',
  flexWrap: 'wrap',
});

export const field = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
});

export const input = style({
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderRadius: vars.radius.md,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  font: 'inherit',
  fontSize: '0.9rem',
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
  selectors: {
    '&:disabled': { opacity: 0.5, cursor: 'not-allowed' },
  },
});

export const buttonGhost = style({
  appearance: 'none',
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[2]} ${vars.space[4]}`,
  font: 'inherit',
  fontSize: '0.9rem',
});

export const danger = style({
  appearance: 'none',
  border: `1px solid ${vars.color.danger}`,
  background: 'transparent',
  color: vars.color.danger,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[2]} ${vars.space[4]}`,
  font: 'inherit',
  fontSize: '0.9rem',
  fontWeight: 600,
});

export const phrase = style({
  fontFamily: vars.font.mono,
  fontSize: '0.95rem',
  lineHeight: 1.7,
  padding: vars.space[4],
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  border: `1px dashed ${vars.color.border}`,
  wordSpacing: '0.3em',
  userSelect: 'all',
});

export const sasGrid = style({
  display: 'flex',
  flexWrap: 'wrap',
  gap: vars.space[2],
});

export const sasWord = style({
  fontFamily: vars.font.mono,
  fontSize: '0.95rem',
  fontWeight: 600,
  padding: `${vars.space[1]} ${vars.space[3]}`,
  borderRadius: vars.radius.pill,
  background: vars.color.bgAlt,
  border: `1px solid ${vars.color.border}`,
});

export const qrFrame = style({
  width: '220px',
  height: '220px',
  padding: vars.space[3],
  background: '#ffffff',
  borderRadius: vars.radius.md,
  border: `1px solid ${vars.color.border}`,
});

export const badge = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  fontSize: '0.75rem',
  fontWeight: 600,
  padding: `${vars.space[1]} ${vars.space[3]}`,
  borderRadius: vars.radius.pill,
});

export const badgeOn = style([
  badge,
  {
    background: vars.color.success,
    color: vars.color.accentText,
  },
]);

export const badgeOff = style([
  badge,
  {
    background: vars.color.bgAlt,
    color: vars.color.textDim,
    border: `1px solid ${vars.color.border}`,
  },
]);

export const error = style({
  fontSize: '0.85rem',
  color: vars.color.danger,
  margin: 0,
});
