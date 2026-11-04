import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
export const menu=style({position:'fixed',zIndex:1500,minWidth:'11rem',maxWidth:'calc(100vw - 16px)',maxHeight:'calc(100dvh - 16px)',overflowY:'auto',overscrollBehavior:'contain',scrollbarWidth:'thin',scrollbarColor:`${vars.color.borderStrong} transparent`,padding:'2px',fontSize:'0.875rem',background:vars.color.bg,color:vars.color.text,border:`2px solid ${vars.color.borderStrong}`,boxShadow:vars.elevation[3]});
const accentSurface=`color-mix(in srgb, ${vars.color.accent} 16%, ${vars.color.bg})`;
const accentHover=`color-mix(in srgb, ${vars.color.accent} 26%, ${vars.color.bg})`;
export const item=style({display:'flex',alignItems:'center',gap:'0.5rem',width:'100%',minHeight:'28px',padding:'3px 10px',lineHeight:1.25,'@media':{'(pointer: coarse)':{minHeight:'36px',padding:'6px 12px'}},border:0,borderInlineStart:'3px solid transparent',background:'transparent',color:'inherit',font:'inherit',textAlign:'start',cursor:'pointer',selectors:{'&:focus-visible':{outline:`2px solid ${vars.color.accent}`,outlineOffset:'-2px'},'&:hover:not([aria-disabled="true"])':{background:`color-mix(in srgb, ${vars.color.accent} 8%, ${vars.color.bg})`},'&[aria-disabled="true"]':{opacity:0.5,cursor:'default'},'&[aria-checked="true"], &[data-current="true"]':{background:accentSurface,borderInlineStartColor:vars.color.accent,fontWeight:vars.weight.medium},'&[aria-checked="true"]:hover:not([aria-disabled="true"]), &[data-current="true"]:hover:not([aria-disabled="true"])':{background:accentHover}}});
export const separator=style({height:'1px',margin:'2px 6px',background:vars.color.borderStrong});
export const folderRow=style({display:'flex',alignItems:'stretch',minWidth:0,paddingInlineStart:'calc(var(--folder-depth, 0) * 0.9rem)'});
// The icon-only rail has no room for nesting or twisties.
globalStyle('.workspace-shell[data-folder-mode="compact"] .sidebar__twisty',{'@media':{'(min-width: 761px)':{display:'none'}}});
globalStyle(`.workspace-shell[data-folder-mode="compact"] .${folderRow}`,{'@media':{'(min-width: 761px)':{paddingInlineStart:0}}});
globalStyle('.sidebar__twisty',{flex:'none',display:'inline-flex',alignItems:'center',justifyContent:'center',width:'1.75rem',border:0,background:'transparent',color:'inherit',cursor:'pointer'});
globalStyle(`.${folderRow} > .sidebar__box`,{flex:'1 1 auto',minWidth:0,width:'auto'});
/** The trailing check on the current option (`ContextAction.current`). */
export const current=style({marginInlineStart:'auto',paddingInlineStart:'0.75rem',display:'inline-flex',color:vars.color.accent});

// A menu has keyboard focus; its target keeps a separate visual cue without
// changing the open folder/message or the user's bulk selection.
globalStyle('.list__row[data-context-open="true"]',{background:accentSurface,outline:`1px solid ${vars.color.accent}`,outlineOffset:'-1px'});
globalStyle(`${folderRow}[data-context-open="true"] > .sidebar__box`,{background:accentSurface,outline:`1px solid ${vars.color.accent}`,outlineOffset:'-1px'});
globalStyle(`.${menu}::-webkit-scrollbar`,{width:'8px'});
globalStyle(`.${menu}::-webkit-scrollbar-thumb`,{background:vars.color.borderStrong,borderRadius:vars.radius.sm});
