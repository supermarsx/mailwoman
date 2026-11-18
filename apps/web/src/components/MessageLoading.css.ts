import {style,keyframes} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';

/** Reader-only spacing; compact loading labels elsewhere keep their own layout. */
export const loading=style({selectors:{'&[data-visible="true"]':{
  margin:'0.5rem 1rem',paddingBlock:'0.375rem',gap:vars.space[2],
  maxInlineSize:'calc(100% - 2rem)',fontSize:'0.9rem',lineHeight:1.5,
}}});
const sweep=keyframes({from:{backgroundPosition:'200% 0'},to:{backgroundPosition:'-200% 0'}});
export const skeleton=style({display:'flex',flexDirection:'column',gap:vars.space[5],inlineSize:'calc(100% - 2rem)',maxInlineSize:'56rem',boxSizing:'border-box',margin:'1rem',paddingBlock:vars.space[2],minBlockSize:'12rem'});
export const paragraph=style({display:'flex',flexDirection:'column',gap:vars.space[3]});
export const line=style({display:'block',blockSize:'.7rem',inlineSize:'92%',borderRadius:vars.radius.sm,background:`linear-gradient(90deg, ${vars.color.bgAlt} 25%, color-mix(in srgb, ${vars.color.textDim} 18%, ${vars.color.surface}) 50%, ${vars.color.bgAlt} 75%)`,backgroundSize:'200% 100%',animation:`${sweep} 1.8s linear infinite`,selectors:{'&:nth-child(4n+1)':{inlineSize:'76%'},'&:nth-child(4n+3)':{inlineSize:'86%'},'&:nth-child(4n)':{inlineSize:'58%'}},'@media':{'(prefers-reduced-motion: reduce)':{animation:'none'},'(forced-colors: active)':{background:'GrayText'}}});
