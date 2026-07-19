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
  borderBottom: '1px solid var(--border)',
  background: 'transparent',
  color: 'var(--text-dim)',
  cursor: 'pointer',
  fontSize: '0.8rem',
  lineHeight: 1,
});

/** The primary button of a head row fills the remaining slot width. */
export const headRow = style({
  flex: '1 1 auto',
  minWidth: 0,
});

/** Conversation-count badge on the head row (e.g. "3"). */
export const count = style({
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  minWidth: '1.25rem',
  padding: '0 0.35rem',
  marginLeft: '0.35rem',
  borderRadius: vars.radius.pill,
  background: 'var(--bg-sink)',
  color: 'var(--text-dim)',
  fontSize: '0.72rem',
  fontVariantNumeric: 'tabular-nums',
});

/** A member row inside an expanded conversation: indented + a quieter left rail
 *  so it reads as nested under its head. */
export const childRow = style({
  paddingLeft: '2rem',
  boxShadow: 'inset 3px 0 0 var(--border)',
});

/** The always-visible list toolbar that hosts the reading-pane control. */
export const toolbar = style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'center',
  gap: '0.35rem',
  padding: '0.4rem 0.6rem',
  borderBottom: '1px solid var(--border)',
  background: 'var(--bg-alt)',
  fontSize: '0.78rem',
});

export const toolbarLabel = style({
  color: 'var(--text-dim)',
  marginRight: '0.15rem',
});

/** A segmented option button in the toolbar (reading-pane right/bottom/off). */
export const segBtn = style({
  padding: '0.15rem 0.5rem',
  border: '1px solid var(--border)',
  borderRadius: vars.radius.sm,
  background: 'transparent',
  color: 'inherit',
  font: 'inherit',
  cursor: 'pointer',
  selectors: {
    '&[aria-pressed="true"]': {
      background: 'var(--accent)',
      color: 'var(--accent-text)',
      borderColor: 'var(--accent)',
    },
  },
});
