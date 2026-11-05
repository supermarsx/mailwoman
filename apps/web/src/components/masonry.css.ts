// Masonry: children flow into balanced CSS columns, top-aligned, with no row
// stretching, so a short panel never leaves a gap under it. DOM order is the
// reading order and the tab order. One column below `minColumn`, more as the
// container widens. Children must be direct, block-level boxes.

import { style, globalStyle } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const masonry = style({
  columnWidth: '20rem',
  columnGap: vars.space[4],
  minInlineSize: 0,
});
globalStyle(`${masonry} > *`, {
  breakInside: 'avoid',
  display: 'block',
  inlineSize: '100%',
  boxSizing: 'border-box',
  marginBlockEnd: vars.space[4],
});
