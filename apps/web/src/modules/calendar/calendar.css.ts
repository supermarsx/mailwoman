// Token-native styling for the calendar module (plan §2.5: themed via the V2
// design tokens). New module → references `vars.*` directly (same posture as the
// V2 Ribbon). Dynamic geometry (event block top/height, grid column counts) is
// applied inline by the views; everything static lives here.
//
// t32 (flat, square, never bigger): 2px control and panel rules in
// `borderStrong` with 1px less padding wherever a border got thicker, 1px grid
// lines, focus as an outline, today as a yellow `signal` block. Event colours
// are the user's (inline) and are not touched.

import { globalStyle, style, styleVariants } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

/** Ink on a user-chosen event colour. The colour is user data with no theme
 *  counterpart, so the ink is fixed rather than a token (unchanged by t32). */
const userColourInk = '#fff';

/** The focus outline (forced-colours mode keeps outlines and drops box-shadows). */
const focusOutline = { outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`, outlineOffset: '2px' };

/** The same outline drawn inside the box, for grid cells whose neighbours or
 *  scrolling parent would cover an outer one. */
const focusOutlineInset = {
  outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`,
  outlineOffset: `calc(-1 * ${vars.a11y.focusRingWidth})`,
};

/** Small tracked capitals for headers and labels (t32 P5); the tracking token
 *  is 0 for ar/ja/zh. */
const caps = { textTransform: 'uppercase', letterSpacing: vars.tracking.label, fontWeight: vars.weight.bold } as const;

/** Event and day buttons that keep their own font size take the UI face
 *  instead of the browser's control font. The line height is pinned at 1.05,
 *  the line box they had with the control font (12px at 0.72rem, measured):
 *  the face's own `normal` (about 1.45) or the page's 1.2 would make a
 *  two-line event label taller than its block. */
const controlFace = { fontFamily: 'inherit', lineHeight: 1.05 } as const;

const hair = `${vars.border.hair} solid ${vars.color.border}`;
const rule = `${vars.border.rule} solid ${vars.color.borderStrong}`;

export const module = style({
  display: 'flex',
  flexDirection: 'column',
  height: '100%',
  minHeight: 0,
  color: vars.color.text,
  background: vars.color.bg,
  fontFamily: vars.font.ui,
});

export const toolbar = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[3],
  // The rule under the toolbar is 1px thicker than before. The padding does not
  // give it back: each wrapped line of controls is 1px shorter with the pinned
  // 1.2 line height (25 -> 24px), so the toolbar is no taller with one line and
  // 1px shorter with two, and the calendar below it does not gain height.
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderBlockEnd: rule,
  flexWrap: 'wrap',
});

// Visually hidden but exposed to assistive tech — used for the calendar's
// polite live region that announces the focused day + its event count.
export const srOnly = style({
  position: 'absolute',
  width: '1px',
  height: '1px',
  padding: 0,
  margin: '-1px',
  overflow: 'hidden',
  clip: 'rect(0, 0, 0, 0)',
  whiteSpace: 'nowrap',
  border: 0,
});

export const title = style({
  fontSize: '1.1rem',
  fontWeight: vars.weight.heavy,
  minWidth: '10ch',
});

export const spacer = style({ flex: 1 });

export const button = style({
  appearance: 'none',
  border: rule,
  background: 'transparent',
  color: vars.color.text,
  borderRadius: vars.radius.md,
  padding: `calc(${vars.space[1]} - 1px) calc(${vars.space[2]} - 1px)`,
  // WCAG 2.2 §2.5.8 — interactive controls are at least 24×24 CSS px.
  minHeight: vars.a11y.touchTarget,
  minWidth: vars.a11y.touchTarget,
  cursor: 'pointer',
  font: 'inherit',
  transitionProperty: 'background, color',
  transitionDuration: vars.a11y.motionDuration,
  selectors: {
    // Hover lifts a hard shadow, which changes no box.
    '&:hover': { background: vars.color.bgAlt, boxShadow: vars.elevation[1] },
    '&:active': { boxShadow: 'none' },
    '&:focus-visible': focusOutline,
    // Not colour alone: a disabled button also has a dashed border.
    '&:disabled': { opacity: 0.6, cursor: 'default', borderStyle: 'dashed', boxShadow: 'none' },
  },
});

export const primaryButton = style([
  button,
  {
    background: vars.color.accent,
    color: vars.color.accentText,
    borderColor: vars.color.accent,
    selectors: { '&:hover': { background: vars.color.accent } },
  },
]);

export const conflictButton = style([
  button,
  {
    background: vars.color.danger,
    color: vars.color.accentText,
    borderColor: vars.color.danger,
    selectors: { '&:hover': { background: vars.color.danger } },
  },
]);

export const viewSwitch = style({ display: 'flex', gap: vars.space[1], flexWrap: 'wrap' });

// The active view is an accent fill with a 3px bar along its bottom edge (an
// inset shadow, so no box changes); the bar stays on hover.
const activeBar = `inset 0 -3px 0 ${vars.color.accentText}`;

export const viewButton = styleVariants({
  base: [button],
  active: [
    button,
    {
      background: vars.color.accent,
      color: vars.color.accentText,
      borderColor: vars.color.accent,
      boxShadow: activeBar,
      selectors: { '&:hover': { background: vars.color.accent, boxShadow: activeBar } },
    },
  ],
});

export const body = style({ display: 'flex', flex: 1, minHeight: 0 });

export const sidebar = style({
  width: '15rem',
  flexShrink: 0,
  borderInlineEnd: rule,
  padding: vars.space[3],
  paddingInlineEnd: `calc(${vars.space[3]} - 1px)`,
  overflowY: 'auto',
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
});

/** The `<h3>` heading above each sidebar group. A class rather than an inline
 *  `style={{…}}`: a fully-static style object is hoisted by Solid's compiler into
 *  the template as a literal `style="…"` attribute, which the shell's
 *  `style-src 'self'` blocks (t24-e13). Tracked capitals one step smaller. */
export const sideHeading = style({ margin: '0 0 0.25rem', fontSize: '0.75rem', ...caps, color: vars.color.textDim });

export const calList = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1], listStyle: 'none', margin: 0, padding: 0 });

export const calItem = style({ display: 'flex', alignItems: 'center', gap: vars.space[2] });

export const swatch = style({ width: '0.9rem', height: '0.9rem', borderRadius: vars.radius.sm, flexShrink: 0 });

export const main = style({ flex: 1, minWidth: 0, overflow: 'auto', position: 'relative' });

// ── time grid (day / 3-day / work-week / week) ──────────────────────────────

export const timeGrid = style({ display: 'grid', gridTemplateColumns: 'auto 1fr', minHeight: '100%' });

export const allDayRow = style({
  display: 'grid',
  gridTemplateColumns: 'auto 1fr',
  borderBlockEnd: hair,
  minHeight: '1.5rem',
});

export const gutter = style({ width: '3.5rem', flexShrink: 0, color: vars.color.textDim, fontSize: '0.75rem' });

// 48px = HOUR_PX in views.tsx: events are placed in px, so the hour rows are px too.
export const hourCell = style({ height: '48px', borderBlockEnd: hair, textAlign: 'end', paddingInlineEnd: vars.space[1], boxSizing: 'border-box' });

export const dayColumns = style({ display: 'grid', position: 'relative' });

export const dayColumn = style({ position: 'relative', borderInlineEnd: hair });

export const dayHeader = style({
  position: 'sticky',
  insetBlockStart: 0,
  textAlign: 'center',
  padding: vars.space[1],
  borderBlockEnd: hair,
  background: vars.color.bgAlt,
  fontSize: '0.75rem',
  ...caps,
  zIndex: 2,
  selectors: {
    // Today: a yellow block with dark ink (filled, so it also reads on light packs).
    '&[aria-current="date"]': { background: vars.color.signal, color: vars.color.signalText },
    '&:focus-visible': focusOutlineInset,
  },
});

export const hourLine = style({ height: '48px', borderBlockEnd: hair, boxSizing: 'border-box' });

export const eventBlock = style({
  position: 'absolute',
  insetInlineStart: '2px',
  insetInlineEnd: '2px',
  borderRadius: vars.radius.sm,
  padding: '1px 4px',
  fontSize: '0.72rem',
  // Keeps the browser's control font: the title is an inline <bdi> that wraps,
  // and the bundled face's taller glyph box (about 1.45em) makes a two-line
  // title 4px taller than before even at the same line height.
  color: userColourInk,
  overflow: 'hidden',
  cursor: 'pointer',
  boxSizing: 'border-box',
  selectors: { '&:focus-visible': focusOutline },
});

export const allDayChip = style({
  display: 'inline-block',
  borderRadius: vars.radius.sm,
  padding: '0 6px',
  margin: '1px',
  fontSize: '0.72rem',
  ...controlFace,
  color: userColourInk,
  cursor: 'pointer',
  selectors: { '&:focus-visible': focusOutline },
});

// ── month / tri-month ────────────────────────────────────────────────────────

export const monthGridEl = style({ display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)', gridAutoRows: '1fr', height: '100%', minHeight: '30rem' });

// Row-based month grid (WAI-ARIA grid = rows of cells) for the interactive Month
// view: a flex column of `role=row` weeks, each a 7-column CSS grid of day cells.
export const monthGridRows = style({ display: 'flex', flexDirection: 'column', flex: 1, minHeight: '30rem' });

export const monthRow = style({ display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)', flex: 1 });

export const weekdayHead = style({ display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)', borderBlockEnd: hair, background: vars.color.bgAlt });

export const weekdayCell = style({ padding: vars.space[1], textAlign: 'center', fontSize: '0.75rem', ...caps, color: vars.color.textDim });

export const monthCell = style({
  border: hair,
  padding: '2px',
  minHeight: '4.5rem',
  overflow: 'hidden',
  display: 'flex',
  flexDirection: 'column',
  gap: '1px',
  cursor: 'pointer',
  selectors: {
    // Roving-tabindex date cell — a clearly visible focus ring for keyboard nav.
    '&:focus-visible': { ...focusOutlineInset, position: 'relative', zIndex: 1 },
    '&[aria-selected="true"]': { outline: `${vars.border.rule} solid ${vars.color.accent}`, outlineOffset: `calc(-1 * ${vars.border.rule})` },
  },
});

export const monthCellOut = style([monthCell, { background: vars.color.bgSink, color: vars.color.textDim }]);

export const dayNum = style({ fontSize: '0.75rem', alignSelf: 'flex-end' });

// Today: a square yellow block behind the date number with dark ink.
export const dayNumToday = style([
  dayNum,
  { background: vars.color.signal, color: vars.color.signalText, fontWeight: vars.weight.bold, borderRadius: vars.radius.sm, padding: '0 6px' },
]);

export const monthEvent = style({ fontSize: '0.7rem', ...controlFace, borderRadius: vars.radius.sm, padding: '0 3px', color: userColourInk, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', cursor: 'pointer', appearance: 'none', border: 'none', textAlign: 'start', selectors: { '&:focus-visible': focusOutlineInset } });

export const triMonth = style({ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: vars.space[3], padding: vars.space[3] });

// ── agenda / schedule ────────────────────────────────────────────────────────

export const agenda = style({ display: 'flex', flexDirection: 'column', padding: vars.space[3], gap: vars.space[2] });

export const agendaDay = style({ display: 'flex', flexDirection: 'column', gap: '2px' });

// Day header of the agenda list: tracked capitals over a 2px rule. The rule is
// 1px thicker than before, so the space under the text gives back 1px.
export const agendaDate = style({ ...caps, fontSize: '0.8rem', borderBlockEnd: rule, paddingBlockEnd: '1px', marginBlockEnd: '2px' });

export const agendaRow = style({ display: 'flex', gap: vars.space[3], alignItems: 'center', padding: '2px 0', cursor: 'pointer', appearance: 'none', border: 'none', background: 'none', color: 'inherit', font: 'inherit', width: '100%', textAlign: 'start', minHeight: vars.a11y.touchTarget, selectors: { '&:hover': { background: vars.color.bgAlt }, '&:focus-visible': focusOutline } });

export const agendaTime = style({ width: '9rem', flexShrink: 0, color: vars.color.textDim, fontSize: '0.85rem' });

export const agendaDot = style({ width: '0.6rem', height: '0.6rem', borderRadius: vars.radius.pill, flexShrink: 0 });

// ── year ─────────────────────────────────────────────────────────────────────

export const yearGrid = style({ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: vars.space[3], padding: vars.space[3] });

export const miniMonth = style({ border: `${vars.border.hair} solid ${vars.color.borderStrong}`, borderRadius: vars.radius.md, padding: vars.space[2] });

export const miniTitle = style({ textAlign: 'center', fontSize: '0.85rem', fontWeight: vars.weight.heavy, marginBlockEnd: '2px' });

export const miniGrid = style({ display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)', gap: '1px' });

export const miniDay = style({ fontSize: '0.65rem', ...controlFace, textAlign: 'center', padding: '1px', cursor: 'pointer', borderRadius: vars.radius.sm, appearance: 'none', border: 'none', background: 'none', color: 'inherit', selectors: { '&:focus-visible': focusOutlineInset } });

export const miniDayEvent = style([miniDay, { fontWeight: vars.weight.bold, color: vars.color.accent }]);

export const miniDayToday = style([miniDay, { background: vars.color.signal, color: vars.color.signalText, fontWeight: vars.weight.bold }]);

// ── conflict badge / participation ──────────────────────────────────────────

export const conflictBadge = style({
  display: 'inline-block',
  background: vars.color.danger,
  color: vars.color.accentText,
  fontWeight: vars.weight.bold,
  borderRadius: vars.radius.sm,
  fontSize: '0.6rem',
  padding: '0 4px',
  marginInlineStart: '2px',
});

// ── editor dialog ────────────────────────────────────────────────────────────

export const dialogBackdrop = style({
  position: 'fixed',
  inset: 0,
  // The theme's modal scrim, as the other dialogs use (no colour literal).
  background: vars.color.scrim,
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  zIndex: 50,
});

// A 2px frame in `borderStrong` (the hard shadow vanishes under forced
// colours); the padding gives the 2px back so the dialog keeps its size.
export const dialog = style({
  background: vars.color.surface,
  color: vars.color.text,
  border: rule,
  borderRadius: vars.radius.lg,
  padding: `calc(${vars.space[4]} - ${vars.border.rule})`,
  width: 'min(32rem, 92vw)',
  maxHeight: '90vh',
  overflowY: 'auto',
  boxShadow: vars.elevation[3],
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
});

export const field = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1] });

export const label = style({ fontSize: '0.8rem', ...caps, color: vars.color.textDim });

export const input = style({
  border: rule,
  background: vars.color.bgSink,
  color: vars.color.text,
  borderRadius: vars.radius.md,
  padding: `calc(${vars.space[1]} - 1px) calc(${vars.space[2]} - 1px)`,
  // The pinned 1.2 line height made these fields 22.8px; they keep the 24px
  // target floor like the buttons next to them.
  minHeight: vars.a11y.touchTarget,
  font: 'inherit',
  selectors: {
    // A light page puts fields on the brighter surface instead of the sink.
    ':root[data-appearance="light"] &': { background: vars.color.surface },
    '&:focus-visible': focusOutline,
  },
});

export const row = style({ display: 'flex', gap: vars.space[2], alignItems: 'center', flexWrap: 'wrap' });

export const dialogActions = style({ display: 'flex', gap: vars.space[2], justifyContent: 'flex-end', marginBlockStart: vars.space[2] });

export const inviteBar = style({
  display: 'flex',
  gap: vars.space[2],
  padding: vars.space[2],
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  flexWrap: 'wrap',
  alignItems: 'center',
});

export const chip = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  borderRadius: vars.radius.sm,
  border: hair,
  padding: '0 8px',
  fontSize: '0.8rem',
});

export const dangerText = style({ color: vars.color.danger });
export const dimText = style({ color: vars.color.textDim, fontSize: '0.85rem' });

// ── schedule view (distinct from the agenda list) ───────────────────────────

export const schedule = style({ display: 'flex', flexDirection: 'column', padding: vars.space[3], gap: '2px' });

export const scheduleDay = style({
  ...caps,
  fontSize: '0.8rem',
  padding: `${vars.space[2]} 0 ${vars.space[1]}`,
  borderBlockEnd: rule,
  marginBlockStart: vars.space[2],
});

export const scheduleGap = style({
  color: vars.color.textDim,
  fontSize: '0.75rem',
  fontStyle: 'italic',
  paddingBlock: '1px',
  paddingInline: '9rem 0',
  borderInlineStart: `${vars.border.rule} dashed ${vars.color.border}`,
  marginInlineStart: '4.4rem',
});

export const scheduleRow = style({
  display: 'flex',
  gap: vars.space[3],
  alignItems: 'center',
  padding: `${vars.space[1]} ${vars.space[2]}`,
  cursor: 'pointer',
  appearance: 'none',
  border: `1px solid transparent`,
  background: 'none',
  color: 'inherit',
  font: 'inherit',
  width: '100%',
  textAlign: 'start',
  borderRadius: vars.radius.md,
  minHeight: vars.a11y.touchTarget,
  selectors: {
    '&:hover': { background: vars.color.bgAlt, borderColor: vars.color.border },
    '&:focus-visible': focusOutline,
  },
});

export const scheduleRange = style({ width: '9rem', flexShrink: 0, color: vars.color.textDim, fontSize: '0.85rem', fontVariantNumeric: 'tabular-nums' });

export const scheduleDot = style({ width: '0.6rem', height: '0.6rem', borderRadius: vars.radius.pill, flexShrink: 0 });

export const scheduleTitle = style({ fontWeight: 500 });

// ── attendee rows (role / cutype pickers + reply status) ────────────────────

export const attendeeList = style({ listStyle: 'none', margin: 0, padding: 0, display: 'flex', flexDirection: 'column', gap: vars.space[1] });

export const attendeeRow = style({ display: 'flex', alignItems: 'center', gap: vars.space[2], flexWrap: 'wrap' });

export const attendeeEmail = style({ flex: 1, minWidth: '8rem', fontSize: '0.85rem' });

// Reply status badge: square, always carries its text; tracked capitals one
// step smaller so the wider capitals do not lengthen the row.
export const attendeeStatus = style({
  fontSize: '0.65rem',
  ...caps,
  borderRadius: vars.radius.sm,
  padding: '0 6px',
  border: hair,
  color: vars.color.textDim,
  whiteSpace: 'nowrap',
});

// ── conflict resolver ───────────────────────────────────────────────────────

export const resolverDialog = style([dialog, { width: 'min(46rem, 94vw)' }]);

export const resolverGridTwo = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(2, minmax(0, 1fr))',
  gap: vars.space[3],
  '@media': { 'screen and (max-width: 40rem)': { gridTemplateColumns: '1fr' } },
});

export const resolverSide = style({
  border: rule,
  borderRadius: vars.radius.md,
  padding: `calc(${vars.space[2]} - 1px)`,
  minWidth: 0,
});

export const resolverSideTitle = style({ margin: `0 0 ${vars.space[1]}`, fontSize: '0.95rem', fontWeight: vars.weight.heavy, overflowWrap: 'anywhere' });

export const resolverMeta = style({
  display: 'grid',
  gridTemplateColumns: 'auto 1fr',
  gap: '2px 8px',
  margin: 0,
  fontSize: '0.8rem',
});
globalStyle(`${resolverMeta} dt`, { color: vars.color.textDim });
globalStyle(`${resolverMeta} dd`, { margin: 0, overflowWrap: 'anywhere' });

export const resolverActions = style({ display: 'flex', gap: vars.space[2], flexWrap: 'wrap', marginBlockStart: vars.space[2] });

// ── free/busy grid ──────────────────────────────────────────────────────────

export const fbScroll = style({ overflowX: 'auto', border: `${vars.border.hair} solid ${vars.color.borderStrong}`, borderRadius: vars.radius.md });

export const fbGrid = style({ borderCollapse: 'collapse', fontSize: '0.7rem', width: '100%' });

export const fbCorner = style({ textAlign: 'start', padding: '2px 6px', position: 'sticky', insetInlineStart: 0, background: vars.color.surface, color: vars.color.textDim, ...caps });

export const fbHead = style({ padding: '2px 4px', color: vars.color.textDim, fontWeight: vars.weight.bold, textAlign: 'center', fontVariantNumeric: 'tabular-nums' });

export const fbRowHead = style({ textAlign: 'start', padding: '2px 6px', position: 'sticky', insetInlineStart: 0, background: vars.color.surface, whiteSpace: 'nowrap', maxWidth: '10rem', overflow: 'hidden', textOverflow: 'ellipsis' });

const fbCellBase = style({ textAlign: 'center', border: hair, minWidth: '1.4rem', height: '1.4rem', color: vars.color.text });

// ── former inline styles (t32-h12) ──────────────────────────────────────────
// Static `style={{…}}` objects compile into the template as a literal `style=`
// attribute, which the shell CSP (`style-src 'self'`) drops. They live here.

/** A dialog's own heading (conflict resolver, share dialog). */
export const dialogTitle = style({ margin: 0, fontSize: '1.1rem' });
/** Takes the free space in a flex row. */
export const grow = style({ flex: 1 });
export const durationInput = style({ width: '6rem' });
export const intervalInput = style({ width: '4rem' });
export const countInput = style({ width: '5rem' });
export const quickAdd = style({ minWidth: '12rem' });
/** The file input behind the Import button. */
export const fileInput = style({ display: 'none' });
/** The colour picker beside a calendar's name. */
export const colorInput = style({ width: '1.4rem', height: '1.4rem', padding: 0, border: 'none', background: 'none' });
export const addCalendar = style({ marginBlockStart: '0.5rem' });
/** A view that stacks a header over a body filling the rest of the height. */
export const viewColumn = style({ display: 'flex', flexDirection: 'column', height: '100%' });
export const timeGridView = style({ display: 'flex', flexDirection: 'column', minHeight: '100%' });
/** A CSS grid whose column track is set inline (it depends on the day count). */
export const gridRow = style({ display: 'grid' });
export const timeBody = style({ display: 'grid', flex: 1, position: 'relative' });
export const allDayLabel = style({ fontSize: '0.7rem', textAlign: 'end', paddingInlineEnd: '4px' });
export const allDayCell = style({ minHeight: '1.5rem', padding: '2px' });

export const fbCell = styleVariants({
  free: [fbCellBase, { background: vars.color.bg }],
  busy: [fbCellBase, { background: vars.color.danger, color: vars.color.accentText }],
  tentative: [fbCellBase, { background: vars.color.bgAlt }],
});
