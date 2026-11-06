import { style, globalStyle } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

export const dialogForm = style({ width: 'min(34rem, 100%)' });
globalStyle(`${dialogForm} .mw-insert-actions`, { display: 'flex', justifyContent: 'flex-end', gap: vars.space[2] });
globalStyle(`${dialogForm} .mw-insert-row`, { display: 'flex', gap: vars.space[2], flexWrap: 'wrap', alignItems: 'end' });
globalStyle(`${dialogForm} .mw-insert-row > *`, { flex: '1 1 8rem', minWidth: 0 });
globalStyle(`${dialogForm} .mw-insert-check`, { display: 'flex', flexDirection: 'row', alignItems: 'center', gap: vars.space[2] });
globalStyle(`${dialogForm} .mw-insert-check input`, { width: 'auto' });
globalStyle(`${dialogForm} .mw-insert-warn`, { margin: 0, fontSize: '.8rem', color: vars.color.warning });
globalStyle(`${dialogForm} .mw-insert-preview`, { border: `1px solid ${vars.color.border}`, borderRadius: vars.radius.sm, padding: vars.space[2], overflow: 'auto', maxHeight: '12rem', background: '#ffffff', color: '#222222' });
globalStyle(`${dialogForm} .mw-insert-grid`, { display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(5.5rem, 1fr))', gap: vars.space[2], maxHeight: '18rem', overflow: 'auto' });
globalStyle(`${dialogForm} .mw-insert-thumb`, { display: 'grid', gap: '.2rem', padding: '.25rem', border: `2px solid transparent`, borderRadius: vars.radius.sm, background: 'transparent', cursor: 'pointer', font: 'inherit', color: 'inherit', textAlign: 'start' });
globalStyle(`${dialogForm} .mw-insert-thumb[aria-pressed="true"]`, { borderColor: vars.color.accent });
globalStyle(`${dialogForm} .mw-insert-thumb img`, { width: '100%', height: '4.5rem', objectFit: 'cover', borderRadius: '2px', background: vars.color.surface });
globalStyle(`${dialogForm} .mw-insert-thumb span`, { fontSize: '.7rem', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' });

export const shapeGallery = style({ display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)', gap: vars.space[1] });
globalStyle(`${shapeGallery} button`, { display: 'grid', placeItems: 'center', padding: '.25rem', border: `1px solid ${vars.color.border}`, borderRadius: vars.radius.sm, background: vars.color.bg, cursor: 'pointer' });
globalStyle(`${shapeGallery} button[aria-pressed="true"]`, { borderColor: vars.color.accent, outline: `2px solid ${vars.color.accent}` });

export const dropOverlay = style({
  position: 'fixed', inset: 0, zIndex: 1400, display: 'grid', placeItems: 'center', padding: vars.space[4],
  background: vars.color.scrim, pointerEvents: 'none',
});
export const dropCard = style({
  display: 'grid', gap: vars.space[2], justifyItems: 'center', textAlign: 'center', padding: vars.space[5],
  border: `3px dashed ${vars.color.accent}`, borderRadius: vars.radius.md, background: vars.color.bg, color: vars.color.text,
  maxWidth: 'min(32rem, 100%)', boxShadow: vars.elevation[2],
});
globalStyle(`${dropCard} p`, { margin: 0 });
globalStyle(`${dropCard} .mw-drop-note`, { fontSize: '.85rem', color: vars.color.textDim });
globalStyle(`${dropCard} .mw-drop-bad`, { fontSize: '.85rem', color: vars.color.warning });
