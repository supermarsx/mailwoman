import { style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

export const listEditor = style({ display: 'flex', flexDirection: 'column', gap: vars.space[3], minInlineSize: 0 });
export const addRow = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[3], alignItems: 'center' });
export const chips = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[2], listStyle: 'none', margin: 0, padding: 0 });
export const chip = style({
  display: 'inline-flex', alignItems: 'center', gap: vars.space[2], paddingBlock: '2px', paddingInline: vars.space[3],
  border: `${vars.border.hair} solid ${vars.color.border}`, background: vars.color.surface, minInlineSize: 0, overflowWrap: 'anywhere',
});
export const chipRemove = style({
  appearance: 'none', background: 'transparent', border: 0, color: 'inherit', cursor: 'pointer', paddingInline: vars.space[2],
  minBlockSize: '1.5rem', selectors: { '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing } },
});
export const tools = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[3], alignItems: 'flex-start' });
export const notes = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2] });
export const warning = style({ margin: 0, overflowWrap: 'anywhere' });
export const muted = style({ margin: 0, color: vars.color.textDim, fontSize: '0.875rem' });
export const readonlyLabel = style({ color: vars.color.textDim });
export const byteSize = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[3], alignItems: 'center' });
export const simulator = style({
  display: 'grid', gap: `${vars.space[3]} ${vars.space[4]}`, alignItems: 'start', maxInlineSize: '44rem',
  gridTemplateColumns: 'repeat(auto-fit, minmax(min(100%, 12rem), 1fr))',
});
const simLabel = { display: 'flex', flexDirection: 'column', gap: vars.space[2], minInlineSize: 0 } as const;
export const simFile = style({ ...simLabel, gridColumn: '1 / -1' });
export const simName = style({ ...simLabel });
export const simMime = style({ minInlineSize: 0 });
export const simSize = style({ ...simLabel });
export const simUnit = style({ ...simLabel });
export const simRun = style({ justifySelf: 'start', alignSelf: 'end' });
export const result = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2], padding: vars.space[4], border: `${vars.border.hair} solid ${vars.color.border}` });
