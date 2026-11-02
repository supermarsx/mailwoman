import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';

// Share the app's .btn border, font, surface, hover and density rules. Only the
// icon/chevron geometry belongs here; workspace bars supply their compact size.
export const root=style({position:'relative',gap:'3px',flex:'0 0 auto',minWidth:'36px',selectors:{'&[aria-expanded="true"]':{background:vars.color.selection},'&:focus-visible':{outline:`2px solid ${vars.color.focus}`,outlineOffset:'2px'}}});
export const chevron=style({display:'inline-flex'});
globalStyle(`${root}.btn ${chevron} > svg`,{width:'10px',height:'10px'});
