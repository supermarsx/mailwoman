import {style,globalStyle} from '@vanilla-extract/css';
import {vars} from '../../../theme/contract.css.ts';
export const grid=style({display:'grid',gridTemplateColumns:'repeat(auto-fit,minmax(min(100%,18rem),1fr))',gap:vars.space[4],minWidth:0});
export const check=style({display:'flex',alignItems:'center',gap:vars.space[2]});
export const tableWrap=style({overflowX:'auto',maxWidth:'100%',minWidth:0});
export const table=style({width:'100%',borderCollapse:'collapse',textAlign:'start'});
export const rowActions=style({display:'flex',justifyContent:'flex-end',gap:vars.space[2]});

globalStyle(`${table} th, ${table} td`,{padding:vars.space[3],borderBottom:`1px solid ${vars.color.border}`,overflowWrap:'anywhere'});

export const wizard=style({minWidth:0});
globalStyle(`${wizard} [data-step="review"]`,{gridTemplateColumns:'minmax(8rem,1fr) minmax(0,3fr)',gap:vars.space[3],'@media':{'(max-width: 600px)':{gridTemplateColumns:'minmax(0,1fr)',gap:vars.space[2]}}});
globalStyle(`${wizard} [data-step="review"] dd`,{margin:0,overflowWrap:'anywhere','@media':{'(max-width: 600px)':{marginBottom:vars.space[3]}}});
