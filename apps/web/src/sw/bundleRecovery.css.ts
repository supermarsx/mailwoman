// The "new version, reloading" notice for when no toast region is on screen
// (t102): a chunk failure can take down the surface that mounts the region (the
// admin console's own boundary), so the notice cannot rely on it. Tokens only,
// logical properties only, like the toasts it stands in for.
import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const notice = style({
  position: 'fixed',
  insetBlockEnd: vars.space[5],
  insetInlineEnd: vars.space[5],
  zIndex: 1100,
  maxWidth: `min(28rem, calc(100vw - 2 * ${vars.space[5]}))`,
  paddingBlock: vars.space[2],
  paddingInline: vars.space[3],
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderInlineStartWidth: vars.border.heavy,
  borderInlineStartColor: vars.color.accent,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  color: vars.color.text,
  boxShadow: vars.elevation[2],
  fontSize: vars.fontSize.base,
  lineHeight: vars.lineHeight.ui,
  '@media': {
    '(max-width: 760px)': {
      insetBlockStart: 'calc(3.75rem + env(safe-area-inset-top))',
      insetBlockEnd: 'auto',
      insetInlineStart: vars.space[3],
      insetInlineEnd: vars.space[3],
      maxWidth: 'none',
    },
  },
});
