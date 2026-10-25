import { style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

export const listRow = style({
  flexWrap: 'wrap',
  gap: vars.space[4],
  minInlineSize: 0,
});
