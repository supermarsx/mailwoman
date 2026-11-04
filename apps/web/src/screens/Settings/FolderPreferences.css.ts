import { style, globalStyle } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const heading=style({display:'flex',alignItems:'start',justifyContent:'space-between',flexWrap:'wrap',gap:vars.space[3]});
export const tools=style({display:'flex',alignItems:'center',flexWrap:'wrap',gap:vars.space[2],maxInlineSize:'64rem'});
export const search=style({display:'flex',alignItems:'center',gap:vars.space[2],flex:'1 1 16rem',maxInlineSize:'28rem',minInlineSize:0});
globalStyle(`${search} > input:not([type="checkbox"]):not([type="radio"])`,{minInlineSize:0,inlineSize:'100%'});
export const bulk=style({display:'flex',alignItems:'center',flexWrap:'wrap',gap:vars.space[2],padding:vars.space[2],border:`${vars.border.hair} solid ${vars.color.borderStrong}`,background:vars.color.bgAlt,maxInlineSize:'64rem'});
globalStyle(`${bulk} > span`,{fontWeight:vars.weight.medium,marginInlineEnd:vars.space[2]});
export const wrap = style({maxInlineSize:'64rem',minInlineSize:0,overflowX:'auto'});
export const table = style({inlineSize:'100%',borderCollapse:'collapse',tableLayout:'auto'});
globalStyle(`${table} th, ${table} td`,{padding:`${vars.space[2]} ${vars.space[2]}`,textAlign:'start',verticalAlign:'middle',borderBlockEnd:`${vars.border.hair} solid ${vars.color.border}`,overflowWrap:'anywhere'});
globalStyle(`${table} thead`,{background:vars.color.bgAlt,fontSize:'0.8rem'});
globalStyle(`${table} tbody tr:hover`,{background:vars.color.bgAlt});
globalStyle(`${table} tbody tr[data-selected="true"]`,{background:vars.color.selection});
globalStyle(`${table} tbody tr[data-drop-target="true"]`,{boxShadow:`inset 0 3px 0 ${vars.color.accent}`,background:vars.color.selection});
globalStyle(`${table} th:last-child, ${table} td:last-child`,{inlineSize:'9rem',minInlineSize:'9rem'});
export const selectCell=style({inlineSize:'2rem',textAlign:'center'});
export const location=style({maxInlineSize:'18rem',fontSize:'0.8rem',color:vars.color.textDim,'@media':{'(max-width: 600px)':{display:'none'}}});
export const mobileLocation=style({display:'none',overflowWrap:'anywhere','@media':{'(max-width: 600px)':{display:'inline'}}});
export const name=style({display:'flex',alignItems:'center',gap:vars.space[2],minInlineSize:0});
export const folderName=style({fontWeight:vars.weight.medium,overflowWrap:'anywhere',minInlineSize:0});
export const metadata=style({display:'flex',flexWrap:'wrap',gap:vars.space[2],fontSize:'0.75rem',color:vars.color.textDim,paddingInlineStart:'2rem'});
export const actions = style({position:'relative',display:'flex',alignItems:'center',gap:'2px',flexWrap:'nowrap'});
globalStyle(`${actions} > button`,{flex:'0 0 auto',minInlineSize:'30px',minBlockSize:'30px',padding:'3px','@media':{'(pointer: coarse)':{minInlineSize:'44px',minBlockSize:'44px'}}});
export const drag=style({flex:'0 0 auto',minInlineSize:'24px',minBlockSize:'30px',padding:'2px',cursor:'grab',borderColor:'transparent',selectors:{'&:active':{cursor:'grabbing'}}});
// Move buttons remain keyboard reachable; the same actions are in More.
export const keyboardAction=style({position:'absolute',inlineSize:'1px',blockSize:'1px',minInlineSize:'1px',minBlockSize:'1px',padding:0,clipPath:'inset(50%)',overflow:'hidden',selectors:{'&:focus-visible':{position:'relative',inlineSize:'auto',blockSize:'auto',minInlineSize:'30px',minBlockSize:'30px',clipPath:'none',overflow:'visible'}}});
export const empty=style({textAlign:'center',color:vars.color.textDim,paddingBlock:vars.space[4]});

globalStyle(`${wrap}:focus-visible`,{outline:`${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`,outlineOffset:'-2px'});

export const iconGrid=style({display:'grid',gridTemplateColumns:'repeat(auto-fill,minmax(44px,1fr))',gap:'4px',maxInlineSize:'28rem'});
globalStyle(`${iconGrid} button[aria-checked="true"]`,{outline:`2px solid ${vars.color.accent}`,outlineOffset:'-2px'});
