import {style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
export const root=style({minInlineSize:0,minBlockSize:0,flex:1});
export const title=style({display:'flex',alignItems:'center',gap:vars.space[3],flexWrap:'wrap'});
export const badge=style({fontSize:'.8rem',color:vars.color.accent,fontWeight:vars.weight.bold});
export const saved=style({fontSize:'.8rem',color:vars.color.textDim,marginBlock:vars.space[2]});
export const actions=style({display:'flex',alignItems:'center',gap:vars.space[2],flexWrap:'wrap',marginBlock:vars.space[3]});
export const attachments=style({padding:vars.space[3],margin:vars.space[3],border:`${vars.border.hair} solid ${vars.color.border}`});
export const file=style({display:'flex',alignItems:'center',gap:vars.space[2],paddingBlock:vars.space[2],overflowWrap:'anywhere'});
