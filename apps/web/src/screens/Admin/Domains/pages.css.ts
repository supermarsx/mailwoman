import { style,globalStyle } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

export const listRow = style({
  flexWrap: 'wrap',
  gap: vars.space[4],
  minInlineSize: 0,
});

/** Domain names navigate to their overview without a button-shaped surface. */
export const domainLink = style({
  display: 'inline-flex', alignItems: 'center', gap: vars.space[2],
  maxInlineSize: '100%', minInlineSize: 0, minBlockSize: vars.a11y.touchTarget,
  color: vars.color.text, fontWeight: vars.weight.medium, textDecoration: 'none',
  overflowWrap: 'anywhere', borderRadius: vars.radius.sm,
  selectors: {
    '&:hover': { color: vars.color.link, textDecoration: 'underline', textUnderlineOffset: '0.2em' },
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  },
  '@media': { '(forced-colors: active)': { selectors: { '&:focus-visible': { outline: `${vars.a11y.focusRingWidth} solid Highlight` } } } },
});
globalStyle(`${domainLink} > svg`, { flexShrink: 0, color: vars.color.textDim });
globalStyle(`tr:has(${domainLink}) > td`, { verticalAlign: 'middle' });
export const domainStatus = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1], lineHeight: 1.5 });
export const domainMeta = style({ color: vars.color.textDim, fontSize: '0.8rem' });
export const domainConnections = style({ color: vars.color.textDim, overflowWrap: 'anywhere', lineHeight: 1.5 });
export const domainActions = style({ display: 'flex', alignItems: 'center', flexWrap: 'nowrap', gap: vars.space[1] });

/** Aliases are short mappings: one compact row, with actions kept together. */
export const aliasList = style({margin:0,padding:0,listStyle:'none',display:'flex',flexWrap:'wrap',columnGap:vars.space[3],rowGap:vars.space[1],overflowWrap:'anywhere'});
export const aliasTable = style({width:'100%',tableLayout:'fixed',borderCollapse:'collapse'});
export const aliasName = style({overflowWrap:'anywhere',fontWeight:500});
export const aliasActionCell = style({width:'8.5rem','@media':{'(pointer: coarse)':{width:'9.25rem'}}});
export const aliasActions = style({display:'flex',justifyContent:'flex-end',gap:vars.space[1],flexWrap:'nowrap'});
export const aliasAction = style({display:'inline-flex',alignItems:'center',justifyContent:'center',width:'2.5rem',height:'2.5rem',minWidth:'2.5rem',padding:0,'@media':{'(pointer: coarse)':{width:'2.75rem',height:'2.75rem',minWidth:'2.75rem'}}});
globalStyle(`${aliasTable} th, ${aliasTable} td`,{padding:`${vars.space[1]} ${vars.space[2]}`,textAlign:'start',verticalAlign:'middle',borderBottom:`${vars.border.hair} solid ${vars.color.border}`});
