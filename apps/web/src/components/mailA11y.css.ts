// Local WCAG 2.2 AA primitives for the mail-core area (t8-e1).
//
// These are SELF-CONTAINED, token-native helpers applied alongside the legacy
// app.css class strings on the mail components (message list, reader, composer,
// ribbon, dialogs). They are intentionally not imported from the e3-owned
// `src/components/a11y/**` primitives (which may not exist yet); a later
// consolidation pass folds these into the shared contract. Everything here is
// built on the frozen `vars.a11y.*` contract so focus/motion/high-contrast stay
// centrally switchable.

import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

/** Visible focus ring for keyboard users (WCAG 2.2 §2.4.11/2.4.13). Applied next
 *  to a component's own class: `class={`btn ${a11y.focusable}`}`. */
export const focusable = style({
  selectors: {
    '&:focus-visible': {
      outline: 'none',
      boxShadow: vars.a11y.focusRing,
      // Keep the ring visible above neighbouring rows/controls.
      position: 'relative',
      zIndex: 1,
    },
  },
});

/** Minimum 24×24 CSS-px interactive target (WCAG 2.2 §2.5.8). For icon-only
 *  controls that would otherwise render smaller than the target floor. */
export const touchTarget = style({
  minWidth: vars.a11y.touchTarget,
  minHeight: vars.a11y.touchTarget,
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
});

/** Icon button: a touch-target-sized control with a visible focus ring. */
export const iconButton = style([focusable, touchTarget]);

/** Visually hidden but exposed to assistive tech (the `.sr-only` pattern). Used
 *  for row status text ("Unread") and off-screen labels. */
export const srOnly = style({
  position: 'absolute',
  width: '1px',
  height: '1px',
  padding: 0,
  margin: '-1px',
  overflow: 'hidden',
  clip: 'rect(0, 0, 0, 0)',
  whiteSpace: 'nowrap',
  border: 0,
});
