import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const list=style({maxInlineSize:'42rem',inlineSize:'100%',minInlineSize:0});
export const row=style({display:'grid',gridTemplateColumns:'1.25rem minmax(0,1fr) minmax(7rem,10rem) auto',gap:vars.space[2],alignItems:'center',minBlockSize:'2.5rem',paddingBlock:vars.space[1],borderBlockEnd:`1px solid ${vars.color.border}`,minInlineSize:0,'@media':{'(max-width: 420px)':{gridTemplateColumns:'1.25rem minmax(0,1fr) auto'}}});
export const label=style({minInlineSize:0,fontSize:'.875rem',overflowWrap:'anywhere'});
export const select=style({inlineSize:'100%',minInlineSize:0,maxInlineSize:'10rem',padding:'3px 6px',fontSize:'.8125rem','@media':{'(max-width: 420px)':{gridColumn:'2',gridRow:'2',maxInlineSize:'100%'}}});
export const reset=style({inlineSize:'2rem',blockSize:'2rem',minInlineSize:'2rem',padding:'4px',display:'inline-flex',justifyContent:'center',alignItems:'center','@media':{'(max-width: 420px)':{gridColumn:'3',gridRow:'1 / 3'},'(pointer: coarse)':{minBlockSize:'44px'}}});
export const note=style({gridColumn:'2 / -1',fontSize:'.8rem'});

export const preview=style({display:'flex',alignItems:'center',flexWrap:'wrap',gap:'.25rem',minInlineSize:0,padding:vars.space[3],marginBlock:vars.space[3],border:`1px solid ${vars.color.borderStrong}`,background:vars.color.bg});

export const order=style({display:'flex',gap:vars.space[1],gridColumn:'4','@media':{'(max-width: 420px)':{gridColumn:'3',gridRow:'1 / 3'}}});
globalStyle(`${order} > button`,{display:'inline-flex',alignItems:'center',justifyContent:'center',padding:'2px',minHeight:'1.75rem',inlineSize:'1.75rem'});
