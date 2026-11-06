import {style} from '@vanilla-extract/css';
import {vars} from '../theme/contract.css.ts';
// The floating bar over the sidebar folder list while folders are selected.
export const bar=style({position:'sticky',insetBlockStart:0,zIndex:4,display:'flex',flexWrap:'wrap',alignItems:'center',gap:'0.25rem',padding:'0.3rem 0.4rem',background:vars.color.bg,borderBottom:`1px solid ${vars.color.borderStrong}`,boxShadow:`0 1px 0 ${vars.color.borderStrong}`});
export const count=style({flex:'1 0 100%',fontSize:'0.8em',color:vars.color.textDim});
export const action=style({display:'inline-flex',alignItems:'center',gap:'0.25rem',padding:'0.15rem 0.4rem',fontSize:'0.8em'});
