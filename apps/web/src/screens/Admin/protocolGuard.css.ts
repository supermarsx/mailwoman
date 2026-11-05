import { style, globalStyle } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

const PHONE = '(max-width: 560px)';

export const tiles = style({ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(10rem, 1fr))', gap: vars.space[3], margin: `${vars.space[3]} 0` });
export const tile = style({ display: 'grid', gap: vars.space[1], padding: vars.space[3], border: `${vars.border.hair} solid ${vars.color.borderStrong}`, borderRadius: vars.radius.md, background: vars.color.bg, minInlineSize: 0 });
export const tileValue = style({ fontSize: '1.5rem', fontVariantNumeric: 'tabular-nums', fontWeight: 700, overflowWrap: 'anywhere', minInlineSize: 0 });
export const tileValueCompact = style({ fontSize: '0.95rem', fontWeight: 700, lineHeight: 1.3, overflowWrap: 'anywhere', minInlineSize: 0 });
export const tileCaption = style({ color: vars.color.textDim, fontSize: '0.75rem', margin: 0 });
export const tileLabel = style({ color: vars.color.textDim, fontSize: '0.8rem' });
export const toolbar = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[3], alignItems: 'center', justifyContent: 'space-between', margin: `${vars.space[2]} 0` });
export const groupHead = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[2], alignItems: 'baseline', justifyContent: 'space-between' });
export const table = style({ inlineSize: '100%', borderCollapse: 'collapse' });
globalStyle(`${table} th`, { textAlign: 'start', fontWeight: 600, fontSize: '0.8rem', color: vars.color.textDim, padding: `${vars.space[1]} ${vars.space[2]}` });
globalStyle(`${table} td`, { padding: `${vars.space[1]} ${vars.space[2]}`, borderBlockStart: `${vars.border.hair} solid ${vars.color.border}`, verticalAlign: 'middle' });
globalStyle(`${table} .num`, { textAlign: 'end', fontVariantNumeric: 'tabular-nums', whiteSpace: 'nowrap' });
export const zero = style({ color: vars.color.textDim, opacity: 0.7 });
export const hit = style({ fontWeight: 700, display: 'inline-flex', gap: vars.space[1], alignItems: 'center', color: vars.color.text });
export const bar = style({ appearance: 'none', inlineSize: '100%', maxInlineSize: '12rem', blockSize: '4px', border: 0, background: vars.color.bgSink, color: vars.color.accent, accentColor: vars.color.accent });
export const label = style({ display: 'grid', gap: '2px', minInlineSize: 0 });
// Phone width: rows become cards, each cell carrying its own label.
globalStyle(`${table} thead`, { '@media': { [PHONE]: { position: 'absolute', insetInlineStart: '-9999px' } } });
globalStyle(`${table} tr, ${table} td`, { '@media': { [PHONE]: { display: 'block' } } });
globalStyle(`${table} tr`, { '@media': { [PHONE]: { padding: vars.space[2], borderBlockStart: `${vars.border.hair} solid ${vars.color.border}` } } });
globalStyle(`${table} td`, { '@media': { [PHONE]: { border: 0, display: 'flex', justifyContent: 'space-between', gap: vars.space[3] } } });
globalStyle(`${table} td[data-label]::before`, { '@media': { [PHONE]: { content: 'attr(data-label)', color: vars.color.textDim } } });
export const editRow = style({ display: 'grid', gap: vars.space[1], marginBlockEnd: vars.space[3] });
export const statLine = style({ fontSize: '0.8rem', color: vars.color.textDim, fontVariantNumeric: 'tabular-nums', margin: 0 });
export const modeSwitch = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[2], alignItems: 'center' });

export const rangeField = style({ display: 'inline-flex', gap: vars.space[2], alignItems: 'center', margin: `${vars.space[2]} 0` });
