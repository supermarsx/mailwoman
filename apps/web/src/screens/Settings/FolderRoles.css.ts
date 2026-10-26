import {style} from '@vanilla-extract/css';
export const panel=style({minInlineSize:0,marginBlock:'1rem'});
export const grid=style({display:'grid',gridTemplateColumns:'repeat(auto-fit,minmax(min(100%,16rem),1fr))',gap:'1rem',minInlineSize:0});
export const select=style({inlineSize:'100%',maxInlineSize:'28rem',minInlineSize:0,boxSizing:'border-box'});
