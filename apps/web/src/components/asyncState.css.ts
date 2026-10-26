import { keyframes, style } from '@vanilla-extract/css';
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
