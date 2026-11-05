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

export const row = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: vars.space[3] });
export const textarea = style({
  inlineSize: '100%', minBlockSize: '6rem', font: 'inherit', padding: vars.space[2],
  border: `${vars.border.hair} solid ${vars.color.border}`, borderRadius: vars.radius.sm, background: vars.color.bg, color: vars.color.text,
  ':focus-visible': { outline: `2px solid ${vars.color.text}`, outlineOffset: '2px' },
});
export const field = style({
  font: 'inherit', padding: vars.space[2], minInlineSize: 0,
  border: `${vars.border.hair} solid ${vars.color.border}`, borderRadius: vars.radius.sm, background: vars.color.bg, color: vars.color.text,
  ':focus-visible': { outline: `2px solid ${vars.color.text}`, outlineOffset: '2px' },
});
export const score = style({ inlineSize: '4.5rem' });
export const body = style({ whiteSpace: 'pre-wrap', margin: 0 });
export const sectionBox = style({
  display: 'flex', flexDirection: 'column', gap: vars.space[2], padding: vars.space[3],
  border: `${vars.border.hair} solid ${vars.color.border}`, borderRadius: vars.radius.md,
});
export const warn = style({ margin: 0, paddingInlineStart: vars.space[5] });
export const matrix = style({ borderCollapse: 'collapse' });
globalStyle(`${matrix} td, ${matrix} th`, { border: `${vars.border.hair} solid ${vars.color.border}`, padding: vars.space[2], textAlign: 'center', minInlineSize: '3rem' });
export const linkButton = style({
  background: 'none', border: 0, padding: 0, font: 'inherit', color: vars.color.text, textDecoration: 'underline', cursor: 'pointer',
  ':focus-visible': { outline: `2px solid ${vars.color.text}`, outlineOffset: '2px' },
  ':hover': { textDecoration: 'none' },
});
