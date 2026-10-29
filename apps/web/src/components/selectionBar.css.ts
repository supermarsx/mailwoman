// The floating selection bar (t58): a compact, non-modal bar over the bottom
// centre of the message list while messages are selected. It is positioned
// absolutely inside the list pane, so it never pushes or reflows the rows; the
// list scroller gets `listInset` instead, a bottom padding of the bar's height
// plus its offset, so the last row can always be scrolled clear of it.
//
// Sizes are fixed rems so the bar is one line at every width: the controls that
// do not fit go into its More menu (see `inlineCount` in SelectionBar.tsx).

import { keyframes, style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

/** The bar's height and its distance from the pane's bottom edge. */
export const BAR_HEIGHT = '2.75rem';
export const BAR_OFFSET = '0.75rem';
/** Bottom padding the list takes while the bar shows: bar + offset + a gap. */
export const LIST_INSET = `calc(${BAR_HEIGHT} + ${BAR_OFFSET} + 0.5rem)`;

const rise = keyframes({
  from: { opacity: 0, transform: 'translate(-50%, 0.5rem)' },
  to: { opacity: 1, transform: 'translate(-50%, 0)' },
});

export const bar = style({
  position: 'absolute',
  zIndex: 20,
  // Physical `left` + translate centres the bar in either direction.
  left: '50%',
  bottom: BAR_OFFSET,
  transform: 'translate(-50%, 0)',
  boxSizing: 'border-box',
  display: 'flex',
  flexWrap: 'nowrap',
  alignItems: 'center',
  gap: '0.25rem',
  height: BAR_HEIGHT,
  maxWidth: 'calc(100% - 16px)',
  padding: '0.25rem 0.375rem',
  overflow: 'hidden',
  whiteSpace: 'nowrap',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  color: vars.color.text,
  boxShadow: vars.elevation[2],
  fontSize: '0.85rem',
  '@media': {
    '(prefers-reduced-motion: no-preference)': { animation: `${rise} 140ms ease-out` },
  },
  selectors: { ':root[data-reduced-motion] &': { animation: 'none' } },
});

/** "3 selected": tabular digits so the buttons do not jump as the count changes. */
export const count = style({
  flex: '0 0 auto',
  paddingInline: '0.375rem 0.25rem',
  fontWeight: vars.weight.bold,
  fontVariantNumeric: 'tabular-nums',
});

export const separator = style({
  flex: '0 0 auto',
  alignSelf: 'stretch',
  width: vars.border.hair,
  marginBlock: '0.375rem',
  background: vars.color.border,
});

/** A square icon button. */
export const button = style({
  flex: '0 0 auto',
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  boxSizing: 'border-box',
  width: '2.25rem',
  height: '2.25rem',
  padding: 0,
  border: `${vars.border.rule} solid transparent`,
  borderRadius: vars.radius.sm,
  background: 'transparent',
  color: 'inherit',
  font: 'inherit',
  cursor: 'pointer',
  selectors: {
    '&:hover:not(:disabled)': { background: vars.color.bgAlt, borderColor: vars.color.border },
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
    '&[aria-expanded="true"]': { background: vars.color.bgAlt, borderColor: vars.color.borderStrong },
    '&:disabled': { opacity: 0.5, cursor: 'default' },
  },
});

/** The live region that announces the count (visually hidden). */
export const live = style({
  position: 'absolute',
  width: '1px',
  height: '1px',
  margin: '-1px',
  padding: 0,
  overflow: 'hidden',
  clip: 'rect(0 0 0 0)',
  whiteSpace: 'nowrap',
  border: 0,
});

/** On the list's scroller while the bar shows: every row can scroll clear of
 *  the bar, including for focus and `scrollIntoView`. Doubled for specificity
 *  over the scroller's own padding. */
export const listInset = style({
  selectors: {
    '&&': { paddingBlockEnd: LIST_INSET, scrollPaddingBlockEnd: LIST_INSET },
  },
});

/** A positioned box around a pane's scroller, for panes (a folder tab) whose
 *  bar must sit over the rows and not over the pane's header or footer. */
export const paneBody = style({
  position: 'relative',
  display: 'flex',
  flexDirection: 'column',
  flex: '1 1 0',
  minHeight: 0,
  minWidth: 0,
});
