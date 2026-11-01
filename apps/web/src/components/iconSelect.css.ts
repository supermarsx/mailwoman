import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';

export const root=style({position:'relative',display:'inline-flex',alignItems:'center',justifyContent:'center',gap:'3px',flex:'0 0 auto',minWidth:'36px',minHeight:vars.a11y.touchTarget,padding:'0 5px',boxSizing:'border-box',color:vars.color.text,borderRadius:vars.radius.sm,cursor:'pointer',selectors:{'&:hover':{background:vars.color.bgAlt},'&:focus-within':{outline:`2px solid ${vars.color.focus}`,outlineOffset:'2px'}}});
export const chevron=style({display:'inline-flex'});
globalStyle(`${chevron} > svg`,{width:'10px',height:'10px'});
// The native select covers the whole trigger; its popup retains the option labels.
export const select=style({position:'absolute',inset:0,width:'100%',height:'100%',margin:0,opacity:0,cursor:'pointer',font:'inherit'});
