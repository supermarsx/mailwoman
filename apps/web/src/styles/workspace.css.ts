import { globalStyle } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';
const desktop='(min-width: 761px)';
globalStyle('.shell.workspace-shell',{gridTemplateColumns:'var(--folder-width,220px) var(--folder-handle,6px) minmax(0,1fr)' ,gridTemplateRows:'minmax(0,1fr)'});
globalStyle(':root[data-reading-pane] .shell.workspace-shell',{'@media':{[desktop]:{gridTemplateColumns:'var(--folder-width,220px) var(--folder-handle,6px) minmax(0,1fr)' ,gridTemplateRows:'minmax(0,1fr)'}}});
globalStyle(':root[data-reading-pane] .workspace-shell > .sidebar',{'@media':{[desktop]:{gridColumn:'1',gridRow:'1'}}});
globalStyle('.mail-workspace',{display:'flex',flexDirection:'column',minWidth:0,minHeight:0,overflow:'hidden',gridColumn:'3 / -1',gridRow:'1',flex:'1 1 0'});
globalStyle('.mail-workspace[hidden], .mail-workspace [hidden]',{display:'none'});
globalStyle('.workspace-bar',{display:'flex',alignItems:'center',gap:vars.space[1],padding:'2px 4px',flexShrink:0,borderBlockEnd:`${vars.border.hair} solid ${vars.color.border}`,overflowX:'auto'});
globalStyle('.workspace-bar .btn',{flex:'none','@media':{'(min-width: 761px) and (pointer: fine)':{minHeight:'28px',paddingBlock:'2px'}}});
globalStyle('.workspace-body',{display:'grid',gridTemplateColumns:'minmax(0,1fr)',gridTemplateRows:'minmax(0,1fr)',alignItems:'stretch',minHeight:0,minWidth:0,flex:'1 1 0',overflow:'hidden'});
globalStyle('.workspace-body--split',{gridTemplateColumns:'var(--list-size,340px) 6px minmax(0,1fr)'});
globalStyle(':root[data-reading-pane] .mail-workspace .mail-pane',{gridColumn:'auto',gridRow:'auto',minHeight:0});
globalStyle('.workspace-panes',{display:'grid',gridTemplateColumns:'minmax(0,1fr)',minWidth:0,minHeight:0,overflow:'auto'});
globalStyle('.workspace-panes[data-layout="side-by-side"]',{gridTemplateColumns:'repeat(auto-fit,minmax(min(100%,320px),1fr))',gridAutoRows:'minmax(320px,1fr)'});
globalStyle('.workspace-panes[data-layout="stack"]',{gridTemplateColumns:'minmax(0,1fr)',gridAutoRows:'minmax(300px,1fr)'});
globalStyle('.workspace-pane',{minWidth:0,minHeight:0,overflow:'auto',borderInlineStart:`${vars.border.hair} solid ${vars.color.border}`});
globalStyle('.workspace-shell .mail-workspace .reader, :root[data-reading-pane] .mail-workspace .reader.reader--open',{position:'static',inset:'auto',gridColumn:'auto',gridRow:'auto',height:'100%',width:'100%',maxHeight:'none',minHeight:0,zIndex:'auto',display:'flex'});
globalStyle('.workspace-compose',{height:'100%',minHeight:0,overflow:'auto'});
globalStyle('.workspace-compose > .compose',{width:'100%',maxWidth:'none',maxHeight:'none',minHeight:'100%',border:0,boxShadow:'none'});

globalStyle('.workspace-shell > [role="separator"]',{gridColumn:'2',gridRow:'1'});
globalStyle('.workspace-shell > .sidebar[hidden]',{display:'none'});
globalStyle('.workspace-shell > .module-pane, .workspace-shell > .outbox, .workspace-shell > .attachments-pane',{gridColumn:'3 / -1'});
globalStyle('.workspace-body--split[data-reading-pane="bottom"]',{gridTemplateColumns:'minmax(0,1fr)',gridTemplateRows:'var(--list-size,320px) 6px minmax(0,1fr)'});
globalStyle('.workspace-panes[data-pair="true"][data-layout="side-by-side"]',{gridTemplateColumns:'var(--tile-first,50%) 6px minmax(0,1fr)',gridTemplateRows:'minmax(0,1fr)',gridAutoRows:'auto'});
globalStyle('.workspace-panes[data-pair="true"][data-layout="stack"]',{gridTemplateColumns:'minmax(0,1fr)',gridTemplateRows:'var(--tile-first,50%) 6px minmax(0,1fr)',gridAutoRows:'auto'});
globalStyle('.workspace-shell[data-folder-mode="compact"] .sidebar__box-name, .workspace-shell[data-folder-mode="compact"] .sidebar__brand, .workspace-shell[data-folder-mode="compact"] .sidebar__user',{'@media':{[desktop]:{display:'none'}}});
globalStyle('.workspace-shell[data-folder-mode="compact"] .sidebar',{'@media':{[desktop]:{paddingInline:vars.space[1]}}});
globalStyle('.workspace-shell[data-folder-mode="compact"] .sidebar__box',{'@media':{[desktop]:{justifyContent:'center',position:'relative',paddingInline:vars.space[1]}}});
globalStyle('.workspace-shell[data-folder-mode="compact"] .sidebar__badge',{'@media':{[desktop]:{fontSize:'0.65rem',paddingInline:'2px'}}});

globalStyle('.mail-workspace, .workspace-body, .workspace-pane',{position:'relative'});
globalStyle('.workspace-shell[data-folder-mode="compact"] .sidebar__total',{'@media':{[desktop]:{display:'none'}}});
globalStyle('.workspace-show-folders',{position:'fixed',insetBlockStart:vars.space[2],insetInlineStart:vars.space[2],zIndex:40});
globalStyle('.mail-workspace .subtab-strip',{display:'flex',alignItems:'stretch',gap:vars.space[1],overflowX:'auto',flexShrink:0,padding:vars.space[1],borderBlockEnd:`${vars.border.hair} solid ${vars.color.border}`});
globalStyle('.mail-workspace .subtab',{display:'flex',alignItems:'center',flex:'0 0 auto',maxWidth:'min(70vw,320px)',gap:'2px',padding:'2px',border:`1px solid ${vars.color.border}`,borderRadius:vars.radius.sm,background:vars.color.bgAlt,color:vars.color.textDim});
globalStyle('.mail-workspace .subtab__label',{minWidth:0,maxWidth:'220px',overflow:'hidden',textOverflow:'ellipsis',whiteSpace:'nowrap',minHeight:'32px',paddingInline:vars.space[2],border:0,background:'transparent',color:'inherit',font:'inherit',textAlign:'start'});
globalStyle('.mail-workspace .subtab--active',{background:vars.color.surface,color:vars.color.text,borderColor:vars.color.accent,boxShadow:`inset 0 3px 0 ${vars.color.accent}`});
globalStyle('.mail-workspace .subtab--active .subtab__label',{fontWeight:vars.weight.bold});
const tabActions='.mail-workspace :is(.subtab__pin, .subtab__close, .subtab__tearoff)';
globalStyle(tabActions,{flex:'0 0 auto',width:'24px',minWidth:'24px',height:'24px',minHeight:'24px',marginInline:0,padding:0,display:'inline-flex',alignItems:'center',justifyContent:'center',border:'1px solid transparent',borderRadius:vars.radius.sm,background:'transparent',color:vars.color.textDim,'@media':{'(pointer: coarse)':{width:'44px',minWidth:'44px',height:'44px',minHeight:'44px'}}});
globalStyle(`${tabActions} svg`,{width:'11px',height:'11px'});
globalStyle(`${tabActions}:hover`,{background:vars.color.surface,color:vars.color.text,borderColor:vars.color.borderStrong});
globalStyle('.mail-workspace .subtab__pin[aria-pressed="true"]',{background:vars.color.accent,color:vars.color.accentText,borderColor:vars.color.accent});
globalStyle('.mail-workspace .subtab__close:hover',{color:vars.color.danger});

// Reserve control space so revealing actions never shifts the tab title.
// Focus reveals them too; coarse/non-hover input keeps actions discoverable.
globalStyle('.mail-workspace .subtab :is(.subtab__pin, .subtab__close)', {
  '@media': { '(hover: hover) and (pointer: fine)': { opacity: 0, pointerEvents: 'none' } },
});
globalStyle('.mail-workspace .subtab:is(:hover, :focus-within) :is(.subtab__pin, .subtab__close)', {
  '@media': { '(hover: hover) and (pointer: fine)': { opacity: 1, pointerEvents: 'auto' } },
});

// Column scaffolds belong to the stretched containers, not their content lists.
globalStyle('.mail-workspace .mail-pane', { alignSelf: 'stretch', background: vars.color.bg, borderInlineEnd: `1px solid ${vars.color.border}` });
globalStyle('.mail-workspace .mail-pane > .list', { borderInlineEnd: 0 });
globalStyle('.workspace-panes[data-layout="tabs"]', { gridTemplateRows: 'minmax(0,1fr)' });
globalStyle('.workspace-pane', { alignSelf: 'stretch', background: vars.color.bg });
globalStyle('.workspace-body--split[data-reading-pane="bottom"] > .mail-pane', { borderInlineEnd: 0, borderBlockEnd: `1px solid ${vars.color.border}` });

// Five view arrangements keep the same retained pane elements.
globalStyle('.workspace-layout-modes',{display:'flex',alignItems:'center',gap:'2px',flex:'0 0 auto',paddingInline:'4px',borderInlineStart:`1px solid ${vars.color.border}`,borderInlineEnd:`1px solid ${vars.color.border}`});
globalStyle('.workspace-bar .workspace-layout-mode',{padding:'4px',width:'32px',minWidth:'32px','@media':{'(pointer: coarse)':{width:'44px',minWidth:'44px',minHeight:'44px'}}});
globalStyle('.workspace-layout-mode[aria-pressed="true"]',{background:vars.color.accent,color:vars.color.accentText,borderColor:vars.color.accent});
globalStyle('.workspace-panes[data-layout="side-by-side"] > [data-view-slot="0"]',{gridColumn:'1',gridRow:'1'});
globalStyle('.workspace-panes[data-layout="side-by-side"] > [data-view-slot="1"]',{gridColumn:'3',gridRow:'1'});
globalStyle('.workspace-panes[data-layout="side-by-side"] > [role="separator"]',{gridColumn:'2',gridRow:'1'});
globalStyle('.workspace-panes[data-layout="stack"] > [data-view-slot="0"]',{gridColumn:'1',gridRow:'1'});
globalStyle('.workspace-panes[data-layout="stack"] > [data-view-slot="1"]',{gridColumn:'1',gridRow:'3'});
globalStyle('.workspace-panes[data-layout="stack"] > [role="separator"]',{gridColumn:'1',gridRow:'2'});
globalStyle('.workspace-view-empty',{display:'flex',flexDirection:'column',alignItems:'center',justifyContent:'center',padding:vars.space[4],minWidth:0,minHeight:0,color:vars.color.textDim,border:`1px dashed ${vars.color.border}`,textAlign:'center'});
globalStyle('.workspace-panes [data-drop-target="true"]',{outline:`2px solid ${vars.color.accent}`,outlineOffset:'-2px'});
globalStyle('.workspace-panes:is([data-layout="masonry-side-by-side"],[data-layout="masonry-stack"])',{containerType:'inline-size',gridTemplateColumns:'repeat(auto-fit,minmax(min(100%,280px),1fr))',gridTemplateRows:'none',gridAutoRows:'minmax(220px,1fr)',gridAutoFlow:'row dense',gap:vars.space[2],padding:vars.space[2],alignContent:'start'});
globalStyle('.workspace-panes:is([data-layout="masonry-side-by-side"],[data-layout="masonry-stack"]) > .workspace-pane',{border:`1px solid ${vars.color.border}`,borderRadius:vars.radius.sm});
globalStyle('.workspace-panes[data-layout="masonry-side-by-side"] > .workspace-pane:nth-of-type(3n+1)',{gridRow:'span 2'});
globalStyle('.workspace-panes[data-layout="masonry-stack"] > .workspace-pane:nth-of-type(3n+1)',{'@container':{'(min-width: 576px)':{gridColumn:'span 2'}}});

// Keep account actions reachable while the folder and application lists scroll.
globalStyle('.sidebar',{overflow:'hidden'});
globalStyle('.sidebar__scroll',{flex:'1 1 auto',minHeight:0,overflowY:'auto',overflowX:'hidden'});
globalStyle('.sidebar__footer',{display:'flex',flexDirection:'column',gap:vars.space[1],flex:'0 0 auto',marginTop:'auto'});
globalStyle('.sidebar__footer .sidebar__settings',{marginInlineStart:0,padding:'0.5rem 0.6rem',justifyContent:'flex-start'});
