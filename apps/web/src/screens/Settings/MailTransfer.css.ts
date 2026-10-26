import {style,globalStyle} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const section=style({display:'flex',flexDirection:'column',gap:vars.space[3],minWidth:0});
export const fields=style({display:'flex',flexDirection:'column',gap:vars.space[4],minInlineSize:0,minWidth:0,margin:0,padding:0,border:0});
export const label=style({display:'flex',flexDirection:'column',alignItems:'stretch',gap:vars.space[2],minWidth:0});
globalStyle(`${section} input:not([type="checkbox"], [type="radio"]), ${section} select`,{maxWidth:'100%',minWidth:0,width:'100%',boxSizing:'border-box'});
