import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const root=style({minInlineSize:0,minBlockSize:0});
export const panel=style({display:'flex',flexDirection:'column',flex:'1 1 0',minInlineSize:0,minBlockSize:0});
export const visual=style({display:'flex',flexDirection:'column',flex:'1 1 auto',minInlineSize:0,minBlockSize:0});
export const frame=style({display:'block',boxSizing:'border-box',width:'100%',height:'18rem',flex:'1 1 18rem',minHeight:0,border:`${vars.border.rule} solid ${vars.color.borderStrong}`,borderRadius:vars.radius.md,background:vars.color.bg,color:vars.color.text,
 selectors:{'&[data-surface="dark"]':{background:'#1e1f22',color:'#e8e8ea',colorScheme:'dark'},'&[data-surface="light"]':{background:'#ffffff',color:'#1f1f1f',colorScheme:'light'}},
});
export const toolbar=style({display:'flex',flexWrap:'wrap',gap:vars.space[1],marginBlockEnd:vars.space[2]});
export const views=style({display:'flex',flexDirection:'column',gap:vars.space[1]});
export const viewTab=style({selectors:{'&[aria-selected="true"]':{background:vars.color.selection,color:vars.color.text,borderColor:vars.color.borderStrong}}});
