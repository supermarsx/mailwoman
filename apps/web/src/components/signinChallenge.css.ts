import {style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
export const verification=style({
  minInlineSize:0,
  paddingBlock:vars.space[3],
  vars:{
    '--altcha-color-base':vars.color.surface,
    '--altcha-color-base-content':vars.color.text,
    '--altcha-color-primary':vars.color.accent,
    '--altcha-color-primary-content':vars.color.accentText,
    '--altcha-color-neutral':vars.color.bgAlt,
    '--altcha-color-neutral-content':vars.color.textDim,
    '--altcha-border-color':vars.color.borderStrong,
    '--altcha-checkbox-border-color':vars.color.borderStrong,
    '--altcha-checkbox-outline-color':vars.color.focus,
    '--altcha-border-width':'2px',
    '--altcha-border-radius':'0',
    '--altcha-checkbox-border-radius':'0',
    '--altcha-shadow':'none',
    '--altcha-max-width':'100%',
    '--altcha-padding':vars.space[5],
  },
});
