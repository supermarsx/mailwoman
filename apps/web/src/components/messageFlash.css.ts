import {globalStyle,keyframes} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
// A row that was navigated to (Go to reply, folder indicator): a ~1.5s accent wash, or a static outline when motion is reduced.
const flash=keyframes({'0%':{backgroundColor:vars.color.selection},'100%':{backgroundColor:'transparent'}});
globalStyle('.list__row[data-flash="true"]',{
 outline:`2px solid ${vars.color.accent}`,outlineOffset:'-2px',animation:`${flash} 1.5s ease-out 1`,
 '@media':{'(prefers-reduced-motion: reduce)':{animation:'none'},'(forced-colors: active)':{outline:'3px solid Highlight',animation:'none'}},
});
globalStyle(':root[data-reduced-motion] .list__row[data-flash="true"]',{animation:'none'});
// The folder indicator on a cross-folder row: a real button beside (never inside) the row button.
globalStyle('.list__folder-chip',{
 position:'absolute',insetInlineEnd:vars.space[2],bottom:vars.space[1],display:'inline-flex',alignItems:'center',gap:vars.space[1],maxWidth:'40%',
 overflow:'hidden',textOverflow:'ellipsis',whiteSpace:'nowrap',font:'inherit',fontSize:'0.75em',padding:`0 ${vars.space[2]}`,
 color:vars.color.textDim,background:vars.color.bgSink,border:`1px solid ${vars.color.borderStrong}`,borderRadius:vars.radius.sm,cursor:'pointer',
});
globalStyle('.list__folder-chip:hover',{color:vars.color.text,borderColor:vars.color.accent});
globalStyle('.list__folder-chip:focus-visible',{outline:`2px solid ${vars.color.focus}`,outlineOffset:'1px'});
globalStyle('.list__folder-chip',{'@media':{'(forced-colors: active)':{color:'ButtonText',background:'ButtonFace',borderColor:'ButtonText'}}});
