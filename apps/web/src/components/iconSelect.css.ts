import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';

export const root=style({position:'relative',display:'inline-flex',alignItems:'center',justifyContent:'center',gap:'3px',flex:'0 0 auto',minWidth:'36px',minHeight:vars.a11y.touchTarget,padding:'0 5px',boxSizing:'border-box',font:'inherit',color:vars.color.text,background:vars.color.surface,border:`1px solid ${vars.color.border}`,borderRadius:vars.radius.sm,cursor:'pointer',selectors:{'&:hover':{background:vars.color.bgAlt},'&[aria-expanded="true"]':{background:vars.color.selection,borderColor:vars.color.borderStrong},'&:focus-visible':{outline:`2px solid ${vars.color.focus}`,outlineOffset:'2px'}}});
export const chevron=style({display:'inline-flex'});
globalStyle(`${chevron} > svg`,{width:'10px',height:'10px'});
