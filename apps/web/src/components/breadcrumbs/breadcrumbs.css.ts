// t95: the shared breadcrumb trail (admin console and user Settings).
// Tokens only, logical properties (RTL lays the trail out right to left and the
// chevron is a directional icon, so it mirrors). Each label is cut with an
// ellipsis; the full text is in the tooltip.

import { globalStyle, keyframes, style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';
import { calmAnimation, calmForce } from '../spinner/spinner.css.ts';
import { linkColor } from '../../theme/accentLink.ts';

export const nav = style({ fontSize: '0.8rem', minInlineSize: 0, maxInlineSize: '100%' });
export const list = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: vars.space[2], listStyle: 'none', margin: 0, padding: 0, minInlineSize: 0 });
export const item = style({ display: 'inline-flex', alignItems: 'center', gap: vars.space[2], color: vars.color.textDim, minInlineSize: 0, maxInlineSize: '100%' });
export const separator = style({ flex: 'none', display: 'inline-flex', color: vars.color.textDim });

// One level's text. Earlier levels are held shorter than the current page so a
// long trail keeps the page name readable; on a phone both are narrower.
export const label = style({
  display: 'inline-block',
  minInlineSize: 0,
  maxInlineSize: '18rem',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
  verticalAlign: 'bottom',
  '@media': { '(max-width: 760px)': { maxInlineSize: '9rem' } },
});
export const current = style({
  color: vars.color.text,
  fontWeight: vars.weight.bold,
  maxInlineSize: '28rem',
  '@media': { '(max-width: 760px)': { maxInlineSize: '14rem' } },
});
// Ancestor crumbs: the accent-derived link colour (4.5:1 on the surface), else
// the theme's link. Underline kept so colour is never the only cue; visited
// links read the same as unvisited ones (a trail is navigation, not history).
const linkBase = {
  color: linkColor,
  textDecoration: 'underline',
  textDecorationThickness: '1px',
  textUnderlineOffset: '0.2em',
  cursor: 'pointer',
  minInlineSize: 0,
} as const;
export const linkButton = style({
  ...linkBase,
  font: 'inherit',
  background: 'none',
  border: 0,
  padding: 0,
  maxInlineSize: '100%',
});
const linkStates = {
  ':visited': { color: linkColor },
  ':hover': { textDecorationThickness: '2px', color: vars.color.text },
  ':active': { color: vars.color.text },
  ':focus-visible': { outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`, outlineOffset: '2px' },
} as const;
globalStyle(`${item} a`, linkBase);
for (const [state, rule] of Object.entries(linkStates)) {
  globalStyle(`${item} a${state}`, rule);
  globalStyle(`${linkButton}${state}`, rule);
}

const pulse = keyframes({ '0%': { opacity: 0.45 }, '50%': { opacity: 0.8 }, '100%': { opacity: 0.45 } });
// A name still being fetched: a quiet bar the size of a short word, never the id.
export const placeholder = style({
  display: 'inline-block',
  inlineSize: '6rem',
  blockSize: '0.8em',
  verticalAlign: 'middle',
  background: vars.color.border,
  animation: `${pulse} 1.2s ease-in-out infinite`,
  '@media': { '(prefers-reduced-motion: reduce)': { animation: calmAnimation } },
  selectors: { ':root[data-reduced-motion] &': { animation: calmAnimation, ...calmForce } },
});
export const srOnly = style({
  position: 'absolute',
  width: '1px',
  height: '1px',
  overflow: 'hidden',
  clipPath: 'inset(50%)',
  whiteSpace: 'nowrap',
});
