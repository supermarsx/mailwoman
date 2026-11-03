import {style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';

export const sheet=style({display:'flex',flexDirection:'column',gap:vars.space[3],minInlineSize:0});
export const note=style({margin:0,color:vars.color.textDim,fontSize:'0.8125rem',lineHeight:1.5});
export const actions=style({display:'flex',alignItems:'center',flexWrap:'wrap',gap:vars.space[2]});
export const headers=style({
  margin:0,padding:vars.space[3],maxBlockSize:'min(60dvh,32rem)',overflow:'auto',
  minInlineSize:0,maxInlineSize:'100%',boxSizing:'border-box',
  fontFamily:vars.font.mono,fontSize:'0.75rem',lineHeight:1.5,whiteSpace:'pre',
  color:vars.color.text,background:vars.color.bg,border:`${vars.border.hair} solid ${vars.color.border}`,
  borderRadius:vars.radius.sm,userSelect:'text',
});
