import { keyframes, style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

const appear = keyframes({ from: { opacity: 0 }, to: { opacity: 1 } });

// Reserve the same small footprint while fast requests finish. Delay only the
// visual indicator, never the request, resolved content, or live announcement.
export const pending = style({
  minBlockSize: '2.75rem',
  display: 'flex',
  alignItems: 'center',
  marginBlock: vars.space[3],
});

export const indicator = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[2],
  animation: `${appear} 160ms ease-out 120ms both`,
  '::before': {
    content: '""',
    inlineSize: '0.5rem',
    blockSize: '0.5rem',
    flexShrink: 0,
    background: vars.color.accent,
  },
  selectors: { ':root[data-reduced-motion] &': { animation: 'none' } },
  '@media': { '(prefers-reduced-motion: reduce)': { animation: 'none' } },
});
