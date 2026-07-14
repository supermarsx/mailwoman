// Password-change UI styles (plan §3 e7). Token-native — design tokens unchanged.

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const panel = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  fontFamily: vars.font.ui,
  color: vars.color.text,
  maxWidth: '32rem',
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
});

export const policyList = style({
  margin: 0,
  paddingLeft: vars.space[5],
  fontSize: '0.82rem',
  color: vars.color.textDim,
  display: 'flex',
  flexDirection: 'column',
  gap: '2px',
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
  selectors: { '&:disabled': { opacity: 0.5, cursor: 'not-allowed' } },
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
});

export const check = style({
  display: 'flex',
  alignItems: 'flex-start',
  gap: vars.space[2],
  fontSize: '0.88rem',
  cursor: 'pointer',
  lineHeight: 1.4,
});

export const phrase = style({
  fontFamily: vars.font.mono,
  fontSize: '0.95rem',
  lineHeight: 1.6,
  padding: vars.space[4],
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  border: `1px dashed ${vars.color.warning}`,
  color: vars.color.text,
  wordBreak: 'break-word',
  userSelect: 'all',
});

export const warn = style({
  fontSize: '0.88rem',
  lineHeight: 1.5,
  margin: 0,
  padding: vars.space[4],
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  borderLeft: `3px solid ${vars.color.warning}`,
});

export const banner = style({
  fontSize: '0.88rem',
  lineHeight: 1.5,
  margin: 0,
  padding: vars.space[4],
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  borderLeft: `3px solid ${vars.color.danger}`,
  fontWeight: 600,
});

export const success = style({ fontSize: '0.9rem', color: vars.color.success, margin: 0, fontWeight: 600 });
export const error = style({ fontSize: '0.85rem', color: vars.color.danger, margin: 0 });
