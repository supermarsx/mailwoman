import { keyframes, style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';
const enter = keyframes({ from: { opacity: 0 }, to: { opacity: 1 } });
export const panel = style({
  padding: vars.space[6], minBlockSize: '12rem', boxSizing: 'border-box',
  animation: `${enter} 160ms ease-out var(--reader-loading-delay, 120ms) both`,
  selectors: { ':root[data-reduced-motion] &': { animation: 'none' } },
  '@media': { '(prefers-reduced-motion: reduce)': { animation: 'none' } },
});
export const lines = style({ display: 'grid', gap: vars.space[5], marginBlockStart: vars.space[5], maxInlineSize: '42rem' });
export const line = style({ blockSize: '0.65rem', background: vars.color.border, selectors: { '&:last-child': { inlineSize: '60%' } } });
