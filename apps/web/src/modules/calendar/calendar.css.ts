// Token-native styling for the calendar module (plan §2.5: themed via the V2
// design tokens). New module → references `vars.*` directly (same posture as the
// V2 Ribbon). Dynamic geometry (event block top/height, grid column counts) is
// applied inline by the views; everything static lives here.

import { globalStyle, style, styleVariants } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

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
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderBottom: `1px solid ${vars.color.border}`,
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
  fontWeight: 600,
  minWidth: '10ch',
});

export const spacer = style({ flex: 1 });

export const button = style({
  appearance: 'none',
  border: `1px solid ${vars.color.border}`,
  background: vars.color.surface,
  color: vars.color.text,
  borderRadius: vars.radius.md,
  padding: `${vars.space[1]} ${vars.space[2]}`,
  // WCAG 2.2 §2.5.8 — interactive controls are at least 24×24 CSS px.
  minHeight: vars.a11y.touchTarget,
  minWidth: vars.a11y.touchTarget,
  cursor: 'pointer',
  font: 'inherit',
  transitionProperty: 'background',
  transitionDuration: vars.a11y.motionDuration,
  selectors: {
    '&:hover': { background: vars.color.bgAlt },
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  },
});

export const primaryButton = style([button, { background: vars.color.accent, color: vars.color.accentText, borderColor: vars.color.accent }]);

export const conflictButton = style([button, { background: vars.color.danger, color: '#fff', borderColor: vars.color.danger }]);

export const viewSwitch = style({ display: 'flex', gap: vars.space[1], flexWrap: 'wrap' });

export const viewButton = styleVariants({
  base: [button],
  active: [button, { background: vars.color.accent, color: vars.color.accentText, borderColor: vars.color.accent }],
});

export const body = style({ display: 'flex', flex: 1, minHeight: 0 });

export const sidebar = style({
  width: '15rem',
  flexShrink: 0,
  borderInlineEnd: `1px solid ${vars.color.border}`,
  padding: vars.space[3],
  overflowY: 'auto',
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
});

export const calList = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1], listStyle: 'none', margin: 0, padding: 0 });

export const calItem = style({ display: 'flex', alignItems: 'center', gap: vars.space[2] });

export const swatch = style({ width: '0.9rem', height: '0.9rem', borderRadius: vars.radius.sm, flexShrink: 0 });

export const main = style({ flex: 1, minWidth: 0, overflow: 'auto', position: 'relative' });

// ── time grid (day / 3-day / work-week / week) ──────────────────────────────

export const timeGrid = style({ display: 'grid', gridTemplateColumns: 'auto 1fr', minHeight: '100%' });

export const allDayRow = style({
  display: 'grid',
  gridTemplateColumns: 'auto 1fr',
  borderBottom: `1px solid ${vars.color.border}`,
  minHeight: '1.5rem',
});

export const gutter = style({ width: '3.5rem', flexShrink: 0, color: vars.color.textDim, fontSize: '0.75rem' });

export const hourCell = style({ height: '3rem', borderBottom: `1px solid ${vars.color.border}`, textAlign: 'end', paddingInlineEnd: vars.space[1], boxSizing: 'border-box' });

export const dayColumns = style({ display: 'grid', position: 'relative' });

export const dayColumn = style({ position: 'relative', borderInlineEnd: `1px solid ${vars.color.border}` });

export const dayHeader = style({ position: 'sticky', top: 0, textAlign: 'center', padding: vars.space[1], borderBottom: `1px solid ${vars.color.border}`, background: vars.color.bgAlt, fontSize: '0.8rem', zIndex: 2 });

export const hourLine = style({ height: '3rem', borderBottom: `1px solid ${vars.color.border}`, boxSizing: 'border-box' });

export const eventBlock = style({
  position: 'absolute',
  insetInlineStart: '2px',
  insetInlineEnd: '2px',
  borderRadius: vars.radius.sm,
  padding: '1px 4px',
  fontSize: '0.72rem',
  color: '#fff',
  overflow: 'hidden',
  cursor: 'pointer',
  boxSizing: 'border-box',
  selectors: { '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing } },
});

export const allDayChip = style({
  display: 'inline-block',
  borderRadius: vars.radius.pill,
  padding: '0 6px',
  margin: '1px',
  fontSize: '0.72rem',
  color: '#fff',
  cursor: 'pointer',
});

// ── month / tri-month ────────────────────────────────────────────────────────

export const monthGridEl = style({ display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)', gridAutoRows: '1fr', height: '100%', minHeight: '30rem' });

// Row-based month grid (WAI-ARIA grid = rows of cells) for the interactive Month
// view: a flex column of `role=row` weeks, each a 7-column CSS grid of day cells.
export const monthGridRows = style({ display: 'flex', flexDirection: 'column', flex: 1, minHeight: '30rem' });

export const monthRow = style({ display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)', flex: 1 });

export const weekdayHead = style({ display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)', borderBottom: `1px solid ${vars.color.border}`, background: vars.color.bgAlt });

export const weekdayCell = style({ padding: vars.space[1], textAlign: 'center', fontSize: '0.75rem', color: vars.color.textDim });

export const monthCell = style({
  border: `1px solid ${vars.color.border}`,
  padding: '2px',
  minHeight: '4.5rem',
  overflow: 'hidden',
  display: 'flex',
  flexDirection: 'column',
  gap: '1px',
  cursor: 'pointer',
  selectors: {
    // Roving-tabindex date cell — a clearly visible focus ring for keyboard nav.
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing, position: 'relative', zIndex: 1 },
    '&[aria-selected="true"]': { outline: `2px solid ${vars.color.accent}`, outlineOffset: '-2px' },
  },
});

export const monthCellOut = style([monthCell, { background: vars.color.bgSink, color: vars.color.textDim }]);

export const dayNum = style({ fontSize: '0.75rem', alignSelf: 'flex-end' });

export const dayNumToday = style([dayNum, { background: vars.color.accent, color: vars.color.accentText, borderRadius: vars.radius.pill, padding: '0 6px' }]);

export const monthEvent = style({ fontSize: '0.7rem', borderRadius: vars.radius.sm, padding: '0 3px', color: '#fff', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', cursor: 'pointer', appearance: 'none', border: 'none', textAlign: 'start', selectors: { '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing } } });

export const triMonth = style({ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: vars.space[3], padding: vars.space[3] });

// ── agenda / schedule ────────────────────────────────────────────────────────

export const agenda = style({ display: 'flex', flexDirection: 'column', padding: vars.space[3], gap: vars.space[2] });

export const agendaDay = style({ display: 'flex', flexDirection: 'column', gap: '2px' });

export const agendaDate = style({ fontWeight: 600, borderBottom: `1px solid ${vars.color.border}`, paddingBottom: '2px', marginBottom: '2px' });

export const agendaRow = style({ display: 'flex', gap: vars.space[3], alignItems: 'center', padding: '2px 0', cursor: 'pointer', appearance: 'none', border: 'none', background: 'none', color: 'inherit', font: 'inherit', width: '100%', textAlign: 'start', minHeight: vars.a11y.touchTarget, selectors: { '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing } } });

export const agendaTime = style({ width: '9rem', flexShrink: 0, color: vars.color.textDim, fontSize: '0.85rem' });

export const agendaDot = style({ width: '0.6rem', height: '0.6rem', borderRadius: '50%', flexShrink: 0 });

// ── year ─────────────────────────────────────────────────────────────────────

export const yearGrid = style({ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: vars.space[3], padding: vars.space[3] });

export const miniMonth = style({ border: `1px solid ${vars.color.border}`, borderRadius: vars.radius.md, padding: vars.space[2] });

export const miniTitle = style({ textAlign: 'center', fontSize: '0.85rem', fontWeight: 600, marginBottom: '2px' });

export const miniGrid = style({ display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)', gap: '1px' });

export const miniDay = style({ fontSize: '0.65rem', textAlign: 'center', padding: '1px', cursor: 'pointer', borderRadius: vars.radius.sm, appearance: 'none', border: 'none', background: 'none', color: 'inherit', selectors: { '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing } } });

export const miniDayEvent = style([miniDay, { fontWeight: 700, color: vars.color.accent }]);

export const miniDayToday = style([miniDay, { background: vars.color.accent, color: vars.color.accentText }]);

// ── conflict badge / participation ──────────────────────────────────────────

export const conflictBadge = style({
  display: 'inline-block',
  background: vars.color.danger,
  color: '#fff',
  borderRadius: vars.radius.pill,
  fontSize: '0.6rem',
  padding: '0 4px',
  marginInlineStart: '2px',
});

// ── editor dialog ────────────────────────────────────────────────────────────

export const dialogBackdrop = style({
  position: 'fixed',
  inset: 0,
  background: 'rgba(0,0,0,0.4)',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  zIndex: 50,
});

export const dialog = style({
  background: vars.color.surface,
  color: vars.color.text,
  borderRadius: vars.radius.lg,
  padding: vars.space[4],
  width: 'min(32rem, 92vw)',
  maxHeight: '90vh',
  overflowY: 'auto',
  boxShadow: vars.elevation[3],
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
});

export const field = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1] });

export const label = style({ fontSize: '0.8rem', color: vars.color.textDim });

export const input = style({
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.md,
  padding: `${vars.space[1]} ${vars.space[2]}`,
  font: 'inherit',
  selectors: { '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing } },
});

export const row = style({ display: 'flex', gap: vars.space[2], alignItems: 'center', flexWrap: 'wrap' });

export const dialogActions = style({ display: 'flex', gap: vars.space[2], justifyContent: 'flex-end', marginTop: vars.space[2] });

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
  borderRadius: vars.radius.pill,
  border: `1px solid ${vars.color.border}`,
  padding: '0 8px',
  fontSize: '0.8rem',
});

export const dangerText = style({ color: vars.color.danger });
export const dimText = style({ color: vars.color.textDim, fontSize: '0.85rem' });

// ── schedule view (distinct from the agenda list) ───────────────────────────

export const schedule = style({ display: 'flex', flexDirection: 'column', padding: vars.space[3], gap: '2px' });

export const scheduleDay = style({
  fontWeight: 600,
  fontSize: '0.9rem',
  padding: `${vars.space[2]} 0 ${vars.space[1]}`,
  borderBottom: `2px solid ${vars.color.border}`,
  marginTop: vars.space[2],
});

export const scheduleGap = style({
  color: vars.color.textDim,
  fontSize: '0.75rem',
  fontStyle: 'italic',
  padding: '1px 0 1px 9rem',
  borderInlineStart: `2px dashed ${vars.color.border}`,
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
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  },
});

export const scheduleRange = style({ width: '9rem', flexShrink: 0, color: vars.color.textDim, fontSize: '0.85rem', fontVariantNumeric: 'tabular-nums' });

export const scheduleDot = style({ width: '0.6rem', height: '0.6rem', borderRadius: '50%', flexShrink: 0 });

export const scheduleTitle = style({ fontWeight: 500 });

// ── attendee rows (role / cutype pickers + reply status) ────────────────────

export const attendeeList = style({ listStyle: 'none', margin: 0, padding: 0, display: 'flex', flexDirection: 'column', gap: vars.space[1] });

export const attendeeRow = style({ display: 'flex', alignItems: 'center', gap: vars.space[2], flexWrap: 'wrap' });

export const attendeeEmail = style({ flex: 1, minWidth: '8rem', fontSize: '0.85rem' });

export const attendeeStatus = style({
  fontSize: '0.7rem',
  borderRadius: vars.radius.pill,
  padding: '0 6px',
  border: `1px solid ${vars.color.border}`,
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
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  padding: vars.space[2],
  minWidth: 0,
});

export const resolverSideTitle = style({ margin: `0 0 ${vars.space[1]}`, fontSize: '0.95rem', overflowWrap: 'anywhere' });

export const resolverMeta = style({
  display: 'grid',
  gridTemplateColumns: 'auto 1fr',
  gap: '2px 8px',
  margin: 0,
  fontSize: '0.8rem',
});
globalStyle(`${resolverMeta} dt`, { color: vars.color.textDim });
globalStyle(`${resolverMeta} dd`, { margin: 0, overflowWrap: 'anywhere' });

export const resolverActions = style({ display: 'flex', gap: vars.space[2], flexWrap: 'wrap', marginTop: vars.space[2] });

// ── free/busy grid ──────────────────────────────────────────────────────────

export const fbScroll = style({ overflowX: 'auto', border: `1px solid ${vars.color.border}`, borderRadius: vars.radius.md });

export const fbGrid = style({ borderCollapse: 'collapse', fontSize: '0.7rem', width: '100%' });

export const fbCorner = style({ textAlign: 'start', padding: '2px 6px', position: 'sticky', insetInlineStart: 0, background: vars.color.surface, color: vars.color.textDim, fontWeight: 600 });

export const fbHead = style({ padding: '2px 4px', color: vars.color.textDim, fontWeight: 500, textAlign: 'center', fontVariantNumeric: 'tabular-nums' });

export const fbRowHead = style({ textAlign: 'start', padding: '2px 6px', position: 'sticky', insetInlineStart: 0, background: vars.color.surface, whiteSpace: 'nowrap', maxWidth: '10rem', overflow: 'hidden', textOverflow: 'ellipsis' });

const fbCellBase = style({ textAlign: 'center', border: `1px solid ${vars.color.border}`, minWidth: '1.4rem', height: '1.4rem', color: vars.color.text });

export const fbCell = styleVariants({
  free: [fbCellBase, { background: vars.color.bg }],
  busy: [fbCellBase, { background: vars.color.danger, color: '#fff' }],
  tentative: [fbCellBase, { background: vars.color.bgAlt }],
});
