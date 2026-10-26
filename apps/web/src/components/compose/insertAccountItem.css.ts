import {style,globalStyle} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const root=style({position:'relative',minWidth:0});
export const panel=style({display:'grid',gap:vars.space[2],padding:vars.space[3],border:`1px solid ${vars.color.border}`,borderRadius:vars.radius.sm,background:vars.color.bg,minWidth:0,maxWidth:'36rem'});
export const header=style({display:'flex',alignItems:'center',justifyContent:'space-between',gap:vars.space[2]});
export const help=style({fontSize:'.8rem',color:vars.color.textDim,margin:0});
globalStyle(`${panel} label`,{display:'grid',gap:vars.space[1],minWidth:0});
globalStyle(`${panel} :is(input,select)`,{maxWidth:'100%',width:'100%',minWidth:0});
