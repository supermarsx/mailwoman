import {style,globalStyle} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const notice=style({display:'flex',flexDirection:'column',gap:vars.space[2],padding:vars.space[3],border:`1px solid ${vars.color.border}`,borderRadius:vars.radius.md,minInlineSize:0,selectors:{'&[data-unread="true"]':{borderInlineStart:`3px solid ${vars.color.accent}`}}});
export const heading=style({display:'flex',alignItems:'center',justifyContent:'space-between',flexWrap:'wrap',gap:vars.space[2],minInlineSize:0});
export const facts=style({display:'grid',gridTemplateColumns:'repeat(auto-fit,minmax(min(100%, 20rem),1fr))',columnGap:vars.space[4],rowGap:vars.space[2],margin:0,minInlineSize:0});
export const fact=style({display:'grid',gridTemplateColumns:'minmax(0, .9fr) minmax(0, 1.1fr)',alignItems:'start',gap:vars.space[2],minInlineSize:0});
export const label=style({display:'flex',alignItems:'center',gap:vars.space[2],fontSize:'.8rem',lineHeight:1.4,color:vars.color.textDim,minInlineSize:0});
export const value=style({margin:0,fontSize:'.85rem',lineHeight:1.4,overflowWrap:'anywhere',unicodeBidi:'plaintext',minInlineSize:0});
export const details=style({minInlineSize:0,overflowWrap:'anywhere'});

globalStyle(`${label} > svg`,{width:'1rem',height:'1rem',flexShrink:0});
