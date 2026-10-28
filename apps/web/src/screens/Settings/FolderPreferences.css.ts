import { style, globalStyle } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const wrap = style({maxInlineSize:'52rem',minInlineSize:0,overflowX:'auto'});
export const table = style({inlineSize:'100%',borderCollapse:'collapse',tableLayout:'auto'});
globalStyle(`${table} th, ${table} td`,{padding:`${vars.space[3]} ${vars.space[3]}`,textAlign:'start',verticalAlign:'middle',borderBlockEnd:`${vars.border.hair} solid ${vars.color.border}`,overflowWrap:'anywhere'});
globalStyle(`${table} th:last-child, ${table} td:last-child`,{inlineSize:'17rem',minInlineSize:'17rem','@media':{'(max-width: 600px)':{inlineSize:'9rem',minInlineSize:'9rem'}}});
export const actions = style({display:'flex',alignItems:'center',flexWrap:'wrap',gap:vars.space[3]});
globalStyle(`${actions} > button`,{flex:'0 0 auto',minInlineSize:'44px',minBlockSize:'44px',paddingInline:vars.space[2]});

globalStyle(`${wrap}:focus-visible`,{outline:`${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`,outlineOffset:'-2px'});
