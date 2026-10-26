import { style, globalStyle } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';
export const field = style({ position: 'relative', display: 'flex', flexDirection: 'column', gap: vars.space[2], minInlineSize: 0 });
export const inputRow = style({ display: 'flex', minInlineSize: 0 });
globalStyle(`${inputRow} > input:not([type="checkbox"], [type="radio"])`, { flex: '1 1 auto', inlineSize: 0, minInlineSize: 0 });
export const toggle = style({ flex: '0 0 auto', display: 'grid', placeItems: 'center', minInlineSize: vars.a11y.touchTarget, background: vars.color.surface, color: vars.color.text, border: `1px solid ${vars.color.borderStrong}`, cursor: 'pointer', selectors: { '&:focus-visible': { outline: `2px solid ${vars.a11y.focusRingColor}`, outlineOffset: 2 } } });
export const container = style({ position: 'absolute', insetInline: 0, top: '100%', zIndex: 10, marginBlockStart: vars.space[1], maxBlockSize: 'min(18rem, 45dvh)', overflowY: 'auto', background: vars.color.surface, boxShadow: vars.elevation[2], border: `${vars.border.rule} solid ${vars.color.borderStrong}`, padding: vars.space[3], minInlineSize: 0 });
export const list = style({ listStyle: 'none', padding: 0, margin: 0, display: 'flex', flexDirection: 'column', gap: vars.space[2] });
export const row = style({ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: vars.space[2], minInlineSize: 0 });
export const address = style({ display: 'flex', gap: vars.space[2], minInlineSize: 0, overflowWrap: 'anywhere', textAlign: 'start' });

export const remove = style({
  display: 'grid', placeItems: 'center', flex: '0 0 auto',
  inlineSize: '1.75rem', blockSize: '1.75rem', minInlineSize: 0, minBlockSize: 0,
  padding: 0, margin: 0, border: 0,
  color: vars.color.text, background: 'transparent', cursor: 'pointer',
  selectors: {
    '&:hover': { background: vars.color.surface },
    '&:focus-visible': { outline: `2px solid ${vars.a11y.focusRingColor}`, outlineOffset: 2 },
    '&:disabled': { opacity: 0.5, cursor: 'default' },
  },
  '@media': { '(pointer: coarse)': { inlineSize: vars.a11y.touchTarget, blockSize: vars.a11y.touchTarget } },
});
globalStyle(`${remove} svg`, { inlineSize: '0.875rem', blockSize: '0.875rem' });
