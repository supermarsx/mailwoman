import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const criteria = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fit, minmax(min(100%, 18rem), 1fr))',
  gap: vars.space[4],
  padding: vars.space[4],
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  minInlineSize: 0,
});
