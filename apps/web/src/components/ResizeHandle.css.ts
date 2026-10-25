import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const handle = style({
  position: 'relative',
  flexShrink: 0,
  touchAction: 'none',
  userSelect: 'none',
  background: vars.color.borderStrong,
  outlineOffset: -2,
  selectors: {
    '&[data-orientation="vertical"]': { width: 6, height: '100%', cursor: 'col-resize' },
    '&[data-orientation="horizontal"]': { height: 6, width: '100%', cursor: 'row-resize' },
    '&:hover, &:focus-visible': { background: vars.color.text, outline: `2px solid ${vars.color.text}` },
  },
});
