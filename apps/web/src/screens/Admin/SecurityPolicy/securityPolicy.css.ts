// Security policy screen: the DLP rule groups (tokens only, logical properties).

import { style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

export const dlp = style({ display: 'flex', flexDirection: 'column', gap: vars.space[3], minWidth: 0 });
export const rule = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  margin: 0,
  minWidth: 0,
  padding: `calc(${vars.space[3]} - 1px)`,
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  background: vars.color.bg,
});
export const ruleLegend = style({ padding: `0 ${vars.space[1]}`, fontWeight: vars.weight.heavy, overflowWrap: 'anywhere' });
export const ruleGrid = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fit, minmax(min(100%, 12rem), 1fr))',
  gap: vars.space[3],
  minWidth: 0,
});
