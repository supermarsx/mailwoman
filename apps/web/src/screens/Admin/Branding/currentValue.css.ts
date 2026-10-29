import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../../../theme/contract.css.ts';

/** The "current value" row and its tooltip in the branding editor (t62). */
export const row=style({display:'flex',alignItems:'center',flexWrap:'wrap',gap:vars.space[2],minInlineSize:0});
export const labelRow=style({display:'inline-flex',alignItems:'center',gap:vars.space[1]});

export const trigger=style({
 display:'inline-flex',alignItems:'center',gap:vars.space[2],minBlockSize:'2rem',minInlineSize:'2rem',
 padding:vars.space[1],margin:0,font:'inherit',fontSize:'0.8125rem',color:vars.color.text,cursor:'help',
 background:vars.color.bgSink,border:`${vars.border.rule} solid ${vars.color.border}`,borderRadius:vars.radius.sm,
 selectors:{'&:hover':{borderColor:vars.color.borderStrong},'&:focus-visible':{outline:`${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`,outlineOffset:'2px'}},
});
/** The ⓘ trigger beside a text field label. */
export const infoTrigger=style([trigger,{minBlockSize:'1.5rem',minInlineSize:'1.5rem',padding:0,justifyContent:'center',color:vars.color.textDim,textTransform:'none',letterSpacing:'normal'}]);
export const triggerText=style({whiteSpace:'nowrap'});
export const arrow=style({color:vars.color.textDim});

/** An image over a theme-coloured checkerboard, so transparent pixels stay visible. */
export const checker=style({position:'relative',display:'inline-grid',placeItems:'center',overflow:'hidden',flex:'none',background:vars.color.surface,border:`${vars.border.hair} solid ${vars.color.border}`,borderRadius:vars.radius.sm});
export const checkerPattern=style({position:'absolute',inset:0,inlineSize:'100%',blockSize:'100%',pointerEvents:'none'});
globalStyle(`${checkerPattern} rect`,{fill:vars.color.border});
export const checkerImage=style({position:'relative',display:'block',maxInlineSize:'100%',maxBlockSize:'100%',objectFit:'contain'});
export const thumb=style({inlineSize:'1.5rem',blockSize:'1.5rem'});
export const preview=style({inlineSize:'7.5rem',blockSize:'5rem'});
export const emptyMark=style({position:'relative',color:vars.color.textDim,fontSize:'0.75rem'});

export const bubble=style({minInlineSize:'min(14rem, calc(100vw - 16px))'});
export const columns=style({display:'flex',flexWrap:'wrap',gap:vars.space[3]});
export const figure=style({display:'flex',flexDirection:'column',gap:vars.space[1],margin:0,minInlineSize:0});
export const caption=style({fontWeight:vars.weight.bold});
export const facts=style({display:'grid',gridTemplateColumns:'auto 1fr',columnGap:vars.space[2],rowGap:0,margin:0});
globalStyle(`${facts} dt`,{color:vars.color.textDim});
globalStyle(`${facts} dd`,{margin:0,overflowWrap:'anywhere',whiteSpace:'pre-wrap'});
export const scope=style({margin:0,marginBlockStart:vars.space[2],color:vars.color.textDim});
