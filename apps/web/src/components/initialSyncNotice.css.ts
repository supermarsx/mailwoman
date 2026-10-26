import {globalStyle} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
globalStyle('.mail-sync-notice',{margin:vars.space[3],padding:vars.space[3],border:`${vars.border.hair} solid ${vars.color.border}`,color:vars.color.textDim,flexShrink:0,overflowWrap:'anywhere'});
