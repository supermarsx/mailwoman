import {globalStyle} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
globalStyle('.conversation',{display:'flex',flexDirection:'column',alignItems:'flex-start',gap:vars.space[3],minInlineSize:0});
globalStyle('.conversation p',{margin:0});
globalStyle('.conversation__members',{listStyle:'none',margin:0,padding:0,maxBlockSize:'18rem',overflowY:'auto',maxInlineSize:'100%',display:'grid',gap:vars.space[3]});
globalStyle('.conversation__members .btn',{flexDirection:'column',alignItems:'flex-start',whiteSpace:'normal',textAlign:'start',overflowWrap:'anywhere',maxInlineSize:'100%'});
globalStyle('.conversation__members [aria-current="true"]',{borderColor:vars.color.signal});

globalStyle('.conversation__trigger',{padding:vars.space[1],minInlineSize:'1.75rem',minBlockSize:'1.75rem','@media':{'(pointer: coarse)':{minInlineSize:vars.a11y.touchTarget,minBlockSize:vars.a11y.touchTarget}}});
