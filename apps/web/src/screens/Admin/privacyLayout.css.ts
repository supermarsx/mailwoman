import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const policyText=style({minInlineSize:0,overflowWrap:'anywhere'});
export const policyAction=style({flexShrink:0,whiteSpace:'nowrap',selectors:{'&:focus-visible':{outline:'none',boxShadow:vars.a11y.focusRing}}});
export const concurrency=style({display:'grid',gap:vars.space[2],minInlineSize:0});
export const profileControls=style({display:'flex',alignItems:'center',flexWrap:'wrap',gap:vars.space[2]});
export const capacities=style({display:'grid',gridTemplateColumns:'repeat(auto-fit,minmax(min(100%,9rem),1fr))',gap:vars.space[2],margin:0,minInlineSize:0});
