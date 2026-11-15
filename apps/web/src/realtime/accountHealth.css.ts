import {globalStyle} from '@vanilla-extract/css';
globalStyle('.connection-toast--accounts',{display:'block',paddingBlock:'0.35rem',fontSize:'0.8rem'});
globalStyle('.connection-toast--accounts summary',{cursor:'pointer',fontWeight:600});
globalStyle('.connection-toast--accounts ul',{listStyle:'none',padding:0,margin:'0.4rem 0 0',display:'grid',gap:'0.35rem'});
globalStyle('.connection-toast--accounts li',{display:'flex',alignItems:'center',flexWrap:'wrap',gap:'0.35rem 0.75rem',minWidth:0});
globalStyle('.connection-toast--accounts li bdi',{maxWidth:'28ch',overflow:'hidden',textOverflow:'ellipsis',whiteSpace:'nowrap'});
