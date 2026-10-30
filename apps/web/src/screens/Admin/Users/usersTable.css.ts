// The admin Users table (t107). Admin rules (adminStyle.rules.test.ts): tokens
// only, logical properties, every interactive class has its own :focus-visible,
// state is never shown by colour alone (each chip carries its word).

import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

const ring = { outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`, outlineOffset: '2px' } as const;
const ringInset = { ...ring, outlineOffset: `calc(-1 * ${vars.a11y.focusRingWidth})` } as const;
const smallLabel = { textTransform: 'uppercase', letterSpacing: vars.tracking.label, fontWeight: vars.weight.bold } as const;

export const root = style({ display: 'flex', flexDirection: 'column', gap: vars.space[3], minInlineSize: 0, inlineSize: '100%' });

export const toolbar = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'flex-end', gap: vars.space[3], minInlineSize: 0 });
export const search = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1], flex: '1 1 18rem', minInlineSize: 0, maxInlineSize: '28rem', margin: 0 });
export const searchInput = style({ inlineSize: '100%', maxInlineSize: 'none' });
export const toolbarEnd = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: vars.space[2], marginInlineStart: 'auto' });
export const count = style({ fontSize: '0.85rem', color: vars.color.textDim, margin: 0 });

// ── Filter chips ──────────────────────────────────────────────────────────────
export const filterBar = style({ position: 'relative', display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: vars.space[2], minInlineSize: 0 });
export const chip = style({
  display: 'inline-flex',
  alignItems: 'stretch',
  maxInlineSize: '100%',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.pill,
  background: vars.color.surface,
  overflow: 'hidden',
  fontSize: '0.8rem',
});
const chipPart = {
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  minBlockSize: vars.a11y.touchTarget,
  border: 0,
  background: 'transparent',
  color: 'inherit',
  font: 'inherit',
  cursor: 'pointer',
} as const;
export const chipLabel = style({
  ...chipPart,
  minInlineSize: 0,
  paddingBlock: 0,
  paddingInline: `${vars.space[3]} ${vars.space[1]}`,
  selectors: { '&:hover': { background: vars.color.selection }, '&:focus-visible': ringInset },
});
export const chipText = style({ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' });
export const chipField = style({ color: vars.color.textDim });
export const chipRemove = style({
  ...chipPart,
  justifyContent: 'center',
  minInlineSize: vars.a11y.touchTarget,
  paddingInline: vars.space[1],
  color: vars.color.textDim,
  selectors: { '&:hover': { background: vars.color.selection, color: vars.color.text }, '&:focus-visible': ringInset },
});
globalStyle(`${chipRemove} svg, ${chipLabel} svg`, { inlineSize: '0.85rem', blockSize: '0.85rem' });
export const addFilter = style({
  ...chipPart,
  paddingInline: vars.space[3],
  border: `${vars.border.rule} dashed ${vars.color.borderStrong}`,
  borderRadius: vars.radius.pill,
  color: vars.color.textDim,
  fontSize: '0.8rem',
  selectors: { '&:hover': { color: vars.color.text, background: vars.color.selection }, '&:focus-visible': ring },
});
export const textButton = style({
  ...chipPart,
  paddingInline: vars.space[2],
  color: vars.color.link,
  fontSize: '0.8rem',
  textDecoration: 'underline',
  selectors: { '&:hover': { color: vars.color.text }, '&:focus-visible': ring },
});

/** A small panel under the toolbar (chip editor, column list): placed by CSS, no script. */
export const popover = style({
  position: 'absolute',
  insetBlockStart: 'calc(100% + 4px)',
  insetInlineStart: 0,
  zIndex: 30,
  inlineSize: 'min(22rem, calc(100vw - 32px))',
  maxBlockSize: '70vh',
  overflowY: 'auto',
  padding: vars.space[4],
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  color: vars.color.text,
  boxShadow: vars.elevation[2],
});
export const popoverEnd = style({ insetInlineStart: 'auto', insetInlineEnd: 0 });
export const anchor = style({ position: 'relative' });
export const popoverForm = style({ display: 'grid', gap: vars.space[3], margin: 0 });
export const popoverTitle = style({ margin: 0, fontSize: '0.9rem', fontWeight: vars.weight.bold });
export const options = style({ display: 'grid', gap: vars.space[1], margin: 0, padding: 0, border: 0 });
export const option = style({ display: 'flex', alignItems: 'center', gap: vars.space[2], fontSize: '0.85rem', minBlockSize: vars.a11y.touchTarget });
export const popoverActions = style({ display: 'flex', justifyContent: 'flex-end', flexWrap: 'wrap', gap: vars.space[2] });
export const popoverRemove = style({ marginInlineEnd: 'auto' });
export const inlineNumber = style({ inlineSize: '5rem' });

// ── Bulk bar ──────────────────────────────────────────────────────────────────
export const bulk = style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'center',
  gap: vars.space[2],
  padding: `${vars.space[2]} ${vars.space[3]}`,
  border: `${vars.border.rule} solid ${vars.color.accent}`,
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
});
export const bulkCount = style({ fontWeight: vars.weight.bold, marginInlineEnd: vars.space[2] });

// ── Table ─────────────────────────────────────────────────────────────────────
/** The scroll box: the header sticks to its top while the rows scroll under it. */
export const scroller = style({
  position: 'relative',
  overflow: 'auto',
  maxBlockSize: 'max(24rem, calc(100vh - 18rem))',
  inlineSize: '100%',
  border: `${vars.border.hair} solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
});
export const table = style({ inlineSize: '100%', borderCollapse: 'separate', borderSpacing: 0, fontSize: '0.85rem' });
globalStyle(`${table} th, ${table} td`, {
  textAlign: 'start',
  paddingBlock: vars.space[3],
  paddingInline: vars.space[3],
  borderBlockEnd: `${vars.border.hair} solid ${vars.color.border}`,
  verticalAlign: 'middle',
});
globalStyle(`${table} thead th`, {
  ...smallLabel,
  position: 'sticky',
  insetBlockStart: 0,
  zIndex: 2,
  background: vars.color.surface,
  color: vars.color.textDim,
  fontSize: '0.72rem',
  whiteSpace: 'nowrap',
  borderBlockEnd: `${vars.border.rule} solid ${vars.color.borderStrong}`,
});
globalStyle(`:lang(ar) ${table} thead th, :lang(ja) ${table} thead th, :lang(zh) ${table} thead th`, { textTransform: 'none', letterSpacing: vars.tracking.none });
export const compact = style({});
globalStyle(`${compact} th, ${compact} td`, { paddingBlock: vars.space[1], paddingInline: vars.space[2] });
export const row = style({
  selectors: {
    '&:hover': { background: vars.color.bgAlt },
    '&:focus-visible': ringInset,
    '&[data-selected="true"]': { background: vars.color.selection },
  },
});
export const selectCell = style({ inlineSize: '1%', whiteSpace: 'nowrap' });
/** The actions stay in view at the row's end while wide tables scroll sideways. */
export const actionsCell = style({ inlineSize: '1%', whiteSpace: 'nowrap', textAlign: 'end', position: 'sticky', insetInlineEnd: 0, zIndex: 1, background: vars.color.surface });
globalStyle(`tr > th${actionsCell}, tr > td${actionsCell}`, { textAlign: 'end' });
globalStyle(`thead th${actionsCell}`, { zIndex: 3 });
export const actions = style({ display: 'inline-flex', alignItems: 'center', justifyContent: 'flex-end', gap: vars.space[1] });
export const sort = style({
  appearance: 'none',
  font: 'inherit',
  color: 'inherit',
  letterSpacing: 'inherit',
  textTransform: 'inherit',
  fontWeight: 'inherit',
  background: 'transparent',
  border: 0,
  padding: 0,
  cursor: 'pointer',
  display: 'inline-flex',
  gap: vars.space[1],
  alignItems: 'center',
  selectors: { '&:hover': { color: vars.color.text }, '&:focus-visible': ring },
});
globalStyle(`${sort} svg`, { inlineSize: '0.85rem', blockSize: '0.85rem' });

export const person = style({ display: 'flex', alignItems: 'center', gap: vars.space[3], minInlineSize: '16rem' });
export const avatar = style({
  flex: 'none',
  display: 'grid',
  placeItems: 'center',
  inlineSize: '2rem',
  blockSize: '2rem',
  borderRadius: vars.radius.pill,
  background: vars.color.accent,
  color: vars.color.accentText,
  fontSize: '0.75rem',
  fontWeight: vars.weight.bold,
});
globalStyle(`${compact} ${avatar}`, { inlineSize: '1.5rem', blockSize: '1.5rem', fontSize: '0.65rem' });
export const personText = style({ display: 'flex', flexDirection: 'column', minInlineSize: 0 });
export const personName = style({ fontWeight: vars.weight.bold, color: vars.color.link, textDecoration: 'none', overflowWrap: 'anywhere', selectors: { '&:hover': { textDecoration: 'underline' }, '&:focus-visible': ring } });
export const personAddress = style({ color: vars.color.textDim, fontSize: '0.8rem', overflowWrap: 'anywhere' });
export const mark = style({ background: vars.color.signal, color: vars.color.signalText, padding: 0, borderRadius: vars.radius.sm });
export const dim = style({ color: vars.color.textDim });
export const nowrap = style({ whiteSpace: 'nowrap' });

export const status = style({
  ...smallLabel,
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  paddingBlock: '1px',
  paddingInline: vars.space[2],
  borderRadius: vars.radius.sm,
  fontSize: '0.68rem',
  whiteSpace: 'nowrap',
  border: `${vars.border.hair} solid ${vars.color.border}`,
  background: vars.color.bgAlt,
  color: vars.color.text,
  selectors: {
    '&[data-state="active"]': { borderColor: vars.color.success, color: vars.color.success },
    '&[data-state="blocked"], &[data-state="disabled"]': { borderColor: vars.color.danger, color: vars.color.danger },
    '&[data-state="pending"]': { borderColor: vars.color.warning, color: vars.color.text },
    '&[data-state="deleted"]': { color: vars.color.textDim, borderStyle: 'dashed' },
  },
});
globalStyle(`${status} svg`, { inlineSize: '0.8rem', blockSize: '0.8rem' });
export const badge = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  paddingBlock: '1px',
  paddingInline: vars.space[2],
  borderRadius: vars.radius.sm,
  fontSize: '0.75rem',
  whiteSpace: 'nowrap',
  border: `${vars.border.hair} solid ${vars.color.border}`,
  background: vars.color.bg,
});
globalStyle(`${badge} svg`, { inlineSize: '0.8rem', blockSize: '0.8rem' });

// Skeleton rows: the table keeps its frame while the first page loads (t75).
export const skeletonCell = style({ display: 'block', blockSize: '0.75rem', background: vars.color.border, borderRadius: vars.radius.sm });
export const skeletonShort = style({ inlineSize: '40%' });
export const skeletonMid = style({ inlineSize: '65%' });
export const skeletonLong = style({ inlineSize: '90%' });

export const empty = style({ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', gap: vars.space[3], padding: vars.space[5] });

export const pager = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'center', justifyContent: 'space-between', gap: vars.space[3] });
export const pagerButtons = style({ display: 'flex', alignItems: 'center', gap: vars.space[2] });
export const pageSize = style({ display: 'inline-flex', alignItems: 'center', gap: vars.space[2], fontSize: '0.85rem' });

// ── Phone: stacked cards ──────────────────────────────────────────────────────
export const cards = style({ listStyle: 'none', margin: 0, padding: 0, display: 'grid', gap: vars.space[3], minInlineSize: 0 });
export const card = style({
  display: 'grid',
  gridTemplateColumns: 'auto minmax(0, 1fr) auto',
  alignItems: 'center',
  gap: vars.space[3],
  padding: vars.space[3],
  border: `${vars.border.hair} solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  minInlineSize: 0,
  selectors: { '&:focus-within': { borderColor: vars.color.borderStrong }, '&[data-selected="true"]': { background: vars.color.selection } },
});
export const cardBody = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1], minInlineSize: 0 });
export const cardMeta = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: vars.space[2], fontSize: '0.8rem' });
export const cardActions = style({ gridColumn: '1 / -1', display: 'flex', justifyContent: 'flex-end', gap: vars.space[1] });

// In a card the person shares a narrow row with the checkbox and actions: no minimum there.
globalStyle(`${cardBody} ${person}`, { minInlineSize: 0 });
