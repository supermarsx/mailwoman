// Styles of the shared admin pieces added by f33 (list editor, status badge,
// key/value summary, file drop). Same rules as ux.css.ts: tokens only, logical
// properties only, every interactive class with its own :focus-visible, state
// never by colour alone (each badge has an icon and text).

import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

const FORCED = '(forced-colors: active)';
const focusRing = { outline: 'none', boxShadow: vars.a11y.focusRing } as const;

// ── List editor ───────────────────────────────────────────────────────────────
export const listEditor = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2], minWidth: 0 });
export const listItems = style({ listStyle: 'none', margin: 0, padding: 0, display: 'flex', flexDirection: 'column', gap: vars.space[1] });
export const listItem = style({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  gap: vars.space[2],
  minWidth: 0,
  paddingBlock: 0,
  paddingInlineStart: vars.space[2],
  paddingInlineEnd: 0,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  borderRadius: vars.radius.sm,
  background: vars.color.bg,
});
export const listText = style({ minWidth: 0, overflowWrap: 'anywhere', fontSize: '0.86rem' });
export const listMono = style({ fontFamily: vars.font.mono, fontSize: '0.8rem' });
export const listAdd = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[2], alignItems: 'center', minWidth: 0 });
export const listInput = style({ flex: '1 1 12rem', minWidth: 0, maxWidth: '100%' });
export const listEmpty = style({ margin: 0, fontSize: '0.8rem', color: vars.color.textDim });

// ── Status badge ──────────────────────────────────────────────────────────────
export const statusBadge = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  fontSize: '0.72rem',
  fontWeight: vars.weight.bold,
  padding: `0 ${vars.space[2]}`,
  minHeight: '20px',
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  color: vars.color.text,
  background: vars.color.bg,
  whiteSpace: 'nowrap',
  selectors: {
    '&[data-tone="ok"]': { boxShadow: `inset 3px 0 0 ${vars.color.success}` },
    '&[data-tone="warn"]': { boxShadow: `inset 3px 0 0 ${vars.color.warning}` },
    '&[data-tone="danger"]': { boxShadow: `inset 3px 0 0 ${vars.color.danger}` },
    '[dir="rtl"] &[data-tone="ok"]': { boxShadow: `inset -3px 0 0 ${vars.color.success}` },
    '[dir="rtl"] &[data-tone="warn"]': { boxShadow: `inset -3px 0 0 ${vars.color.warning}` },
    '[dir="rtl"] &[data-tone="danger"]': { boxShadow: `inset -3px 0 0 ${vars.color.danger}` },
    '&[data-tone="off"]': { color: vars.color.textDim, borderStyle: 'dashed' },
  },
  '@media': { [FORCED]: { border: `${vars.border.hair} solid CanvasText` } },
});
globalStyle(`${statusBadge} svg`, { flex: 'none', width: '12px', height: '12px' });
export const badgeRow = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[1], alignItems: 'center' });

// ── Key / value summary (a preview, a detail header) ─────────────────────────
export const summary = style({
  display: 'grid',
  gridTemplateColumns: 'minmax(0, 12rem) minmax(0, 1fr)',
  gap: `${vars.space[1]} ${vars.space[3]}`,
  margin: 0,
  fontSize: '0.86rem',
  '@media': { 'screen and (max-width: 760px)': { gridTemplateColumns: 'minmax(0, 1fr)' } },
});
export const summaryTerm = style({ color: vars.color.textDim, fontWeight: vars.weight.bold });
export const summaryValue = style({ margin: 0, minWidth: 0, overflowWrap: 'anywhere' });
export const mono = style({ fontFamily: vars.font.mono, fontSize: '0.8rem', overflowWrap: 'anywhere' });

/** A block that groups a sub-task inside a page (a card with a heading). */
export const panel = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  minWidth: 0,
  padding: `calc(${vars.space[4]} - 1px)`,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
});
export const panelTitle = style({ margin: 0, fontSize: '1rem', fontWeight: vars.weight.heavy });
export const panelText = style({ margin: 0, fontSize: '0.86rem', color: vars.color.textDim });
export const actions = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[2], alignItems: 'center' });

/** Verbatim component output (a classifier test answer). */
export const pre = style({
  margin: 0,
  padding: vars.space[2],
  fontFamily: vars.font.mono,
  fontSize: '0.76rem',
  whiteSpace: 'pre-wrap',
  overflowWrap: 'anywhere',
  borderRadius: vars.radius.sm,
  background: vars.color.bgSink,
  maxBlockSize: '20rem',
  overflow: 'auto',
});

/** A file picker shown as a labelled control (the native input stays focusable). */
export const fileInput = style({
  maxWidth: '100%',
  font: 'inherit',
  fontSize: '0.86rem',
  selectors: { '&:focus-visible': focusRing },
});

/** A checkbox list (capability grants). */
export const checks = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1], margin: 0, padding: 0, border: 0, minWidth: 0 });
export const checkRow = style({ display: 'flex', alignItems: 'center', gap: vars.space[2], minHeight: '24px', fontSize: '0.86rem' });
globalStyle(`${checkRow} input`, { accentColor: vars.color.accent, flex: 'none' });
