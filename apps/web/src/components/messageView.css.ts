import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
export const cell=style({minInlineSize:0,overflow:'hidden',textOverflow:'ellipsis',whiteSpace:'nowrap',display:'flex',alignItems:'center',gap:'.3rem',gridRow:1,fontSize:'var(--mail-row-font,14px)'});
export const tablePreview=style({gridColumn:'1 / -1',gridRow:2});
export const headers=style({minInlineSize:'var(--folder-view-width)',display:'grid',gap:'12px',padding:'4px 12px',fontSize:'.8rem',color:vars.color.textDim});
globalStyle('.list[data-view-highlight="none"] .list__row--unread',{background:'transparent'});
globalStyle('.list[data-view-highlight="unread"] .list__row--unread',{background:`color-mix(in srgb, ${vars.color.accent} 12%, ${vars.color.bg})`});
globalStyle('.list[data-view-highlight="flagged"] .list__row:has([data-testid="row-flagged"])',{background:`color-mix(in srgb, ${vars.color.warning} 12%, ${vars.color.bg})`});
globalStyle('.list[data-view-highlight="important"] .list__row:has([data-priority="high"])',{background:`color-mix(in srgb, ${vars.color.danger} 10%, ${vars.color.bg})`});
globalStyle('.list[data-list-style="table"] .list__row',{gridTemplateColumns:'var(--folder-view-columns)',minInlineSize:'var(--folder-view-width)',rowGap:'var(--mail-row-gap,2px)'});
globalStyle('.list .list__row',{fontFamily:'var(--folder-view-font,inherit)',fontSize:'var(--mail-row-font,14px)'});

export const headerViewport=style({overflow:'hidden',flexShrink:0});
export const tableOptions=style({display:'flex',justifyContent:'flex-end',padding:'2px 12px',minInlineSize:0,flexShrink:0});
globalStyle(`${tableOptions} select`,{maxInlineSize:'16rem',minInlineSize:0,font:'inherit',fontSize:'.8rem'});

globalStyle('.list:not([data-list-style="table"]) .list__row',{gridTemplateRows:'var(--mail-row-line,18px) var(--mail-row-line,18px) minmax(0,1fr)'});
