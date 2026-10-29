// The Events view's filter row above the list (EventsView.tsx, t78): the same
// slim header row as the Focused / Other tabs, with one toggle chip.

import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const row = style({
  display: 'flex',
  alignItems: 'center',
  gap: '0.4rem',
  padding: '0.2rem 0.6rem',
  borderBlockEnd: `${vars.border.hair} solid ${vars.color.border}`,
  background: vars.color.bgAlt,
  fontSize: '0.78rem',
});

/** A toggle chip; 24px minimum target. Pressed reads as the accent fill. */
export const chip = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: '0.3rem',
  minBlockSize: '24px',
  padding: '0.1rem 0.6rem',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  background: 'transparent',
  color: 'inherit',
  font: 'inherit',
  lineHeight: 1.2,
  cursor: 'pointer',
  selectors: {
    '&[aria-pressed="true"]': {
      background: vars.color.accent,
      color: vars.color.accentText,
      borderColor: vars.color.accent,
    },
  },
});
