// Styles for the Focused / Other tab pair above the Inbox list (InboxTabs.tsx, t73).

import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

/** One slim row in the list header style (the view toolbar's colours, rule and
 *  font size), with less block padding so it adds as little height as it can. */
export const row = style({
  display: 'flex',
  alignItems: 'center',
  gap: 0,
  padding: '0.2rem 0.6rem',
  borderBlockEnd: `${vars.border.hair} solid ${vars.color.border}`,
  background: vars.color.bgAlt,
  fontSize: '0.78rem',
});

/** The two tabs read as one segmented control: shared border, joined edges.
 *  The block padding keeps the 24px minimum target size. */
export const tab = style({
  display: 'inline-flex',
  alignItems: 'center',
  minBlockSize: '24px',
  padding: '0.1rem calc(0.6rem - 1px)',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: 'transparent',
  color: 'inherit',
  font: 'inherit',
  lineHeight: 1.2,
  cursor: 'pointer',
  selectors: {
    '&:first-child': { borderStartStartRadius: vars.radius.sm, borderEndStartRadius: vars.radius.sm },
    '&:last-child': { borderStartEndRadius: vars.radius.sm, borderEndEndRadius: vars.radius.sm, marginInlineStart: `calc(-1 * ${vars.border.rule})` },
    '&[aria-selected="true"]': {
      background: vars.color.accent,
      color: vars.color.accentText,
      borderColor: vars.color.accent,
    },
  },
});

/** The count beside a tab's name: a small gap and tabular digits. With a badge
 *  colour set (`:root[data-badge]`) app.css turns it into a filled badge. */
export const count = style({
  marginInlineStart: '0.35rem',
  fontVariantNumeric: 'tabular-nums',
});
