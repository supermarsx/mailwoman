// The one busy indicator (toasts, async states, inline loading). A ring with one
// arc in currentColor; rotation is transform-only (compositor friendly), 0.9 s
// linear. Reduced motion (the OS setting or the in-app switch) slows the
// rotation while keeping the ring visibly spinning.
// The global reduced-motion rule in theme/themes.css.ts skips [data-mw-spinner],
// otherwise its `animation-duration: 0.01ms !important` froze every spinner.
// Forced colours: the arc is currentColor (CanvasText family), the track GrayText.

import { keyframes, style } from '@vanilla-extract/css';

const spin = keyframes({ from: { transform: 'rotate(0deg)' }, to: { transform: 'rotate(360deg)' } });

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
    '(prefers-reduced-motion: reduce)': { animationDuration: '1.8s' },
    '(forced-colors: active)': { borderColor: 'GrayText', borderBlockStartColor: 'CanvasText' },
  },
  selectors: {
    ':root[data-reduced-motion] &': { animationDuration: '1.8s' },
  },
});

// The reduced-motion stand-in for any looping busy motion (skeleton shimmer,
// sync icons): a slow opacity fade, never a frozen shape. `calmForce` beats the
// global `animation-duration: 0.01ms !important` that the in-app switch applies
// (that rule has zero specificity except its :not(); a class rule wins).
const calm = keyframes({ '0%, 100%': { opacity: 1 }, '50%': { opacity: 0.55 } });
export const calmAnimation = `${calm} 3s ease-in-out infinite`;
export const calmForce = { animationDuration: '3s !important', animationIterationCount: 'infinite !important' } as const;

export const label = style({
  position: 'absolute',
  inlineSize: '1px',
  blockSize: '1px',
  overflow: 'hidden',
  clipPath: 'inset(50%)',
  whiteSpace: 'nowrap',
});
