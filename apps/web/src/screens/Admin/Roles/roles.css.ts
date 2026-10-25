import { style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';
export const steps = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[3], listStyle: 'none', padding: 0, margin: 0, fontWeight: vars.weight.bold });
export const panel = style({ display: 'flex', flexDirection: 'column', gap: vars.space[3], minInlineSize: 0, padding: vars.space[4], border: `${vars.border.rule} solid ${vars.color.borderStrong}`, background: vars.color.surface });
export const group = style({ minInlineSize: 0, padding: vars.space[3], border: `${vars.border.hair} solid ${vars.color.border}` });
export const permissions = style({ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(min(100%, 15rem), 1fr))', gap: vars.space[3], marginBlockStart: vars.space[3] });
export const permission = style({ display: 'flex', alignItems: 'flex-start', gap: vars.space[2], cursor: 'pointer', minInlineSize: 0 });
export const capability = style({ display: 'block', fontSize: '0.75rem', color: vars.color.textDim, overflowWrap: 'anywhere' });
export const summary = style({ columns: '2 15rem', overflowWrap: 'anywhere' });
