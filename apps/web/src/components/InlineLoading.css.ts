import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

// A compact label next to a spinner. Never a card, never a backdrop: it takes
// the height of one line and leaves everything around it interactive.
export const root = style({
  display: 'inline-flex', alignItems: 'center', gap: vars.space[2], minInlineSize: 0, maxInlineSize: '100%',
  paddingBlock: vars.space[2], color: vars.color.textDim, lineHeight: 1.5, overflowWrap: 'anywhere',
});
// Not yet past the delay: the text is still announced, nothing is drawn.
export const quiet = style({
  position: 'absolute', inlineSize: '1px', blockSize: '1px', overflow: 'hidden', clipPath: 'inset(50%)', whiteSpace: 'nowrap', padding: 0,
});
