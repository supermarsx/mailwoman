import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
// Folder drag and drop: the insertion line, the "inside" highlight, the grab handle and the nest dialog.
// A row sets `data-drop` (before | after | inside, or the same with -refused) and `--drop-depth`; the line is drawn here.
const indent='var(--folder-indent, 0.9rem)';
export const row='folder-dnd-row';
globalStyle(`[data-drop], [data-dragging]`,{position:'relative'});
globalStyle('[data-dragging="true"]',{opacity:0.5});
for(const zone of ['before','after'] as const){
 const edge=zone==='before'?{top:'-1px'}:{bottom:'-1px'};
 globalStyle(`[data-drop="${zone}"]::after`,{content:'""',position:'absolute',insetInlineStart:`calc(var(--drop-depth, 0) * ${indent})`,insetInlineEnd:0,height:'2px',borderRadius:'1px',background:vars.color.accent,pointerEvents:'none',zIndex:2,...edge});
 globalStyle(`[data-drop="${zone}"]::before`,{content:'""',position:'absolute',insetInlineStart:`calc(var(--drop-depth, 0) * ${indent} - 3px)`,width:'6px',height:'6px',borderRadius:'50%',background:vars.color.accent,pointerEvents:'none',zIndex:2,marginTop:'-2px',...(zone==='before'?{top:'-1px'}:{bottom:'-1px'})});
 globalStyle(`[data-drop="${zone}-refused"]::after`,{content:'""',position:'absolute',insetInlineStart:`calc(var(--drop-depth, 0) * ${indent})`,insetInlineEnd:0,height:'2px',background:`repeating-linear-gradient(90deg, ${vars.color.textDim} 0 4px, transparent 4px 8px)`,pointerEvents:'none',zIndex:2,...edge});
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
export const lockIcon=style({flex:'none',display:'inline-flex',alignItems:'center',color:vars.color.textDim,paddingInline:'0.25rem'});
export const choices=style({border:0,margin:0,padding:0,display:'grid',gap:'0.6rem'});
export const legend=style({fontWeight:600,marginBottom:'0.4rem'});
export const choice=style({display:'flex',gap:'0.6rem',alignItems:'flex-start',cursor:'pointer'});
export const hint=style({display:'block',color:vars.color.textDim,fontSize:'0.85em'});
export const pinnedGroup=style({display:'grid',gap:0,paddingBottom:'0.25rem',marginBottom:'0.25rem',borderBottom:`1px solid ${vars.color.borderStrong}`});
export const groupLabel=style({fontSize:'0.75em',textTransform:'uppercase',letterSpacing:'0.04em',color:vars.color.textDim,padding:'0.2rem 0.6rem'});
export const topTarget=style({margin:'0.25rem 0.6rem',padding:'0.3rem 0.5rem',border:`1px dashed ${vars.color.borderStrong}`,borderRadius:'4px',color:vars.color.textDim,fontSize:'0.85em'});
