// Tasks module styles (plan §3 e5). Token-native scoped classes — reuses the V2
// design-token contract so the module themes with the rest of the shell.
//
// t32 (flat, square, never bigger): 2px control and panel rules in
// `borderStrong` with 1px less padding wherever a border got thicker, focus as
// an outline, the current list marked by a 4px bar.

import { style, globalStyle } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

/** The focus outline (forced-colours mode keeps outlines and drops box-shadows). */
const focusOutline = { outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`, outlineOffset: '2px' };

const hair = `${vars.border.hair} solid ${vars.color.border}`;
const rule = `${vars.border.rule} solid ${vars.color.borderStrong}`;

export const layout = style({
  display: 'grid',
  gridTemplateColumns: 'minmax(160px, 220px) 1fr',
  gap: vars.space[5],
  height: '100%',
  padding: vars.space[5],
  color: vars.color.text,
  fontFamily: vars.font.ui,
});

export const sidebar = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  borderInlineEnd: rule,
  paddingInlineEnd: `calc(${vars.space[4]} - 1px)`,
});

// Small tracked capitals (t32 P5); the tracking token is 0 for ar/ja/zh.
export const sidebarHeading = style({
  fontSize: '0.72rem',
  fontWeight: vars.weight.bold,
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
  color: vars.color.textDim,
  margin: `${vars.space[3]} 0 ${vars.space[1]}`,
});

export const navButton = style({
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'center',
  gap: vars.space[2],
  padding: `${vars.space[2]} ${vars.space[3]}`,
  border: '1px solid transparent',
  borderRadius: vars.radius.md,
  background: 'transparent',
  color: vars.color.text,
  font: 'inherit',
  textAlign: 'start',
  minHeight: vars.a11y.touchTarget,
  cursor: 'pointer',
  selectors: {
    '&:hover': { background: vars.color.bgAlt },
    // Not colour alone: the current list also has a 4px inset bar on its
    // inline-start edge (a shadow, so no box changes) and bold text.
    // The frame takes the control border colour so the boundary reaches 3:1.
    '&[aria-current="true"]': {
      background: vars.color.selection,
      borderColor: vars.color.borderStrong,
      fontWeight: 600,
      boxShadow: `inset 4px 0 0 ${vars.color.accent}`,
    },
    '[dir="rtl"] &[aria-current="true"]': { boxShadow: `inset -4px 0 0 ${vars.color.accent}` },
    '&:focus-visible': focusOutline,
  },
});

export const colorDot = style({
  width: '0.6rem',
  height: '0.6rem',
  borderRadius: vars.radius.pill,
  flex: '0 0 auto',
});

export const main = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  minWidth: 0,
});

export const addForm = style({
  display: 'flex',
  gap: vars.space[2],
});

export const input = style({
  flex: '1 1 auto',
  padding: `calc(${vars.space[2]} - 1px) calc(${vars.space[3]} - 1px)`,
  border: rule,
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  color: vars.color.text,
  font: 'inherit',
  minHeight: vars.a11y.touchTarget,
  selectors: {
    // A light page puts fields on the brighter surface instead of the sink.
    ':root[data-appearance="light"] &': { background: vars.color.surface },
    '&:focus-visible': focusOutline,
  },
});

export const button = style({
  padding: `calc(${vars.space[2]} - 1px) calc(${vars.space[4]} - 1px)`,
  border: `${vars.border.rule} solid ${vars.color.accent}`,
  borderRadius: vars.radius.md,
  background: vars.color.accent,
  color: vars.color.accentText,
  font: 'inherit',
  minHeight: vars.a11y.touchTarget,
  cursor: 'pointer',
  selectors: {
    // Hover lifts a hard shadow, which changes no box.
    '&:hover:not(:disabled)': { boxShadow: vars.elevation[1] },
    '&:active': { boxShadow: 'none' },
    '&:focus-visible': focusOutline,
    // Not colour alone: a disabled button also has a dashed border.
    '&:disabled': { opacity: 0.6, cursor: 'default', borderStyle: 'dashed' },
  },
});

/** The confirm step of a destructive action (delete). */
export const dangerButton = style([
  button,
  { background: vars.color.danger, color: vars.color.accentText, borderColor: vars.color.danger },
]);

// WCAG 2.2 §2.5.8 — the complete/reopen checkbox is at least 24×24 CSS px.
export const checkbox = style({
  width: vars.a11y.touchTarget,
  height: vars.a11y.touchTarget,
  cursor: 'pointer',
  accentColor: vars.color.accent,
});

export const taskList = style({
  listStyle: 'none',
  margin: 0,
  padding: 0,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
});

export const subtasks = style({
  listStyle: 'none',
  marginBlock: `${vars.space[1]} 0`,
  marginInline: `${vars.space[6]} 0`,
  padding: 0,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
});

export const row = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[3],
  padding: `${vars.space[2]} ${vars.space[3]}`,
  border: hair,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
});

/** The inline edit form that replaces a row while it is being edited: a panel
 *  with a 2px frame (the padding gives back the extra 1px). */
export const editForm = style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'flex-end',
  gap: vars.space[3],
  padding: `calc(${vars.space[2]} - 1px) calc(${vars.space[3]} - 1px)`,
  border: rule,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
});

export const field = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
  flex: '1 1 10rem',
  minWidth: 0,
});

export const editActions = style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'center',
  gap: vars.space[2],
});

export const rowDone = style({
  opacity: 0.6,
  textDecoration: 'line-through',
});

export const title = style({ flex: '1 1 auto', minWidth: 0 });

export const meta = style({
  fontSize: '0.75rem',
  color: vars.color.textDim,
  whiteSpace: 'nowrap',
});

export const priorityHigh = style({ color: vars.color.danger, fontWeight: vars.weight.bold });

// Separated from the list by a 2px rule; the top padding gives back the extra 1px.
export const convert = style({
  display: 'flex',
  flexWrap: 'wrap',
  gap: vars.space[2],
  alignItems: 'center',
  marginBlockStart: 'auto',
  paddingBlockStart: `calc(${vars.space[3]} - 1px)`,
  borderBlockStart: rule,
});

export const empty = style({
  color: vars.color.textDim,
  fontStyle: 'italic',
  padding: vars.space[4],
});

// Phone-sized panes must not leave the task editor in a narrow second column.
globalStyle(layout,{'@media':{'screen and (max-width: 40rem)':{gridTemplateColumns:'minmax(0,1fr)',gridTemplateRows:'minmax(6rem,30%) minmax(0,1fr)',gap:vars.space[3],padding:vars.space[3],minHeight:0}}});
globalStyle(sidebar,{'@media':{'screen and (max-width: 40rem)':{minHeight:0,overflow:'auto',borderInlineEnd:0,borderBlockEnd:rule,paddingInlineEnd:0}}});
globalStyle(main,{'@media':{'screen and (max-width: 40rem)':{minWidth:0,minHeight:0,overflow:'auto'}}});

// Scope containment to this module's existing shell host; no global layout changes.
globalStyle('.module-pane[data-surface="tasks"]',{containerType:'inline-size',containerName:'mw-tasks'});
globalStyle(layout,{'@container':{'mw-tasks (max-width: 40rem)':{gridTemplateColumns:'minmax(0,1fr)',gridTemplateRows:'minmax(6rem,30%) minmax(0,1fr)',gap:vars.space[3],padding:vars.space[3],minHeight:0}}});
globalStyle(sidebar,{'@container':{'mw-tasks (max-width: 40rem)':{minHeight:0,overflow:'auto',borderInlineEnd:0,borderBlockEnd:rule,paddingInlineEnd:0}}});
globalStyle(main,{'@container':{'mw-tasks (max-width: 40rem)':{minWidth:0,minHeight:0,overflow:'auto'}}});
