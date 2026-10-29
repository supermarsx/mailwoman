import { globalStyle, keyframes, style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

const appear = keyframes({ from: { opacity: 0 }, to: { opacity: 1 } });

export const errorCard = style({
  boxSizing:'border-box', display:'grid', gap:vars.space[4], minInlineSize:0,
  maxInlineSize:'42rem', margin:vars.space[4], padding:vars.space[5],
  border:`2px solid ${vars.color.borderStrong}`, borderInlineStart:`5px solid ${vars.color.danger}`,
  background:vars.color.surface, color:vars.color.text, fontFamily:vars.font.ui,
  overflowWrap:'anywhere', alignSelf:'start',
});
export const errorHeading = style({display:'flex',alignItems:'center',gap:vars.space[3],margin:0,fontSize:'1.125rem',fontWeight:vars.weight.bold,lineHeight:1.4});
export const errorMark = style({flexShrink:0,color:vars.color.danger,inlineSize:'1.5rem',blockSize:'1.5rem'});
export const errorHelp = style({margin:0,color:vars.color.textDim,lineHeight:1.5});
export const retry = style({
  display:'inline-flex',alignItems:'center',justifyContent:'center',justifySelf:'start',gap:vars.space[2],
  inlineSize:'fit-content',maxInlineSize:'100%',minBlockSize:'44px',padding:`${vars.space[2]} ${vars.space[4]}`,
  border:`2px solid ${vars.color.borderStrong}`,borderRadius:0,background:vars.color.accent,color:vars.color.accentText,
  font:'inherit',fontWeight:vars.weight.bold,cursor:'pointer',
  ':hover':{filter:'brightness(1.08)'},':active':{filter:'brightness(0.95)'},
});

// Reserve the same small footprint while fast requests finish. Delay only the
// visual indicator, never the request, resolved content, or live announcement.
export const pending = style({
  minBlockSize: '2.75rem',
  display: 'flex',
  alignItems: 'center',
  marginBlock: vars.space[3],
});

export const indicator = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[2],
  animation: `${appear} 160ms ease-out 120ms both`,
  '::before': {
    content: '""',
    inlineSize: '0.5rem',
    blockSize: '0.5rem',
    flexShrink: 0,
    background: vars.color.accent,
  },
  selectors: { ':root[data-reduced-motion] &': { animation: 'none' } },
  '@media': { '(prefers-reduced-motion: reduce)': { animation: 'none' } },
});

// Quiet content-shaped placeholders. One live status announces loading; these
// shapes are decorative, bounded and never pretend to be loaded information.
export const skeletonPanel = style({flexDirection:'column',alignItems:'stretch',gap:vars.space[3],inlineSize:'100%',maxInlineSize:'64rem',minInlineSize:0,boxSizing:'border-box'});
export const skeletonBody = style({display:'grid',gap:vars.space[3],paddingBlock:vars.space[2],animation:`${appear} 120ms ease-out 120ms both`,selectors:{':root[data-reduced-motion] &':{animation:'none'}},'@media':{'(prefers-reduced-motion: reduce)':{animation:'none'}}});
export const skeletonRow = style({display:'flex',alignItems:'center',gap:vars.space[3],minBlockSize:'2rem',minInlineSize:0});
export const skeletonLine = style({display:'block',blockSize:'.75rem',flex:'1 1 0',minInlineSize:0,background:vars.color.border,borderRadius:vars.radius.sm});
export const skeletonShort = style([skeletonLine,{flex:'0 1 18%'}]);
export const skeletonAvatar = style({display:'block',inlineSize:'2rem',blockSize:'2rem',flex:'0 0 auto',background:vars.color.bgAlt,border:`${vars.border.hair} solid ${vars.color.border}`,borderRadius:vars.radius.sm});
export const skeletonCards = style({display:'grid',gridTemplateColumns:'repeat(auto-fit,minmax(min(100%,9rem),1fr))',gap:vars.space[3]});
export const skeletonCard = style({display:'block',blockSize:'5rem',background:vars.color.bgAlt,border:`${vars.border.hair} solid ${vars.color.border}`,borderRadius:vars.radius.sm});
export const boot = style({minBlockSize:'100dvh',maxInlineSize:'38rem',margin:'0 auto',padding:vars.space[6],justifyContent:'center',fontFamily:vars.font.ui,color:vars.color.text});

// Message-list placeholders (`variant="messages"`). Each row uses the list
// row's own density variables (styles/app.css `--mail-row-*`) and is given the
// virtualized row height, so the skeleton has the same rhythm as the rows that
// replace it. The shapes pulse slowly between two flat fills (the t32 style
// has no gradients); with reduced motion they are still.
const pulse = keyframes({ '0%, 100%': { backgroundColor: vars.color.border }, '50%': { backgroundColor: `color-mix(in srgb, ${vars.color.border} 45%, ${vars.color.bg})` } });
const STILL = { selectors: { ':root[data-reduced-motion] &': { animation: 'none' } }, '@media': { '(prefers-reduced-motion: reduce)': { animation: 'none' } } } as const;
export const messagesPanel = style({margin:0,maxInlineSize:'none',flex:'1 1 auto',minBlockSize:0,overflow:'hidden',gap:0});
export const messageSkeletons = style({display:'block',minInlineSize:0,overflow:'hidden',animation:`${appear} 120ms ease-out 120ms both`,...STILL});
export const messageSkeleton = style({
  display:'grid',boxSizing:'border-box',minInlineSize:0,overflow:'hidden',alignItems:'center',alignContent:'start',
  gridTemplateRows:'var(--mail-row-line, 18px) var(--mail-row-line, 18px) var(--mail-preview-line, 16px)',
  rowGap:'var(--mail-row-gap, 2px)',padding:'var(--mail-row-padding, 7px) 0.8rem',
  borderBlockEnd:`${vars.border.hair} solid ${vars.color.border}`,
});
export const messageSkeletonHead = style({display:'flex',alignItems:'center',gap:vars.space[2],minInlineSize:0});
export const messageSkeletonAvatar = style({display:'block',flex:'0 0 auto',inlineSize:'var(--mail-row-line, 18px)',blockSize:'var(--mail-row-line, 18px)',borderRadius:'50%',background:vars.color.border});
export const messageSkeletonDate = style({display:'block',flex:'0 0 3rem',marginInlineStart:'auto',blockSize:'.6rem',background:vars.color.border,borderRadius:vars.radius.sm});
export const messageSkeletonDim = style({opacity:.6});
globalStyle(`${messageSkeletons} span`, {animation:`${pulse} 1.6s ease-in-out infinite`});
globalStyle(`:root[data-reduced-motion] ${messageSkeletons} span`, {animation:'none'});
globalStyle(`${messageSkeletons} span`, {'@media':{'(prefers-reduced-motion: reduce)':{animation:'none'}}});

// A thin indeterminate line over a list that is still indexing in the
// background while some rows are already shown. Decorative: the list carries
// the state for assistive technology. Still (and partly filled) with reduced motion.
const travel = keyframes({ from: { transform: 'translateX(-100%)' }, to: { transform: 'translateX(250%)' } });
export const syncProgress = style({
  position:'relative',flex:'0 0 auto',blockSize:'2px',overflow:'hidden',background:`color-mix(in srgb, ${vars.color.accent} 18%, transparent)`,
  '::after':{content:'""',position:'absolute',insetBlock:0,insetInlineStart:0,inlineSize:'40%',background:vars.color.accent,animation:`${travel} 1.4s ease-in-out infinite`},
  selectors:{':root[data-reduced-motion] &::after':{animation:'none',transform:'none'}},
  '@media':{'(prefers-reduced-motion: reduce)':{selectors:{'&::after':{animation:'none',transform:'none'}}}},
});
