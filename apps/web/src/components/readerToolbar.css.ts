// Styles for the reader action row (ReaderToolbar.tsx, t57). The row is one
// line that never wraps: actions that do not fit fold into its "More" menu, so
// `overflow` is clipped only as a guard for the frame before the first measure.
// A rule after Forward separates the reply group from the actions on the
// message itself.

import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const row = style({
  display: 'flex',
  flexWrap: 'nowrap',
  alignItems: 'center',
  gap: '0.25rem',
  minInlineSize: 0,
  inlineSize: '100%',
  overflow: 'hidden',
  position: 'relative',
  // Room for the focus ring of the first and last buttons inside the clip.
  padding: '3px',
  margin: '-3px',
  boxSizing: 'content-box',
});

export const item = style({
  display: 'inline-flex',
  alignItems: 'center',
  flex: '0 0 auto',
});
globalStyle(`${item}[hidden]`, { display: 'none' });

export const groupEnd = style({
  marginInlineEnd: '0.25rem',
  paddingInlineEnd: '0.5rem',
  borderInlineEnd: `${vars.border.hair} solid ${vars.color.border}`,
});

/** The More button pushed to the row's end. While nothing is folded it keeps
 *  its size (so the row can be measured) but takes no space and no focus. */
export const more = style({ marginInlineStart: 'auto', flex: '0 0 auto' });
export const moreIdle = style({ position: 'absolute', visibility: 'hidden', pointerEvents: 'none' });
