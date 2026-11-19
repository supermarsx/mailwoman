import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const graphs=style({display:'grid',gridTemplateColumns:'repeat(2,minmax(0,1fr))',gap:vars.space[5],minInlineSize:0,
 '@media':{'screen and (max-width: 760px)':{gridTemplateColumns:'minmax(0,1fr)'}}});
export const selectedRange=style({background:vars.color.accent,color:vars.color.accentText,fontWeight:vars.weight.bold});
