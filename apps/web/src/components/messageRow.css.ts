// Layout for one slot of the virtualized message list and for the per-row
// action cluster inside it (MessageList.tsx, MessageActions.tsx). Self-contained
// vanilla-extract classes applied ALONGSIDE the legacy BEM strings (`list__slot`,
// `list__row`, `msg-actions`, `msg-menu`), which the specs locate by — the same
// convention as threadList.css.ts.
//
// The slot. The list positions each slot with `translateY(index × rowHeight)`
// inside a spacer as tall as the whole list. That only lands a row where the
// windowing math expects it if the slot is taken OUT of normal flow; left in
// flow, every slot was offset by its flow position as well, so rows were drawn
// two row-heights apart and the action cluster filled the gap.
//
// The cluster. Two presentations of the same buttons:
//   - a pointer that can hover (and a viewport wider than the narrow layout):
//     the cluster sits over the bottom-right of the row, transparent until the
//     slot is hovered, holds focus, or has a menu open. It stays in the
//     accessibility tree throughout.
//   - no hover, or the narrow (phone) layout: nothing can be revealed by
//     hovering, so a "More actions" button at the row's edge opens and closes
//     the cluster, which is `display: none` until then and covers the row as a
//     bar of touch-sized buttons while open.

import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

/** Where the cluster is opened by its toggle instead of revealed by hover:
 *  touch-only pointers, and the narrow layout (styles/app.css, max-width 760px). */
const TOGGLED = '(hover: none), (max-width: 760px)';
/** The complement: a hovering pointer on a wide viewport. */
const HOVERED = '(hover: hover) and (min-width: 761px)';

/** One slot of the virtualized list: out of flow, so `translateY` alone places it. */
export const slot = style({
  position: 'absolute',
  top: 0,
  left: 0,
  right: 0,
  boxSizing: 'border-box',
  selectors: {
    // A slot's `transform` makes it a stacking context, and later slots paint
    // over earlier ones — which would bury a menu that drops below its row.
    // The slot in use is lifted. Two rules, not one selector list: a browser
    // without `:has()` would otherwise drop the `:focus-within` half too.
    '&:focus-within': { zIndex: 2 },
    '&:has([aria-expanded="true"])': { zIndex: 2 },
  },
});

/** The row button fills its slot exactly; whatever does not fit is clipped
 *  rather than spilling onto the next row. Doubled selector so it outranks the
 *  base `.list__row` rule whichever stylesheet loads last. */
export const row = style({
  selectors: {
    '&&': {
      height: '100%',
      boxSizing: 'border-box',
      overflow: 'hidden',
      paddingBlock: '0.35rem',
      alignContent: 'start',
    },
  },
});

/** The first line: sender, indicators, date. The sender takes what is left and
 *  is cut with an ellipsis; the date keeps its width at the far end. It stays in
 *  the grid's first column: label chips are placed in the second, beside it. */
export const line1 = style({
  display: 'flex',
  alignItems: 'baseline',
  gap: '0.4rem',
  minWidth: 0,
});

export const sender = style({
  flex: '0 1 auto',
  minWidth: 0,
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
});

export const date = style({
  flex: '0 0 auto',
  marginLeft: 'auto',
  whiteSpace: 'nowrap',
});

/** The preview line, dropped at compact density where the row is two lines tall. */
export const preview = style({
  selectors: {
    ':root[data-density="compact"] &': { display: 'none' },
  },
});

/** Wrapper for the toggle and the cluster. It has no box of its own, so the
 *  slot (a containing block, because it is transformed) positions both. */
export const more = style({
  display: 'contents',
});

/** "More actions": only where the cluster cannot be revealed by hover. */
export const moreToggle = style({
  position: 'absolute',
  right: '0.25rem',
  bottom: '0.25rem',
  zIndex: 2,
  minWidth: '2.75rem',
  minHeight: '2rem',
  border: '1px solid var(--border)',
  borderRadius: vars.radius.sm,
  background: 'var(--bg-alt)',
  color: 'inherit',
  font: 'inherit',
  cursor: 'pointer',
  '@media': {
    [HOVERED]: { display: 'none' },
  },
});

/** The cluster of action buttons. */
export const actions = style({
  position: 'absolute',
  display: 'flex',
  alignItems: 'center',
  gap: '0.15rem',
  background: 'var(--bg-alt)',
  border: '1px solid var(--border)',
  borderRadius: vars.radius.sm,
  zIndex: 1,
  '@media': {
    [HOVERED]: {
      right: '0.4rem',
      bottom: '0.3rem',
      padding: '0.1rem 0.2rem',
      opacity: 0,
      pointerEvents: 'none',
      selectors: {
        // `li` so this only ever matches the slot the cluster sits in.
        'li:hover > * > &, li:focus-within > * > &': { opacity: 1, pointerEvents: 'auto' },
      },
    },
    [TOGGLED]: {
      display: 'none',
      // A bar across the row, leaving the toggle (which closes it) uncovered.
      top: 0,
      bottom: 0,
      left: 0,
      right: '3.25rem',
      justifyContent: 'space-around',
      padding: '0 0.25rem',
      borderWidth: '0 1px 1px 0',
      borderRadius: 0,
    },
  },
});

/** The cluster while it is in use: its toggle is on, or one of its menus is open. */
export const actionsOpen = style({
  '@media': {
    [HOVERED]: { opacity: 1, pointerEvents: 'auto' },
    [TOGGLED]: { display: 'flex' },
  },
});

/** One action button. */
export const actionBtn = style({
  border: 'none',
  borderRadius: vars.radius.sm,
  background: 'transparent',
  color: 'inherit',
  font: 'inherit',
  lineHeight: 1,
  padding: '0.2rem 0.3rem',
  cursor: 'pointer',
  selectors: {
    '&:hover': { background: 'var(--bg-sink)' },
    '&[aria-pressed="true"]': { background: 'var(--bg-sink)' },
  },
  '@media': {
    [TOGGLED]: { minWidth: '2.75rem', minHeight: '2.75rem' },
  },
});

/** Anchor for a button's dropdown menu. */
export const menuWrap = style({
  position: 'relative',
  display: 'inline-flex',
});

/** A dropdown (snooze presets, labels) under its button. */
export const menu = style({
  position: 'absolute',
  top: 'calc(100% + 0.2rem)',
  right: 0,
  zIndex: 3,
  display: 'flex',
  flexDirection: 'column',
  minWidth: '11rem',
  maxWidth: 'min(18rem, 80vw)',
  padding: '0.2rem',
  background: 'var(--bg-alt)',
  border: '1px solid var(--border)',
  borderRadius: vars.radius.sm,
  boxShadow: '0 4px 12px rgba(0, 0, 0, 0.18)',
  '@media': {
    // In the bar the first buttons sit at the left edge, where a right-aligned
    // menu would run off-screen.
    [TOGGLED]: { right: 'auto', left: 0 },
  },
});

export const menuItem = style({
  display: 'flex',
  alignItems: 'center',
  gap: '0.4rem',
  padding: '0.35rem 0.5rem',
  border: 'none',
  borderRadius: vars.radius.sm,
  background: 'transparent',
  color: 'inherit',
  font: 'inherit',
  textAlign: 'left',
  whiteSpace: 'nowrap',
  cursor: 'pointer',
  selectors: {
    '&:hover': { background: 'var(--bg-sink)' },
  },
  '@media': {
    [TOGGLED]: { minHeight: '2.75rem' },
  },
});

/** The colour dot beside a label in the labels menu. */
export const swatch = style({
  flex: '0 0 auto',
  width: '0.7rem',
  height: '0.7rem',
  borderRadius: '50%',
});
