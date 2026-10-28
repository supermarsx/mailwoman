import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../../../theme/contract.css.ts';
// The scroller and the tab row are separate boxes so the scrollbar runs in its own lane BELOW the tab
// baseline: a classic scrollbar takes its own height there, and an overlay one paints over the block-end
// padding instead of over the labels and the active tab. The lane comes out of the bottom margin, so a strip
// that fits sits where it did.
export const tabs=style({overflowX:'auto',overflowY:'hidden',paddingBlockEnd:vars.space[3],marginBlockEnd:vars.space[3],scrollbarWidth:'thin',scrollbarColor:`${vars.color.borderStrong} transparent`,overscrollBehaviorX:'contain'});
/** The tab row: at least as wide as the scroller, so the baseline spans the whole strip when it scrolls. */
export const strip=style({display:'flex',alignItems:'end',justifyContent:'flex-start',gap:vars.space[1],width:'max-content',minWidth:'100%',boxSizing:'border-box',paddingInline:vars.space[1],paddingBlockStart:vars.space[1],borderBottom:`1px solid ${vars.color.border}`});
globalStyle(`${strip} > a`,{display:'inline-flex',alignItems:'center',justifyContent:'center',flex:'0 0 auto',minHeight:vars.a11y.touchTarget,padding:`${vars.space[2]} ${vars.space[4]}`,border:'1px solid transparent',borderBottom:0,color:vars.color.textDim,textDecoration:'none',whiteSpace:'nowrap',fontWeight:vars.weight.medium,transition:`background ${vars.a11y.motionDuration}, color ${vars.a11y.motionDuration}`});
globalStyle(`${strip} > a:hover`,{background:vars.color.bgAlt,color:vars.color.text});
globalStyle(`${strip} > a[aria-current="page"]`,{background:vars.color.surface,color:vars.color.text,borderColor:vars.color.border,boxShadow:`inset 0 3px 0 ${vars.color.accent}`,fontWeight:vars.weight.bold});
globalStyle(`${strip} > a:focus-visible`,{outline:`2px solid ${vars.a11y.focusRingColor}`,outlineOffset:-3});
