import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';
export const root=style({containerType:'inline-size',containerName:'settings-workspace',height:'100%',minHeight:0,overflow:'hidden'});
export const panel=style({maxWidth:'none',maxHeight:'none',height:'100%',padding:vars.space[4],border:0,boxShadow:'none',overflow:'hidden',gap:vars.space[4]});
export const body=style({display:'grid',gridTemplateColumns:'minmax(9rem,12rem) minmax(0,1fr)',gap:vars.space[5],minHeight:0,flex:1,overflow:'hidden','@container':{'settings-workspace (max-width: 760px)':{gridTemplateColumns:'minmax(0,1fr)',gridTemplateRows:'auto minmax(0,1fr)'}}});
export const legacy=style({display:'contents'});
export const nav=style({display:'flex',flexDirection:'column',gap:vars.space[1],overflow:'auto',alignItems:'stretch','@container':{'settings-workspace (max-width: 760px)':{flexDirection:'row',maxHeight:'3.5rem'}}});
globalStyle(`${nav} button`,{display:'flex',alignItems:'center',gap:vars.space[2],textAlign:'start',padding:vars.space[2],whiteSpace:'nowrap',background:'transparent',color:vars.color.text,border:`1px solid transparent`});
globalStyle(`${nav} button svg`,{flex:'0 0 auto',inlineSize:'16px',blockSize:'16px'});
globalStyle(`${nav} button[aria-current="page"]`,{borderColor:vars.color.borderStrong,background:vars.color.surface});
export const content=style({minWidth:0,minHeight:0,overflow:'auto',display:'flex',flexDirection:'column',gap:vars.space[5]});
export const appearance=style({selectors:{'&:not([hidden])':{display:'flex',flexDirection:'column',gap:vars.space[6]}}});

// The embedded workspace must override the legacy modal's later-loaded 520px cap.
globalStyle(`${root} > section`,{maxWidth:'none',maxHeight:'none',minWidth:0,width:'100%',boxSizing:'border-box'});
