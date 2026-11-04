import {globalStyle,style} from '@vanilla-extract/css';
export const field=style({minInlineSize:0,maxInlineSize:'100%'});
globalStyle(`${field} button[aria-haspopup="menu"]`,{inlineSize:'100%',minInlineSize:0,justifyContent:'space-between'});
globalStyle(`${field} button[aria-haspopup="menu"] > span:not(:last-child)`,{flex:'1 1 auto',minInlineSize:0,overflow:'hidden',textOverflow:'ellipsis',whiteSpace:'nowrap',textAlign:'start'});
globalStyle(`${field} button[aria-haspopup="menu"] > svg`,{flexShrink:0});
