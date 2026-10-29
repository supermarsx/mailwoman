// t95: the shared breadcrumb trail (admin console and user Settings).
// Tokens only, logical properties (RTL lays the trail out right to left and the
// chevron is a directional icon, so it mirrors). Each label is cut with an
// ellipsis; the full text is in the tooltip.

import { globalStyle, keyframes, style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const nav = style({ fontSize: '0.8rem', minInlineSize: 0, maxInlineSize: '100%' });
export const list = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: vars.space[2], listStyle: 'none', margin: 0, padding: 0, minInlineSize: 0 });
export const item = style({ display: 'inline-flex', alignItems: 'center', gap: vars.space[2], color: vars.color.textDim, minInlineSize: 0, maxInlineSize: '100%' });
export const separator = style({ flex: 'none', display: 'inline-flex' });

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
export const linkButton = style({
  font: 'inherit',
  color: vars.color.link,
  textDecoration: 'underline',
  background: 'none',
  border: 0,
  padding: 0,
  cursor: 'pointer',
  minInlineSize: 0,
  maxInlineSize: '100%',
});
globalStyle(`${item} a`, { color: vars.color.link, textDecoration: 'underline', minInlineSize: 0 });

const pulse = keyframes({ '0%': { opacity: 0.45 }, '50%': { opacity: 0.8 }, '100%': { opacity: 0.45 } });
// A name still being fetched: a quiet bar the size of a short word, never the id.
export const placeholder = style({
  display: 'inline-block',
  inlineSize: '6rem',
  blockSize: '0.8em',
  verticalAlign: 'middle',
  background: vars.color.border,
  animation: `${pulse} 1.2s ease-in-out infinite`,
  '@media': { '(prefers-reduced-motion: reduce)': { animation: 'none' } },
});
export const srOnly = style({
  position: 'absolute',
  width: '1px',
  height: '1px',
  overflow: 'hidden',
  clipPath: 'inset(50%)',
  whiteSpace: 'nowrap',
});
