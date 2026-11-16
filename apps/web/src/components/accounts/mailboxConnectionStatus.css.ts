import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const status=style({display:'inline-flex',alignItems:'center',verticalAlign:'middle',flexShrink:0,gap:'.3rem',marginInlineStart:'.35rem',fontSize:'.75rem',lineHeight:1.3,color:vars.color.textDim,selectors:{'&[data-mailbox-connection="online"]':{color:vars.color.success},'&[data-mailbox-connection="auth-expired"], &[data-mailbox-connection="degraded"], &[data-mailbox-connection="offline"]':{color:vars.color.warning}}});
