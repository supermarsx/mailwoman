// Styles for the failed-offline-queue notice above the message list
// (OfflineQueueNotice.tsx). It sits between the list toolbar and the scroller
// and never grows past a third of the pane: a long list of failures scrolls
// inside the notice instead of pushing the messages off screen.

import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const notice = style({
  flex: '0 0 auto',
  maxHeight: '33%',
  overflowY: 'auto',
  padding: '0.5rem 0.7rem',
  borderBlockEnd: `${vars.border.hair} solid ${vars.color.border}`,
  borderInlineStart: `${vars.border.heavy} solid ${vars.color.danger}`,
  background: vars.color.bgAlt,
  fontSize: '0.85rem',
});

export const title = style({
  margin: '0 0 0.35rem',
  fontSize: '0.85rem',
  fontWeight: vars.weight.heavy,
});

export const list = style({
  listStyle: 'none',
  margin: 0,
  padding: 0,
  display: 'flex',
  flexDirection: 'column',
  gap: '0.4rem',
});

export const item = style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'center',
  gap: '0.3rem 0.6rem',
});

export const what = style({
  flex: '1 1 12rem',
  minWidth: 0,
  display: 'flex',
  flexDirection: 'column',
  gap: '0.1rem',
  overflowWrap: 'anywhere',
});

export const reason = style({
  color: vars.color.textDim,
  fontSize: '0.78rem',
});

export const buttons = style({
  display: 'flex',
  gap: '0.35rem',
});

export const button = style({
  minHeight: '2rem',
  padding: 'calc(0.2rem - 1px) calc(0.7rem - 1px)',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  background: 'transparent',
  color: 'inherit',
  font: 'inherit',
  cursor: 'pointer',
  '@media': {
    '(hover: none), (max-width: 760px)': { minHeight: '2.75rem' },
  },
});
