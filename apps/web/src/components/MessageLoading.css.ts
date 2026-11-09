import {style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';

/** Reader-only spacing; compact loading labels elsewhere keep their own layout. */
export const loading=style({selectors:{'&[data-visible="true"]':{
  margin:'0.5rem 1rem',paddingBlock:'0.375rem',gap:vars.space[2],
  maxInlineSize:'calc(100% - 2rem)',fontSize:'0.9rem',lineHeight:1.5,
}}});
