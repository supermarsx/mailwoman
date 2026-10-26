import {style} from '@vanilla-extract/css';

export const loginBackdropHost=style({position:'relative',isolation:'isolate'});
export const loginBackdrop=style({position:'absolute',inset:0,inlineSize:'100%',blockSize:'100%',objectFit:'cover',objectPosition:'center',pointerEvents:'none',zIndex:-1});
export const loginLogo=style({maxInlineSize:'12rem',maxBlockSize:'6rem',objectFit:'contain'});
export const shellLogo=style({maxInlineSize:'3rem',maxBlockSize:'2rem',objectFit:'contain'});
export const assetPreview=style({maxInlineSize:'min(100%, 12rem)',maxBlockSize:'8rem',objectFit:'contain'});
