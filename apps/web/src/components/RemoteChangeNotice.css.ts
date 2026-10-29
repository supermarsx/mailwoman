import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

// One line above the open message: what changed elsewhere, and a way to close it.
export const notice = style({
  display: 'flex',
  alignItems: 'center',
  flexWrap: 'wrap',
  gap: vars.space[2],
  margin: `0 0 ${vars.space[2]}`,
  padding: `${vars.space[2]} ${vars.space[3]}`,
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  color: vars.color.text,
  fontSize: '0.88rem',
});
