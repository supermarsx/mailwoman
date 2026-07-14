// OAuth consent screen styles (plan §3 e8). Token-native — design tokens unchanged.

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const backdrop = style({
  minHeight: '100vh',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  padding: vars.space[5],
  background: vars.color.bg,
  fontFamily: vars.font.ui,
  color: vars.color.text,
});

export const card = style({
  width: '100%',
  maxWidth: '480px',
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  padding: vars.space[6],
  borderRadius: vars.radius.lg,
  background: vars.color.surface,
  border: `1px solid ${vars.color.border}`,
  boxShadow: vars.elevation[3],
});

export const heading = style({ fontSize: '1.1rem', fontWeight: 600, margin: 0 });
export const client = style({ fontSize: '1rem', fontWeight: 600 });
export const prose = style({ fontSize: '0.9rem', lineHeight: 1.5, margin: 0, color: vars.color.text });
export const subHeading = style({
  fontSize: '0.78rem',
  fontWeight: 600,
  color: vars.color.textDim,
  textTransform: 'uppercase',
  letterSpacing: '0.04em',
});

export const scopeList = style({
  margin: 0,
  paddingLeft: vars.space[5],
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  fontSize: '0.9rem',
  lineHeight: 1.45,
});

export const approved = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  fontSize: '0.75rem',
  fontWeight: 600,
  padding: `${vars.space[1]} ${vars.space[3]}`,
  borderRadius: vars.radius.pill,
  background: vars.color.success,
  color: vars.color.accentText,
  width: 'fit-content',
});

export const unapproved = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  fontSize: '0.75rem',
  fontWeight: 600,
  padding: `${vars.space[1]} ${vars.space[3]}`,
  borderRadius: vars.radius.pill,
  background: vars.color.bgAlt,
  color: vars.color.danger,
  border: `1px solid ${vars.color.danger}`,
  width: 'fit-content',
});

export const meta = style({ fontSize: '0.78rem', color: vars.color.textDim, wordBreak: 'break-all' });

export const actions = style({ display: 'flex', gap: vars.space[3], justifyContent: 'flex-end' });

export const grant = style({
  appearance: 'none',
  border: `1px solid ${vars.color.accent}`,
  background: vars.color.accent,
  color: vars.color.accentText,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[2]} ${vars.space[5]}`,
  font: 'inherit',
  fontSize: '0.9rem',
  fontWeight: 600,
  selectors: { '&:disabled': { opacity: 0.5, cursor: 'not-allowed' } },
});

export const deny = style({
  appearance: 'none',
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[2]} ${vars.space[5]}`,
  font: 'inherit',
  fontSize: '0.9rem',
});

export const error = style({ fontSize: '0.85rem', color: vars.color.danger, margin: 0 });
