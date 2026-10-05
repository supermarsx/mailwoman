// Styles for the reader toolbar's Reply / Reply all / Forward group
// (Reader.tsx). The three buttons stay together when the toolbar wraps, and a
// rule separates them from the actions on the message itself.

import { style } from '@vanilla-extract/css';

export const replyGroup = style({
  display: 'inline-flex',
  flexWrap: 'nowrap',
  gap: '0.25rem',
  marginInlineEnd: '0.5rem',
  paddingInlineEnd: '0.5rem',
  borderInlineEnd: '1px solid var(--border)',
});
