// Styles of the app colour picker (t81). Every chrome colour is a theme token;
// the picked colour itself (swatch fills, the preview, thumb positions) is set
// through the CSSOM at run time, which the CSP allows. The saturation/value
// area and the hue and alpha strips are canvases, so no CSS gradient is needed.

import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const root = style({ display: 'inline-flex', position: 'relative', verticalAlign: 'middle' });

/** The trigger: a swatch of the current value, optionally an icon and a label. */
export const trigger = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[2],
  minBlockSize: vars.a11y.touchTarget,
  minInlineSize: vars.a11y.touchTarget,
  padding: `${vars.space[1]} ${vars.space[2]}`,
  color: vars.color.text,
  background: vars.color.surface,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  font: 'inherit',
  cursor: 'pointer',
  selectors: {
    '&:disabled': { cursor: 'not-allowed', opacity: 0.6 },
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  },
});

/** A bare trigger for ribbons: the host's button class gives it its shape. */
export const triggerBare = style({ display: 'inline-flex', alignItems: 'center', gap: vars.space[1] });

export const chip = style({
  display: 'inline-block',
  inlineSize: '1.1rem',
  blockSize: '1.1rem',
  flex: 'none',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  background: vars.color.bgSink,
});

/** The value bar under a ribbon icon (the text or highlight colour). */
export const bar = style({
  display: 'block',
  inlineSize: '1rem',
  blockSize: '4px',
  marginBlockStart: '1px',
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  background: vars.color.bgSink,
});

export const stack = style({ display: 'inline-flex', flexDirection: 'column', alignItems: 'center' });

export const popover = style({
  position: 'fixed',
  zIndex: 10001,
  inset: 'auto',
  margin: 0,
  boxSizing: 'border-box',
  inlineSize: 'min(18.5rem, calc(100vw - 16px))',
  overflowY: 'auto',
  overscrollBehavior: 'contain',
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  padding: vars.space[3],
  color: vars.color.text,
  background: vars.color.surface,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  boxShadow: vars.elevation[3],
  fontFamily: vars.font.ui,
  fontSize: '0.875rem',
  selectors: {
    '&[hidden]': { display: 'none' },
    // The UA popover style would centre it in the top layer; this one places it.
    '&:popover-open': { inset: 'auto' },
  },
});

export const title = style({ margin: 0, fontSize: '0.875rem', fontWeight: vars.weight.bold });

export const area = style({
  position: 'relative',
  direction: 'ltr',
  inlineSize: '100%',
  blockSize: '9.5rem',
  touchAction: 'none',
  cursor: 'crosshair',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  background: vars.color.bgSink,
  selectors: { '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing } },
});

export const strip = style([area, { blockSize: '1.25rem', cursor: 'ew-resize' }]);

export const canvas = style({ display: 'block', inlineSize: '100%', blockSize: '100%', pointerEvents: 'none' });

export const thumb = style({
  position: 'absolute',
  inlineSize: '14px',
  blockSize: '14px',
  marginInlineStart: '-7px',
  marginBlockStart: '-7px',
  boxSizing: 'border-box',
  borderRadius: vars.radius.pill,
  border: `2px solid ${vars.color.surface}`,
  outline: `${vars.border.rule} solid ${vars.color.text}`,
  pointerEvents: 'none',
});

export const stripThumb = style([thumb, { insetBlockStart: 0, blockSize: 'calc(100% + 4px)', marginBlockStart: '-2px', borderRadius: vars.radius.sm }]);

export const row = style({ display: 'flex', alignItems: 'center', gap: vars.space[2], flexWrap: 'wrap' });

export const fields = style({ display: 'grid', gridTemplateColumns: 'minmax(5.75rem, 1.7fr) repeat(3, minmax(2.5rem, 1fr))', gap: vars.space[1], alignItems: 'end' });

export const field = style({ display: 'flex', flexDirection: 'column', gap: '2px', minInlineSize: 0, fontSize: '0.75rem', color: vars.color.textDim });

export const input = style({
  inlineSize: '100%',
  minInlineSize: 0,
  boxSizing: 'border-box',
  minBlockSize: vars.a11y.touchTarget,
  fontFamily: vars.font.mono,
  fontSize: '0.8125rem',
  // Number fields without spinners: the arrow keys still step them.
  MozAppearance: 'textfield',
});

globalStyle(`${input}::-webkit-inner-spin-button, ${input}::-webkit-outer-spin-button`, { WebkitAppearance: 'none', margin: 0 });

export const preview = style({
  inlineSize: '2.25rem',
  blockSize: '2.25rem',
  flex: 'none',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  background: vars.color.bgSink,
});

export const swatches = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[1], margin: 0, padding: 0, border: 0 });

export const groupLabel = style({ inlineSize: '100%', margin: 0, padding: 0, fontSize: '0.75rem', color: vars.color.textDim });

export const swatch = style({
  inlineSize: '28px',
  blockSize: '28px',
  padding: 0,
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  color: vars.color.text,
  background: vars.color.bgSink,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  cursor: 'pointer',
  selectors: {
    '&[aria-pressed="true"]': { boxShadow: `0 0 0 2px ${vars.color.surface}, 0 0 0 4px ${vars.color.text}` },
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  },
});

export const button = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  minBlockSize: vars.a11y.touchTarget,
  padding: `${vars.space[1]} ${vars.space[2]}`,
  color: vars.color.text,
  background: vars.color.bgAlt,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  font: 'inherit',
  cursor: 'pointer',
  selectors: { '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing } },
});

export const primary = style([button, { color: vars.color.accentText, background: vars.color.accent, borderColor: vars.color.accent }]);

export const actions = style({ display: 'flex', gap: vars.space[2], justifyContent: 'flex-end', flexWrap: 'wrap' });

export const grow = style({ flex: '1 1 auto' });

export const hint = style({ margin: 0, fontSize: '0.75rem', color: vars.color.textDim, display: 'flex', flexWrap: 'wrap', gap: vars.space[1], alignItems: 'center' });

export const badge = style({
  display: 'inline-block',
  padding: `0 ${vars.space[1]}`,
  borderRadius: vars.radius.sm,
  fontWeight: vars.weight.bold,
  border: `${vars.border.rule} solid currentColor`,
  selectors: {
    '&[data-level="aa"]': { color: vars.color.success },
    '&[data-level="large"]': { color: vars.color.warning },
    '&[data-level="fail"]': { color: vars.color.danger },
  },
});

export const error = style({ margin: 0, fontSize: '0.75rem', color: vars.color.danger });

export const srOnly = style({
  position: 'absolute',
  inlineSize: '1px',
  blockSize: '1px',
  overflow: 'hidden',
  clipPath: 'inset(50%)',
  whiteSpace: 'nowrap',
});
