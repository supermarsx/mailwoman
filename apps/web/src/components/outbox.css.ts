// Styles for the Outbox pane (Outbox.tsx): the list of submissions the server
// holds, the note on a row that waits for the owner, and its Release / Discard
// controls.
//
// Wide layout: the pane takes the list and reader columns of the shell grid
// (as `.module-pane` does), and a row is three columns — state, what would be
// sent, and the time with the controls under it. Narrow layout (styles/app.css,
// max-width 760px): the row is one column in reading order, with the controls
// last, full width and at least 2.75rem tall.

import { style, styleVariants } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

const NARROW = '(max-width: 760px)';

export const root = style({
  gridColumn: '2 / 4',
  minWidth: 0,
  overflow: 'auto',
  padding: '1rem 1.25rem',
  '@media': {
    // Keeps the inset styles/app.css gives the pane at this width.
    [NARROW]: { padding: '0.75rem 0.75rem calc(0.75rem + env(safe-area-inset-bottom))' },
  },
});

export const header = style({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  gap: '0.75rem',
  marginBottom: '0.75rem',
});

export const title = style({
  margin: 0,
  fontSize: '1.1rem',
  fontWeight: vars.weight.heavy,
});

export const empty = style({
  margin: 0,
  padding: '1rem 0',
  color: vars.color.textDim,
});

export const items = style({
  listStyle: 'none',
  margin: 0,
  padding: 0,
  borderBlockStart: `${vars.border.hair} solid ${vars.color.border}`,
});

export const row = style({
  display: 'grid',
  gridTemplateColumns: '8rem minmax(0, 1fr) auto',
  gridTemplateAreas: '"state subject when" "state to actions" "state note actions" "state error actions"',
  columnGap: '0.9rem',
  alignItems: 'start',
  padding: '0.7rem 0.6rem',
  borderBlockEnd: `${vars.border.hair} solid ${vars.color.border}`,
  borderInlineStart: `${vars.border.heavy} solid transparent`,
  selectors: {
    // A row that will not be sent until its owner acts.
    '&[data-state="held"]': {
      borderInlineStartColor: vars.color.accent,
      background: vars.color.bgAlt,
    },
    '&[data-state="failed"]': {
      borderInlineStartColor: vars.color.danger,
    },
  },
  '@media': {
    [NARROW]: {
      gridTemplateColumns: 'minmax(0, 1fr) auto',
      gridTemplateAreas:
        '"state when" "subject subject" "to to" "note note" "error error" "actions actions"',
      padding: '0.7rem 0.5rem',
    },
  },
});

/** The state badge: square, small tracked capitals, a step smaller. Its text
 *  always names the state, so the colour is never the only sign. */
export const state = style({
  gridArea: 'state',
  justifySelf: 'start',
  padding: '0.1rem 0.55rem',
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  fontSize: '0.7rem',
  fontWeight: vars.weight.bold,
  letterSpacing: vars.tracking.label,
  textTransform: 'uppercase',
  whiteSpace: 'nowrap',
});

export const stateOf = styleVariants({
  held: { borderColor: vars.color.accent, color: vars.color.accent },
  uncertain: { borderColor: vars.color.accent, color: vars.color.accent },
  scheduled: {},
  holding: {},
  sent: { borderColor: vars.color.success, color: vars.color.success },
  canceled: { color: vars.color.textDim },
  failed: { borderColor: vars.color.danger, color: vars.color.danger },
});

export const when = style({
  gridArea: 'when',
  justifySelf: 'end',
  color: vars.color.textDim,
  fontSize: '0.8rem',
  whiteSpace: 'nowrap',
});

export const subject = style({
  gridArea: 'subject',
  fontWeight: vars.weight.bold,
  overflowWrap: 'anywhere',
  '@media': {
    [NARROW]: { marginTop: '0.35rem' },
  },
});

export const to = style({
  gridArea: 'to',
  marginTop: '0.15rem',
  color: vars.color.textDim,
  fontSize: '0.85rem',
  overflowWrap: 'anywhere',
});

export const note = style({
  gridArea: 'note',
  margin: '0.35rem 0 0',
  fontSize: '0.85rem',
  overflowWrap: 'anywhere',
});

export const error = style({
  gridArea: 'error',
  margin: '0.35rem 0 0',
  color: vars.color.danger,
  fontSize: '0.85rem',
  overflowWrap: 'anywhere',
});

export const actions = style({
  gridArea: 'actions',
  justifySelf: 'end',
  display: 'flex',
  flexWrap: 'wrap',
  justifyContent: 'flex-end',
  gap: '0.4rem',
  marginTop: '0.35rem',
  '@media': {
    [NARROW]: {
      justifySelf: 'stretch',
      marginTop: '0.6rem',
    },
  },
});

export const action = style({
  '@media': {
    [NARROW]: { flex: '1 1 0', minHeight: '2.75rem' },
  },
});
