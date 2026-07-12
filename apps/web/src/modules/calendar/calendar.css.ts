// Token-native styling for the calendar module (plan §2.5: themed via the V2
// design tokens). New module → references `vars.*` directly (same posture as the
// V2 Ribbon). Dynamic geometry (event block top/height, grid column counts) is
// applied inline by the views; everything static lives here.

import { style, styleVariants } from '@vanilla-extract/css';
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
  cursor: 'pointer',
  font: 'inherit',
  selectors: {
    '&:hover': { background: vars.color.bgAlt },
    '&:focus-visible': { outline: `2px solid ${vars.color.accent}`, outlineOffset: '1px' },
  },
});

export const primaryButton = style([button, { background: vars.color.accent, color: vars.color.accentText, borderColor: vars.color.accent }]);

export const viewSwitch = style({ display: 'flex', gap: vars.space[1], flexWrap: 'wrap' });

export const viewButton = styleVariants({
  base: [button],
  active: [button, { background: vars.color.accent, color: vars.color.accentText, borderColor: vars.color.accent }],
});

export const body = style({ display: 'flex', flex: 1, minHeight: 0 });

export const sidebar = style({
  width: '15rem',
  flexShrink: 0,
  borderRight: `1px solid ${vars.color.border}`,
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

export const hourCell = style({ height: '3rem', borderBottom: `1px solid ${vars.color.border}`, textAlign: 'right', paddingRight: vars.space[1], boxSizing: 'border-box' });

export const dayColumns = style({ display: 'grid', position: 'relative' });

export const dayColumn = style({ position: 'relative', borderRight: `1px solid ${vars.color.border}` });

export const dayHeader = style({ position: 'sticky', top: 0, textAlign: 'center', padding: vars.space[1], borderBottom: `1px solid ${vars.color.border}`, background: vars.color.bgAlt, fontSize: '0.8rem', zIndex: 2 });

export const hourLine = style({ height: '3rem', borderBottom: `1px solid ${vars.color.border}`, boxSizing: 'border-box' });

export const eventBlock = style({
  position: 'absolute',
  left: '2px',
  right: '2px',
  borderRadius: vars.radius.sm,
  padding: '1px 4px',
  fontSize: '0.72rem',
  color: '#fff',
  overflow: 'hidden',
  cursor: 'pointer',
  boxSizing: 'border-box',
  selectors: { '&:focus-visible': { outline: `2px solid ${vars.color.text}` } },
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

export const weekdayHead = style({ display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)', borderBottom: `1px solid ${vars.color.border}`, background: vars.color.bgAlt });

export const weekdayCell = style({ padding: vars.space[1], textAlign: 'center', fontSize: '0.75rem', color: vars.color.textDim });

export const monthCell = style({ border: `1px solid ${vars.color.border}`, padding: '2px', minHeight: '4.5rem', overflow: 'hidden', display: 'flex', flexDirection: 'column', gap: '1px' });

export const monthCellOut = style([monthCell, { background: vars.color.bgSink, color: vars.color.textDim }]);

export const dayNum = style({ fontSize: '0.75rem', alignSelf: 'flex-end' });

export const dayNumToday = style([dayNum, { background: vars.color.accent, color: vars.color.accentText, borderRadius: vars.radius.pill, padding: '0 6px' }]);

export const monthEvent = style({ fontSize: '0.7rem', borderRadius: vars.radius.sm, padding: '0 3px', color: '#fff', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', cursor: 'pointer' });

export const triMonth = style({ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: vars.space[3], padding: vars.space[3] });

// ── agenda / schedule ────────────────────────────────────────────────────────

export const agenda = style({ display: 'flex', flexDirection: 'column', padding: vars.space[3], gap: vars.space[2] });

export const agendaDay = style({ display: 'flex', flexDirection: 'column', gap: '2px' });

export const agendaDate = style({ fontWeight: 600, borderBottom: `1px solid ${vars.color.border}`, paddingBottom: '2px', marginBottom: '2px' });

export const agendaRow = style({ display: 'flex', gap: vars.space[3], alignItems: 'center', padding: '2px 0', cursor: 'pointer' });

export const agendaTime = style({ width: '9rem', flexShrink: 0, color: vars.color.textDim, fontSize: '0.85rem' });

export const agendaDot = style({ width: '0.6rem', height: '0.6rem', borderRadius: '50%', flexShrink: 0 });

// ── year ─────────────────────────────────────────────────────────────────────

export const yearGrid = style({ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: vars.space[3], padding: vars.space[3] });

export const miniMonth = style({ border: `1px solid ${vars.color.border}`, borderRadius: vars.radius.md, padding: vars.space[2] });

export const miniTitle = style({ textAlign: 'center', fontSize: '0.85rem', fontWeight: 600, marginBottom: '2px' });

export const miniGrid = style({ display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)', gap: '1px' });

export const miniDay = style({ fontSize: '0.65rem', textAlign: 'center', padding: '1px', cursor: 'pointer', borderRadius: vars.radius.sm });

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
  marginLeft: '2px',
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
