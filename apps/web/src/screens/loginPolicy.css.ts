import {style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';

export const legal=style({position:'fixed',insetInlineStart:'50%',insetBlockEnd:'max(1rem, env(safe-area-inset-bottom))',transform:'translateX(-50%)',display:'grid',justifyItems:'center',gap:vars.space[2],maxInlineSize:'calc(calc(100vw / var(--mw-ui-zoom, 1)) - 2rem)',zIndex:20});
export const policy=style({display:'inline-flex',alignItems:'center',justifyContent:'center',gap:vars.space[2],padding:'0.65rem 1rem',background:vars.color.surface,color:vars.color.text,border:`2px solid ${vars.color.borderStrong}`,borderRadius:vars.radius.lg,boxShadow:vars.elevation[2],textDecoration:'none',whiteSpace:'nowrap',selectors:{'&:hover':{background:vars.color.bgAlt},'&:focus-visible':{outline:`2px solid ${vars.color.accent}`,outlineOffset:3}}});
export const cookies=style({fontSize:'0.75rem',padding:'0.15rem 0.45rem',color:vars.color.textDim,background:vars.color.bg,borderRadius:vars.radius.sm});
