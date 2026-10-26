import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const tabs=style({display:'flex',gap:vars.space[1],overflowX:'auto',borderBottom:`1px solid ${vars.color.border}`,marginBottom:vars.space[4]});
export const tab=style({flex:'0 0 auto',padding:`${vars.space[3]} ${vars.space[4]}`,background:'transparent',color:vars.color.textDim,border:'1px solid transparent',borderBottom:0,cursor:'pointer',selectors:{'&[aria-selected="true"]':{background:vars.color.surface,color:vars.color.text,borderColor:vars.color.border,boxShadow:`inset 0 3px 0 ${vars.color.accent}`},'&:focus-visible':{outline:`2px solid ${vars.a11y.focusRingColor}`,outlineOffset:-3}}});
export const value=style({whiteSpace:'pre-wrap',overflowWrap:'anywhere',minWidth:'14rem',maxWidth:'32rem',margin:0,fontSize:'0.8rem'});
