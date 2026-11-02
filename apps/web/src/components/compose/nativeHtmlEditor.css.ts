import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const frame=style({display:'block',width:'100%',minHeight:'18rem',border:`${vars.border.rule} solid ${vars.color.borderStrong}`,background:'#fff'});
export const toolbar=style({display:'flex',flexWrap:'wrap',gap:vars.space[1],marginBlockEnd:vars.space[2]});
export const views=style({display:'flex',gap:vars.space[1],marginBlockEnd:vars.space[2]});
export const viewTab=style({selectors:{'&[aria-selected="true"]':{background:vars.color.selection,color:vars.color.text,borderColor:vars.color.borderStrong}}});
