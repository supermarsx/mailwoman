import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

export const cards = style({ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(min(100%, 16rem), 1fr))', gap: vars.space[4] });
export const areaCard = style({
  display: 'flex', flexDirection: 'column', gap: vars.space[2], padding: vars.space[4],
  border: `${vars.border.hair} solid ${vars.color.border}`, borderRadius: vars.radius.md,
  background: vars.color.bg, color: vars.color.text, textDecoration: 'none',
  ':hover': { background: vars.color.bgAlt },
  ':focus-visible': { outline: `2px solid ${vars.color.text}`, outlineOffset: '2px' },
});
globalStyle(`${areaCard} h3`, { margin: 0, fontSize: '1rem' });
globalStyle(`${areaCard} p`, { margin: 0, color: vars.color.textDim });
export const stateLine = style({ fontSize: '0.8rem', color: vars.color.textDim });
