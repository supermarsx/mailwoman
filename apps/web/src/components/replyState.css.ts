import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
export const indicator=style({display:'inline-flex',alignItems:'center',gap:'0.3rem',flexShrink:0,color:vars.color.accent,fontSize:'0.85em'});
export const reader=style({display:'flex',alignItems:'center',gap:'0.25rem 0.5rem',flexWrap:'wrap',minInlineSize:0,marginBlock:'0.125rem'});
// Keep draft actions together at the end, including when the notice wraps.
export const notice=style({display:'flex',alignItems:'center',justifyContent:'space-between',flexWrap:'wrap',gap:'0.25rem 0.5rem',flex:'1 1 16rem',minInlineSize:0,boxSizing:'border-box',padding:'0.125rem 0.5rem',border:`${vars.border.rule} solid ${vars.color.accent}`,borderRadius:vars.radius.sm,background:vars.color.selection,color:vars.color.accent});
export const noticeClickable=style({cursor:'pointer'});
export const noticeText=style({display:'inline-flex',alignItems:'center',gap:'0.25rem',minInlineSize:0,fontSize:'0.9em',fontWeight:600});

export const noticeActions=style({display:'inline-flex',alignItems:'center',justifyContent:'flex-end',gap:'0.25rem',flexShrink:0,marginInlineStart:'auto'});

globalStyle(`${noticeActions} .btn`,{minInlineSize:vars.a11y.touchTarget,minBlockSize:vars.a11y.touchTarget,padding:'0.125rem 0.25rem'});
