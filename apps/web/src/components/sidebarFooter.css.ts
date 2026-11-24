import {globalStyle,style} from '@vanilla-extract/css';
export const actions=style({display:'flex',alignItems:'stretch',gap:'.25rem',flexWrap:'wrap',minInlineSize:0});

// Both session actions share their vertical metrics, including touch targets.
globalStyle(`${actions} > button.btn`,{minBlockSize:'2.75rem',paddingBlock:'.5rem',lineHeight:1.25});
