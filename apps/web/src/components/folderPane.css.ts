import {style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
export const pane=style({display:'flex',flexDirection:'column',height:'100%',minHeight:0,minWidth:0});
export const toolbar=style({display:'flex',alignItems:'center',justifyContent:'space-between',gap:'.5rem',padding:'.5rem',flexWrap:'wrap',flexShrink:0});
export const confirmation=style({padding:'.5rem',border:`2px solid ${vars.color.warning}`,background:vars.color.surface,color:vars.color.text});
export const viewport=style({overflow:'auto',flex:'1 1 0',minHeight:0,padding:'.25rem'});
export const list=style({listStyle:'none',margin:0,padding:0});
export const row=style({display:'flex',alignItems:'center',gap:'.5rem',padding:'.25rem',borderBlockEnd:`1px solid ${vars.color.border}`,selectors:{'&[data-selected="true"]':{outline:`1px solid ${vars.color.focus}`,outlineOffset:-1},'&[data-unread="true"]:not([data-selected="true"])':{background:`color-mix(in srgb, ${vars.color.accent} 9%, ${vars.color.bg})`},'&[data-unread="true"]:not([data-selected="true"]):hover':{background:`color-mix(in srgb, ${vars.color.accent} 14%, ${vars.color.bgAlt})`}}});
export const message=style({display:'flex',flexDirection:'column',alignItems:'stretch',gap:'.25rem',textAlign:'start',minWidth:0,flex:'1 1 0',padding:'.5rem',background:'transparent',color:vars.color.text,border:0,font:'inherit',overflowWrap:'anywhere',cursor:'pointer',selectors:{'&:focus-visible':{outline:`2px solid ${vars.color.focus}`,outlineOffset:-2}}});
