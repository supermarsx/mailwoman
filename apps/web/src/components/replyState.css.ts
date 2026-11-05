import {style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
export const indicator=style({display:'inline-flex',alignItems:'center',gap:'0.3rem',flexShrink:0,color:vars.color.accent,fontSize:'0.85em'});
export const reader=style({display:'flex',alignItems:'center',gap:'0.5rem 0.75rem',flexWrap:'wrap',minInlineSize:0,marginBlock:'0.25rem'});
// An alert-style bar: one row, even inset, the action at the end; wraps on narrow panes.
export const notice=style({display:'flex',alignItems:'center',justifyContent:'space-between',flexWrap:'wrap',gap:'0.5rem 0.75rem',flex:'1 1 16rem',minInlineSize:0,boxSizing:'border-box',padding:'0.375rem 0.75rem',border:`${vars.border.rule} solid ${vars.color.accent}`,borderRadius:vars.radius.sm,background:vars.color.selection,color:vars.color.accent});
export const noticeClickable=style({cursor:'pointer'});
export const noticeText=style({display:'inline-flex',alignItems:'center',gap:'0.375rem',minInlineSize:0,fontSize:'0.9em',fontWeight:600});
