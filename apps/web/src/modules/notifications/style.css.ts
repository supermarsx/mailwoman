import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const iconLabel=style({display:'inline-flex',alignItems:'center',gap:vars.space[2],minInlineSize:0,textAlign:'start'});
globalStyle(`.${iconLabel} > svg`,{flexShrink:0,alignSelf:'center'});
