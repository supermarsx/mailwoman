import {style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';

/** The single-line table mode uses its own virtual row height and column rhythm. */
export const table=style({display:'grid',gridTemplateColumns:'minmax(0,1fr) minmax(0,2fr) 110px',columnGap:'12px',alignItems:'center',boxSizing:'border-box',minInlineSize:0,overflow:'hidden',padding:'4px 12px',borderBlockEnd:`${vars.border.hair} solid ${vars.color.border}`,pointerEvents:'none'});

export const row=style({blockSize:'100%'});
export const sender=style({maxInlineSize:'42%'});
export const subject=style({maxInlineSize:'72%'});
export const preview=style({maxInlineSize:'88%'});
export const subjectWithoutPreview=style({maxInlineSize:'64%'});
export const date=style({inlineSize:'3rem'});
