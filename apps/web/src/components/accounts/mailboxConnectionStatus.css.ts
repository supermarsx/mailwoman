import {style,globalStyle} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const status=style({display:'inline-flex',alignItems:'center',verticalAlign:'middle',flexShrink:0,gap:'.25rem',marginInlineStart:'.375rem',marginInlineEnd:'.125rem',inlineSize:'.875rem',blockSize:'.875rem',justifyContent:'center',borderRadius:0,background:'transparent',fontSize:'.75rem',lineHeight:1,color:vars.color.textDim,selectors:{'&[data-detail]':{inlineSize:'auto',blockSize:'auto',background:'transparent',borderRadius:0},'&[data-mailbox-connection="online"]':{color:vars.color.success},'&[data-mailbox-connection="unstable"], &[data-mailbox-connection="auth-expired"], &[data-mailbox-connection="degraded"], &[data-mailbox-connection="offline"]':{color:vars.color.warning}}});

globalStyle(`${status} > svg`,{inlineSize:'10px',blockSize:'10px',flexShrink:0});
