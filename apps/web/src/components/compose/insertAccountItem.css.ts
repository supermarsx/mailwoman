import {style,globalStyle} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const root=style({position:'relative',minWidth:0});
export const panel=style({display:'grid',gap:vars.space[2],padding:vars.space[3],border:`1px solid ${vars.color.border}`,borderRadius:vars.radius.sm,background:vars.color.bg,minWidth:0,width:'min(36rem, 100%)',maxWidth:'100%',maxHeight:'calc(100dvh - 2rem)',overflowY:'auto',overscrollBehavior:'contain',boxShadow:vars.elevation[2],boxSizing:'border-box'});
export const header=style({display:'flex',alignItems:'center',justifyContent:'space-between',gap:vars.space[2]});
export const help=style({fontSize:'.8rem',color:vars.color.textDim,margin:0});
globalStyle(`${panel} label`,{display:'grid',gap:vars.space[1],minWidth:0});
globalStyle(`${panel} :is(input,select)`,{maxWidth:'100%',width:'100%',minWidth:0});

export const backdrop=style({position:'fixed',inset:0,zIndex:1300,background:vars.color.scrim,display:'grid',placeItems:'center',padding:vars.space[3],boxSizing:'border-box'});
