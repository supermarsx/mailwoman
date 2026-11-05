import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const scrim=style({position:'fixed',inset:0,background:vars.color.bg,zIndex:1000,opacity:.75});
export const dialog=style({position:'fixed',insetBlockStart:'50%',insetInlineStart:'50%',transform:'translate(-50%, -50%)',inlineSize:'min(30rem, calc(100vw - 2rem))',maxBlockSize:'calc(100vh - 2rem)',overflow:'auto',boxSizing:'border-box',background:vars.color.surface,color:vars.color.text,border:`${vars.border.rule} solid ${vars.color.borderStrong}`,padding:vars.space[6],display:'flex',flexDirection:'column',gap:vars.space[5],zIndex:1001});

export const fields=style({border:0,margin:0,padding:0,minInlineSize:0,display:'flex',flexDirection:'column',gap:vars.space[5]});

// Identity editor: fields keep a readable measure instead of stretching the card.
export const form=style({display:'flex',flexDirection:'column',maxInlineSize:'48rem',minInlineSize:0});
export const group=style({border:0,margin:0,padding:0,minInlineSize:0,display:'flex',flexDirection:'column',gap:vars.space[4],paddingBlockEnd:vars.space[5],borderBlockEnd:`${vars.border.hair} solid ${vars.color.border}`,selectors:{'&:last-of-type':{borderBlockEnd:0}}});
export const legend=style({padding:0,marginBlockEnd:vars.space[3],fontSize:'0.9rem',fontWeight:vars.weight.heavy,color:vars.color.text});
export const control=style({inlineSize:'100%',maxInlineSize:'28rem',boxSizing:'border-box'});
export const fieldError=style({fontSize:'0.85rem',color:vars.color.danger});
