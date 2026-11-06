// Styles of the folder and message Properties dialogs. Tokens only; a narrow
// screen turns the tab strip into a scrolling row and the key/value grid into one column.
import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';

export const body=style({display:'flex',flexDirection:'column',gap:vars.space[3],minWidth:0,maxWidth:'min(92vw, 44rem)',maxHeight:'min(80vh, 40rem)'});
/** Marks the dialog for the phone layout (the media rules on `grid` and `overrideRow`); also a test hook. */
export const phone=style({});
export const tabs=style({display:'flex',gap:vars.space[1],overflowX:'auto',borderBottom:`1px solid ${vars.color.border}`,flex:'0 0 auto',scrollbarWidth:'thin'});
export const tab=style({flex:'0 0 auto',background:'transparent',border:'none',borderBottom:'2px solid transparent',padding:`${vars.space[2]} ${vars.space[3]}`,color:vars.color.textDim,font:'inherit',cursor:'pointer',whiteSpace:'nowrap',
 selectors:{'&[aria-selected="true"]':{color:vars.color.text,borderBottomColor:vars.color.accent,fontWeight:vars.weight.bold}}});
export const panel=style({overflowY:'auto',minHeight:0,display:'flex',flexDirection:'column',gap:vars.space[3],paddingBlock:vars.space[2]});
export const grid=style({display:'grid',gridTemplateColumns:'minmax(8rem, 12rem) 1fr',gap:`${vars.space[1]} ${vars.space[3]}`,margin:0,'@media':{'(max-width: 40rem)':{gridTemplateColumns:'1fr'}}});
globalStyle(`${grid} dt`,{color:vars.color.textDim,margin:0});
globalStyle(`${grid} dd`,{margin:0,minWidth:0,overflowWrap:'anywhere'});
export const mono=style({fontFamily:'ui-monospace, SFMono-Regular, Menlo, monospace',fontSize:'0.85em'});
export const note=style({color:vars.color.textDim,margin:0,fontSize:'0.9em'});
export const row=style({display:'flex',flexWrap:'wrap',alignItems:'center',gap:vars.space[2]});
export const actions=style({display:'flex',flexWrap:'wrap',gap:vars.space[2]});
export const locked=style({display:'inline-flex',alignItems:'center',gap:vars.space[1],color:vars.color.textDim,fontSize:'0.85em'});
export const source=style({color:vars.color.textDim,fontSize:'0.85em'});
export const overrideRow=style({display:'grid',gridTemplateColumns:'minmax(8rem, 12rem) 1fr',gap:vars.space[1],alignItems:'center',paddingBlock:vars.space[1],borderBottom:`1px solid ${vars.color.border}`,'@media':{'(max-width: 40rem)':{gridTemplateColumns:'1fr'}}});
export const textarea=style({width:'100%',minHeight:'7rem',resize:'vertical',font:'inherit',boxSizing:'border-box'});
export const headersPre=style({margin:0,padding:vars.space[2],border:`1px solid ${vars.color.border}`,borderRadius:'4px',maxHeight:'16rem',overflow:'auto',whiteSpace:'pre-wrap',overflowWrap:'anywhere',fontSize:'0.8em'});
export const hop=style({display:'grid',gridTemplateColumns:'2rem 1fr auto',gap:vars.space[2],alignItems:'center'});
export const bar=style({height:'6px',background:vars.color.accent,borderRadius:'3px',minWidth:'2px'});
export const danger=style({color:vars.color.danger});
