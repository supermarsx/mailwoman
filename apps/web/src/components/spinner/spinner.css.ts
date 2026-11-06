// The one busy indicator (toasts, async states, inline loading). A ring with one
// arc in currentColor; rotation is transform-only (compositor friendly), 0.9 s
// linear. Reduced motion (the OS setting or the in-app switch) swaps the
// rotation for a slow opacity pulse: still visibly busy, nothing travels.
// The global reduced-motion rule in theme/themes.css.ts skips [data-mw-spinner],
// otherwise its `animation-duration: 0.01ms !important` froze every spinner.
// Forced colours: the arc is currentColor (CanvasText family), the track GrayText.

import { keyframes, style } from '@vanilla-extract/css';

const spin = keyframes({ from: { transform: 'rotate(0deg)' }, to: { transform: 'rotate(360deg)' } });
const pulse = keyframes({ '0%, 100%': { opacity: 1 }, '50%': { opacity: 0.35 } });

export const spinner = style({
  display: 'inline-block',
  flex: 'none',
  boxSizing: 'border-box',
  inlineSize: '16px',
  blockSize: '16px',
  border: `2px solid color-mix(in srgb, currentColor 25%, transparent)`,
  borderBlockStartColor: 'currentColor',
  borderRadius: '50%',
  transformOrigin: 'center',
  transformBox: 'fill-box',
  animation: `${spin} 0.9s linear infinite`,
  '@media': {
    '(prefers-reduced-motion: reduce)': { animation: `${pulse} 1.6s ease-in-out infinite` },
    '(forced-colors: active)': { borderColor: 'GrayText', borderBlockStartColor: 'CanvasText' },
  },
  selectors: {
    ':root[data-reduced-motion] &': { animation: `${pulse} 1.6s ease-in-out infinite` },
  },
});

export const label = style({
  position: 'absolute',
  inlineSize: '1px',
  blockSize: '1px',
  overflow: 'hidden',
  clipPath: 'inset(50%)',
  whiteSpace: 'nowrap',
});
