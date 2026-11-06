import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
// Folder drag and drop: the insertion line, the "inside" highlight, the grab handle and the nest dialog.
// A row sets `data-drop` (before | after | inside, or the same with -refused) and `--drop-depth`; the line is drawn here.
const indent='var(--folder-indent, 0.9rem)';
export const row='folder-dnd-row';
globalStyle(`[data-drop], [data-dragging]`,{position:'relative'});
globalStyle('[data-dragging="true"]',{opacity:0.5});
// Touch drag: a long press lifts the row; a floating label follows the finger. The page must not select text or open a callout.
globalStyle('[data-dnd-id]',{WebkitTouchCallout:'none',WebkitUserSelect:'none',userSelect:'none'});
globalStyle('[data-lifted="true"]',{opacity:1,transform:'scale(1.03)',outline:`2px solid ${vars.color.accent}`,background:`color-mix(in srgb, ${vars.color.accent} 12%, ${vars.color.bg})`,zIndex:3,transition:'transform 120ms, box-shadow 120ms','@media':{'(prefers-reduced-motion: reduce)':{transition:'none'}}});
globalStyle('.folder-touch-ghost',{position:'fixed',zIndex:2000,pointerEvents:'none',transform:'translate(12px, -50%)',padding:'0.3rem 0.6rem',background:vars.color.bg,color:vars.color.text,border:`1px solid ${vars.color.accent}`,fontSize:'0.9rem',maxInlineSize:'60vw',overflow:'hidden',textOverflow:'ellipsis',whiteSpace:'nowrap'});
for(const zone of ['before','after'] as const){
 const edge=zone==='before'?{top:'-1px'}:{bottom:'-1px'};
 globalStyle(`[data-drop="${zone}"]::after`,{content:'""',position:'absolute',insetInlineStart:`calc(var(--drop-depth, 0) * ${indent})`,insetInlineEnd:0,height:'2px',background:vars.color.accent,pointerEvents:'none',zIndex:2,...edge});
 globalStyle(`[data-drop="${zone}"]::before`,{content:'""',position:'absolute',insetInlineStart:`calc(var(--drop-depth, 0) * ${indent} - 3px)`,width:'6px',height:'6px',borderRadius:'50%',background:vars.color.accent,pointerEvents:'none',zIndex:2,marginTop:'-2px',...(zone==='before'?{top:'-1px'}:{bottom:'-1px'})});
 globalStyle(`[data-drop="${zone}-refused"]::after`,{content:'""',position:'absolute',insetInlineStart:`calc(var(--drop-depth, 0) * ${indent})`,insetInlineEnd:0,height:0,borderBlockStart:`2px dashed ${vars.color.textDim}`,pointerEvents:'none',zIndex:2,...edge});
}
globalStyle('[data-drop="inside"]',{outline:`2px solid ${vars.color.accent}`,outlineOffset:'-2px',background:`color-mix(in srgb, ${vars.color.accent} 14%, ${vars.color.bg})`});
globalStyle('[data-drop$="-refused"], [data-drop="inside-refused"]',{cursor:'not-allowed'});
globalStyle('[data-drop="inside-refused"]',{outline:`2px dashed ${vars.color.textDim}`,outlineOffset:'-2px'});
// Forced colours drop the accent: use the system highlight.
globalStyle('[data-drop="before"]::after, [data-drop="after"]::after, [data-drop="before"]::before, [data-drop="after"]::before',{'@media':{'(forced-colors: active)':{background:'Highlight',forcedColorAdjust:'none'}}});
globalStyle('[data-drop="inside"]',{'@media':{'(forced-colors: active)':{outlineColor:'Highlight'}}});
/** The grab handle: always visible, or only on hover and focus (the user's choice). */
export const handle=style({flex:'none',display:'inline-flex',alignItems:'center',justifyContent:'center',width:'1.25rem',border:0,background:'transparent',color:vars.color.textDim,cursor:'grab',padding:0,touchAction:'none',opacity:0,selectors:{
 '&:active':{cursor:'grabbing'},
 '&:focus-visible':{opacity:1},
 '[data-handles="always"] &':{opacity:1},
}});
globalStyle(`*:hover > .${handle}, *:focus-within > .${handle}`,{opacity:1});
// The compact icon rail has no room for it; long press and keyboard moves still work there.
globalStyle('.workspace-shell[data-folder-mode="compact"] [data-testid="folder-grab-handle"]',{'@media':{'(min-width: 761px)':{display:'none'}}});
export const lockIcon=style({flex:'none',display:'inline-flex',alignItems:'center',color:vars.color.textDim,paddingInline:'0.25rem'});
export const choices=style({border:0,margin:0,padding:0,display:'grid',gap:'0.6rem'});
export const legend=style({fontWeight:600,marginBottom:'0.4rem'});
export const choice=style({display:'flex',gap:'0.6rem',alignItems:'flex-start',cursor:'pointer'});
export const hint=style({display:'block',color:vars.color.textDim,fontSize:'0.85em'});
export const pinnedGroup=style({display:'grid',gap:0,paddingBottom:'0.25rem',marginBottom:'0.25rem',borderBottom:`1px solid ${vars.color.borderStrong}`});
export const groupLabel=style({fontSize:'0.75em',textTransform:'uppercase',letterSpacing:'0.04em',color:vars.color.textDim,padding:'0.2rem 0.6rem'});
export const topTarget=style({margin:'0.25rem 0.6rem',padding:'0.3rem 0.5rem',border:`1px dashed ${vars.color.borderStrong}`,color:vars.color.textDim,fontSize:'0.85em'});
// The starred marker: a small filled star, decorative (the row's description says "Starred").
globalStyle('.sidebar__box-star',{flex:'none',display:'inline-flex',alignItems:'center',inlineSize:'0.8rem',blockSize:'0.8rem',marginInline:'0.25rem',color:vars.color.accent,'@media':{'(forced-colors: active)':{color:'Highlight',forcedColorAdjust:'none'}}});
globalStyle('.sidebar__box-star svg',{inlineSize:'100%',blockSize:'100%'});
// The icon-only rail has no name: the star sits on the icon's corner.
globalStyle('.workspace-shell[data-folder-mode="compact"] .sidebar__box',{'@media':{'(min-width: 761px)':{position:'relative'}}});
globalStyle('.workspace-shell[data-folder-mode="compact"] .sidebar__box-star',{'@media':{'(min-width: 761px)':{position:'absolute',insetBlockStart:'2px',insetInlineEnd:'4px',margin:0,inlineSize:'0.6rem',blockSize:'0.6rem'}}});
globalStyle('.sidebar__starred-only',{justifyContent:'flex-start',gap:'0.4rem',marginInline:'0.4rem',inlineSize:'calc(100% - 0.8rem)'});
globalStyle('.sidebar__starred-only[aria-pressed="true"]',{background:`color-mix(in srgb, ${vars.color.accent} 16%, ${vars.color.bg})`});
globalStyle('.workspace-shell[data-folder-mode="compact"] .sidebar__starred-only .sidebar__box-name',{'@media':{'(min-width: 761px)':{display:'none'}}});
