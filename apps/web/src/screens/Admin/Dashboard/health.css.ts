// Health panel (t103): one row per component, a status chip, the reason, then
// small label/value stat rows. Same t32 rules as dashboard.css.ts.

import { style, styleVariants, globalStyle } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

export const head = style({ display: 'flex', alignItems: 'center', gap: vars.space[3], flexWrap: 'wrap' });
export const checked = style({ fontSize: '0.8rem', color: vars.color.textDim, marginInlineStart: 'auto' });

export const items = style({ listStyle: 'none', margin: 0, padding: 0, display: 'flex', flexDirection: 'column' });

export const item = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  padding: `${vars.space[3]} 0`,
  borderBottom: `${vars.border.hair} solid ${vars.color.border}`,
  minWidth: 0,
});

export const itemHead = style({ display: 'flex', alignItems: 'baseline', gap: vars.space[3], flexWrap: 'wrap' });
export const label = style({ fontWeight: vars.weight.bold, minWidth: '9rem' });
export const reason = style({ flex: '1 1 16rem', minWidth: 0, overflowWrap: 'anywhere' });
export const links = style({ display: 'inline-flex', gap: vars.space[3], flexWrap: 'wrap', fontSize: '0.85rem' });
globalStyle(`${links} a`, { color: vars.color.link, textDecoration: 'underline' });

const chipBase = {
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  minWidth: '6rem',
  fontSize: '0.8rem',
  fontWeight: vars.weight.bold,
  flex: 'none',
} as const;
export const chip = styleVariants({
  ok: { ...chipBase, color: vars.color.success },
  warn: { ...chipBase, color: vars.color.warning },
  fail: { ...chipBase, color: vars.color.danger },
  disabled: { ...chipBase, color: vars.color.textDim },
});

export const stats = style({
  display: 'grid',
  gridTemplateColumns: 'minmax(0, 14rem) minmax(0, 1fr)',
  gap: `${vars.space[1]} ${vars.space[4]}`,
  margin: 0,
  fontSize: '0.85rem',
  paddingInlineStart: `calc(6rem + ${vars.space[3]})`,
  '@media': { 'screen and (max-width: 760px)': { gridTemplateColumns: 'minmax(0, 1fr)', paddingInlineStart: 0 } },
});
globalStyle(`${stats} dt`, { color: vars.color.textDim });
globalStyle(`${stats} dd`, { margin: 0, overflowWrap: 'anywhere', fontVariantNumeric: 'tabular-nums' });
globalStyle(`${stats} ul`, { margin: 0, paddingInlineStart: vars.space[4] });

export const note = style({
  margin: 0,
  fontSize: '0.8rem',
  color: vars.color.textDim,
  paddingInlineStart: `calc(6rem + ${vars.space[3]})`,
  '@media': { 'screen and (max-width: 760px)': { paddingInlineStart: 0 } },
});

export const charts = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
  gap: vars.space[5],
  paddingTop: vars.space[3],
});
