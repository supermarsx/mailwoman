import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const iconLabel=style({display:'inline-flex',alignItems:'center',gap:vars.space[2],minInlineSize:0,textAlign:'start'});
globalStyle(`.${iconLabel} > svg`,{flexShrink:0,alignSelf:'center'});

export const filters=style({display:'grid',gridTemplateColumns:'minmax(0,1fr) minmax(10rem,14rem)',gap:vars.space[4],alignItems:'end',padding:vars.space[4],marginBlock:vars.space[4],border:`1px solid ${vars.color.border}`,borderRadius:vars.radius.md,background:vars.color.surface,'@media':{'(max-width: 620px)':{gridTemplateColumns:'minmax(0,1fr)'}}});
export const filterField=style({display:'grid',gap:vars.space[2],minInlineSize:0,color:vars.color.textDim,fontSize:'0.875rem'});
globalStyle(`${filterField} > span`,{minInlineSize:0,inlineSize:'100%'});
