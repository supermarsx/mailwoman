import {globalStyle,style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';

export const backdrop=style({position:'fixed',inset:0,zIndex:1900,display:'flex',alignItems:'center',justifyContent:'center',padding:'1rem',background:'rgba(0,0,0,.45)'});
export const dialog=style({boxSizing:'border-box',inlineSize:'min(46rem, 100%)',maxBlockSize:'calc(100dvh - 2rem)',display:'flex',flexDirection:'column',gap:'.75rem',padding:'1rem',overflow:'auto',background:vars.color.bg,color:vars.color.text,border:`${vars.border.rule} solid ${vars.color.borderStrong}`,borderRadius:vars.radius.md,boxShadow:vars.elevation[3]});
export const title=style({margin:0,fontSize:'1.1em'});
export const list=style({listStyle:'none',margin:0,padding:0,display:'flex',flexDirection:'column',gap:'.5rem'});
export const row=style({display:'grid',gridTemplateColumns:'auto minmax(0,1fr) minmax(0,1fr)',gap:'.4rem .6rem',alignItems:'center',paddingBlock:'.5rem',borderBlockEnd:`${vars.border.hair} solid ${vars.color.border}`});
export const wide=style({gridColumn:'2 / -1',display:'grid',gridTemplateColumns:'repeat(auto-fit, minmax(10rem, 1fr))',gap:'.4rem'});
export const badge=style({display:'inline-block',marginInlineEnd:'.25rem',padding:'0 .4rem',border:`${vars.border.hair} solid ${vars.color.borderStrong}`,borderRadius:vars.radius.sm,fontSize:'.8em'});
export const email=style({color:vars.color.textDim,overflowWrap:'anywhere'});
export const field=style({display:'flex',flexDirection:'column',gap:'.15rem',fontSize:'.85em',minInlineSize:0});
export const actions=style({display:'flex',justifyContent:'flex-end',gap:'.5rem'});
export const muted=style({color:vars.color.textDim,margin:0});
globalStyle(`${field} input, ${field} select`,{minInlineSize:0,font:'inherit'});
