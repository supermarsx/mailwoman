import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';

export const choices=style({display:'grid',gridTemplateColumns:'repeat(auto-fit, minmax(min(100%, 17rem), 1fr))',gap:vars.space[3],maxInlineSize:'52rem'});
export const choice=style({alignItems:'center',gap:vars.space[3],padding:vars.space[3],border:`${vars.border.hair} solid ${vars.color.border}`,borderRadius:vars.radius.md,minInlineSize:0,selectors:{'&:focus-within':{outline:`2px solid ${vars.color.accent}`,outlineOffset:'2px'},'&:has(input:disabled)':{cursor:'default'}}});
