// Contacts module styles (plan §3 e7). Token-native scoped classes reusing the
// V2 design-token contract so the module themes with the rest of the shell.
//
// t32 (flat, square, never bigger): 2px control and panel rules in
// `borderStrong` with 1px less padding wherever a border got thicker, focus as
// an outline, the current address book and contact marked by a 4px bar.

import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';
import * as controls from '../../styles/controls.css.ts';

/** The focus outline (forced-colours mode keeps outlines and drops box-shadows). */
const focusOutline = { outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`, outlineOffset: '2px' };

/** Small tracked capitals for headers and labels (t32 P5); the tracking token
 *  is 0 for ar/ja/zh. */
const caps = { textTransform: 'uppercase', letterSpacing: vars.tracking.label, fontWeight: vars.weight.bold } as const;

const hair = `${vars.border.hair} solid ${vars.color.border}`;
const rule = `${vars.border.rule} solid ${vars.color.borderStrong}`;

/** The selected-item bar: a 4px inset shadow on the inline-start edge, so no
 *  box changes and the state is not shown by colour alone. */
const startBar = `inset 4px 0 0 ${vars.color.accent}`;
const startBarRtl = `inset -4px 0 0 ${vars.color.accent}`;

export const groupNavigation = style({display:'grid',gridTemplateColumns:'minmax(0,1fr) auto',alignItems:'center',gap:vars.space[2]});

export const layout = style({
  display: 'grid',
  gridTemplateColumns: 'minmax(160px, 220px) minmax(220px, 340px) 1fr',
  gap: vars.space[5],
  height: '100%',
  padding: vars.space[5],
  color: vars.color.text,
  fontFamily: vars.font.ui,
  '@media': { '(max-width: 900px)': { gridTemplateColumns: 'minmax(0, 1fr)', overflowY: 'auto', alignContent: 'start' } },
});

export const sidebar = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
  borderInlineEnd: rule,
  paddingInlineEnd: `calc(${vars.space[4]} - 1px)`,
  minWidth: 0,
});

export const heading = style({
  fontSize: '0.72rem',
  ...caps,
  color: vars.color.textDim,
  margin: `${vars.space[3]} 0 ${vars.space[1]}`,
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'center',
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
  width: '100%',
  selectors: {
    '&:hover': { background: vars.color.bgAlt },
    // The frame takes the control border colour so the selected item's
    // boundary reaches 3:1 on the page.
    '&[aria-current="true"]': {
      background: vars.color.selection,
      borderColor: vars.color.borderStrong,
      fontWeight: vars.weight.bold,
      boxShadow: startBar,
    },
    '[dir="rtl"] &[aria-current="true"]': { boxShadow: startBarRtl },
    '&:focus-visible': focusOutline,
  },
});

export const count = style({
  fontSize: '0.72rem',
  color: vars.color.textDim,
  selectors: {
    // The dim colour is below 4.5:1 on the selection fill; the selected item's
    // count uses the item's own text colour.
    [`${navButton}[aria-current="true"] &`]: { color: 'inherit' },
  },
});

export const listPane = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  minWidth: 0,
  borderInlineEnd: rule,
  paddingInlineEnd: `calc(${vars.space[4]} - 1px)`,
});

export const toolbar = style({
  display: 'flex',
  gap: vars.space[2],
  flexWrap: 'wrap',
  alignItems: 'center',
});

// Fields and buttons compose the shared primitive (styles/controls.css.ts, t39 L15).
export const input = style([controls.textField, { flex: '1 1 auto' }]);

export const button = style([controls.button]);

export const buttonGhost = style([controls.buttonGhost, { paddingInline: `calc(${vars.space[3]} - 1px)` }]);

export const contactList = style({
  listStyle: 'none',
  margin: 0,
  padding: 0,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
  overflowY: 'auto',
});

export const contactRow = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[2],
  padding: `${vars.space[2]} ${vars.space[3]}`,
  border: '1px solid transparent',
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  cursor: 'pointer',
  width: '100%',
  textAlign: 'start',
  font: 'inherit',
  color: vars.color.text,
  selectors: {
    '&:hover': { background: vars.color.bgAlt },
    '&[aria-current="true"]': { borderColor: vars.color.accent, background: vars.color.selection, boxShadow: startBar },
    '[dir="rtl"] &[aria-current="true"]': { boxShadow: startBarRtl },
    '&:focus-visible': focusOutline,
  },
});

export const star = style({
  border: 'none',
  background: 'transparent',
  cursor: 'pointer',
  fontSize: '1rem',
  lineHeight: 1,
  padding: vars.space[1],
  minWidth: vars.a11y.touchTarget,
  minHeight: vars.a11y.touchTarget,
  color: vars.color.textDim,
  selectors: {
    '&[aria-pressed="true"]': { color: vars.color.warning },
    '&:focus-visible': focusOutline,
  },
});

export const rowBody = style({ flex: '1 1 auto', minWidth: 0, display: 'flex', flexDirection: 'column' });
export const rowName = style({ fontWeight: 600, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' });
export const rowMeta = style({
  fontSize: '0.75rem',
  color: vars.color.textDim,
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
  // Below 4.5:1 on the selection fill in Dark: the open contact's line uses the
  // row's own text colour.
  selectors: { [`${contactRow}[aria-current="true"] &`]: { color: 'inherit' } },
});

export const detail = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  minWidth: 0,
  overflowY: 'auto',
});

export const card = style({
  border: rule,
  borderRadius: vars.radius.lg,
  background: vars.color.surface,
  padding: `calc(${vars.space[5]} - 1px)`,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
});

export const cardName = style({ fontSize: '1.4rem', fontWeight: vars.weight.heavy, margin: 0 });
export const cardSub = style({ color: vars.color.textDim, margin: 0 });

export const fieldGroup = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2] });
export const fieldLabel = style({ fontSize: '0.72rem', ...caps, color: vars.color.textDim });
export const fieldRow = style({ display: 'flex', gap: vars.space[2], alignItems: 'center', flexWrap: 'wrap' });

export const empty = style({ color: vars.color.textDim, fontStyle: 'italic', padding: vars.space[4] });

export const dialogBackdrop = style({
  position: 'fixed',
  inset: 0,
  // The theme's modal scrim, as the other dialogs use (no colour literal).
  background: vars.color.scrim,
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  padding: vars.space[5],
  zIndex: 50,
});

// A 2px frame in `borderStrong` plus a hard shadow (the shadow vanishes under
// forced colours, the frame does not); the padding gives back the extra 1px.
export const dialog = style({
  background: vars.color.surface,
  color: vars.color.text,
  border: rule,
  borderRadius: vars.radius.lg,
  boxShadow: vars.elevation[3],
  padding: `calc(${vars.space[5]} - 1px)`,
  maxWidth: '52rem',
  width: '100%',
  maxHeight: '85vh',
  overflowY: 'auto',
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
});

export const table = style({
  borderCollapse: 'collapse',
  width: '100%',
  fontSize: '0.85rem',
});

// Header cells: tracked capitals one step smaller over a 2px rule (the bottom
// padding gives back the extra 1px).
export const th = style({
  textAlign: 'start',
  padding: vars.space[2],
  paddingBlockEnd: `calc(${vars.space[2]} - 1px)`,
  borderBlockEnd: rule,
  fontSize: '0.75rem',
  ...caps,
});

export const td = style({
  padding: vars.space[2],
  borderBlockEnd: hair,
});

export const select = style({
  padding: `calc(${vars.space[1]} - 1px) calc(${vars.space[2]} - 1px)`,
  border: rule,
  borderRadius: vars.radius.sm,
  background: vars.color.bgSink,
  color: vars.color.text,
  font: 'inherit',
  selectors: {
    ':root[data-appearance="light"] &': { background: vars.color.surface },
    '&:focus-visible': focusOutline,
  },
});

export const actions = style([controls.actionsEnd]);

// Square chip with a 1px frame; the padding gives that pixel back.
export const chip = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  padding: `calc(${vars.space[1]} - 1px) calc(${vars.space[2]} - 1px)`,
  border: hair,
  borderRadius: vars.radius.sm,
  background: vars.color.bgAlt,
  fontSize: '0.75rem',
});

// ── former inline styles (t32-h12) ──────────────────────────────────────────
// Static `style={{…}}` objects compile into the template as a literal `style=`
// attribute, which the shell CSP (`style-src 'self'`) drops.
/** The card's header row: name block at the start, actions at the end. */
export const cardHeader = style({ justifyContent: 'space-between' });
export const notesText = style({ whiteSpace: 'pre-wrap', margin: 0 });
/** A fieldset that only groups (no frame, no inset). */
export const bareFieldset = style({ border: 'none', padding: 0, margin: 0 });

/** Exclusive contact editor, independent of the list's three-column grid. */
export const editorPage = style({height: '100%', minWidth: 0, overflow: 'auto', padding: vars.space[5], color: vars.color.text});
export const editorSection = style({display: 'grid', gap: vars.space[3], border: hair, padding: vars.space[4], minWidth: 0, margin: 0});

// ── Management pages (t39 R6): membership, import, merge ─────────────────────
// A page header, then a list pane and a detail pane; one column on a phone.
export const page = style({
  display: 'flex', flexDirection: 'column', gap: vars.space[4],
  height: '100%', minWidth: 0, overflow: 'auto', padding: vars.space[5], color: vars.color.text, fontFamily: vars.font.ui,
  '@media': { '(max-width: 600px)': { padding: vars.space[4] } },
});
export const pageHeader = style({ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', gap: vars.space[2] });
// The heading takes focus on arrival (programmatic); a keyboard user still sees the ring.
globalStyle(`${pageHeader} h1:focus:not(:focus-visible)`, { outline: 'none' });
export const pageBody = style({
  display: 'grid', gridTemplateColumns: 'minmax(200px, 320px) minmax(0, 1fr)', gap: vars.space[5], alignItems: 'start', minWidth: 0,
  '@media': { '(max-width: 900px)': { gridTemplateColumns: 'minmax(0, 1fr)' } },
});
export const pageList = style({
  display: 'flex', flexDirection: 'column', gap: vars.space[2], minWidth: 0,
  borderInlineEnd: rule, paddingInlineEnd: `calc(${vars.space[4]} - 1px)`,
  '@media': { '(max-width: 900px)': { borderInlineEnd: 'none', paddingInlineEnd: 0, borderBlockEnd: rule, paddingBlockEnd: `calc(${vars.space[4]} - 1px)` } },
});
export const pageDetail = style({ display: 'flex', flexDirection: 'column', gap: vars.space[4], minWidth: 0 });
export const pageItems = style({ listStyle: 'none', margin: 0, padding: 0, display: 'flex', flexDirection: 'column', gap: vars.space[1] });
export const pageSubheading = style({ fontSize: '1rem', fontWeight: vars.weight.heavy, margin: 0, overflowWrap: 'anywhere' });
export const actionsStart = style([controls.actions]);
export const tableScroll = style({ maxInlineSize: '100%', overflowX: 'auto' });
/** The host of a management page (the page scrolls itself). */
export const pageHost = style({ height: '100%', minWidth: 0, overflow: 'hidden' });
