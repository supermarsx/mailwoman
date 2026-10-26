import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const policyText=style({minInlineSize:0,overflowWrap:'anywhere'});
export const policyAction=style({flexShrink:0,whiteSpace:'nowrap',selectors:{'&:focus-visible':{outline:'none',boxShadow:vars.a11y.focusRing}}});
