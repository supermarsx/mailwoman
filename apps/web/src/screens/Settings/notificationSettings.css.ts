import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const container=style({display:'flex',flexDirection:'column',gap:vars.space[5],minInlineSize:0});
export const permissionBody=style({display:'flex',flexDirection:'column',gap:vars.space[3],minInlineSize:0});
globalStyle(`.${permissionBody} > p`,{margin:0});
