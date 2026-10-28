// Ribbon groups at exactly three rows (t38 §2.2).
//
// A group's controls area is a column-flowing grid of three rows with a FIXED
// block size, 3 × row + 2 × gap, and the caption is a fixed line under it (not
// a row). Fewer controls leave rows empty; more controls add columns, never
// rows. `small` items take one row, `large` items all three. Nothing in a
// ribbon scrolls vertically: every box has a fixed block size and
// `overflow-y: hidden`. The panel reserves room for a thin horizontal
// scrollbar, used only once every group has folded into a dropdown.

import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';
import {
  RIBBON_CAPTION,
  RIBBON_GAP,
  RIBBON_PAD,
  RIBBON_ROW_COARSE,
  RIBBON_ROW_FINE,
  RIBBON_SCROLL_RESERVE,
} from './ribbonMetrics.ts';

globalStyle(':root', {
  vars: {
    '--mw-ribbon-row': `${RIBBON_ROW_FINE}px`,
    '--mw-ribbon-gap': `${RIBBON_GAP}px`,
    '--mw-ribbon-caption': `${RIBBON_CAPTION}px`,
    '--mw-ribbon-pad': `${RIBBON_PAD}px`,
    '--mw-ribbon-scroll-reserve': `${RIBBON_SCROLL_RESERVE}px`,
  },
  '@media': { '(pointer: coarse)': { vars: { '--mw-ribbon-row': `${RIBBON_ROW_COARSE}px` } } },
});

const controlsBlock = 'calc(3 * var(--mw-ribbon-row) + 2 * var(--mw-ribbon-gap))';
const groupBlock = `calc(${controlsBlock} + var(--mw-ribbon-caption))`;
const trackBlock = `calc(${groupBlock} + 2 * var(--mw-ribbon-pad))`;

export const panel = style({
  display: 'block',
  minInlineSize: 0,
  maxInlineSize: '100%',
  blockSize: `calc(${trackBlock} + var(--mw-ribbon-scroll-reserve))`,
  overflowX: 'hidden',
  overflowY: 'hidden',
  scrollbarWidth: 'thin',
  selectors: { '&[data-scroll="true"]': { overflowX: 'auto' } },
});

export const track = style({
  display: 'flex',
  flexWrap: 'nowrap',
  alignItems: 'flex-start',
  gap: vars.space[2],
  inlineSize: 'max-content',
  blockSize: trackBlock,
  padding: 'var(--mw-ribbon-pad)',
  overflow: 'hidden',
});

export const group = style({
  position: 'relative',
  display: 'flex',
  flexDirection: 'column',
  flex: 'none',
  blockSize: groupBlock,
  paddingInline: vars.space[1],
  borderInlineEnd: `${vars.border.hair} solid ${vars.color.border}`,
  overflow: 'hidden',
  selectors: { '&:last-child': { borderInlineEnd: 'none' } },
});

export const body = style({
  display: 'flex',
  blockSize: controlsBlock,
  overflow: 'hidden',
});

export const controls = style({
  display: 'grid',
  gridAutoFlow: 'column',
  gridTemplateRows: 'repeat(3, var(--mw-ribbon-row))',
  gridAutoColumns: 'max-content',
  rowGap: 'var(--mw-ribbon-gap)',
  columnGap: '2px',
  alignItems: 'center',
  blockSize: controlsBlock,
  overflow: 'hidden',
});

const item = style({
  display: 'flex',
  alignItems: 'center',
  minInlineSize: 0,
  overflow: 'hidden',
  whiteSpace: 'nowrap',
});

export const itemSmall = style([item, { gridRow: 'span 1', blockSize: 'var(--mw-ribbon-row)' }]);

export const itemLarge = style([
  item,
  { gridRow: 'span 3', blockSize: controlsBlock, flexDirection: 'column', justifyContent: 'center' },
]);

export const caption = style({
  blockSize: 'var(--mw-ribbon-caption)',
  lineHeight: 'var(--mw-ribbon-caption)',
  fontSize: '0.7rem',
  textAlign: 'center',
  color: vars.color.textDim,
  whiteSpace: 'nowrap',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
});

// A folded group's caption keeps its line (the height stays fixed) but not its text: the button says it.
export const captionFolded = style({ visibility: 'hidden' });

// The dropdown a folded group turns into: one `large` button.
export const more = style({
  appearance: 'none',
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'center',
  justifyContent: 'center',
  gap: vars.space[1],
  blockSize: controlsBlock,
  minInlineSize: '3.5rem',
  maxInlineSize: '7rem',
  padding: `0 ${vars.space[2]}`,
  border: `${vars.border.rule} solid transparent`,
  borderRadius: vars.radius.sm,
  background: 'transparent',
  color: vars.color.text,
  font: 'inherit',
  fontSize: '0.75rem',
  cursor: 'pointer',
  overflow: 'hidden',
  selectors: {
    '&:hover, &[aria-expanded="true"]': { borderColor: vars.color.borderStrong, background: vars.color.bgAlt },
  },
  ':focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
});

export const moreText = style({
  maxInlineSize: '100%',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
});

// The folded group's controls, in the same 3-row grid, under the button.
// Fixed, so the ribbon's `overflow: hidden` does not clip it; above the bar
// (900) and under context menus (1500).
export const popover = style({
  position: 'fixed',
  zIndex: 1400,
  maxInlineSize: 'calc(100vw - 16px)',
  padding: vars.space[1],
  background: vars.color.surface,
  color: vars.color.text,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  boxShadow: vars.elevation[3],
  overflowX: 'auto',
  overflowY: 'hidden',
  selectors: { '&[hidden]': { display: 'none' } },
});
