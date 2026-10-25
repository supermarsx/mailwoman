import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

// A readable minimum inside the bounded keyboard-scrollable wrapper on phones.
export const table = style({ tableLayout: 'fixed', minInlineSize: '46rem', maxInlineSize: '100%' });
globalStyle(`${table} th:nth-child(1)`, { inlineSize: '32%' });
globalStyle(`${table} th:nth-child(2)`, { inlineSize: '25%' });
globalStyle(`${table} th:nth-child(3)`, { inlineSize: '19%' });
globalStyle(`${table} thead th, ${table} tbody th, ${table} td`, { overflowWrap: 'anywhere', whiteSpace: 'normal' });
export const title = style({ flex: '1 1 24rem', minInlineSize: 0 });
export const cell = style({ display: 'flex', flexDirection: 'column', gap: vars.space[3], minInlineSize: 0 });
export const key = style({ fontSize: '0.75rem', fontFamily: vars.font.mono, color: vars.color.textDim, overflowWrap: 'anywhere' });
export const value = style({ margin: 0, whiteSpace: 'pre-wrap', overflowWrap: 'anywhere', font: 'inherit', lineHeight: 1.5 });
export const details = style({ borderBlockStart: `${vars.border.hair} solid ${vars.color.border}`, paddingBlockStart: vars.space[3] });
globalStyle(`${details} summary`, { cursor: 'pointer', minBlockSize: vars.a11y.touchTarget });
export const scroller = style({ minInlineSize: 0, maxInlineSize: '100%', selectors: { '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing } } });
