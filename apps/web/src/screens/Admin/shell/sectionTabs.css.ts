import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../../../theme/contract.css.ts';
export const tabs=style({display:'flex',alignItems:'end',justifyContent:'flex-start',gap:vars.space[1],overflowX:'auto',paddingInline:vars.space[1],paddingBlockStart:vars.space[1],marginBlockEnd:vars.space[5],borderBottom:`1px solid ${vars.color.border}`,scrollbarWidth:'thin'});
globalStyle(`${tabs} a`,{display:'inline-flex',alignItems:'center',justifyContent:'center',flex:'0 0 auto',minHeight:vars.a11y.touchTarget,padding:`${vars.space[2]} ${vars.space[4]}`,border:'1px solid transparent',borderBottom:0,color:vars.color.textDim,textDecoration:'none',whiteSpace:'nowrap',fontWeight:vars.weight.medium,transition:`background ${vars.a11y.motionDuration}, color ${vars.a11y.motionDuration}`});
globalStyle(`${tabs} a:hover`,{background:vars.color.bgAlt,color:vars.color.text});
globalStyle(`${tabs} a[aria-current="page"]`,{background:vars.color.surface,color:vars.color.text,borderColor:vars.color.border,boxShadow:`inset 0 3px 0 ${vars.color.accent}`,fontWeight:vars.weight.bold});
globalStyle(`${tabs} a:focus-visible`,{outline:`2px solid ${vars.a11y.focusRingColor}`,outlineOffset:-3});
