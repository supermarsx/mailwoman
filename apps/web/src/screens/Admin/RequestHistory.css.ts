import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const controls=style({display:'flex',flexWrap:'wrap',alignItems:'flex-end',gap:vars.space[3]});
export const field=style({display:'flex',flexDirection:'column',gap:vars.space[1],maxWidth:'100%',minWidth:0});
export const input=style({width:'auto',maxWidth:'100%',minWidth:0});
