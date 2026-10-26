import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const handle = style({
  position: 'relative',
  flexShrink: 0,
  alignSelf: 'stretch',
  justifySelf: 'stretch',
  touchAction: 'none',
  userSelect: 'none',
  background: vars.color.borderStrong,
  outlineOffset: -2,
  selectors: {
    '&[data-orientation="vertical"]': { width: 6, height: 'auto', cursor: 'col-resize' },
    '&[data-orientation="horizontal"]': { height: 6, width: 'auto', cursor: 'row-resize' },
    '&:hover, &:focus-visible': { background: vars.color.text, outline: `2px solid ${vars.color.text}` },
  },
});
