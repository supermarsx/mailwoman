import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

// MultiSelect (t39 L15): a chip field over a filtered listbox. Square, ruled,
// token-only, like the shared native controls in styles/controls.css.ts.
export const root = style({ position: 'relative', display: 'flex', flexDirection: 'column', gap: vars.space[2], minInlineSize: 0, maxInlineSize: '100%' });
export const field = style({
  display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: vars.space[2],
  minBlockSize: vars.a11y.touchTarget,
  paddingBlock: vars.space[1], paddingInline: vars.space[2],
  backgroundColor: vars.color.bgSink, color: vars.color.text,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`, borderRadius: vars.radius.sm,
  cursor: 'text',
  selectors: { '&:focus-within': { outline: `2px solid ${vars.color.accent}`, outlineOffset: '1px' }, '&[data-disabled]': { opacity: 0.6, borderStyle: 'dashed', cursor: 'not-allowed' } },
  '@media': { '(forced-colors: active)': { borderColor: 'CanvasText' } },
});
export const chips = style({ display: 'contents', listStyle: 'none', margin: 0, padding: 0 });
export const chip = style({
  display: 'inline-flex', alignItems: 'center', gap: vars.space[1], maxInlineSize: '100%',
  paddingInlineStart: vars.space[2],
  backgroundColor: vars.color.surface,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`, borderRadius: vars.radius.sm,
  fontSize: '0.875rem', fontWeight: vars.weight.medium,
});
export const chipLabel = style({ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', minInlineSize: 0 });
export const chipRemove = style({
  display: 'inline-grid', placeItems: 'center',
  inlineSize: vars.a11y.touchTarget, blockSize: vars.a11y.touchTarget,
  padding: 0, border: 'none', borderInlineStart: `${vars.border.rule} solid ${vars.color.borderStrong}`, borderRadius: 0,
  background: 'transparent', color: 'inherit', cursor: 'pointer',
  selectors: { '&:focus-visible': { outline: `2px solid ${vars.color.accent}`, outlineOffset: '-2px' } },
});
export const input = style({
  flex: '1 1 8rem', minInlineSize: '6rem', minBlockSize: vars.a11y.touchTarget,
  border: 'none', background: 'transparent', color: 'inherit', font: 'inherit', padding: 0,
  selectors: { '&:focus': { outline: 'none' } },
});
// The field's own outline shows focus; the inner input must not draw a second box.
globalStyle(`${field} ${input}`, { border: 'none', backgroundColor: 'transparent', paddingBlock: 0, paddingInline: 0 });
export const listbox = style({
  position: 'absolute', insetBlockStart: '100%', insetInline: 0, zIndex: 20,
  marginBlockStart: vars.space[1], padding: vars.space[1], listStyle: 'none',
  maxBlockSize: '16rem', overflowY: 'auto',
});
export const option = style({
  display: 'flex', alignItems: 'center', gap: vars.space[2], minBlockSize: vars.a11y.touchTarget,
  paddingBlock: vars.space[1], paddingInline: vars.space[2], cursor: 'pointer', borderRadius: vars.radius.sm,
  selectors: {
    '&[data-active]': { outline: `2px solid ${vars.color.accent}`, outlineOffset: '-2px' },
  },
  '@media': { '(forced-colors: active)': { selectors: { '&[data-active]': { outlineColor: 'Highlight' } } } },
});
export const tick = style({ inlineSize: '1rem', flex: 'none', textAlign: 'center', fontWeight: vars.weight.bold });
export const optionText = style({ display: 'flex', flexDirection: 'column', minInlineSize: 0 });
export const optionDetail = style({ fontSize: '0.8rem', color: vars.color.textDim, overflowWrap: 'anywhere' });
export const empty = style({ paddingBlock: vars.space[2], paddingInline: vars.space[2], color: vars.color.textDim });
export const srOnly = style({ position: 'absolute', width: '1px', height: '1px', padding: 0, margin: '-1px', overflow: 'hidden', clip: 'rect(0, 0, 0, 0)', whiteSpace: 'nowrap', border: 0 });
