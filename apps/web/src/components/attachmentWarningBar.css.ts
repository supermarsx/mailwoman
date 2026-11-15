import {style,globalStyle} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
export const bar=style({display:'flex',alignItems:'flex-start',gap:vars.space[3],padding:vars.space[3],marginBlock:vars.space[2],border:`${vars.border.hair} solid ${vars.color.borderStrong}`,borderInlineStart:`3px solid ${vars.color.danger}`,borderRadius:vars.radius.md,background:vars.color.bgAlt,fontSize:'.85rem',minInlineSize:0});
export const body=style({display:'flex',flexDirection:'column',gap:vars.space[2],minInlineSize:0});
globalStyle(`${bar} > svg`,{flexShrink:0});
globalStyle(`${body} p`,{margin:0,color:vars.color.textDim,maxInlineSize:'75ch'});
globalStyle(`${body} ul`,{margin:0,paddingInlineStart:'1.25rem',display:'flex',flexDirection:'column',gap:vars.space[2]});
globalStyle(`${body} li`,{overflowWrap:'anywhere'});
globalStyle(`${body} li > span`,{display:'flex',flexDirection:'column',gap:vars.space[1],color:vars.color.textDim});
