import { style, globalStyle } from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
import {linkColor} from '../theme/accentLink.ts';
export const wrap=style({overflow:'auto',minInlineSize:0,minBlockSize:'12rem',flex:'1 1 auto'});
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

export const filters=style({display:'flex',flexDirection:'column',gap:vars.space[3],padding:`${vars.space[3]} ${vars.space[4]}`,borderBlockEnd:`${vars.border.hair} solid ${vars.color.border}`,flexShrink:0,minInlineSize:0});
export const searchRow=style({display:'flex',alignItems:'end',flexWrap:'wrap',gap:vars.space[2],minInlineSize:0});
export const field=style({display:'flex',flexDirection:'column',gap:vars.space[1],minInlineSize:0,maxInlineSize:'24rem',fontSize:'.85rem'});
export const searchField=style([field,{flex:'1 1 18rem',maxInlineSize:'36rem'}]);
export const control=style({boxSizing:'border-box',inlineSize:'100%',minInlineSize:0,minBlockSize:'2rem',padding:'.4rem .55rem',border:`${vars.border.rule} solid ${vars.color.borderStrong}`,borderRadius:vars.radius.sm,background:vars.color.bgSink,color:vars.color.text,font:'inherit',':focus-visible':{outline:`${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`,outlineOffset:2}});
export const filterGrid=style({display:'grid',gridTemplateColumns:'repeat(auto-fit, minmax(min(100%, 11rem), 1fr))',gap:vars.space[3],maxInlineSize:'80rem',minInlineSize:0,'@media':{'(max-width: 600px)':{gridTemplateColumns:'repeat(2, minmax(0, 1fr))',gap:vars.space[2]}}});
export const more=style({fontSize:'.85rem',minInlineSize:0});
export const help=style({fontSize:'.8rem',color:vars.color.textDim,marginBlockEnd:0});
export const resultCount=style({fontSize:'.8rem',color:vars.color.textDim,marginInlineStart:'auto',fontVariantNumeric:'tabular-nums'});
export const chips=style({display:'flex',flexWrap:'wrap',gap:vars.space[2],minInlineSize:0});
export const chip=style({display:'inline-flex',alignItems:'center',gap:vars.space[2],maxInlineSize:'100%',padding:'.25rem .5rem',background:vars.color.bgAlt,color:vars.color.text,border:`${vars.border.hair} solid ${vars.color.borderStrong}`,borderRadius:vars.radius.sm,font:'inherit',fontSize:'.8rem',cursor:'pointer',':focus-visible':{outline:`${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`,outlineOffset:2}});
export const validation=style({fontSize:'.85rem',color:vars.color.danger,margin:0});
globalStyle(`${field} > span`,{display:'inline-flex',alignItems:'center',gap:vars.space[1],color:vars.color.textDim});
globalStyle(`${more} > summary`,{display:'flex',alignItems:'center',gap:vars.space[2],cursor:'pointer',inlineSize:'fit-content',paddingBlock:vars.space[1]});
globalStyle(`${more}[open] > summary`,{marginBlockEnd:vars.space[2]});
globalStyle(`${chip} > span`,{overflow:'hidden',textOverflow:'ellipsis',whiteSpace:'nowrap'});
globalStyle(`${chip} svg`,{flexShrink:0});

globalStyle('.mw-attach',{overflowY:'auto'});
