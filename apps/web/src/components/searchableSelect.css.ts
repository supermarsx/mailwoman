import {style,globalStyle} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
export const root=style({position:'relative',display:'flex',flexDirection:'column',gap:vars.space[2],inlineSize:'min(100%, 24rem)'});
export const options=style({position:'absolute',insetInline:0,insetBlockStart:'100%',zIndex:30,maxBlockSize:'16rem',overflowY:'auto',background:vars.color.surface,color:vars.color.text,border:`${vars.border.rule} solid ${vars.color.borderStrong}`,boxShadow:vars.elevation[2],padding:vars.space[1]});
export const option=style({padding:`${vars.space[2]} ${vars.space[3]}`,cursor:'pointer',overflowWrap:'anywhere',':hover':{background:vars.color.bgAlt}});
globalStyle(`${option}[aria-selected="true"]`,{background:vars.color.bgAlt,boxShadow:`inset 3px 0 ${vars.color.accent}`});
