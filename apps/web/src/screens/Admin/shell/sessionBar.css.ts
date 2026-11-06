// The admin session bar (bottom status bar): tokens only, logical properties,
// every control keeps a visible focus ring, forced-colours safe.
//
// The bar is fixed to the bottom edge and `--admin-bar-h` (set on the console
// frame while the bar is shown) is subtracted from every full-height column, so
// nothing the bar covers is content.

import { style, keyframes, globalStyle } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

const PHONE = 'screen and (max-width: 760px)';
const ring = { outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`, outlineOffset: '2px' } as const;

const pulse = keyframes({
  '0%, 100%': { opacity: 1 },
  '50%': { opacity: 0.55 },
});

export const bar = style({
  position: 'fixed',
  insetInline: 0,
  insetBlockEnd: 0,
  blockSize: 'var(--admin-bar-h, 28px)',
  zIndex: 4,
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[3],
  paddingInline: vars.space[4],
  background: vars.color.bgAlt,
  color: vars.color.textDim,
  borderBlockStart: `1px solid ${vars.color.borderStrong}`,
  fontSize: '0.8125rem',
  lineHeight: 1,
  fontVariantNumeric: 'tabular-nums',
  overflow: 'visible',
  selectors: {
    '&[data-level="warning"]': { color: vars.color.text, borderBlockStartColor: vars.color.warning },
    '&[data-level="critical"]': { color: vars.color.text, borderBlockStartColor: vars.color.danger },
    '&[data-level="expired"]': { color: vars.color.text, borderBlockStartColor: vars.color.danger },
  },
  '@media': {
    '(forced-colors: active)': { borderBlockStart: '2px solid CanvasText' },
    [PHONE]: { paddingInline: vars.space[3], gap: vars.space[2] },
  },
});

/** The remaining fraction of the window, as a sliver along the top edge. */
export const sliver = style({
  position: 'absolute',
  insetBlockStart: '-1px',
  insetInlineStart: 0,
  blockSize: '2px',
  inlineSize: 'calc(100% * var(--admin-bar-fraction, 0))',
  background: vars.color.accent,
  pointerEvents: 'none',
  selectors: {
    [`${bar}[data-level="warning"] &`]: { background: vars.color.warning },
    [`${bar}[data-level="critical"] &, ${bar}[data-level="expired"] &`]: { background: vars.color.danger },
  },
  '@media': { '(forced-colors: active)': { background: 'Highlight' } },
});

export const detailsButton = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[2],
  minBlockSize: '24px',
  paddingInline: vars.space[2],
  background: 'transparent',
  color: 'inherit',
  border: '1px solid transparent',
  font: 'inherit',
  cursor: 'pointer',
  minInlineSize: 0,
  selectors: {
    '&:hover': { color: vars.color.text, borderColor: vars.color.border },
    '&:focus-visible': ring,
    [`${bar}[data-level="critical"] &`]: { fontWeight: vars.weight.bold },
  },
  '@media': {
    '(prefers-reduced-motion: no-preference)': {
      selectors: { [`${bar}[data-level="critical"] &`]: { animation: `${pulse} 1s ease-in-out infinite` } },
    },
    [PHONE]: { minBlockSize: '28px' },
  },
});

export const label = style({ whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' });
export const longLabel = style({ '@media': { [PHONE]: { display: 'none' } } });
export const shortLabel = style({ display: 'none', '@media': { [PHONE]: { display: 'inline' } } });

export const spacer = style({ flex: 1 });

export const action = style({
  display: 'inline-flex',
  alignItems: 'center',
  minBlockSize: '22px',
  paddingInline: vars.space[3],
  background: vars.color.surface,
  color: vars.color.text,
  border: `1px solid ${vars.color.borderStrong}`,
  font: 'inherit',
  fontWeight: vars.weight.bold,
  cursor: 'pointer',
  selectors: {
    '&:hover': { background: vars.color.bgSink },
    '&:focus-visible': ring,
    '&:disabled': { cursor: 'default', opacity: 0.6 },
  },
  '@media': { [PHONE]: { minBlockSize: '28px' } },
});

export const popover = style({
  position: 'absolute',
  insetInlineStart: vars.space[3],
  insetBlockEnd: 'calc(100% + 6px)',
  maxInlineSize: 'calc(100vw - 2 * 12px)',
  minInlineSize: '14rem',
  padding: vars.space[3],
  background: vars.color.surface,
  color: vars.color.text,
  border: `1px solid ${vars.color.borderStrong}`,
  boxShadow: vars.elevation[2],
  display: 'grid',
  gap: vars.space[2],
  fontSize: '0.8125rem',
  lineHeight: 1.35,
});

export const popoverRow = style({ display: 'flex', justifyContent: 'space-between', gap: vars.space[4] });
export const popoverTerm = style({ color: vars.color.textDim });
export const popoverNote = style({ margin: 0, color: vars.color.textDim });

/** Bottom-of-frame room for the bar: the page itself (phone) and the frame's own padding. */
globalStyle('[data-screen="admin"][data-session-bar]', {
  vars: { '--admin-bar-h': '28px' },
  paddingBlockEnd: 'var(--admin-bar-h)',
});

// Columns that were `100vh` tall are shorter by the bar while it is shown.
globalStyle('[data-screen="admin"][data-session-bar] [data-admin-column]', {
  maxBlockSize: 'calc(100vh - var(--admin-bar-h, 0px))',
  '@media': { [PHONE]: { maxBlockSize: 'none' } },
});
globalStyle('[data-screen="admin"][data-session-bar] [data-admin-column="resize"]', {
  blockSize: 'calc(100vh - var(--admin-bar-h, 0px))',
});
