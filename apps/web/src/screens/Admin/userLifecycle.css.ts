import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
export const root=style({minWidth:0});
export const actions=style({display:'flex',justifyContent:'flex-end',alignItems:'center',gap:vars.space[1]});
export const detailActions=style({display:'flex',flexWrap:'wrap',justifyContent:'flex-start',alignItems:'center',gap:vars.space[3]});
export const textAction=style({display:'inline-flex',alignItems:'center',justifyContent:'center',gap:vars.space[2],inlineSize:'auto',flex:'0 0 auto',textDecoration:'none'});
/** The list's action column: header and buttons share the end edge. `tr > th&` outranks the table's `th, td { text-align: start }`. */
export const actionColumn=style({selectors:{'tr > th&, tr > td&':{textAlign:'end',inlineSize:'1%',whiteSpace:'nowrap'}}});
/** An icon-only link drawn as the row's icon buttons: `.btn` border, no text padding. */
export const iconLink=style({padding:0,textDecoration:'none'});
export const error=style({color:vars.color.danger,fontSize:'.8rem',maxWidth:'24rem'});
