import {globalStyle} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';

// Keep the platform's normal scrollbar width and touch/overlay behavior.
// Semantic tokens follow both webmail and console theme changes.
globalStyle(':where(*)',{
  scrollbarColor:`${vars.color.textDim} ${vars.color.bgAlt}`,
  '@media':{'(forced-colors: active)':{scrollbarColor:'auto'}},
});

// Older WebKit engines do not implement scrollbar-color.
const fallback='not (scrollbar-color: red blue)';
globalStyle('::-webkit-scrollbar',{'@supports':{[fallback]:{width:'12px',height:'12px'}}});
globalStyle('::-webkit-scrollbar-track',{'@supports':{[fallback]:{background:vars.color.bgAlt}}});
globalStyle('::-webkit-scrollbar-thumb',{'@supports':{[fallback]:{background:vars.color.textDim,border:`3px solid ${vars.color.bgAlt}`,borderRadius:vars.radius.sm}}});
globalStyle('::-webkit-scrollbar-thumb:hover',{'@supports':{[fallback]:{background:vars.color.text}}});
globalStyle('::-webkit-scrollbar-corner',{'@supports':{[fallback]:{background:vars.color.bgAlt}}});
