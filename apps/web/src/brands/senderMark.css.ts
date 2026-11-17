import {style,globalStyle} from '@vanilla-extract/css';
export const mark=style({display:'inline-flex',alignItems:'center',flex:'none'});
globalStyle(`${mark} .person-avatar`,{inlineSize:'1rem',blockSize:'1rem',flexBasis:'1rem',margin:0});
