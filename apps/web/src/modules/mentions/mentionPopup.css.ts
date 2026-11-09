import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const popup=style({position:'fixed',zIndex:2000,maxBlockSize:'16rem',overflowY:'auto',inlineSize:'min(22rem, calc(100vw - 2rem))',background:vars.color.surface,color:vars.color.text,border:`1px solid ${vars.color.borderStrong}`,boxShadow:'0 4px 16px #0004',padding:4,margin:0,listStyle:'none'});
export const option=style({padding:'0.5rem',cursor:'pointer',selectors:{'&[aria-selected="true"]':{background:vars.color.accent,color:vars.color.accentText}}});
