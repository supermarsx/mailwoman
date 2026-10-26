import {globalStyle} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';

globalStyle('.sweep__backdrop',{position:'fixed',inset:0,zIndex:1000,display:'grid',placeItems:'center',padding:vars.space[4],background:vars.color.scrim,boxSizing:'border-box'});
globalStyle('.sweep',{boxSizing:'border-box',inlineSize:'min(100%, 36rem)',minInlineSize:0,maxBlockSize:'calc(100dvh - 2rem)',overflowY:'auto',overscrollBehavior:'contain',background:vars.color.surface,color:vars.color.text,border:`${vars.border.rule} solid ${vars.color.borderStrong}`,padding:vars.space[5],display:'flex',flexDirection:'column',gap:vars.space[4],overflowWrap:'anywhere'});
globalStyle('.sweep h2, .sweep p',{margin:0});
globalStyle('.sweep__header, .sweep__footer',{display:'flex',alignItems:'center',gap:vars.space[3],flexWrap:'wrap'});
globalStyle('.sweep__header',{justifyContent:'space-between'});
globalStyle('.sweep__footer',{justifyContent:'flex-end'});
globalStyle('.sweep__footer button',{display:'inline-flex',alignItems:'center',gap:vars.space[2],maxInlineSize:'100%'});
globalStyle('.sweep__options',{display:'grid',gap:vars.space[3],padding:vars.space[4],border:`${vars.border.hair} solid ${vars.color.border}`,minInlineSize:0});
globalStyle('.sweep__option',{display:'flex',alignItems:'center',gap:vars.space[3]});
globalStyle('.sweep__preview',{listStyle:'none',padding:0,margin:0,maxBlockSize:'14rem',overflowY:'auto'});
globalStyle('.sweep__row',{display:'flex',justifyContent:'space-between',gap:vars.space[3],paddingBlock:vars.space[2],borderBlockEnd:`${vars.border.hair} solid ${vars.color.border}`});
globalStyle('.sweep__subject',{minInlineSize:0});
globalStyle('.sweep__date',{flexShrink:0,color:vars.color.textDim});
