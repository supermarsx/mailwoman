import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
import * as linked from './linkedAccounts.css.ts';

/** The same compact management rows as linked mailboxes, without a drag grip. */
export const row=style([linked.row,{
 gridTemplateColumns:'auto minmax(0, 1fr) auto',
 gridTemplateAreas:'"avatar who actions"',
 paddingInline:vars.space[3],
 '@media':{'screen and (max-width: 600px)':{gridTemplateColumns:'auto minmax(0, 1fr)',gridTemplateAreas:'"avatar who" ". actions"'}},
}]);
export const avatar=style({gridArea:'avatar',display:'inline-flex',alignItems:'center',justifyContent:'center',inlineSize:'2rem',blockSize:'2rem',borderRadius:vars.radius.md,background:vars.color.bgAlt,color:vars.color.textDim});
export const name=style({display:'flex',alignItems:'center',gap:vars.space[2],minInlineSize:0,flexWrap:'wrap'});
export const label=style({overflow:'hidden',textOverflow:'ellipsis',whiteSpace:'nowrap',minInlineSize:0,fontWeight:vars.weight.bold});
export const detail=style({margin:0,fontSize:'.8rem',color:vars.color.textDim,overflowWrap:'anywhere'});
export const variant=style({paddingInlineStart:vars.space[4],borderInlineStart:`3px solid ${vars.color.border}`});
export const choice=style({gridColumn:'1 / -1',padding:vars.space[3],borderBlockStart:`${vars.border.hair} solid ${vars.color.border}`,minInlineSize:0});
