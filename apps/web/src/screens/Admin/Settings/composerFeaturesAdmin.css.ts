import { style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

export const panel = style({
  display: 'flex', flexDirection: 'column', gap: vars.space[2], padding: vars.space[4], marginBlockEnd: vars.space[4],
  borderRadius: vars.radius.lg, background: vars.color.surface, border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
});
export const title = style({ margin: 0, fontSize: '1rem', fontWeight: vars.weight.heavy });
export const help = style({ margin: 0, fontSize: '0.8125rem', color: vars.color.textDim, maxInlineSize: '72ch' });
export const controls = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: vars.space[2] });
export const group = style({ margin: `${vars.space[2]} 0 0`, fontSize: '0.875rem', fontWeight: vars.weight.bold });
export const list = style({ listStyle: 'none', margin: 0, padding: 0, display: 'flex', flexDirection: 'column' });
export const item = style({
  display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: vars.space[3], paddingBlock: vars.space[2],
  borderBlockStart: `1px solid ${vars.color.border}`,
});
export const name = style({ display: 'flex', flexDirection: 'column', flex: '1 1 16rem', minInlineSize: 0 });
export const desc = style({ fontSize: '0.8125rem', color: vars.color.textDim });
export const check = style({ display: 'inline-flex', alignItems: 'center', gap: vars.space[1], fontSize: '0.875rem' });
