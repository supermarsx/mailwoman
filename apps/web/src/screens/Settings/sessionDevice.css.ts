import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';

export const device=style({display:'flex',flexDirection:'column',gap:vars.space[2],minInlineSize:0});
export const line=style({display:'inline-flex',alignItems:'center',flexWrap:'wrap',gap:vars.space[2],minInlineSize:0,overflowWrap:'anywhere',lineHeight:1.4});
globalStyle(`${line} svg`,{inlineSize:'1rem',blockSize:'1rem',flexShrink:0});
export const metadata=style({display:'flex',alignItems:'center',flexWrap:'wrap',gap:`${vars.space[2]} ${vars.space[4]}`,color:vars.color.textDim,fontSize:'.875rem'});
export const heading=style({display:'flex',alignItems:'center',flexWrap:'wrap',gap:vars.space[2],minInlineSize:0});
export const technical=style({fontSize:'.8125rem',color:vars.color.textDim,minInlineSize:0});
export const agent=style({marginBlock:`${vars.space[2]} 0`,overflowWrap:'anywhere',maxInlineSize:'72ch',lineHeight:1.5});
export const row=style({display:'flex',alignItems:'flex-start',justifyContent:'space-between',flexWrap:'wrap',gap:vars.space[3],paddingBlock:vars.space[3]});
