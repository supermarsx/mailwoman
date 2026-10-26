import {globalStyle} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
globalStyle('.detached-workspace',{margin:0,height:'100dvh',display:'flex',flexDirection:'column',overflow:'hidden'});
globalStyle('.detached-workspace__header',{display:'flex',alignItems:'center',justifyContent:'space-between',gap:'1rem',padding:'0.5rem',flexShrink:0,borderBottom:`1px solid ${vars.color.border}`});
globalStyle('.detached-workspace__content,.detached-workspace__home,.detached-workspace__portal',{display:'flex',flexDirection:'column',flex:'1 1 auto',minWidth:0,minHeight:0});
globalStyle('.detached-workspace__content',{overflow:'auto'});
globalStyle('.detached-workspace__placeholder',{display:'grid',placeContent:'center',textAlign:'center',gap:'0.5rem',flex:'1 1 auto',padding:'1rem'});
globalStyle('.workspace-pane > .detached-workspace__home',{height:'100%'});
globalStyle('.detached-workspace__portal',{height:'100%'});
