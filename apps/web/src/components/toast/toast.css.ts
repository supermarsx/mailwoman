// The toast region and its toasts (t32 f39). Tokens only (vars.*), logical
// properties only, so every theme, accent, appearance and text direction
// follows; forced colours keep the borders and the icons (currentColor).
//
// Desktop: a column at the bottom inline-end corner, newest nearest the corner.
// Phone (the shell's 760px breakpoint, components/narrowViewport.ts): full width
// at the top, below the safe area and below the top bar (44px controls plus its
// padding and rule, under 60px), so the menu and Compose stay reachable. The region is an overlay (position: fixed),
// so it never moves the page. The region itself lets the pointer through; only
// the toasts take clicks.
//
// Box size: a toast is one 24px control row (touch target) inside 4px of
// padding and a 2px rule, 36px tall, the height of the toast it replaced.

import { keyframes, style, styleVariants } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

const PHONE = '(max-width: 760px)';
const REDUCED = '(prefers-reduced-motion: reduce)';

export const region = style({
  position: 'fixed',
  // Raised above a floating bar it would cover (ToastRegion sets the lift).
  insetBlockEnd: `max(${vars.space[5]}, var(--mw-toast-lift, 0px))`,
  insetInlineEnd: vars.space[5],
  zIndex: 1100,
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'flex-end',
  width: `min(28rem, calc(100vw - 2 * ${vars.space[5]}))`,
  pointerEvents: 'none',
  '@media': {
    [PHONE]: {
      insetBlockStart: 'calc(3.75rem + env(safe-area-inset-top))',
      insetBlockEnd: 'auto',
      insetInlineStart: vars.space[3],
      insetInlineEnd: vars.space[3],
      width: 'auto',
      alignItems: 'stretch',
    },
  },
});

/** One live region (alerts, then statuses); an empty one takes no space. */
export const stack = style({
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'inherit',
  gap: vars.space[3],
  maxWidth: '100%',
  selectors: {
    '&:not(:empty) + &:not(:empty)': { marginBlockStart: vars.space[3] },
  },
});

const rise = keyframes({
  from: { opacity: 0, transform: `translateY(${vars.space[3]})` },
  to: { opacity: 1, transform: 'none' },
});
const drop = keyframes({
  from: { opacity: 0, transform: `translateY(calc(-1 * ${vars.space[3]}))` },
  to: { opacity: 1, transform: 'none' },
});

export const item = style({
  pointerEvents: 'auto',
  boxSizing: 'border-box',
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[3],
  width: 'fit-content',
  maxWidth: '100%',
  minWidth: 0,
  paddingBlock: vars.space[2],
  paddingInlineStart: vars.space[3],
  paddingInlineEnd: vars.space[2],
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderInlineStartWidth: vars.border.heavy,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  color: vars.color.text,
  boxShadow: vars.elevation[2],
  fontSize: vars.fontSize.base,
  lineHeight: vars.lineHeight.ui,
  animation: `${rise} ${vars.a11y.motionDurationSlow} ease-out`,
  '@media': {
    [PHONE]: { width: 'auto', animationName: drop },
    [REDUCED]: { animation: 'none' },
  },
});

/** The inline-start rule and the icon take the tone's colour; the icon shape says it too. */
export const tone = styleVariants({
  success: { borderInlineStartColor: vars.color.success },
  info: { borderInlineStartColor: vars.color.accent },
  warning: { borderInlineStartColor: vars.color.warning },
  error: { borderInlineStartColor: vars.color.danger },
});

export const icon = style({
  flex: 'none',
  width: '16px',
  height: '16px',
});

const spin = keyframes({ to: { transform: 'rotate(360deg)' } });

/** The busy toast's icon (t111): a ring with one arc in the tone's colour, the icon's size.
 *  Reduced motion: the ring stays still; the message says the work is running. */
export const spinner = style({
  flex: 'none',
  boxSizing: 'border-box',
  width: '16px',
  height: '16px',
  border: `2px solid ${vars.color.borderStrong}`,
  borderTopColor: 'currentColor',
  borderRadius: '50%',
  animation: `${spin} 0.8s linear infinite`,
  '@media': {
    [REDUCED]: { animation: 'none' },
  },
  selectors: {
    ':root[data-reduced-motion] &': { animation: 'none' },
  },
});

export const iconTone = styleVariants({
  success: { color: vars.color.success },
  info: { color: vars.color.accent },
  warning: { color: vars.color.warning },
  error: { color: vars.color.danger },
});

export const message = style({
  flex: '1 1 auto',
  minWidth: 0,
  overflowWrap: 'anywhere',
});

export const countdown = style({
  flex: 'none',
  color: vars.color.textDim,
  fontVariantNumeric: 'tabular-nums',
});

const control = style({
  flex: 'none',
  minHeight: vars.a11y.touchTarget,
  border: `${vars.border.rule} solid transparent`,
  borderRadius: vars.radius.sm,
  background: 'transparent',
  color: 'inherit',
  font: 'inherit',
  cursor: 'pointer',
  selectors: {
    '&:hover': { borderColor: vars.color.borderStrong },
    '&:focus-visible': {
      outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`,
      outlineOffset: '1px',
    },
  },
});

export const action = style([
  control,
  {
    paddingBlock: 0,
    paddingInline: vars.space[3],
    borderColor: vars.color.borderStrong,
    fontWeight: vars.weight.medium,
    whiteSpace: 'nowrap',
  },
]);

export const close = style([
  control,
  {
    display: 'inline-flex',
    alignItems: 'center',
    justifyContent: 'center',
    minWidth: vars.a11y.touchTarget,
    padding: 0,
  },
]);

export const closeIcon = style({
  width: '12px',
  height: '12px',
});
