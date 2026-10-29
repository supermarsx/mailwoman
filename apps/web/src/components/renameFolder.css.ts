// Rename folder (t65): the inline field that stands in for a folder row, and
// the dialog's field on phones.
import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';

export const field=style({display:'flex',flexDirection:'column',gap:'2px',flex:'1 1 auto',minWidth:0,padding:'2px 0'});
export const input=style({width:'100%',minWidth:0,boxSizing:'border-box',minHeight:vars.a11y.touchTarget,padding:`0 ${vars.space[2]}`,border:`${vars.border.rule} solid ${vars.color.borderStrong}`,borderRadius:vars.radius.md,background:vars.color.bgSink,color:vars.color.text,font:'inherit',
 selectors:{':root[data-appearance="light"] &':{background:vars.color.surface},'&:focus-visible':{outline:`2px solid ${vars.color.focus}`,outlineOffset:'1px'},'&[aria-invalid="true"]':{borderColor:vars.color.danger},'&:disabled':{opacity:0.7}}});
export const error=style({margin:0,fontSize:'0.8rem',lineHeight:1.3,color:vars.color.danger,overflowWrap:'anywhere'});
export const dialogField=style({display:'flex',flexDirection:'column',gap:vars.space[2]});
/** While the field is open the row's own button is hidden (kept mounted so focus can return to it). */
export const editing=style({});
globalStyle(`${editing} > .sidebar__box`,{display:'none'});
