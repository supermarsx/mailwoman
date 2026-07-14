// Admin → Plugins styles (plan §3 e7). Token-native — design tokens unchanged.

import { style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

export const screen = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  fontFamily: vars.font.ui,
  color: vars.color.text,
});

export const heading = style({ fontSize: '1.1rem', fontWeight: 600, margin: 0 });
export const prose = style({ fontSize: '0.9rem', lineHeight: 1.5, margin: 0 });
export const meta = style({ fontSize: '0.8rem', color: vars.color.textDim, margin: 0 });

/** The PERMANENT unsigned-plugin banner (non-dismissible while an unsigned plugin runs). */
export const unsignedBanner = style({
  fontSize: '0.9rem',
  lineHeight: 1.5,
  margin: 0,
  padding: vars.space[4],
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  borderLeft: `4px solid ${vars.color.danger}`,
  fontWeight: 600,
});

export const list = style({ listStyle: 'none', margin: 0, padding: 0, display: 'flex', flexDirection: 'column', gap: vars.space[3] });

export const card = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: vars.space[4],
  borderRadius: vars.radius.lg,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.surface,
});

export const cardHead = style({ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', gap: vars.space[3], flexWrap: 'wrap' });
export const title = style({ fontSize: '0.98rem', fontWeight: 600, margin: 0 });
export const row = style({ display: 'flex', gap: vars.space[2], alignItems: 'center', flexWrap: 'wrap' });

export const chip = style({
  fontSize: '0.7rem',
  fontWeight: 700,
  textTransform: 'uppercase',
  letterSpacing: '0.04em',
  padding: `2px ${vars.space[2]}`,
  borderRadius: vars.radius.pill,
  background: vars.color.bgSink,
  color: vars.color.textDim,
});

export const signedChip = style({ background: vars.color.bgSink, color: vars.color.success });
export const unsignedChip = style({ background: vars.color.bgSink, color: vars.color.danger });
export const capChip = style({ fontFamily: vars.font.mono, background: vars.color.bgAlt, color: vars.color.text });

export const button = style({
  appearance: 'none',
  border: `1px solid ${vars.color.accent}`,
  background: vars.color.accent,
  color: vars.color.accentText,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[1]} ${vars.space[3]}`,
  font: 'inherit',
  fontSize: '0.82rem',
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

export const check = style({ display: 'flex', alignItems: 'center', gap: vars.space[2], fontSize: '0.84rem', cursor: 'pointer' });
export const limits = style({ fontSize: '0.78rem', color: vars.color.textDim, fontFamily: vars.font.mono });
export const error = style({ fontSize: '0.85rem', color: vars.color.danger, margin: 0 });
