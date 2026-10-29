// Undo and Redo in the editor ribbon's header (t80), shared by the composer,
// the signature editor and the detached composer.
//
// Each is a split button: the action (icon, then its label at desktop widths)
// and a narrow chevron that opens the history list. Both halves are one ribbon
// row high (`--mw-ribbon-row`: 28px fine, 44px coarse), so the 16px icon sits
// with equal space above and below. The action has the same padding before the
// icon as after the label; with the label hidden (narrow widths) it is a square
// with the icon in the middle. The split draws one frame around both halves;
// the frame goes dashed when the action is disabled (the app's disabled
// convention: not colour alone). The glyphs point a direction, so they are
// mirrored under dir=rtl; the chevron is not.

import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

const row = 'var(--mw-ribbon-row, 28px)';
const narrow = 'screen and (max-width: 600px)';
const focusRing = { outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`, outlineOffset: '2px', zIndex: 1 } as const;

/** The two splits, then a rule between them and the tab names. */
export const history = style({});
// Over the editor's generic `.compose-rt__history` bar and the heading's `border:0`.
globalStyle(`.compose-rt__history${history}`, {
  display: 'flex',
  flexWrap: 'nowrap',
  alignItems: 'center',
  gap: vars.space[2],
  paddingBlock: vars.space[2],
  paddingInline: vars.space[3],
  background: 'transparent',
  border: 0,
});
globalStyle(`.compose-ribbon__heading > .compose-rt__history${history}`, {
  borderInlineEnd: `${vars.border.hair} solid ${vars.color.border}`,
  marginInlineEnd: vars.space[2],
});

export const split = style({
  display: 'inline-flex',
  alignItems: 'stretch',
  boxSizing: 'border-box',
  blockSize: row,
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  background: vars.color.surface,
  selectors: { '&:has(> button:disabled)': { borderStyle: 'dashed' } },
});

const part = {
  appearance: 'none',
  position: 'relative',
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  boxSizing: 'border-box',
  blockSize: '100%',
  margin: 0,
  padding: 0,
  border: 0,
  borderRadius: 0,
  background: 'transparent',
  color: vars.color.text,
  boxShadow: 'none',
  cursor: 'pointer',
  transitionProperty: 'background-color, color',
  transitionDuration: vars.a11y.motionDuration,
} as const;

export const action = style({
  ...part,
  gap: `calc(${vars.space[2]} + ${vars.space[1]})`,
  paddingInline: vars.space[3],
  font: 'inherit',
  fontSize: '0.8rem',
  // Bold, as every other ribbon button label (controls.css.ts button default).
  fontWeight: vars.weight.bold,
  lineHeight: 1,
  whiteSpace: 'nowrap',
  minInlineSize: row,
  maxInlineSize: '9rem',
  '@media': { [narrow]: { inlineSize: `calc(${row} - 2 * ${vars.border.hair})`, minInlineSize: 0, paddingInline: 0 } },
});

export const label = style({
  display: 'block',
  minInlineSize: 0,
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  // Jost at 0.8rem needs a 19px line box (Firefox measures the clipped span 1px
  // taller at 1.4; compose-extras.css .compose-rib__text). 1.6 fits the 28px row.
  lineHeight: '1.6',
  '@media': { [narrow]: { display: 'none' } },
});

export const icon = style({ flex: 'none', inlineSize: '16px', blockSize: '16px' });
globalStyle(`[dir="rtl"] ${icon}`, { transform: 'scaleX(-1)' });

/** The `<details>` holding the history list. */
export const more = style({
  position: 'relative',
  display: 'flex',
  borderInlineStart: `${vars.border.hair} solid ${vars.color.border}`,
});

export const summary = style({
  ...part,
  inlineSize: `calc(${vars.space[5]} + ${vars.space[2]})`,
  listStyle: 'none',
  fontWeight: vars.weight.regular,
  selectors: { '&::-webkit-details-marker': { display: 'none' } },
});

export const chevron = style({ flex: 'none', inlineSize: '12px', blockSize: '12px' });

// Hover, pressed, open and keyboard focus. The box never changes size.
globalStyle(`${action}:hover:enabled, ${summary}:hover`, { background: vars.color.bg, color: vars.color.text, boxShadow: 'none' });
globalStyle(`${action}:active:enabled, ${more}[open] > ${summary}`, { background: vars.color.bgSink, boxShadow: `inset 0 -2px 0 ${vars.color.accent}` });
globalStyle(`${action}:focus-visible, ${summary}:focus-visible`, focusRing);
// The app-wide disabled rule dashes a border and fades the control; here the
// frame is the split's, so only the fade applies to the action itself.
globalStyle(`:root body ${split} > ${action}:disabled`, { opacity: 0.5, border: 0, cursor: 'not-allowed', background: 'transparent' });
globalStyle(`${split}:has(> ${action}:disabled)`, { borderColor: vars.color.border });

globalStyle(`${action} > *, ${summary} > *`, { pointerEvents: 'none' });
