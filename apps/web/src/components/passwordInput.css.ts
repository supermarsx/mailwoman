// Password field with a show/hide button (t32-h17, components/PasswordInput.tsx).
//
// The input and the button share one grid cell: the input fills it and keeps the
// size the caller's field class gives it, and the button sits on top at the inline
// end (the right in LTR, the left in RTL), centred on the input's height. Neither
// is positioned, so the field is exactly as tall and wide as the input alone ("no
// box may grow"). The input's end padding leaves room for the button so typed text
// never runs under it.

import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const wrap = style({
  display: 'grid',
  alignItems: 'center',
  minInlineSize: 0,
});

// The wrapper is a <span> (a <label> holds only phrasing content) and sits where the
// field-name span usually is, so the field-name rules match it too: `.field > span`
// (app.css, 0,1,1) and `[data-screen="admin"] .field > span` (admin.css.ts, 0,3,1)
// make text small, bold, tracked capitals in the dim colour, and the input inherits
// its font from here. The quadrupled class (0,4,0) hands the label's own text style
// back, so the input looks as it did before the wrapper.
globalStyle(`${wrap}${wrap}${wrap}${wrap}`, {
  fontSize: 'inherit',
  fontWeight: 'inherit',
  letterSpacing: 'inherit',
  textTransform: 'inherit',
  color: 'inherit',
});

export const input = style({
  gridArea: '1 / 1',
  inlineSize: '100%',
  minInlineSize: 0,
});

// The callers' field rules set a symmetric padding: `.field input:not(…)` in app.css
// (specificity 0,2,1) or one module class. The doubled class (0,3,0) outranks both,
// so only the inline end changes. Button inset + button + the same gap again.
globalStyle(`${wrap} > ${input}${input}`, {
  paddingInlineEnd: `calc(${vars.a11y.touchTarget} + ${vars.space[3]} + ${vars.space[3]})`,
});

export const toggle = style({
  gridArea: '1 / 1',
  justifySelf: 'end',
  marginInlineEnd: vars.space[3],
  display: 'inline-grid',
  placeItems: 'center',
  inlineSize: vars.a11y.touchTarget,
  blockSize: vars.a11y.touchTarget,
  padding: 0,
  // Transparent here; forced colours paint every border, so the button keeps a
  // visible outline there as well as its glyph (ButtonText via currentColor).
  border: `${vars.border.hair} solid transparent`,
  borderRadius: vars.radius.sm,
  background: 'transparent',
  color: vars.color.textDim,
  cursor: 'pointer',
  transitionProperty: 'color',
  transitionDuration: vars.a11y.motionDuration,
  selectors: {
    '&:hover, &[aria-pressed="true"]': { color: vars.color.text },
    '&:disabled': { cursor: 'default' },
  },
});
