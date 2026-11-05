import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';

// Two split buttons (action + list caret) in the mailbox toolbar. They reuse the
// .btn surface, border and density rules; only the grouping geometry is here,
// in logical properties so a right-to-left layout mirrors.
export const group=style({display:'inline-flex',alignItems:'center',gap:vars.space[1],flex:'0 0 auto'});
export const split=style({display:'inline-flex',alignItems:'stretch',flex:'0 0 auto'});
export const main=style({minWidth:'32px',borderStartEndRadius:0,borderEndEndRadius:0,selectors:{'&:focus-visible':{outline:`2px solid ${vars.color.focus}`,outlineOffset:'2px'}}});
export const caret=style({minWidth:'18px',paddingInline:'2px',borderStartStartRadius:0,borderEndStartRadius:0,borderInlineStartWidth:0,selectors:{'&[aria-expanded="true"]':{background:vars.color.selection},'&:focus-visible':{outline:`2px solid ${vars.color.focus}`,outlineOffset:'2px'}}});
globalStyle(`${caret} > svg`,{width:'10px',height:'10px'});
export const progress=style({fontSize:'0.75rem',color:vars.color.textDim,whiteSpace:'nowrap'});
export const live=style({position:'absolute',width:'1px',height:'1px',margin:'-1px',padding:0,overflow:'hidden',clip:'rect(0 0 0 0)',whiteSpace:'nowrap',border:0});
