import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const root=style({minWidth:0});
export const actions=style({display:'flex',justifyContent:'flex-end',alignItems:'center',gap:vars.space[1]});
export const error=style({color:vars.color.danger,fontSize:'.8rem',maxWidth:'24rem'});
