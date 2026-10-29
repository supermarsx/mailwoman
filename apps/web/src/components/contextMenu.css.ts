import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
export const menu=style({position:'fixed',zIndex:1500,minWidth:'12rem',maxWidth:'calc(100vw - 16px)',maxHeight:'calc(100dvh - 16px)',overflowY:'auto',padding:'4px',background:vars.color.bg,color:vars.color.text,border:`2px solid ${vars.color.borderStrong}`,boxShadow:vars.elevation[3]});
export const item=style({display:'flex',alignItems:'center',gap:'0.6rem',width:'100%',minHeight:'40px',padding:'8px 12px',border:0,background:'transparent',color:'inherit',font:'inherit',textAlign:'start',cursor:'pointer',selectors:{'&:focus-visible':{outline:`2px solid ${vars.color.focus}`,outlineOffset:'-2px'},'&:hover':{background:vars.color.bgAlt},'&[aria-disabled="true"]':{opacity:0.5,cursor:'default'}}});
export const separator=style({height:'1px',margin:'4px 8px',background:vars.color.borderStrong});
export const folderRow=style({display:'flex',alignItems:'stretch',minWidth:0});
globalStyle(`.${folderRow} > .sidebar__box`,{flex:'1 1 auto',minWidth:0,width:'auto'});
