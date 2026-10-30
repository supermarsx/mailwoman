// t105: Diagnostics cards, tiles and rows. Token-native like admin.css.ts:
// square corners, rule borders, state in text and icon (colour only adds an
// edge), logical properties so the layout mirrors under dir=rtl, and nothing
// wider than the pane at 390 px.

import { globalStyle, keyframes, style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';
import { subsectionSurface } from '../subsectionLayout.css.ts';

/** Keyboard focus, as the shell's ux.css.ts draws it. */
const focusRing = { outline: 'none', boxShadow: vars.a11y.focusRing } as const;

/** The section: full pane width (admin.css section caps text forms at 760 px). */
export const section = style({ display: 'flex', flexDirection: 'column', gap: vars.space[6], minInlineSize: 0 });
globalStyle(`${section} > h2`, { margin: 0 });
export const page = style({ display: 'flex', flexDirection: 'column', gap: vars.space[6], minInlineSize: 0 });
export const intro = style({ margin: 0, color: vars.color.textDim, maxInlineSize: '72ch', lineHeight: 1.5 });
export const toolbar = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: vars.space[3], minInlineSize: 0 });
export const checkTableScroll=style({maxInlineSize:'100%',minInlineSize:0,overflowX:'auto',border:`1px solid ${vars.color.border}`,selectors:{'&:focus-visible':focusRing}});
export const checkTable=style({inlineSize:'100%',minInlineSize:'56rem',borderCollapse:'collapse',tableLayout:'fixed'});
globalStyle(`${checkTable} caption`,{textAlign:'start',padding:vars.space[3],fontWeight:vars.weight.bold});
globalStyle(`${checkTable} th, ${checkTable} td`,{padding:vars.space[3],textAlign:'start',verticalAlign:'top',borderBlockEnd:`1px solid ${vars.color.border}`,overflowWrap:'anywhere'});
globalStyle(`${checkTable} thead th`,{background:vars.color.bgAlt,fontWeight:vars.weight.bold});
globalStyle(`${checkTable} th:nth-child(1)`,{inlineSize:'22%'});
globalStyle(`${checkTable} thead th:nth-child(2)`,{inlineSize:'12%'});
globalStyle(`${checkTable} thead th:nth-child(3)`,{inlineSize:'34%'});
globalStyle(`${checkTable} thead th:nth-child(4)`,{inlineSize:'12%'});
globalStyle(`${checkTable} thead th:nth-child(5)`,{inlineSize:'20%'});
globalStyle(`${checkTable} tbody th`,{fontWeight:vars.weight.medium});
globalStyle(`${checkTable} td > * + *`,{marginBlockStart:vars.space[3]});

/** Cards flow in one column on a phone, two or more where there is room. */
export const cards = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fill, minmax(min(100%, 22rem), 1fr))',
  gap: vars.space[5],
  alignItems: 'start',
  minInlineSize: 0,
});
export const sectionTitle = style({ margin: 0, fontSize: '1rem', fontWeight: vars.weight.heavy });
export const group = style({ display: 'flex', flexDirection: 'column', gap: vars.space[4], minInlineSize: 0 });

export const card = style({
  ...subsectionSurface,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  selectors: {
    '&[data-status="warn"]': { borderInlineStartColor: vars.color.warning, borderInlineStartWidth: vars.border.heavy },
    '&[data-status="fail"]': { borderInlineStartColor: vars.color.danger, borderInlineStartWidth: vars.border.heavy },
  },
});
export const cardHead = style({ display: 'flex', alignItems: 'flex-start', gap: vars.space[3], minInlineSize: 0 });
export const cardTitles = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1], flex: '1 1 auto', minInlineSize: 0 });
export const cardTitle = style({ margin: 0, fontSize: '1rem', fontWeight: vars.weight.heavy, overflowWrap: 'anywhere' });
export const purpose = style({ margin: 0, fontSize: '0.875rem', color: vars.color.textDim, lineHeight: 1.45 });
export const cardActions = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[2], alignItems: 'center' });

/** Labelled rows: label above value on a phone, side by side when wider. */
export const rows = style({
  display: 'grid',
  gridTemplateColumns: 'minmax(0, 9rem) minmax(0, 1fr)',
  gap: `${vars.space[2]} ${vars.space[3]}`,
  margin: 0,
  '@media': { 'screen and (max-width: 480px)': { gridTemplateColumns: 'minmax(0, 1fr)', rowGap: vars.space[1] } },
});
globalStyle(`${rows} > dt`, { color: vars.color.textDim, fontSize: '0.8rem', fontWeight: vars.weight.bold, paddingBlockStart: '2px' });
globalStyle(`${rows} > dd`, { margin: 0, display: 'flex', alignItems: 'flex-start', gap: vars.space[2], minInlineSize: 0, overflowWrap: 'anywhere' });
globalStyle(`${rows} > dd + dt`, { '@media': { 'screen and (max-width: 480px)': { paddingBlockStart: vars.space[2] } } });
export const value = style({ minInlineSize: 0, flex: '1 1 auto' });
export const mono = style({ fontFamily: vars.font.mono, fontSize: '0.85rem', overflowWrap: 'anywhere' });

export const copy = style({
  flex: 'none',
  display: 'inline-grid',
  placeItems: 'center',
  inlineSize: '24px',
  blockSize: '24px',
  padding: 0,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  borderRadius: vars.radius.sm,
  background: 'transparent',
  color: vars.color.textDim,
  cursor: 'pointer',
  selectors: { '&:hover': { color: vars.color.text, borderColor: vars.color.borderStrong }, '&:focus-visible': focusRing },
});
globalStyle(`${copy} svg`, { inlineSize: '14px', blockSize: '14px' });

export const fix = style({
  margin: 0,
  padding: vars.space[3],
  borderInlineStart: `${vars.border.heavy} solid ${vars.color.warning}`,
  background: vars.color.bgAlt,
  fontSize: '0.875rem',
  lineHeight: 1.5,
  selectors: { '&[data-status="fail"]': { borderInlineStartColor: vars.color.danger } },
});
export const fixTitle = style({ display: 'block', fontWeight: vars.weight.bold, marginBlockEnd: vars.space[1] });

export const details = style({ minInlineSize: 0 });
globalStyle(`${details} > summary`, { cursor: 'pointer', fontSize: '0.875rem', fontWeight: vars.weight.bold });
export const raw = style({
  margin: `${vars.space[2]} 0 0`,
  padding: vars.space[3],
  maxBlockSize: '18rem',
  overflow: 'auto',
  background: vars.color.bgSink,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  fontFamily: vars.font.mono,
  fontSize: '0.8rem',
  whiteSpace: 'pre-wrap',
  overflowWrap: 'anywhere',
});

const spin = keyframes({ to: { transform: 'rotate(360deg)' } });
export const chip = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  flex: 'none',
  fontSize: '0.75rem',
  fontWeight: vars.weight.bold,
  padding: `0 ${vars.space[2]}`,
  minBlockSize: '22px',
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  background: vars.color.bg,
  color: vars.color.text,
  whiteSpace: 'nowrap',
  selectors: {
    '&[data-status="ok"]': { boxShadow: `inset 3px 0 0 ${vars.color.success}` },
    '&[data-status="warn"]': { boxShadow: `inset 3px 0 0 ${vars.color.warning}` },
    '&[data-status="fail"]': { boxShadow: `inset 3px 0 0 ${vars.color.danger}` },
    '&[data-status="running"]': { boxShadow: `inset 3px 0 0 ${vars.color.accent}` },
    '&[data-status="notRun"], &[data-status="skipped"]': { color: vars.color.textDim, borderStyle: 'dashed' },
    '[dir="rtl"] &': { boxShadow: 'none' },
  },
});
globalStyle(`${chip} svg`, { inlineSize: '12px', blockSize: '12px', flex: 'none' });
globalStyle(`${chip}[data-status="running"] svg`, { animation: `${spin} 1s linear infinite`, '@media': { '(prefers-reduced-motion: reduce)': { animation: 'none' } } });

/** Overview: one tile per group. */
export const tiles = style({ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(min(100%, 14rem), 1fr))', gap: vars.space[4] });
export const tile = style({
  ...subsectionSurface,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  color: vars.color.text,
  textDecoration: 'none',
  selectors: { '&:hover': { borderColor: vars.color.accent }, '&:focus-visible': focusRing },
});
export const tileHead = style({ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: vars.space[2] });
export const tileTitle = style({ fontWeight: vars.weight.heavy });
export const tileCounts = style({ margin: 0, fontSize: '0.85rem', color: vars.color.textDim });

export const progress = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2], minInlineSize: 0 });
export const bar = style({ inlineSize: '100%', blockSize: '8px', accentColor: vars.color.accent });

export const attention = style({ listStyle: 'none', margin: 0, padding: 0, display: 'flex', flexDirection: 'column', gap: vars.space[2] });
export const attentionItem = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: vars.space[2], minInlineSize: 0 });

export const filters = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[3], alignItems: 'flex-end', minInlineSize: 0 });
globalStyle(`${filters} label`, { display: 'flex', flexDirection: 'column', gap: vars.space[1], minInlineSize: 0, fontSize: '0.85rem' });
globalStyle(`${filters} input, ${filters} select`, { maxInlineSize: '100%' });

export const links = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[2] });
export const changeList = style({ margin: 0, paddingInlineStart: vars.space[5], display: 'flex', flexDirection: 'column', gap: vars.space[1] });
