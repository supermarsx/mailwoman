import {style,globalStyle} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
export const pane=style({display:'flex',flexDirection:'column',height:'100%',minHeight:0,minWidth:0});
export const toolbar=style({display:'flex',alignItems:'center',justifyContent:'space-between',gap:'.5rem',padding:'.5rem',flexWrap:'wrap',flexShrink:0});
export const confirmation=style({padding:'.5rem',border:`2px solid ${vars.color.warning}`,background:vars.color.surface,color:vars.color.text});
export const viewport=style({overflow:'auto',flex:'1 1 0',minHeight:0,padding:0});
export const list=style({listStyle:'none',margin:0,padding:0});
export const row=style({display:'flex',alignItems:'center',minWidth:0,position:'relative'});
export const message=style({minWidth:0,flex:'1 1 0',selectors:{'&&':{height:'auto',minHeight:'calc(var(--mail-row-line, 18px) * 2 + var(--mail-preview-line, 16px) + var(--mail-row-padding, 7px) * 2 + var(--mail-row-gap, 2px) * 2)'},'&:focus-visible':{outline:`2px solid ${vars.color.focus}`,outlineOffset:-2}}});

globalStyle(`${row} > input`,{flexShrink:0,marginInline:'.5rem'});
