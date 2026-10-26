import { style, globalStyle } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';
export const wrap = style({ inlineSize: '100%', minInlineSize: 0, overflowX: 'auto', border: `${vars.border.rule} solid ${vars.color.borderStrong}` });
export const table = style({ inlineSize: '100%', borderCollapse: 'collapse', textAlign: 'start' });
globalStyle(`${table} th, ${table} td`, { padding: vars.space[5], borderBlockEnd: `1px solid ${vars.color.border}`, verticalAlign: 'top', overflowWrap: 'anywhere', minInlineSize: '8rem', maxInlineSize: '22rem' });
globalStyle(`${table} caption`, { textAlign: 'start', padding: vars.space[5], fontWeight: 700 });
globalStyle(`${table} td:last-child`, { textAlign: 'end' });
globalStyle(`${table} th:first-child, ${table} td:first-child`, { minInlineSize:'3rem',inlineSize:'3rem',maxInlineSize:'3rem' });
