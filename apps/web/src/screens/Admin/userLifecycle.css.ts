import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const root=style({minWidth:0});
export const actions=style({display:'flex',justifyContent:'flex-end',alignItems:'center',gap:vars.space[1]});
export const detailActions=style({display:'flex',flexWrap:'wrap',justifyContent:'flex-start',alignItems:'center',gap:vars.space[3]});
export const textAction=style({display:'inline-flex',alignItems:'center',justifyContent:'center',gap:vars.space[2],inlineSize:'auto',flex:'0 0 auto',textDecoration:'none'});
export const error=style({color:vars.color.danger,fontSize:'.8rem',maxWidth:'24rem'});
