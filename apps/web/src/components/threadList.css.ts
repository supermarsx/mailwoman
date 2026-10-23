// Styles for the conversation-threading rows (W2) + the list view toolbar.
// Self-contained vanilla-extract classes applied ALONGSIDE the legacy `list__*`
// BEM strings (same convention as mailA11y.css.ts), so the existing `.list__row`
// contract the specs locate by is untouched.

import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

/** The head slot lays the disclosure toggle beside the (flex-filling) row. */
export const headSlot = style({
  display: 'flex',
  alignItems: 'stretch',
});

/** Disclosure control for a collapsed/expanded conversation. A sibling of the
 *  row button (never nested — the row is itself a <button>). */
export const toggle = style({
  flex: '0 0 auto',
  width: '2rem',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  border: 'none',
  borderBlockEnd: `${vars.border.hair} solid ${vars.color.border}`,
  background: 'transparent',
  color: vars.color.textDim,
  cursor: 'pointer',
  fontSize: '0.8rem',
  lineHeight: 1,
});

/** The primary button of a head row fills the remaining slot width. */
export const headRow = style({
  flex: '1 1 auto',
  minWidth: 0,
});

/** Conversation-count badge on the head row (e.g. "3"): square, like every badge. */
export const count = style({
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  minWidth: '1.25rem',
  padding: '0 0.35rem',
  marginInlineStart: '0.35rem',
  borderRadius: vars.radius.sm,
  background: vars.color.bgSink,
  color: vars.color.textDim,
  fontSize: '0.72rem',
  fontWeight: vars.weight.bold,
  fontVariantNumeric: 'tabular-nums',
});

/** A member row inside an expanded conversation: indented + a quieter left rail
 *  so it reads as nested under its head. */
export const childRow = style({
  paddingInlineStart: '2rem',
  boxShadow: `inset 3px 0 0 ${vars.color.border}`,
  selectors: {
    ':root[dir="rtl"] &': { boxShadow: `inset -3px 0 0 ${vars.color.border}` },
  },
});

/** The always-visible list toolbar that hosts the reading-pane control. */
export const toolbar = style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'center',
  gap: '0.35rem',
  padding: '0.4rem 0.6rem',
  borderBlockEnd: `${vars.border.hair} solid ${vars.color.border}`,
  background: vars.color.bgAlt,
  fontSize: '0.78rem',
});

/** "Reading pane": a small tracked capital label, a step smaller than the buttons. */
export const toolbarLabel = style({
  color: vars.color.textDim,
  marginInlineEnd: '0.15rem',
  fontSize: '0.72rem',
  fontWeight: vars.weight.bold,
  letterSpacing: vars.tracking.label,
  textTransform: 'uppercase',
});

/** A segmented option button in the toolbar (reading-pane right/bottom/off).
 *  The 2px control rule takes 1px from each inline padding; the block padding
 *  stays, because the pinned 1.2 line height already made the button shorter
 *  and it must not drop further below the 24px target size. */
export const segBtn = style({
  padding: '0.15rem calc(0.5rem - 1px)',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  background: 'transparent',
  color: 'inherit',
  font: 'inherit',
  cursor: 'pointer',
  selectors: {
    '&[aria-pressed="true"]': {
      background: vars.color.accent,
      color: vars.color.accentText,
      borderColor: vars.color.accent,
    },
  },
});
