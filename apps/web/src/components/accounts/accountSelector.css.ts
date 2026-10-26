import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';

export const control=style({display:'flex',alignItems:'center',gap:'.25rem',minWidth:0,maxWidth:'100%',flexWrap:'wrap',flexShrink:0});
export const refreshHelp=style({flexShrink:0,display:'inline-flex',alignItems:'center',justifyContent:'center',width:'2rem',minHeight:'2rem',color:vars.color.text});
export const select=style({font:'inherit',fontSize:'.875rem',color:vars.color.text,background:vars.color.surface,border:`1px solid ${vars.color.borderStrong}`,borderRadius:'.25rem',padding:'.35rem .5rem',minHeight:'2rem',minWidth:0,maxWidth:'min(22rem, 70vw)',textOverflow:'ellipsis',selectors:{'&:focus-visible':{outline:`2px solid ${vars.color.focus}`,outlineOffset:2}}});
export const manage=style({flexShrink:0,minWidth:'2rem',minHeight:'2rem'});
export const error=style({flexBasis:'100%',fontSize:'.875rem',color:vars.color.danger,overflowWrap:'anywhere'});
