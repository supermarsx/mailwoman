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

/** The message body's slot in the reader (t79). The zoom is applied here, from
 *  outside the frame, so changing it never reloads the message document: the
 *  frame is laid out at 1/zoom of the box and scaled back up to fill it, which
 *  reflows the text exactly as a zoom on the document root would and scales
 *  images by the same factor. The box keeps the frame's flex slot, border and
 *  paper, so the reader chrome around it does not move. The level arrives as a
 *  custom property set through the CSSOM (no inline style attribute). No
 *  transition: a step lands in one frame, with or without reduced motion. */
export const zoomVar = '--mw-reader-zoom';
export const box = style({
  position: 'relative',
  flex: '1 0 12rem',
  minHeight: '12rem',
  width: '100%',
  overflow: 'hidden',
  border: 'var(--border-rule) solid var(--border-strong)',
  background: 'var(--surface)',
  '@media': {
    '(max-width: 760px)': { selectors: { '.reader--open &': { flex: '1 1 0', minHeight: 0 } } },
  },
});
globalStyle(`${box} > iframe.reader__frame`, {
  position: 'absolute',
  top: 0,
  left: 0,
  minHeight: 0,
  border: 0,
  width: `calc(100% / var(${zoomVar}, 1))`,
  height: `calc(100% / var(${zoomVar}, 1))`,
  transform: `scale(var(${zoomVar}, 1))`,
  // Physical on purpose: the scale grows from the corner the frame is pinned
  // to, in either direction (the message document keeps its own direction).
  transformOrigin: '0 0',
});
