import {style,globalStyle} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const status=style({display:'inline-flex',alignItems:'center',verticalAlign:'middle',flexShrink:0,gap:'.25rem',marginInlineStart:'.25rem',inlineSize:'1rem',blockSize:'1rem',justifyContent:'center',borderRadius:'50%',background:'color-mix(in srgb, currentColor 10%, transparent)',fontSize:'.75rem',lineHeight:1,color:vars.color.textDim,selectors:{'&[data-detail]':{inlineSize:'auto',blockSize:'auto',background:'transparent',borderRadius:0},'&[data-mailbox-connection="online"]':{color:vars.color.success},'&[data-mailbox-connection="auth-expired"], &[data-mailbox-connection="degraded"], &[data-mailbox-connection="offline"]':{color:vars.color.warning}}});

globalStyle(`${status} > svg`,{inlineSize:'12px',blockSize:'12px',flexShrink:0});
