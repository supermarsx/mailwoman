import { style, globalStyle } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';
export const wrap = style({ inlineSize: '100%', minInlineSize: 0, overflowX: 'auto', border: `${vars.border.rule} solid ${vars.color.borderStrong}` });
export const table = style({ inlineSize: '100%', borderCollapse: 'collapse', textAlign: 'start' });
globalStyle(`${table} th, ${table} td`, { padding: `${vars.space[2]} ${vars.space[3]}`, borderBlockEnd: `1px solid ${vars.color.border}`, verticalAlign: 'top', overflowWrap: 'anywhere', minInlineSize: '6rem', maxInlineSize: '22rem', fontSize: '0.85rem' });
// The card title already names the table; the caption stays for assistive technology.
globalStyle(`${table} caption`, { position: 'absolute', inlineSize: '1px', blockSize: '1px', overflow: 'hidden', clipPath: 'inset(50%)', whiteSpace: 'nowrap' });
globalStyle(`${table} td:last-child`, { textAlign: 'end' });
globalStyle(`${table} th:first-child, ${table} td:first-child`, { minInlineSize:'3rem',inlineSize:'3rem',maxInlineSize:'3rem' });

globalStyle(`${wrap}:focus-visible`,{outline:`${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`,outlineOffset:'-2px'});

export const originLine = style({ display: 'flex', alignItems: 'center', gap: vars.space[2], margin: `${vars.space[1]} 0 0`, color: vars.color.textDim, fontSize: '0.8rem' });
export const originDetails = style({ display: 'grid', gridTemplateColumns: 'auto 1fr', gap: `${vars.space[1]} ${vars.space[3]}`, margin: `${vars.space[2]} 0`, textAlign: 'start' });
globalStyle(`${originDetails} dt`, { color: vars.color.textDim });
globalStyle(`${originDetails} dd`, { margin: 0, overflowWrap: 'anywhere' });
