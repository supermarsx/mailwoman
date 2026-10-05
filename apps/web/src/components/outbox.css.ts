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
});

export const empty = style({
  margin: 0,
  padding: '1rem 0',
  color: 'var(--text-dim)',
});

export const items = style({
  listStyle: 'none',
  margin: 0,
  padding: 0,
  borderTop: '1px solid var(--border)',
});

export const row = style({
  display: 'grid',
  gridTemplateColumns: '8rem minmax(0, 1fr) auto',
  gridTemplateAreas: '"state subject when" "state to actions" "state note actions" "state error actions"',
  columnGap: '0.9rem',
  alignItems: 'start',
  padding: '0.7rem 0.6rem',
  borderBottom: '1px solid var(--border)',
  borderInlineStart: '3px solid transparent',
  selectors: {
    // A row that will not be sent until its owner acts.
    '&[data-state="held"]': {
      borderInlineStartColor: 'var(--accent)',
      background: 'var(--bg-alt)',
    },
    '&[data-state="failed"]': {
      borderInlineStartColor: 'var(--danger)',
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

export const state = style({
  gridArea: 'state',
  justifySelf: 'start',
  padding: '0.1rem 0.55rem',
  border: '1px solid var(--border)',
  borderRadius: '999px',
  fontSize: '0.75rem',
  fontWeight: 600,
  whiteSpace: 'nowrap',
});

export const stateOf = styleVariants({
  held: { borderColor: 'var(--accent)', color: 'var(--accent)' },
  scheduled: {},
  holding: {},
  sent: { borderColor: 'var(--success)', color: 'var(--success)' },
  canceled: { color: 'var(--text-dim)' },
  failed: { borderColor: 'var(--danger)', color: 'var(--danger)' },
});

export const when = style({
  gridArea: 'when',
  justifySelf: 'end',
  color: 'var(--text-dim)',
  fontSize: '0.8rem',
  whiteSpace: 'nowrap',
});

export const subject = style({
  gridArea: 'subject',
  fontWeight: 600,
  overflowWrap: 'anywhere',
  '@media': {
    [NARROW]: { marginTop: '0.35rem' },
  },
});

export const to = style({
  gridArea: 'to',
  marginTop: '0.15rem',
  color: 'var(--text-dim)',
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
  color: 'var(--danger)',
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
