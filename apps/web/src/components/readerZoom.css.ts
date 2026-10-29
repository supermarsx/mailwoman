// Styles for the reader zoom group (ReaderZoom.tsx, t59).

import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

/** The three controls read as one compact segment on the action row. */
export const group = style({
  display: 'inline-flex',
  alignItems: 'center',
  flex: '0 0 auto',
  gap: 0,
});

/** The current level: a small button that resets to 100%. Tabular digits keep
 *  the group the same width at every level, so the row does not re-fold. */
export const level = style({
  minInlineSize: '3.25rem',
  paddingInline: '0.25rem',
  fontSize: '0.75rem',
  fontVariantNumeric: 'tabular-nums',
  color: vars.color.textDim,
  justifyContent: 'center',
});

/** On the reader root while Ctrl/Cmd is held: the body frame stops taking
 *  pointer events, so Ctrl/Cmd + wheel over it reaches the reader. */
export const gesture = style({});
globalStyle(`${gesture} iframe.reader__frame`, { pointerEvents: 'none' });
