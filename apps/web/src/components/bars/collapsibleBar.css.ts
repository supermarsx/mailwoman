// Collapsible bar (t38 §2.1): a pinned bar sits in the flow; an auto-hide bar
// shows only its header (ribbon tab names) or a slim handle, and its panel
// either slides over the content below (`overlay`) or into the flow (`push`).
//
// State lives in data attributes on the root: `data-mode` (pinned|auto),
// `data-expanded` (true|false), `data-reveal` (overlay|push) and
// `data-motion` (full|reduced). Motion is a 160 ms slide on the block axis only,
// so it reads the same in RTL; reduced motion (the media query or
// `:root[data-reduced-motion]`) shows and hides at once.

import { globalStyle, keyframes, style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const REVEAL_MS = 160;

const slideIn = keyframes({
  from: { opacity: 0, transform: 'translateY(-6px)' },
  to: { opacity: 1, transform: 'none' },
});

export const root = style({
  position: 'relative',
  display: 'grid',
  gridTemplateColumns: 'minmax(0, 1fr) auto',
  flex: 'none',
  minInlineSize: 0,
  userSelect: 'none',
  WebkitUserSelect: 'none',
  // Consumer chrome classes can also declare display:flex; the bar owns its layout.
  selectors: { '&[data-bar]': { display: 'grid' } },
});

globalStyle(`${root} :is(input,textarea,[contenteditable]:not([contenteditable="false"]))`, { userSelect: 'text', WebkitUserSelect: 'text' });

export const header = style({ display: 'block', gridRow: 1, gridColumn: 1, minInlineSize: 0, userSelect:'none', WebkitUserSelect:'none' });
globalStyle(`${header} :is(input,textarea,[contenteditable]:not([contenteditable="false"]))`,{userSelect:'text',WebkitUserSelect:'text'});

// The slim handle: a 6px visual rule inside a 12px (fine pointer) or 24px
// (coarse pointer) hit area. With a label (the active tab's title) it reads as
// a one-line strip of text at the same height.
export const handle = style({
  appearance: 'none',
  position: 'relative',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  inlineSize: '100%',
  gridRow: 1,
  gridColumn: 1,
  alignSelf: 'center',
  blockSize: '12px',
  margin: 0,
  padding: `0 ${vars.space[2]}`,
  border: 'none',
  background: 'transparent',
  color: vars.color.textDim,
  font: 'inherit',
  fontSize: '0.7rem',
  lineHeight: '12px',
  whiteSpace: 'nowrap',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  cursor: 'pointer',
  '::before': {
    content: '""',
    position: 'absolute',
    insetInline: 0,
    insetBlockStart: '3px',
    blockSize: '6px',
    background: vars.color.bgAlt,
    borderBlockEnd: `${vars.border.hair} solid ${vars.color.borderStrong}`,
    zIndex: -1,
  },
  ':focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  '@media': { '(pointer: coarse)': { blockSize: '24px', lineHeight: '24px', fontSize: '0.8rem' } },
});

export const handleText = style({
  display: 'block',
  blockSize: '100%',
  // 10px text keeps the glyph box (ascent + descent) inside the 12px line.
  lineHeight: '12px',
  fontSize: '10px',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  maxInlineSize: '100%',
  '@media': { '(pointer: coarse)': { lineHeight: '24px', fontSize: '0.8rem' } },
});

export const panel = style({
  gridColumn: '1 / -1',
  gridRow: 2,
  display: 'flex',
  alignItems: 'flex-start',
  minInlineSize: 0,
  selectors: {
    [`${root}[data-header="false"][data-mode="pinned"] > &`]: { gridColumn: 1, gridRow: 1 },
    // Auto-hide, overlay: lifted out of the flow, over the content below,
    // under toasts (1100) and menus (1500).
    [`${root}[data-mode="auto"][data-reveal="overlay"] > &`]: {
      position: 'absolute',
      insetInline: 0,
      insetBlockStart: '100%',
      zIndex: 900,
      background: vars.color.surface,
      boxShadow: vars.elevation[2],
      transition: `opacity ${REVEAL_MS}ms ease-out, transform ${REVEAL_MS}ms ease-out, visibility 0s linear 0s`,
    },
    [`${root}[data-mode="auto"][data-reveal="overlay"][data-expanded="false"] > &`]: {
      visibility: 'hidden',
      opacity: 0,
      transform: 'translateY(-6px)',
      pointerEvents: 'none',
      transition: `opacity ${REVEAL_MS}ms ease-in, transform ${REVEAL_MS}ms ease-in, visibility 0s linear ${REVEAL_MS}ms`,
    },
    // Auto-hide, push: in the flow, so the content below shrinks and is never covered.
    [`${root}[data-mode="auto"][data-reveal="push"][data-expanded="false"] > &`]: { display: 'none' },
    [`${root}[data-mode="auto"][data-reveal="push"][data-expanded="true"] > &`]: {
      animation: `${slideIn} ${REVEAL_MS}ms ease-out`,
    },
    // Reduced motion: no slide, no fade. Each selector carries more attributes
    // than the auto-hide rules above (four plus the root), so it wins over
    // their transform, transition and animation rather than losing to them.
    [`${root}[data-bar][data-mode][data-reveal][data-motion="reduced"] > &`]: { transition: 'none', animation: 'none', transform: 'none' },
    [`:root[data-reduced-motion] ${root}[data-bar][data-mode][data-reveal] > &`]: { transition: 'none', animation: 'none', transform: 'none' },
  },
  '@media': {
    '(prefers-reduced-motion: reduce)': {
      selectors: {
        [`${root}[data-bar][data-mode][data-reveal][data-expanded] > &`]: { transition: 'none', animation: 'none', transform: 'none' },
      },
    },
  },
});

export const content = style({ flex: '1 1 auto', minInlineSize: 0 });

// Both modes stay reachable beside the header/handle, outside the hidden panel.
export const modes = style({
  gridColumn: 2,
  gridRow: 1,
  alignSelf: 'center',
  display: 'flex',
  alignItems: 'center',
  gap: '2px',
  padding: '2px',
});

export const modeButton = style({
  appearance: 'none',
  flex: 'none',
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  inlineSize: '24px',
  blockSize: '24px',
  margin: 0,
  padding: 0,
  border: `${vars.border.rule} solid transparent`,
  borderRadius: vars.radius.sm,
  background: 'transparent',
  color: vars.color.textDim,
  cursor: 'pointer',
  transition: 'background-color 120ms ease, border-color 120ms ease, color 120ms ease',
  selectors: {
    '&[aria-pressed="true"]': { color: vars.color.accent, background: `color-mix(in srgb, ${vars.color.accent} 14%, transparent)`, borderColor: `color-mix(in srgb, ${vars.color.accent} 40%, transparent)` },
    '&:hover:not(:disabled)': { color: vars.color.accent, background: `color-mix(in srgb, ${vars.color.accent} 22%, transparent)`, borderColor: vars.color.accent },
    '&:disabled': { cursor: 'not-allowed', opacity: 0.6 },
  },
  ':focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  '@media': {
    '(pointer: coarse)': { inlineSize: '28px', blockSize: '28px' },
    '(prefers-reduced-motion: reduce)': { transition: 'none' },
  },
});
