import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';
export const root=style({containerType:'inline-size',containerName:'settings-workspace',height:'100%',minHeight:0,overflow:'hidden'});
export const panel=style({maxWidth:'none',maxHeight:'none',height:'100%',padding:vars.space[4],border:0,boxShadow:'none',overflow:'hidden',gap:vars.space[4],'@container':{'settings-workspace (max-width: 760px)':{padding:vars.space[2],gap:vars.space[2]}}});
export const body=style({display:'grid',gridTemplateColumns:'minmax(9rem,12rem) minmax(0,1fr)',gap:vars.space[5],minHeight:0,flex:1,overflow:'hidden','@container':{'settings-workspace (max-width: 760px)':{gridTemplateColumns:'minmax(0,1fr)',gridTemplateRows:'auto minmax(0,1fr)',gap:vars.space[2]}}});
export const legacy=style({display:'contents'});
export const search=style({inlineSize:'100%',minInlineSize:0,userSelect:'text',WebkitUserSelect:'text'});
export const navRoot=style({display:'flex',flexDirection:'column',gap:vars.space[2],minInlineSize:0,minBlockSize:0,overflow:'hidden',userSelect:'none',WebkitUserSelect:'none'});
export const navHeader=style({display:'flex',flexDirection:'column',gap:vars.space[2],flex:'0 0 auto',minInlineSize:0,'@container':{'settings-workspace (max-width: 760px)':{display:'grid',gridTemplateColumns:'auto minmax(0,1fr)',alignItems:'center'}}});
export const navTitle=style({fontSize:'0.875rem',fontWeight:vars.weight.bold,whiteSpace:'nowrap'});
globalStyle(`${navHeader} > input`,{blockSize:'32px',minBlockSize:'32px',padding:'3px 6px',fontSize:'0.8125rem','@media':{'(pointer: coarse)':{blockSize:'44px',minBlockSize:'44px'}}});
export const navEmpty=style({margin:0,padding:vars.space[2],fontSize:'0.8125rem',color:vars.color.textDim});
export const nav=style({display:'flex',flexDirection:'column',gap:vars.space[1],overflow:'auto',alignItems:'stretch',minBlockSize:0,flex:'1 1 0','@container':{'settings-workspace (max-width: 760px)':{display:'none'}}});
export const rowHits=style({listStyle:'none',margin:0,padding:0,marginBlockStart:vars.space[3],display:'flex',flexDirection:'column'});
globalStyle(`${rowHits} button`,{display:'flex',flexDirection:'column',alignItems:'flex-start',gap:0,inlineSize:'100%',textAlign:'start',padding:vars.space[2],fontSize:'0.8125rem',background:'transparent',color:vars.color.text,border:'1px solid transparent',cursor:'pointer'});
globalStyle(`${rowHits} button > span`,{fontSize:'0.7rem',color:vars.color.textDim});
export const sectionPicker=style({display:'none',inlineSize:'100%',minInlineSize:0,justifyContent:'start','@container':{'settings-workspace (max-width: 760px)':{display:'flex',blockSize:'32px',minBlockSize:'32px',padding:'3px 8px',fontSize:'0.8125rem',gap:vars.space[2]}}});
globalStyle(`${sectionPicker} > span`,{flex:'1 1 auto',minInlineSize:0,overflow:'hidden',textOverflow:'ellipsis',whiteSpace:'nowrap',textAlign:'start'});
globalStyle(`${sectionPicker} > svg`,{inlineSize:'16px',blockSize:'16px',flex:'0 0 auto'});
globalStyle(`${navRoot} > ${sectionPicker}`,{display:'none',inlineSize:'100%',maxInlineSize:'100%','@container':{'settings-workspace (max-width: 760px)':{display:'flex',padding:'3px 8px',fontSize:'0.8125rem',lineHeight:'1.25','@media':{'(pointer: coarse)':{blockSize:'44px',minBlockSize:'44px'}}}}});
// Desktop categories retain labelled groups; narrow workspaces use the picker.
export const group=style({display:'flex',flexDirection:'column',gap:vars.space[1],selectors:{'& + &':{marginBlockStart:vars.space[3]}}});
const label={fontSize:'0.7rem',fontWeight:vars.weight.bold,textTransform:'uppercase',letterSpacing:vars.tracking.label,color:vars.color.textDim,selectors:{':lang(ar) &, :lang(ja) &, :lang(zh) &':{textTransform:'none',letterSpacing:vars.tracking.none}}} as const;
export const groupHeading=style({...label,padding:`0 ${vars.space[2]}`,whiteSpace:'nowrap'});
export const crumb=style({...label,margin:0});
globalStyle(`${nav} button`,{display:'flex',alignItems:'center',gap:vars.space[2],textAlign:'start',padding:vars.space[2],whiteSpace:'nowrap',background:'transparent',color:vars.color.text,border:`1px solid transparent`});
globalStyle(`${nav} button svg`,{flex:'0 0 auto',inlineSize:'16px',blockSize:'16px'});
globalStyle(`${nav} button[aria-current="page"]`,{borderColor:vars.color.borderStrong,background:vars.color.surface});
export const content=style({minWidth:0,minHeight:0,overflow:'auto',display:'flex',flexDirection:'column',gap:vars.space[5],'@container':{'settings-workspace (max-width: 760px)':{gap:vars.space[3]}}});
// Retained sections and their cards keep natural height inside this one scrollport.
globalStyle(`${content} > *`,{flexShrink:0});
export const appearance=style({selectors:{'&:not([hidden])':{display:'flex',flexDirection:'column',gap:vars.space[6]}}});

// The embedded workspace must override the legacy modal's later-loaded 520px cap.
globalStyle(`${root} > section`,{maxWidth:'none',maxHeight:'none',minWidth:0,minHeight:0,width:'100%',height:'100%',boxSizing:'border-box',padding:vars.space[4],border:0,boxShadow:'none',overflow:'hidden',gap:vars.space[4],'@container':{'settings-workspace (max-width: 760px)':{padding:vars.space[2],gap:vars.space[2]}}});
