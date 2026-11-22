import {style,globalStyle} from '@vanilla-extract/css';
export const mark=style({display:'inline-flex',alignItems:'center',flex:'none'});
globalStyle(`${mark} .person-avatar`,{inlineSize:'1rem',blockSize:'1rem',flexBasis:'1rem',margin:0});

// Reader marks have a 16px artwork slot, matching BrandLogo beside addresses.
globalStyle(`${mark} .person-avatar[data-sender-icon]`,{border:0,padding:0});
