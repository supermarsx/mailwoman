import { globalStyle, style, styleVariants } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

/** The Outlook-style event card at the top of a message (t53). */
export const card = style({
  margin: vars.space[3],
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  overflow: 'hidden',
  overflowWrap: 'anywhere',
  boxShadow: vars.elevation[1],
});

export const head = style({
  display: 'flex',
  alignItems: 'flex-start',
  gap: vars.space[3],
  padding: `${vars.space[3]} ${vars.space[3]} ${vars.space[2]}`,
  borderInlineStart: `4px solid ${vars.color.accent}`,
  background: vars.color.bgAlt,
});
export const headCancelled = style({ borderInlineStartColor: vars.color.danger });

export const dateTile = style({
  flex: '0 0 auto',
  inlineSize: '3.25rem',
  textAlign: 'center',
  borderRadius: vars.radius.sm,
  border: `${vars.border.rule} solid ${vars.color.border}`,
  background: vars.color.surface,
  overflow: 'hidden',
  lineHeight: 1.1,
});
export const dateTileMonth = style({
  display: 'block',
  padding: `2px 0`,
  background: vars.color.accent,
  color: vars.color.accentText,
  fontSize: '0.7rem',
  fontWeight: vars.weight.bold,
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
});
export const dateTileDay = style({ display: 'block', padding: `2px 0 4px`, fontSize: '1.35rem', fontWeight: vars.weight.heavy });

export const headText = style({ flex: '1 1 auto', minInlineSize: 0 });
export const kicker = style({
  margin: 0,
  color: vars.color.textDim,
  fontSize: '0.75rem',
  fontWeight: vars.weight.bold,
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
});
export const title = style({ margin: `2px 0 0`, fontSize: '1.1rem', fontWeight: vars.weight.heavy, lineHeight: 1.3 });
export const titleCancelled = style({ textDecoration: 'line-through', color: vars.color.textDim });

export const badge = style({
  flex: '0 0 auto',
  alignSelf: 'center',
  padding: `2px ${vars.space[2]}`,
  borderRadius: vars.radius.pill,
  fontSize: '0.75rem',
  fontWeight: vars.weight.bold,
  border: `${vars.border.rule} solid currentColor`,
  whiteSpace: 'nowrap',
});
export const badgeTone = styleVariants({
  accepted: { color: vars.color.success },
  tentative: { color: vars.color.warning },
  declined: { color: vars.color.danger },
  info: { color: vars.color.textDim },
});

export const body = style({
  display: 'grid',
  gridTemplateColumns: 'minmax(0, 1fr)',
  gap: vars.space[3],
  padding: vars.space[3],
  '@media': { '(min-width: 900px)': { gridTemplateColumns: 'minmax(0, 1fr) minmax(16rem, 22rem)' } },
});

export const rows = style({ display: 'grid', gap: vars.space[2], margin: 0 });
export const row = style({ display: 'grid', gridTemplateColumns: '1.25rem minmax(0, 1fr)', gap: vars.space[2], alignItems: 'start' });
export const rowIcon = style({ color: vars.color.textDim, paddingBlockStart: '2px' });
export const rowMain = style({ margin: 0, minInlineSize: 0 });
export const dim = style({ color: vars.color.textDim, fontSize: '0.875rem', margin: `2px 0 0` });
globalStyle(`${row} dt`, { position: 'absolute', inlineSize: 1, blockSize: 1, overflow: 'hidden', clip: 'rect(0 0 0 0)' });
globalStyle(`${row} dd`, { margin: 0 });

export const join = style({ display: 'inline-flex', alignItems: 'center', gap: vars.space[1], marginBlockStart: vars.space[1] });

export const people = style({ listStyle: 'none', margin: `${vars.space[1]} 0 0`, padding: 0, display: 'grid', gap: '2px' });
export const person = style({ display: 'flex', alignItems: 'center', gap: vars.space[2], minInlineSize: 0 });
export const personName = style({ minInlineSize: 0, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' });
export const rsvp = style({ flex: '0 0 auto', inlineSize: '0.6rem', blockSize: '0.6rem', borderRadius: '50%', border: `${vars.border.rule} solid ${vars.color.textDim}` });
export const rsvpTone = styleVariants({
  accepted: { background: vars.color.success, borderColor: vars.color.success },
  tentative: { background: `repeating-linear-gradient(45deg, ${vars.color.warning} 0 2px, transparent 2px 4px)`, borderColor: vars.color.warning },
  declined: { background: vars.color.danger, borderColor: vars.color.danger },
  'needs-action': {},
});
export const linkButton = style({
  background: 'none',
  border: 'none',
  padding: 0,
  color: vars.color.link,
  cursor: 'pointer',
  font: 'inherit',
  textDecoration: 'underline',
});

export const description = style({
  margin: 0,
  whiteSpace: 'pre-wrap',
  maxBlockSize: '12rem',
  overflow: 'auto',
  padding: vars.space[2],
  borderRadius: vars.radius.sm,
  background: vars.color.bgAlt,
  fontSize: '0.875rem',
});

export const notice = style({
  margin: `0 ${vars.space[3]}`,
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderRadius: vars.radius.sm,
  background: vars.color.bgAlt,
  borderInlineStart: `3px solid ${vars.color.textDim}`,
});
export const noticeTone = styleVariants({
  warn: { borderInlineStartColor: vars.color.warning },
  danger: { borderInlineStartColor: vars.color.danger },
  ok: { borderInlineStartColor: vars.color.success },
  info: {},
});

export const actions = style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'center',
  gap: vars.space[2],
  padding: vars.space[3],
  borderBlockStart: `${vars.border.rule} solid ${vars.color.border}`,
});
export const actionsGroup = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[2] });
export const options = style({ display: 'grid', gap: vars.space[2], padding: `0 ${vars.space[3]} ${vars.space[3]}` });
export const optionRow = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: vars.space[2] });
export const comment = style({ inlineSize: '100%', minBlockSize: '3rem', resize: 'vertical', font: 'inherit' });

export const itemList = style({ listStyle: 'none', margin: 0, padding: 0, display: 'grid', gap: vars.space[2] });
export const item = style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'center',
  justifyContent: 'space-between',
  gap: vars.space[2],
  padding: vars.space[2],
  border: `${vars.border.rule} solid ${vars.color.border}`,
  borderRadius: vars.radius.sm,
});

// ── the mini day agenda ──────────────────────────────────────────────────────

export const agenda = style({
  border: `${vars.border.rule} solid ${vars.color.border}`,
  borderRadius: vars.radius.sm,
  padding: vars.space[2],
  background: vars.color.bg,
  minInlineSize: 0,
});
export const agendaHead = style({ display: 'flex', justifyContent: 'space-between', gap: vars.space[2], margin: `0 0 ${vars.space[2]}`, fontSize: '0.875rem', fontWeight: vars.weight.bold });
export const agendaGrid = style({ position: 'relative', borderInlineStart: `${vars.border.rule} solid ${vars.color.border}` });
export const agendaHour = style({
  position: 'relative',
  blockSize: '1.75rem',
  borderBlockStart: `${vars.border.rule} solid ${vars.color.border}`,
  fontSize: '0.7rem',
  color: vars.color.textDim,
  paddingInlineStart: vars.space[1],
});
export const agendaBlock = style({
  position: 'absolute',
  boxSizing: 'border-box',
  padding: `1px ${vars.space[1]}`,
  borderRadius: vars.radius.sm,
  fontSize: '0.75rem',
  lineHeight: 1.2,
  overflow: 'hidden',
  whiteSpace: 'nowrap',
  textOverflow: 'ellipsis',
  border: `${vars.border.rule} solid rgba(0,0,0,0.15)`,
});
export const agendaInvite = style({
  background: `repeating-linear-gradient(135deg, color-mix(in srgb, ${vars.color.accent} 22%, transparent) 0 6px, color-mix(in srgb, ${vars.color.accent} 10%, transparent) 6px 12px)`,
  border: `2px dashed ${vars.color.accent}`,
  color: vars.color.text,
  fontWeight: vars.weight.bold,
});
export const agendaConflict = style({ outline: `2px solid ${vars.color.danger}`, outlineOffset: '-2px' });
export const agendaAllDay = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[1], marginBlockEnd: vars.space[1] });
export const agendaChip = style({ padding: `1px ${vars.space[1]}`, borderRadius: vars.radius.sm, fontSize: '0.75rem' });
export const agendaSummary = style({ margin: `${vars.space[2]} 0 0`, fontSize: '0.8rem' });
export const agendaSummaryConflict = style({ color: vars.color.danger, fontWeight: vars.weight.bold });
