import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';
export const container = style({ border: `${vars.border.rule} solid ${vars.color.borderStrong}`, padding: vars.space[3], minInlineSize: 0 });
export const list = style({ listStyle: 'none', padding: 0, margin: 0, display: 'flex', flexDirection: 'column', gap: vars.space[2] });
export const row = style({ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: vars.space[2], minInlineSize: 0 });
export const address = style({ display: 'flex', gap: vars.space[2], minInlineSize: 0, overflowWrap: 'anywhere', textAlign: 'start' });
