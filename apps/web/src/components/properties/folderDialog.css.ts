// Styles of the redesigned folder Properties dialog: a summary header, a tab
// rail (vertical on wide screens, a scrolling row on phones), section cards and
// a sticky footer. Tokens only; motion stops under reduced motion; forced-colors
// keeps borders; logical properties throughout so it mirrors in RTL.
import {globalStyle,keyframes,style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';

const shimmer=keyframes({'0%':{opacity:0.55},'50%':{opacity:1},'100%':{opacity:0.55}});
const NARROW='(max-width: 40rem)';

export const backdrop=style({position:'fixed',inset:0,zIndex:1200,background:vars.color.scrim,display:'grid',placeItems:'center',padding:vars.space[3],'@media':{[NARROW]:{padding:0}}});
export const dialog=style({position:'relative',display:'flex',flexDirection:'column',background:vars.color.surface,color:vars.color.text,border:`${vars.border.rule} solid ${vars.color.borderStrong}`,borderRadius:vars.radius.md,boxShadow:vars.elevation[2],inlineSize:'min(45rem, 100%)',blockSize:'min(46rem, calc(100dvh - 2rem))',overflow:'hidden',minInlineSize:0,
 '@media':{[NARROW]:{blockSize:'100dvh',borderRadius:0,border:'none'}}});
export const closeButton=style({appearance:'none',position:'absolute',insetBlockStart:vars.space[2],insetInlineEnd:vars.space[2],zIndex:1,display:'inline-grid',placeItems:'center',minInlineSize:vars.a11y.touchTarget,minBlockSize:vars.a11y.touchTarget,padding:0,border:'none',background:'transparent',color:vars.color.textDim,cursor:'pointer',borderRadius:vars.radius.sm,selectors:{'&:hover':{color:vars.color.text}}});
/** Marks the dialog for the phone layout; also a test hook. */
export const phone=style({});

// Header
export const header=style({display:'flex',flexDirection:'column',gap:vars.space[3],padding:`${vars.space[4]} ${vars.space[4]} ${vars.space[3]}`,borderBlockEnd:`${vars.border.hair} solid ${vars.color.border}`,background:vars.color.bgAlt,flex:'0 0 auto',overflowY:'auto',maxBlockSize:'55%'});
export const identity=style({display:'flex',alignItems:'flex-start',gap:vars.space[3],minInlineSize:0,paddingInlineEnd:vars.a11y.touchTarget});
export const folderGlyph=style({flex:'0 0 auto',display:'inline-grid',placeItems:'center',inlineSize:'3.25rem',blockSize:'3.25rem',borderRadius:vars.radius.lg,border:`${vars.border.rule} solid ${vars.color.borderStrong}`,background:vars.color.surface,color:vars.color.accent,fontSize:'1.6rem'});
export const identityText=style({display:'flex',flexDirection:'column',gap:vars.space[1],minInlineSize:0,flex:'1 1 auto'});
export const nameRow=style({display:'flex',alignItems:'center',gap:vars.space[2],flexWrap:'wrap',minInlineSize:0});
export const folderName=style({margin:0,fontSize:'1.2rem',fontWeight:vars.weight.heavy,overflowWrap:'anywhere',minInlineSize:0});
export const breadcrumb=style({display:'flex',alignItems:'center',flexWrap:'wrap',gap:vars.space[1],color:vars.color.textDim,fontSize:'0.85rem',margin:0,padding:0,listStyle:'none',minInlineSize:0});
globalStyle(`${breadcrumb} li`,{display:'inline-flex',alignItems:'center',gap:vars.space[1],minInlineSize:0,overflowWrap:'anywhere'});
globalStyle(`${breadcrumb} li + li::before`,{content:'"/"',opacity:0.6});
export const chipRow=style({display:'flex',flexWrap:'wrap',alignItems:'center',gap:vars.space[2]});
export const chip=style({display:'inline-flex',alignItems:'center',gap:vars.space[1],border:`${vars.border.hair} solid ${vars.color.borderStrong}`,borderRadius:vars.radius.pill,paddingInline:vars.space[2],paddingBlock:'1px',fontSize:'0.75rem',color:vars.color.text,background:vars.color.surface,whiteSpace:'nowrap'});
export const chipAccent=style({borderColor:vars.color.accent,color:vars.color.accent});
export const chipLocked=style({borderColor:vars.color.textDim,color:vars.color.textDim});
export const toggle=style({appearance:'none',display:'inline-flex',alignItems:'center',gap:vars.space[1],minBlockSize:'1.75rem',border:`${vars.border.hair} solid ${vars.color.borderStrong}`,borderRadius:vars.radius.pill,paddingInline:vars.space[2],background:vars.color.surface,color:vars.color.textDim,font:'inherit',fontSize:'0.8rem',cursor:'pointer',
 selectors:{'&[aria-pressed="true"]':{color:vars.color.accentText,background:vars.color.accent,borderColor:vars.color.accent},'&:disabled':{opacity:0.55,cursor:'default'}},
 '@media':{'(forced-colors: active)':{selectors:{'&[aria-pressed="true"]':{background:'Highlight',color:'HighlightText'}}}}});
export const iconButton=style({appearance:'none',display:'inline-grid',placeItems:'center',minInlineSize:'1.75rem',minBlockSize:'1.75rem',padding:0,border:'none',background:'transparent',color:vars.color.textDim,cursor:'pointer',borderRadius:vars.radius.sm,selectors:{'&:hover':{color:vars.color.text,background:vars.color.bg}}});

// Stat tiles
export const tiles=style({display:'grid',gridTemplateColumns:'repeat(4, minmax(0, 1fr))',gap:vars.space[2],margin:0,'@media':{[NARROW]:{gridTemplateColumns:'repeat(2, minmax(0, 1fr))'}}});
export const tile=style({display:'flex',flexDirection:'column',gap:'2px',minInlineSize:0,padding:`${vars.space[2]} ${vars.space[3]}`,border:`${vars.border.hair} solid ${vars.color.border}`,borderRadius:vars.radius.md,background:vars.color.surface});
export const tileLabel=style({fontSize:'0.72rem',color:vars.color.textDim,margin:0});
export const tileValue=style({fontSize:'1.05rem',fontWeight:vars.weight.bold,margin:0,minBlockSize:'1.4rem',overflowWrap:'anywhere'});
export const skeleton=style({display:'block',blockSize:'1rem',inlineSize:'60%',borderRadius:vars.radius.sm,background:vars.color.border,animation:`${shimmer} 1.4s ease-in-out infinite`,
 selectors:{':root[data-reduced-motion] &':{animation:'none'}},'@media':{'(prefers-reduced-motion: reduce)':{animation:'none'},'(forced-colors: active)':{background:'GrayText'}}});
export const linkButton=style({appearance:'none',background:'none',border:'none',padding:0,font:'inherit',fontSize:'0.8rem',color:vars.color.link,textDecoration:'underline',cursor:'pointer',textAlign:'start'});

// Body: rail + panel
export const main=style({display:'flex',flex:'1 1 auto',minBlockSize:0,'@media':{[NARROW]:{flexDirection:'column'}}});
export const rail=style({display:'flex',flexDirection:'column',gap:'2px',flex:'0 0 12.5rem',padding:vars.space[2],borderInlineEnd:`${vars.border.hair} solid ${vars.color.border}`,overflowY:'auto',
 '@media':{[NARROW]:{flexDirection:'row',flex:'0 0 auto',overflowX:'auto',overflowY:'hidden',borderInlineEnd:'none',borderBlockEnd:`${vars.border.hair} solid ${vars.color.border}`,scrollbarWidth:'thin'}}});
export const tab=style({appearance:'none',display:'flex',alignItems:'center',gap:vars.space[2],flex:'0 0 auto',minBlockSize:vars.a11y.touchTarget,padding:`${vars.space[1]} ${vars.space[3]}`,border:'none',borderInlineStart:'3px solid transparent',borderRadius:vars.radius.sm,background:'transparent',color:vars.color.textDim,font:'inherit',cursor:'pointer',textAlign:'start',whiteSpace:'nowrap',
 selectors:{'&:hover':{background:vars.color.bgAlt,color:vars.color.text},'&[aria-selected="true"]':{color:vars.color.text,background:vars.color.bgAlt,borderInlineStartColor:vars.color.accent,fontWeight:vars.weight.bold}},
 '@media':{[NARROW]:{borderInlineStart:'none',borderBlockEnd:'3px solid transparent',selectors:{'&[aria-selected="true"]':{borderBlockEndColor:vars.color.accent}}},'(forced-colors: active)':{selectors:{'&[aria-selected="true"]':{outline:'2px solid Highlight'}}}}});
export const panel=style({flex:'1 1 auto',minInlineSize:0,minBlockSize:0,overflowY:'auto',display:'flex',flexDirection:'column',gap:vars.space[3],padding:vars.space[4]});

// Cards
export const card=style({display:'flex',flexDirection:'column',gap:vars.space[3],padding:vars.space[3],border:`${vars.border.hair} solid ${vars.color.border}`,borderRadius:vars.radius.md,background:vars.color.bg,minInlineSize:0});
export const cardDanger=style({borderColor:vars.color.danger});
export const cardTitle=style({margin:0,fontSize:'0.95rem',fontWeight:vars.weight.bold});
export const cardHelp=style({margin:0,color:vars.color.textDim,fontSize:'0.85rem'});
export const grid=style({display:'grid',gridTemplateColumns:'minmax(7rem, 11rem) 1fr',gap:`${vars.space[2]} ${vars.space[3]}`,margin:0,alignItems:'baseline','@media':{[NARROW]:{gridTemplateColumns:'1fr',gap:0}}});
globalStyle(`${grid} dt`,{color:vars.color.textDim,margin:0,fontSize:'0.85rem'});
globalStyle(`${grid} dd`,{margin:0,minInlineSize:0,overflowWrap:'anywhere',display:'flex',alignItems:'center',gap:vars.space[1],flexWrap:'wrap'});
export const mono=style({fontFamily:vars.font.mono,fontSize:'0.85em'});
export const note=style({color:vars.color.textDim,margin:0,fontSize:'0.9em'});
export const row=style({display:'flex',flexWrap:'wrap',alignItems:'center',gap:vars.space[2]});
export const problem=style({display:'flex',alignItems:'center',gap:vars.space[2],flexWrap:'wrap',color:vars.color.danger,margin:0,fontSize:'0.9em'});
export const advanced=style({border:`${vars.border.hair} solid ${vars.color.border}`,borderRadius:vars.radius.md,padding:vars.space[2]});
globalStyle(`${advanced} > summary`,{cursor:'pointer',fontWeight:vars.weight.bold,fontSize:'0.9rem'});

// Usage bars
export const meter=style({display:'flex',flexDirection:'column',gap:'4px'});
export const track=style({position:'relative',blockSize:'0.6rem',borderRadius:vars.radius.pill,background:vars.color.border,overflow:'hidden','@media':{'(forced-colors: active)':{border:'1px solid CanvasText'}}});
export const fill=style({position:'absolute',insetBlock:0,insetInlineStart:0,background:vars.color.accent,minInlineSize:'2px','@media':{'(forced-colors: active)':{background:'Highlight'}}});
export const fillWarn=style({background:vars.color.warning});
export const largest=style({listStyle:'none',margin:0,padding:0,display:'flex',flexDirection:'column',gap:vars.space[2]});
export const largestRow=style({display:'grid',gridTemplateColumns:'1.5rem 1fr auto',alignItems:'center',gap:vars.space[2]});

// Appearance
export const swatches=style({display:'flex',flexWrap:'wrap',gap:vars.space[2],alignItems:'center'});
export const swatch=style({appearance:'none',inlineSize:'1.75rem',blockSize:'1.75rem',borderRadius:vars.radius.pill,border:`2px solid ${vars.color.borderStrong}`,cursor:'pointer',padding:0,
 selectors:{'&[aria-checked="true"]':{outline:`2px solid ${vars.color.accent}`,outlineOffset:'2px'}}});
export const previewRow=style({display:'flex',alignItems:'center',gap:vars.space[2],padding:`${vars.space[2]} ${vars.space[3]}`,border:`${vars.border.hair} dashed ${vars.color.borderStrong}`,borderRadius:vars.radius.md,background:vars.color.surface,minInlineSize:0});
export const previewName=style({flex:'1 1 auto',minInlineSize:0,overflow:'hidden',textOverflow:'ellipsis',whiteSpace:'nowrap'});

// Overrides
export const group=style({display:'flex',flexDirection:'column',gap:vars.space[2]});
export const ovr=style({display:'flex',flexDirection:'column',gap:vars.space[2],paddingBlock:vars.space[2],borderBlockStart:`${vars.border.hair} solid ${vars.color.border}`,selectors:{'&:first-of-type':{borderBlockStart:'none',paddingBlockStart:0}}});
export const ovrHead=style({display:'flex',alignItems:'center',justifyContent:'space-between',gap:vars.space[2],flexWrap:'wrap'});
export const ovrLabel=style({fontWeight:vars.weight.medium});
export const segmented=style({display:'inline-flex',flexWrap:'wrap',border:`${vars.border.rule} solid ${vars.color.borderStrong}`,borderRadius:vars.radius.md,overflow:'hidden',alignSelf:'flex-start',maxInlineSize:'100%'});
export const segment=style({appearance:'none',border:'none',borderInlineEnd:`${vars.border.hair} solid ${vars.color.borderStrong}`,background:vars.color.surface,color:vars.color.text,font:'inherit',fontSize:'0.85rem',minBlockSize:'2rem',paddingInline:vars.space[3],cursor:'pointer',
 selectors:{'&:last-child':{borderInlineEnd:'none'},'&[aria-checked="true"]':{background:vars.color.accent,color:vars.color.accentText},'&:disabled':{opacity:0.6,cursor:'default'}},
 '@media':{'(forced-colors: active)':{selectors:{'&[aria-checked="true"]':{background:'Highlight',color:'HighlightText'}}}}});
export const lockedValue=style({display:'inline-flex',alignItems:'center',gap:vars.space[1],color:vars.color.textDim,fontSize:'0.9rem'});
export const textField=style({font:'inherit',minBlockSize:'2rem',paddingInline:vars.space[2],border:`${vars.border.rule} solid ${vars.color.borderStrong}`,borderRadius:vars.radius.sm,background:vars.color.surface,color:vars.color.text,maxInlineSize:'18rem'});

// Notes
export const textarea=style({inlineSize:'100%',minBlockSize:'7rem',maxBlockSize:'20rem',resize:'none',font:'inherit',boxSizing:'border-box',padding:vars.space[2],border:`${vars.border.rule} solid ${vars.color.borderStrong}`,borderRadius:vars.radius.sm,background:vars.color.surface,color:vars.color.text});
export const saved=style({color:vars.color.success,fontSize:'0.85rem'});

// Actions
export const action=style({appearance:'none',display:'grid',gridTemplateColumns:'2rem 1fr',alignItems:'center',gap:vars.space[3],inlineSize:'100%',padding:`${vars.space[2]} ${vars.space[1]}`,border:'none',borderBlockStart:`${vars.border.hair} solid ${vars.color.border}`,background:'transparent',color:vars.color.text,font:'inherit',textAlign:'start',cursor:'pointer',borderRadius:0,
 selectors:{'&:hover:not(:disabled)':{background:vars.color.bgAlt},'&:disabled':{opacity:0.55,cursor:'default'}}});
export const actionDanger=style({color:vars.color.danger});
export const actionTitle=style({display:'block',fontWeight:vars.weight.medium});
export const actionDesc=style({display:'block',color:vars.color.textDim,fontSize:'0.82rem'});

// Footer
export const footer=style({display:'flex',alignItems:'center',justifyContent:'flex-end',gap:vars.space[2],padding:`${vars.space[3]} ${vars.space[4]}`,borderBlockStart:`${vars.border.hair} solid ${vars.color.border}`,background:vars.color.surface,flex:'0 0 auto'});
export const footerHint=style({marginInlineEnd:'auto',color:vars.color.textDim,fontSize:'0.8rem'});
