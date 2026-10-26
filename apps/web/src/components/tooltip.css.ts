import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const tooltip = style({
  position: 'fixed', zIndex: 10000, inset: 'auto', margin: 0,
  maxWidth: 'min(22rem, calc(100vw - 16px))', maxHeight: 'calc(100vh - 16px)',
  overflow: 'auto', overflowWrap: 'anywhere', whiteSpace: 'normal',
  padding: `${vars.space[2]} ${vars.space[3]}`,
  color: vars.color.text, background: vars.color.surface,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm, boxShadow: vars.elevation[2],
  fontFamily: vars.font.ui, fontSize: '0.8125rem', lineHeight: 1.45,
  pointerEvents: 'auto',
});
