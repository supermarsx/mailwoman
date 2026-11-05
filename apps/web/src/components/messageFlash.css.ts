import {globalStyle,keyframes} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
// A row that was navigated to (Go to reply, folder indicator): a ~1.5s accent wash, or a static outline when motion is reduced.
const flash=keyframes({'0%':{backgroundColor:vars.color.selection},'100%':{backgroundColor:'transparent'}});
globalStyle('.list__row[data-flash="true"]',{
 outline:`2px solid ${vars.color.accent}`,outlineOffset:'-2px',animation:`${flash} 1.5s ease-out 1`,
 '@media':{'(prefers-reduced-motion: reduce)':{animation:'none'},'(forced-colors: active)':{outline:'3px solid Highlight',animation:'none'}},
});
globalStyle(':root[data-reduced-motion] .list__row[data-flash="true"]',{animation:'none'});
