// Settings → Toolbars (t38-e2): a bar's radio pair is a bare fieldset.
import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';

export const group = style({border: 'none', margin: 0, padding: 0, minInlineSize: 0});
export const choices=style({display:'inline-flex',alignItems:'center',alignSelf:'flex-start',gap:vars.space[1]});
export const choice=style({display:'inline-flex',alignItems:'center',justifyContent:'center',padding:0,minInlineSize:`max(1.75rem,${vars.a11y.touchTarget})`,minBlockSize:`max(1.75rem,${vars.a11y.touchTarget})`,selectors:{'&[aria-checked="true"]':{background:vars.color.accent,color:vars.color.accentText,borderColor:vars.color.accent},'&:focus-visible':{outline:`2px solid ${vars.color.focus}`,outlineOffset:'2px'}},'@media':{'(pointer: coarse)':{minInlineSize:'44px',minBlockSize:'44px'}}});
