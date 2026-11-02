import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
export const root=style({position:'fixed',insetInlineEnd:vars.space[3],insetBlockEnd:`max(${vars.space[3]}, env(safe-area-inset-bottom))`,zIndex:1090,inlineSize:'28rem',maxInlineSize:`calc(100vw - 2 * ${vars.space[3]})`,padding:vars.space[3],border:`${vars.border.rule} solid ${vars.color.borderStrong}`,background:vars.color.surface,color:vars.color.text,boxShadow:vars.elevation[2],fontSize:'0.875rem'});
export const message=style({display:'flex',alignItems:'start',gap:vars.space[2],minInlineSize:0});
globalStyle(`${message} > svg`,{flex:'0 0 auto',inlineSize:'18px',blockSize:'18px'});
globalStyle(`${message} p`,{marginBlock:`${vars.space[1]} ${vars.space[2]}`,overflowWrap:'anywhere'});
export const actions=style({display:'flex',justifyContent:'flex-end',flexWrap:'wrap',gap:vars.space[2]});
globalStyle(`${actions} .btn`,{minBlockSize:'32px',padding:'4px 8px','@media':{'(pointer:coarse)':{minBlockSize:'44px'}}});
