import {style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';

/** A fixed-size slot on a row's first line, so a mark never shifts the date or the sender. */
const base={flex:'0 0 auto',display:'inline-flex',alignItems:'center',gap:'0.2rem',fontSize:'0.75rem',lineHeight:1} as const;
export const high=style({...base,color:vars.color.danger,fontWeight:vars.weight.bold});
export const low=style({...base,color:vars.color.textDim});
/** The low mark's arrow points down: the chevron icon turned a quarter. */
export const lowIcon=style({display:'inline-flex',transform:'rotate(90deg)'});
/** Text next to the icon; kept in forced-colours mode, where colour alone says nothing. */
export const text=style({whiteSpace:'nowrap'});
/** Compact density drops the words and keeps the icon. */
export const bar=style({flex:'0 0 auto',alignSelf:'stretch',width:'3px',borderRadius:0});
export const barHigh=style({backgroundColor:vars.color.danger});
export const barLow=style({backgroundColor:vars.color.textDim});
