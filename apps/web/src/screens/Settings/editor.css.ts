import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const scrim=style({position:'fixed',inset:0,background:vars.color.bg,zIndex:1000,opacity:.75});
export const dialog=style({position:'fixed',insetBlockStart:'50%',insetInlineStart:'50%',transform:'translate(-50%, -50%)',inlineSize:'min(30rem, calc(100vw - 2rem))',maxBlockSize:'calc(100vh - 2rem)',overflow:'auto',boxSizing:'border-box',background:vars.color.surface,color:vars.color.text,border:`${vars.border.rule} solid ${vars.color.borderStrong}`,padding:vars.space[6],display:'flex',flexDirection:'column',gap:vars.space[5],zIndex:1001});

export const fields=style({border:0,margin:0,padding:0,minInlineSize:0,display:'flex',flexDirection:'column',gap:vars.space[5]});
