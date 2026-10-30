// Statistics layout (t106): figures, charts and tables in separate sections.
//
// A row of stat tiles, then chart cards, then table cards. Spacing is on the
// token scale; grids reflow to one column at 390 px with no horizontal scroll.
// Numbers in columns (axes, summaries, tables) use tabular figures. Same t32
// rules as the rest of the console: flat, square, token colours, logical sides.

import { globalStyle, keyframes, style, styleVariants } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';
import { series } from '../../../theme/dataviz.css.ts';

const PHONE = 'screen and (max-width: 760px)';
const noTrackingScripts = ':lang(ar) &, :lang(ja) &, :lang(zh) &';

/** The whole statistics view: sections stacked with one gap. */
export const view = style({ display: 'flex', flexDirection: 'column', gap: vars.space[6], minInlineSize: 0 });

/** A section of one kind (figures, charts or tables) with its small heading. */
export const section = style({ display: 'flex', flexDirection: 'column', gap: vars.space[4], minInlineSize: 0, margin: 0 });
export const sectionTitle = style({
  margin: 0,
  fontSize: '0.75rem',
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
  fontWeight: vars.weight.bold,
  color: vars.color.textDim,
  selectors: { [noTrackingScripts]: { textTransform: 'none', letterSpacing: vars.tracking.none } },
});
/** A labelled group of tiles inside the figures section. */
export const group = style({ display: 'flex', flexDirection: 'column', gap: vars.space[3], minInlineSize: 0 });
export const groupTitle = style({ margin: 0, fontSize: '0.85rem', fontWeight: vars.weight.bold });

/** Compact tiles: as many columns as fit, two at phone width. */
export const tiles = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fill, minmax(min(100%, 11rem), 1fr))',
  gap: vars.space[3],
  '@media': { [PHONE]: { gridTemplateColumns: 'repeat(2, minmax(0, 1fr))' } },
});
/** Range, coverage and precision notes: one tight block, not a paragraph each. */
export const notes = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2], minInlineSize: 0 });
globalStyle(`${notes} > p`, { margin: 0 });

/** Chart cards: two columns on wide screens, one on narrow ones. */
export const charts = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fill, minmax(min(100%, 26rem), 1fr))',
  gap: vars.space[5],
});
/** Table cards: one per row; tables are wide. */
export const tables = style({ display: 'grid', gridTemplateColumns: 'minmax(0, 1fr)', gap: vars.space[5] });

/** Refetch keeps the frame: the last render stays, dimmed, while the next loads. */
export const stale = style({ opacity: 0.6, transition: 'opacity 120ms ease-out' });

// ── Cards ─────────────────────────────────────────────────────────────────────
export const card = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  minInlineSize: 0,
  margin: 0,
  padding: `calc(${vars.space[5]} - ${vars.border.rule})`,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  '@media': { [PHONE]: { padding: `calc(${vars.space[4]} - ${vars.border.rule})` } },
});
export const cardHead = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'flex-start', justifyContent: 'space-between', gap: vars.space[3] });
export const cardHeading = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1], minInlineSize: 0 });
export const cardTitle = style({ margin: 0, fontSize: '0.95rem', fontWeight: vars.weight.bold });
export const cardUnit = style({ margin: 0, fontSize: '0.75rem', color: vars.color.textDim });
export const cardSummary = style({ margin: 0, fontSize: '0.8rem', color: vars.color.textDim, fontVariantNumeric: 'tabular-nums' });
export const cardEmpty = style({ margin: 0, paddingBlock: vars.space[5], fontSize: '0.85rem', color: vars.color.textDim, textAlign: 'center' });
export const toggle = style({ flex: 'none', fontSize: '0.8rem' });

export const legend = style({ display: 'flex', flexWrap: 'wrap', gap: `${vars.space[2]} ${vars.space[4]}`, margin: 0, padding: 0, listStyle: 'none', fontSize: '0.8rem' });
export const legendItem = style({ display: 'inline-flex', alignItems: 'center', gap: vars.space[2], color: vars.color.text });
/** A short line key in the series colour (legends mirror the mark). */
export const key = style({ display: 'inline-block', inlineSize: '14px', blockSize: '2px', flex: 'none' });
export const keyColour = styleVariants({ 1: { background: series[1] }, 2: { background: series[2] } });

// ── Line chart ────────────────────────────────────────────────────────────────
export const plot = style({
  display: 'grid',
  gridTemplateColumns: 'auto minmax(0, 1fr)',
  gridTemplateRows: 'auto auto',
  columnGap: vars.space[3],
  rowGap: vars.space[2],
  minInlineSize: 0,
});
export const yAxis = style({
  display: 'flex',
  flexDirection: 'column',
  justifyContent: 'space-between',
  alignItems: 'flex-end',
  fontSize: '0.7rem',
  color: vars.color.textDim,
  fontVariantNumeric: 'tabular-nums',
  minInlineSize: '2.5rem',
});
export const svgWrap = style({ position: 'relative', minInlineSize: 0 });
export const svg = style({ display: 'block', inlineSize: '100%', blockSize: '140px', overflow: 'visible', touchAction: 'pan-y' });
export const grid = style({ stroke: vars.color.border, strokeWidth: 1, vectorEffect: 'non-scaling-stroke' });
export const line = style({ fill: 'none', strokeWidth: 2, strokeLinejoin: 'round', strokeLinecap: 'round', vectorEffect: 'non-scaling-stroke' });
export const lineColour = styleVariants({ 1: { stroke: series[1] }, 2: { stroke: series[2] } });
export const crosshair = style({ stroke: vars.color.textDim, strokeWidth: 1, vectorEffect: 'non-scaling-stroke' });
export const hit = style({ fill: 'transparent' });
export const xAxis = style({
  gridColumn: '2',
  display: 'flex',
  justifyContent: 'space-between',
  gap: vars.space[3],
  fontSize: '0.7rem',
  color: vars.color.textDim,
  fontVariantNumeric: 'tabular-nums',
});
export const tip = style({ gridColumn: '2', display: 'flex', flexWrap: 'wrap', gap: `${vars.space[1]} ${vars.space[4]}`, minBlockSize: '1.3em', margin: 0, fontSize: '0.75rem', fontVariantNumeric: 'tabular-nums' });
export const tipTime = style({ color: vars.color.textDim });
export const tipValue = style({ display: 'inline-flex', alignItems: 'center', gap: vars.space[2] });
globalStyle(`${tipValue} strong`, { fontWeight: vars.weight.bold });

// ── Bars (one measure, one colour) ────────────────────────────────────────────
export const barFill = style({ blockSize: '100%', background: series[1] });

// ── Tables inside cards ───────────────────────────────────────────────────────
export const dataTable = style({ inlineSize: '100%', borderCollapse: 'collapse', fontSize: '0.8rem' });
globalStyle(`${dataTable} th, ${dataTable} td`, {
  paddingBlock: vars.space[2],
  paddingInline: vars.space[3],
  borderBlockEnd: `${vars.border.hair} solid ${vars.color.border}`,
  textAlign: 'start',
});
globalStyle(`${dataTable} th`, { color: vars.color.textDim, fontWeight: vars.weight.bold });
export const num = style({ textAlign: 'end', fontVariantNumeric: 'tabular-nums', whiteSpace: 'nowrap' });
globalStyle(`${dataTable} th${num}, ${dataTable} td${num}`, { textAlign: 'end' });
export const tableScroll = style({ maxBlockSize: '18rem', overflow: 'auto', minInlineSize: 0 });

// ── Tile extras: delta and trend ──────────────────────────────────────────────
export const delta = style({ display: 'inline-flex', alignItems: 'center', gap: vars.space[1], margin: 0, fontSize: '0.75rem', fontVariantNumeric: 'tabular-nums' });
export const deltaTone = styleVariants({
  good: { color: vars.color.success },
  bad: { color: vars.color.danger },
  flat: { color: vars.color.textDim },
});
export const deltaNote = style({ color: vars.color.textDim });
export const trend = style({ display: 'block', inlineSize: '100%', blockSize: '24px', overflow: 'visible' });
export const trendLine = style({ fill: 'none', stroke: vars.color.textDim, strokeWidth: 1.5, strokeLinejoin: 'round', vectorEffect: 'non-scaling-stroke' });
export const trendEnd = style({ fill: series[1] });

// ── Skeletons sized like the cards they stand in for ─────────────────────────
const pulse = keyframes({ '0%': { opacity: 1 }, '50%': { opacity: 0.55 }, '100%': { opacity: 1 } });
const skeletonBlock = {
  display: 'block',
  background: vars.color.bgAlt,
  borderRadius: vars.radius.sm,
  animation: `${pulse} 1.6s ease-in-out infinite`,
  selectors: { ':root[data-reduced-motion] &': { animation: 'none' } },
  '@media': { '(prefers-reduced-motion: reduce)': { animation: 'none' } },
} as const;
export const skeletonLine = style({ ...skeletonBlock, blockSize: '0.75rem', inlineSize: '60%' });
export const skeletonValue = style({ ...skeletonBlock, blockSize: '1.75rem', inlineSize: '45%' });
export const skeletonPlot = style({ ...skeletonBlock, blockSize: '140px', inlineSize: '100%' });
export const skeletonRow = style({ ...skeletonBlock, blockSize: '1.6rem', inlineSize: '100%' });
/** Tile skeletons take the same box as a tile with a value and a delta line. */
export const skeletonTile = style({ minBlockSize: '6.5rem' });
