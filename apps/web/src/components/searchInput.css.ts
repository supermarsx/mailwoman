import {style,globalStyle} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
export const root=style({position:'relative',display:'inline-flex',alignItems:'center',minInlineSize:0,maxInlineSize:'100%',inlineSize:'100%',flex:'1 1 auto'});
export const clear=style({position:'absolute',insetInlineEnd:vars.space[1],insetBlockStart:'50%',transform:'translateY(-50%)',display:'inline-flex',alignItems:'center',justifyContent:'center',inlineSize:'1.75rem',blockSize:'1.75rem',minInlineSize:'1.75rem',minBlockSize:'1.75rem',padding:vars.space[1],border:0,borderRadius:vars.radius.sm,background:'transparent',color:vars.color.textDim,cursor:'pointer',':hover':{background:vars.color.bgAlt,color:vars.color.text},':focus-visible':{outline:`${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`}});
globalStyle(`${root} > input[type="search"]`,{boxSizing:'border-box',inlineSize:'100%',minInlineSize:0,paddingInlineEnd:'2.25rem'});
globalStyle(`${root} > input::-webkit-search-cancel-button`,{WebkitAppearance:'none'});
globalStyle(`${clear} svg`,{inlineSize:'1rem',blockSize:'1rem'});
