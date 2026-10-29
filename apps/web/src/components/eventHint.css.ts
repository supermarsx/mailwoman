// The Events view's row hint (EventHint.tsx, t78). It lives in the preview line
// and never grows it: one line, capped at the preview line height.

import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const hint = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: '0.25em',
  marginInlineEnd: '0.5em',
  verticalAlign: 'top',
  maxBlockSize: 'var(--mail-preview-line, 16px)',
  color: vars.color.text,
  fontWeight: 600,
  selectors: {
    '&[data-event-state="cancelled"]': { color: vars.color.danger },
  },
});

export const icon = style({ display: 'inline-flex', inlineSize: '1em', blockSize: '1em', flex: '0 0 auto' });
