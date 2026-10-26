import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';
import { listRow } from './admin.css.ts';

export const row=style([listRow,{
  display:'grid',gridTemplateColumns:'minmax(0, 1fr) auto',gap:vars.space[5],padding:vars.space[5],
  '@media':{'(max-width: 600px)':{gridTemplateColumns:'minmax(0, 1fr)'}},
}]);
export const content=style({minInlineSize:0,overflowWrap:'anywhere'});
export const action=style({justifySelf:'start',whiteSpace:'nowrap'});
