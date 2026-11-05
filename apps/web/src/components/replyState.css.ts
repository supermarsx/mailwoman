import {style,globalStyle} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
import {actionsOpen} from './messageRow.css.ts';
export const indicator=style({display:'inline-flex',alignItems:'center',gap:'0.3rem',flexShrink:0,color:vars.color.accent,fontSize:'0.85em'});
export const reader=style({display:'flex',alignItems:'center',gap:'0.5rem 0.75rem',flexWrap:'wrap',minInlineSize:0,marginBlock:'0.25rem'});
// An alert-style bar: one row, even inset, the action at the end; wraps on narrow panes.
export const notice=style({display:'flex',alignItems:'center',justifyContent:'space-between',flexWrap:'wrap',gap:'0.5rem 0.75rem',flex:'1 1 16rem',minInlineSize:0,boxSizing:'border-box',padding:'0.375rem 0.75rem',border:`${vars.border.rule} solid ${vars.color.accent}`,borderRadius:vars.radius.sm,background:vars.color.selection,color:vars.color.accent});
export const noticeClickable=style({cursor:'pointer'});
export const noticeText=style({display:'inline-flex',alignItems:'center',gap:'0.375rem',minInlineSize:0,fontSize:'0.9em',fontWeight:600});
export const rowControl=style({
  position:'absolute',insetInlineEnd:'3.5rem',insetBlockStart:'calc(50% - 16px)',zIndex:2,
  display:'inline-flex',alignItems:'center',justifyContent:'center',inlineSize:32,blockSize:32,
  border:`${vars.border.rule} solid ${vars.color.borderStrong}`,borderRadius:vars.radius.sm,
  background:vars.color.surface,color:vars.color.accent,cursor:'pointer',
  selectors:{'&:hover':{background:vars.color.selection}},
  '@media':{'(pointer: coarse)':{inlineSize:44,blockSize:'min(44px, calc(100% - 4px))',insetBlockStart:'50%',transform:'translateY(-50%)'}},
});
globalStyle(`li[data-reply-draft] .list__subject,li[data-reply-draft] .list__preview`,{paddingInlineEnd:'6.5rem',boxSizing:'border-box'});
// Keep the direct draft control beside the existing row action surface. On
// narrow/touch rows its expanded alternative temporarily owns that space.
globalStyle('li[data-reply-draft] .msg-actions',{'@media':{'(hover: hover) and (min-width: 761px)':{insetInlineEnd:'6rem'}}});
globalStyle(`li[data-reply-draft]:has(.${actionsOpen}) > .${rowControl}`,{'@media':{'(hover: none), (max-width: 760px)':{display:'none'}}});
