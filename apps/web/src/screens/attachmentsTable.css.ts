import { style, globalStyle } from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
import {linkColor} from '../theme/accentLink.ts';
export const wrap=style({overflow:'auto',minInlineSize:0,flex:'1 1 auto'});
export const table=style({inlineSize:'100%',minInlineSize:'52rem',borderCollapse:'collapse',tableLayout:'auto',fontSize:'.875rem'});
globalStyle(`${table} th, ${table} td`,{padding:`${vars.space[3]} ${vars.space[4]}`,textAlign:'start',borderBlockEnd:`${vars.border.hair} solid ${vars.color.border}`,verticalAlign:'middle',overflowWrap:'anywhere'});
globalStyle(`${table} th`,{color:vars.color.textDim,fontWeight:vars.weight.bold,background:vars.color.bgAlt});
globalStyle(`${table} th`,{whiteSpace:'nowrap'});
globalStyle(`${table} td:first-child`,{minInlineSize:'12rem',maxInlineSize:'28rem'});
globalStyle(`${table} td:nth-child(3), ${table} td:last-child`,{whiteSpace:'nowrap',fontVariantNumeric:'tabular-nums'});
export const name=style({display:'inline-flex',alignItems:'center',gap:vars.space[2],maxInlineSize:'100%',border:0,background:'transparent',color:linkColor,textAlign:'start',font:'inherit',cursor:'pointer',padding:vars.space[1],':hover':{textDecoration:'underline'},':focus-visible':{outline:`${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`,outlineOffset:'2px'}});
globalStyle(`${name} svg`,{flexShrink:0});
export const paging=style({display:'flex',alignItems:'center',flexWrap:'wrap',gap:vars.space[3],padding:vars.space[3],flexShrink:0});
export const warning=style({display:'inline-flex',verticalAlign:'middle',marginInlineStart:vars.space[2],color:vars.color.text});

globalStyle(`${wrap}:focus-visible`,{outline:`${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`,outlineOffset:'-2px'});

globalStyle(`${table} tbody tr:hover`,{background:`color-mix(in srgb, ${vars.color.accent} 10%, transparent)`});
globalStyle(`${table} tbody tr:focus-within`,{background:`color-mix(in srgb, ${vars.color.accent} 14%, transparent)`});
globalStyle(`${table} th[aria-sort="ascending"], ${table} th[aria-sort="descending"]`,{color:vars.color.text,boxShadow:`inset 0 -2px 0 ${vars.color.accent}`});
globalStyle(`${wrap}`,{scrollbarColor:`${vars.color.borderStrong} ${vars.color.bgAlt}`});
globalStyle(`${table} svg`,{color:'currentColor'});
globalStyle(`${name}[data-category="pdf"] svg`,{color:`color-mix(in srgb, ${vars.color.danger} 65%, ${vars.color.text})`});
globalStyle(`${name}[data-category="image"] svg`,{color:`color-mix(in srgb, ${vars.color.success} 65%, ${vars.color.text})`});
globalStyle(`${name}[data-category="audio"] svg`,{color:`color-mix(in srgb, ${vars.color.warning} 65%, ${vars.color.text})`});
globalStyle(`${name}[data-category="video"] svg`,{color:`color-mix(in srgb, ${vars.color.accent} 65%, ${vars.color.text})`});
globalStyle(`${table} th[aria-sort="ascending"], ${table} th[aria-sort="descending"]`,{'@media':{'(forced-colors: active)':{boxShadow:'none',borderBlockEnd:'2px solid Highlight'}}});
