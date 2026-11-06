import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';

export const filters=style({display:'flex',flexWrap:'wrap',gap:vars.space[3],alignItems:'end'});
export const filterField=style({display:'flex',flexDirection:'column',gap:vars.space[1],minInlineSize:0});
export const toolbar=style({display:'flex',flexWrap:'wrap',gap:vars.space[2],alignItems:'center'});
export const name=style({display:'flex',flexDirection:'column',gap:vars.space[1],minInlineSize:0,maxInlineSize:'38rem'});
export const message=style({color:vars.color.textDim,fontSize:'0.85rem',overflowWrap:'anywhere'});
export const num=style({textAlign:'end',fontVariantNumeric:'tabular-nums'});
export const spark=style({display:'block',inlineSize:'6rem',blockSize:'1.5rem',color:vars.color.accent});
export const sparkFlat=style({color:vars.color.border});
export const statusOpen=style({borderColor:vars.color.danger,color:vars.color.danger});
export const frames=style({margin:0,padding:0,listStyle:'none',fontFamily:vars.font.mono,fontSize:'0.8rem',display:'flex',flexDirection:'column'});
export const frame=style({padding:`${vars.space[1]} ${vars.space[3]}`,overflowWrap:'anywhere',borderInlineStart:`3px solid transparent`,color:vars.color.textDim});
export const frameApp=style({color:vars.color.text,fontWeight:vars.weight.bold,borderInlineStartColor:vars.color.accent,background:vars.color.bgAlt});
export const rawStack=style({margin:0,padding:vars.space[3],fontFamily:vars.font.mono,fontSize:'0.8rem',whiteSpace:'pre-wrap',overflowWrap:'anywhere',background:vars.color.bgAlt,border:`${vars.border.hair} solid ${vars.color.border}`});
export const detailGrid=style({display:'grid',gridTemplateColumns:'repeat(auto-fit,minmax(min(100%,18rem),1fr))',gap:vars.space[4]});
