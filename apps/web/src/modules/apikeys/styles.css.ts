// API-key / MCP-key UI styles (plan §3 e8). Token-native — design tokens unchanged.

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

export const heading = style({ fontSize: '1rem', fontWeight: 600, margin: 0 });
export const subHeading = style({
  fontSize: '0.78rem',
  fontWeight: 600,
  color: vars.color.textDim,
  textTransform: 'uppercase',
  letterSpacing: '0.04em',
});
export const prose = style({ fontSize: '0.9rem', lineHeight: 1.5, margin: 0 });

export const row = style({ display: 'flex', gap: vars.space[3], alignItems: 'center', flexWrap: 'wrap' });
export const field = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2] });

export const grid = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fill, minmax(220px, 1fr))',
  gap: vars.space[3],
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

export const check = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[2],
  fontSize: '0.9rem',
  cursor: 'pointer',
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

export const danger = style({
  appearance: 'none',
  border: `1px solid ${vars.color.danger}`,
  background: 'transparent',
  color: vars.color.danger,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[1]} ${vars.space[3]}`,
  font: 'inherit',
  fontSize: '0.82rem',
  fontWeight: 600,
});

export const token = style({
  fontFamily: vars.font.mono,
  fontSize: '0.9rem',
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  border: `1px dashed ${vars.color.warning}`,
  color: vars.color.text,
  wordBreak: 'break-all',
  userSelect: 'all',
});

export const keyList = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2], listStyle: 'none', margin: 0, padding: 0 });

export const keyItem = style({
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'center',
  gap: vars.space[3],
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  flexWrap: 'wrap',
});

export const meta = style({ fontSize: '0.78rem', color: vars.color.textDim });

export const revoked = style({ opacity: 0.55 });

export const warn = style({
  fontSize: '0.88rem',
  lineHeight: 1.5,
  margin: 0,
  padding: vars.space[4],
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  borderLeft: `3px solid ${vars.color.danger}`,
});

export const error = style({ fontSize: '0.85rem', color: vars.color.danger, margin: 0 });
