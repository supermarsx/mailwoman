import {style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
export const header=style({position:'absolute',insetInline:0,height:32,boxSizing:'border-box',display:'flex',alignItems:'center',justifyContent:'space-between',gap:8,paddingInline:12,background:vars.color.surface,color:vars.color.text,fontSize:'.8rem',fontWeight:600,borderBlockEnd:`1px solid ${vars.color.border}`,overflow:'hidden',whiteSpace:'nowrap',textOverflow:'ellipsis'});
export const count=style({fontWeight:400,fontVariantNumeric:'tabular-nums',flexShrink:0});

export const label=style({minInlineSize:0,overflow:"hidden",textOverflow:"ellipsis"});
