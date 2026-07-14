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

export const item = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[3],
  padding: `${vars.space[2]} ${vars.space[3]}`,
  cursor: 'pointer',
  selectors: {
    '&:hover': { background: vars.color.bgAlt },
    '&[aria-selected="true"]': { background: vars.color.bgAlt },
  },
});

export const name = style({ fontSize: '0.9rem', flex: 1 });
export const size = style({ fontSize: '0.76rem', color: vars.color.textDim });
export const dirIcon = style({ fontSize: '0.9rem' });

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

export const check = style({ display: 'flex', alignItems: 'center', gap: vars.space[2], fontSize: '0.88rem', cursor: 'pointer' });

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
  padding: `${vars.space[1]} ${vars.space[3]}`,
  font: 'inherit',
  fontSize: '0.82rem',
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
