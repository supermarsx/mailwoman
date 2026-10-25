import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const card = style({
  margin: vars.space[3],
  padding: vars.space[3],
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  overflowWrap: 'anywhere',
});
