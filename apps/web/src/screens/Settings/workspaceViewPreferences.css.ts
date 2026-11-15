import {style,globalStyle} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const fields=style({display:'flex',flexDirection:'column',gap:vars.space[4],paddingBlock:vars.space[3]});
export const group=style({padding:vars.space[3],border:`${vars.border.hair} solid ${vars.color.border}`,borderRadius:vars.radius.md});
export const summary=style({display:'flex',alignItems:'center',gap:vars.space[2],cursor:'pointer',fontWeight:vars.weight.bold});
export const header=style({display:'flex',alignItems:'center',justifyContent:'space-between',gap:vars.space[3]});
export const dialog=style({color:vars.color.text,background:vars.color.surface,border:`${vars.border.rule} solid ${vars.color.borderStrong}`,padding:vars.space[5],maxInlineSize:'min(38rem,calc(100vw - 2rem))',inlineSize:'38rem',maxBlockSize:'90vh',overflowY:'auto'});
globalStyle(`${dialog}::backdrop`,{background:'rgba(0,0,0,.55)'});
