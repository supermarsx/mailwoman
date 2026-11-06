import { style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

/** The strip scrolls sideways on a narrow screen; a fade marks each clipped edge. */
export const frame = style({
  position: 'relative',
  marginBlockEnd: vars.space[3],
  selectors: {
    '&[data-start]::before, &[data-end]::after': {
      content: '""', position: 'absolute', insetBlock: 0, inlineSize: vars.space[6], pointerEvents: 'none', zIndex: 1,
    },
    '&[data-start]::before': { insetInlineStart: 0, background: vars.color.bg, opacity: 0.85 },
    '&[data-end]::after': { insetInlineEnd: 0, background: vars.color.bg, opacity: 0.85 },
  },
  '@media': { '(forced-colors: active)': { selectors: { '&[data-start]::before, &[data-end]::after': { display: 'none' } } } },
});
export const scroller = style({ overflowX: 'auto', overflowY: 'hidden', scrollbarWidth: 'thin', scrollbarColor: `${vars.color.borderStrong} transparent`, overscrollBehaviorX: 'contain' });
export const strip = style({ display: 'flex', alignItems: 'end', gap: vars.space[1], inlineSize: 'max-content', minInlineSize: '100%', boxSizing: 'border-box', paddingInline: vars.space[1], paddingBlockStart: vars.space[1], borderBlockEnd: `1px solid ${vars.color.border}` });
export const tab = style({
  display: 'inline-flex', alignItems: 'center', gap: vars.space[2], flex: '0 0 auto', minBlockSize: vars.a11y.touchTarget,
  padding: `${vars.space[2]} ${vars.space[4]}`, border: '1px solid transparent', borderBlockEnd: 0, color: vars.color.textDim,
  textDecoration: 'none', whiteSpace: 'nowrap', fontWeight: vars.weight.medium, cursor: 'pointer',
  selectors: {
    '&:hover': { background: vars.color.bgAlt, color: vars.color.text },
    '&[aria-selected="true"]': { background: vars.color.surface, color: vars.color.text, borderColor: vars.color.border, boxShadow: `inset 0 3px 0 ${vars.color.accent}`, fontWeight: vars.weight.bold },
    '&:focus-visible': { outline: `2px solid ${vars.a11y.focusRingColor}`, outlineOffset: -3 },
  },
  '@media': { '(forced-colors: active)': { selectors: { '&[aria-selected="true"]': { outline: '2px solid Highlight', outlineOffset: -3 } } } },
});
export const badge = style({ minInlineSize: '1.5em', paddingInline: vars.space[1], borderRadius: vars.radius.pill, background: vars.color.bgAlt, border: `1px solid ${vars.color.border}`, textAlign: 'center', fontWeight: vars.weight.bold });
