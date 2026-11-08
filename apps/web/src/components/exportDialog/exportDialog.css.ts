// Extra styles of the export dialog; the frame, grid and note styles come from the
// Properties dialogs. Tokens only; borders stay visible in forced colours.
import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';

export const scopes=style({display:'flex',flexWrap:'wrap',gap:vars.space[1],border:'none',margin:0,padding:0});
export const scope=style({display:'inline-flex',alignItems:'center',gap:vars.space[1],padding:`${vars.space[1]} ${vars.space[3]}`,border:`1px solid ${vars.color.border}`,borderRadius:vars.radius.sm,cursor:'pointer',
 selectors:{'&:has(input:checked)':{borderColor:vars.color.accent,background:vars.color.surface},'&:has(input:focus-visible)':{outline:`2px solid ${vars.color.accent}`,outlineOffset:'2px'}}});
export const section=style({display:'flex',flexDirection:'column',gap:vars.space[2],border:'none',margin:0,padding:0,minWidth:0});
export const legend=style({fontWeight:vars.weight.bold,padding:0,marginBottom:vars.space[1]});
export const field=style({display:'flex',flexWrap:'wrap',alignItems:'center',gap:vars.space[2]});
export const input=style({font:'inherit',minWidth:0,maxWidth:'100%',boxSizing:'border-box'});
export const progress=style({width:'100%',height:'8px',accentColor:vars.color.accent,'@media':{'(forced-colors: active)':{forcedColorAdjust:'auto'}}});
export const verdict=style({margin:0,padding:vars.space[2],border:`1px solid ${vars.color.border}`,borderRadius:vars.radius.sm});
export const footer=style({display:'flex',flexWrap:'wrap',alignItems:'center',justifyContent:'space-between',gap:vars.space[2]});
export const body=style({overflowY:'auto',minHeight:0});
