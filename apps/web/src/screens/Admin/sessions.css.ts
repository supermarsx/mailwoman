import { style } from '@vanilla-extract/css';
export const cell = style({ display: 'block', maxInlineSize: '16rem', overflowWrap: 'anywhere' });
export const origin = style({ display: 'flex', flexDirection: 'column', gap: '2px', marginTop: '4px', fontSize: '0.85em', color: 'var(--text-dim)' });
