import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const senders=style({listStyle:'none',margin:0,padding:0,minWidth:0});
export const sender=style({display:'flex',alignItems:'center',justifyContent:'space-between',flexWrap:'wrap',gap:vars.space[2],paddingBlock:vars.space[1],minWidth:0});
export const address=style({overflowWrap:'anywhere',minWidth:0,maxWidth:'100%'});
