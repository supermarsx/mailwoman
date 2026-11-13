import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const notice=style({display:'flex',flexDirection:'column',gap:vars.space[3],padding:vars.space[4],border:`1px solid ${vars.color.border}`,borderRadius:vars.radius.md,minInlineSize:0,selectors:{'&[data-unread="true"]':{borderInlineStart:`3px solid ${vars.color.accent}`}}});
export const facts=style({display:'grid',gridTemplateColumns:'repeat(auto-fit,minmax(min(100%, 12rem),1fr))',gap:vars.space[3],margin:0,minInlineSize:0});
export const fact=style({display:'flex',flexDirection:'column',gap:vars.space[1],minInlineSize:0});
export const label=style({fontSize:'.8rem',color:vars.color.textDim});
export const value=style({margin:0,overflowWrap:'anywhere',unicodeBidi:'plaintext'});
export const details=style({minInlineSize:0,overflowWrap:'anywhere'});
