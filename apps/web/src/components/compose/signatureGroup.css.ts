import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const split = style({ display: 'inline-flex', alignItems: 'stretch' });
export const menu = style({ minWidth: '16rem', maxWidth: '22rem', maxHeight: '70vh', overflow: 'auto' });
export const heading = style({ margin: 0, padding: '.35rem .6rem .1rem', fontSize: '.72rem', fontWeight: 'var(--weight-bold)', opacity: 0.75 });
export const rule = style({ margin: '.25rem 0', border: 0, borderTop: `1px solid ${vars.color.border}` });
export const choice = style({ display: 'grid', gridTemplateColumns: '1rem 1fr auto', alignItems: 'center', columnGap: '.4rem', width: '100%', textAlign: 'start' });
export const preview = style({ gridColumn: '2 / 4', fontSize: '.72rem', opacity: 0.7, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' });
export const badge = style({ fontSize: '.68rem', padding: '0 .35rem', borderRadius: vars.radius.sm, border: `1px solid ${vars.color.border}` });
export const chip = style({ display: 'flex', alignItems: 'center', gap: '.5rem', padding: '.25rem .5rem', fontSize: '.8rem', border: `1px solid ${vars.color.border}`, borderRadius: vars.radius.sm });
export const dialogBody = style({ display: 'grid', gridTemplateColumns: 'minmax(11rem, 14rem) 1fr', gap: vars.space[3], minHeight: '22rem', width: 'min(60rem, 92vw)' });
export const list = style({ listStyle: 'none', margin: 0, padding: 0, overflow: 'auto', maxHeight: '60vh' });
export const row = style({ display: 'grid', gap: '.1rem', width: '100%', textAlign: 'start' });
export const editorPane = style({ minWidth: 0, overflow: 'auto', maxHeight: '70vh' });

export const scrim=style({position:'fixed',inset:0,zIndex:2000,background:'rgba(0,0,0,.4)',display:'grid',placeItems:'center',padding:'1rem'});
export const manager=style({background:vars.color.surface,borderRadius:vars.radius.sm,padding:'1rem',width:'min(64rem,96vw)',maxHeight:'90vh',overflow:'auto'});
