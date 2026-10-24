// Styles for list selection and the bulk bar (t28-e16), applied beside the
// existing `list__*` classes and the list toolbar's classes (threadList.css.ts).
//
// The bulk bar takes the list toolbar's place while messages are selected, in
// the same box: the toolbar's class sets its padding, border and font, and
// `bar` only stops it wrapping, so it is never taller than the toolbar it
// replaces. Controls that do not fit scroll sideways inside it.
//
// A selected row is marked three ways, so colour is never the only cue: a check
// mark before the sender, a 2px frame drawn inside the row (it takes no space),
// and "Selected" for screen readers.

import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const bar = style({
  flexWrap: 'nowrap',
  overflowX: 'auto',
  overflowY: 'hidden',
  whiteSpace: 'nowrap',
  scrollbarWidth: 'none',
});

/** "3 selected": the bar's label, a fixed width of digits so buttons do not jump. */
export const count = style({
  flex: '0 0 auto',
  marginInlineEnd: '0.25rem',
  fontWeight: vars.weight.bold,
  fontVariantNumeric: 'tabular-nums',
});

/** The folder picker, matched to the toolbar's segment buttons. */
export const select = style({
  flex: '0 0 auto',
  padding: '0.15rem calc(0.5rem - 1px)',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  background: 'transparent',
  color: 'inherit',
  font: 'inherit',
  lineHeight: 'inherit',
  cursor: 'pointer',
});

/** A selected row: a 2px frame drawn inside it (an inset shadow, so the row
 *  keeps its size and the focus outline stays free for focus). */
export const selectedRow = style({
  selectors: {
    '&&': {
      boxShadow: `inset 0 0 0 ${vars.border.rule} ${vars.color.accent}`,
    },
  },
});

/** The check mark before a selected row's sender. */
export const check = style({
  flex: '0 0 auto',
  fontWeight: vars.weight.bold,
  color: vars.color.accent,
});

/** The $flagged mark on a row's first line. */
export const flagMark = style({
  flex: '0 0 auto',
  color: vars.color.accent,
});
