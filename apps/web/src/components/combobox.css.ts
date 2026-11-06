import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

// Every colour is explicit: the base stylesheet gives `[role="option"][aria-selected="true"]`
// the accent fill with accent text, which would otherwise leak onto a row that is only
// highlighted (dark text on a dark row in dark themes).
export const root = style({ position: 'relative', display: 'flex', flexDirection: 'column', gap: vars.space[2], inlineSize: 'min(100%, 28rem)', minInlineSize: 0 });
export const field = style({ position: 'relative', display: 'flex', alignItems: 'stretch', minInlineSize: 0 });
export const input = style({ flex: '1 1 auto', minInlineSize: 0, paddingInlineEnd: '2.5rem' });
export const toggle = style({
  position: 'absolute', insetBlock: 0, insetInlineEnd: 0, inlineSize: '2.5rem', appearance: 'none', background: 'transparent',
  border: 0, color: vars.color.textDim, cursor: 'pointer',
  selectors: { '&:focus-visible': { outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`, outlineOffset: '-2px' } },
  '@media': { '(forced-colors: active)': { color: 'ButtonText' } },
});
export const list = style({
  position: 'absolute', insetInline: 0, insetBlockStart: '100%', zIndex: 30, maxBlockSize: 'min(20rem, 50dvh)', overflowY: 'auto', overscrollBehavior: 'contain',
  background: vars.color.surface, color: vars.color.text, border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  boxShadow: vars.elevation[2], padding: vars.space[1],
  '@media': { '(forced-colors: active)': { background: 'Canvas', color: 'CanvasText', borderColor: 'CanvasText' } },
});
export const groupLabel = style({
  padding: `${vars.space[2]} ${vars.space[3]} ${vars.space[1]}`, color: vars.color.textDim, fontSize: '0.75rem', textTransform: 'uppercase', letterSpacing: '0.04em',
  '@media': { '(forced-colors: active)': { color: 'CanvasText' } },
});
export const option = style({
  display: 'flex', flexWrap: 'wrap', justifyContent: 'space-between', alignItems: 'baseline', columnGap: vars.space[3], rowGap: 0,
  minBlockSize: vars.a11y.touchTarget, padding: `${vars.space[2]} ${vars.space[3]}`, cursor: 'pointer', overflowWrap: 'anywhere',
  background: 'transparent', color: vars.color.text,
  selectors: { '&:hover': { background: vars.color.bgAlt, color: vars.color.text } },
});
export const detail = style({ color: vars.color.textDim, fontSize: '0.8125rem', fontVariantNumeric: 'tabular-nums' });
globalStyle(`${option}[aria-selected="true"]`, { fontWeight: vars.weight.bold, background: 'transparent', color: vars.color.text });
globalStyle(`${option}[data-active="true"]`, { background: vars.color.bgAlt, color: vars.color.text, boxShadow: `inset 3px 0 ${vars.color.accent}` });
globalStyle(`${option}[data-active="true"]`, {
  '@media': { '(forced-colors: active)': { background: 'Highlight', color: 'HighlightText', forcedColorAdjust: 'none' } },
});
globalStyle(`${option}[data-active="true"] ${detail}`, {
  '@media': { '(forced-colors: active)': { color: 'HighlightText' } },
});
export const empty = style({ padding: `${vars.space[2]} ${vars.space[3]}`, color: vars.color.textDim });
