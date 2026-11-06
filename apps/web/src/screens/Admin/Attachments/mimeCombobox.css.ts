import { style, globalStyle } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

export const root = style({ position: 'relative', display: 'flex', flexDirection: 'column', gap: vars.space[2], minInlineSize: 0 });
export const field = style({ position: 'relative', display: 'flex', alignItems: 'stretch', minInlineSize: 0 });
export const input = style({ flex: '1 1 auto', minInlineSize: 0, paddingInlineEnd: '2.25rem' });
export const toggle = style({
  position: 'absolute', insetBlock: 0, insetInlineEnd: 0, inlineSize: '2.25rem', appearance: 'none', background: 'transparent',
  border: 0, color: vars.color.textDim, cursor: 'pointer',
  selectors: { '&:focus-visible': { outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`, outlineOffset: '-2px' } },
  '@media': { '(forced-colors: active)': { color: 'ButtonText', selectors: { '&:focus-visible': { outlineColor: 'Highlight' } } } },
});
export const list = style({
  position: 'absolute', insetInline: 0, insetBlockStart: '100%', zIndex: 30, maxBlockSize: '18rem', overflowY: 'auto',
  background: vars.color.surface, color: vars.color.text, border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  boxShadow: vars.elevation[2], padding: vars.space[1],
  '@media': { '(forced-colors: active)': { background: 'Canvas', color: 'CanvasText', borderColor: 'CanvasText' } },
});
export const groupLabel = style({
  padding: `${vars.space[2]} ${vars.space[3]} ${vars.space[1]}`, color: vars.color.textDim, fontSize: '0.75rem', textTransform: 'uppercase', letterSpacing: '0.04em',
});
export const option = style({
  display: 'flex', justifyContent: 'space-between', gap: vars.space[3], alignItems: 'baseline', padding: `${vars.space[2]} ${vars.space[3]}`, cursor: 'pointer',
  overflowWrap: 'anywhere', ':hover': { background: vars.color.bgAlt },
  selectors: { '&:focus-visible': { outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`, outlineOffset: '-2px' } },
  '@media': { '(forced-colors: active)': { selectors: { '&:focus-visible': { outlineColor: 'CanvasText' } } } },
});
globalStyle(`${option}[aria-selected="true"]`, {
  background: vars.color.bgAlt, color: vars.color.text, boxShadow: `inset 3px 0 ${vars.color.accent}`,
});
globalStyle(`${option}[aria-selected="true"]`, {
  '@media': { '(forced-colors: active)': { background: 'Highlight', color: 'HighlightText', forcedColorAdjust: 'none' } },
});
export const code = style({ color: vars.color.textDim, fontSize: '0.75rem', textAlign: 'end', overflowWrap: 'anywhere' });
export const hint = style({ margin: 0, color: vars.color.textDim, fontSize: '0.8125rem' });
export const hintInvalid = style({ margin: 0, color: vars.color.danger, fontSize: '0.8125rem' });
