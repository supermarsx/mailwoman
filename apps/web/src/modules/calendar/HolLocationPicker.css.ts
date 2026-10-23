// Styles of the holiday-pack location picker (`HolLocationPicker.tsx`). The
// dialog frame, buttons and inputs are the calendar module's (`calendar.css.ts`).

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const heading = style({ margin: 0, fontSize: '1.1rem', fontWeight: vars.weight.heavy });

// Outlook's own pack has 116 locations: the list scrolls inside the dialog. Its
// frame keeps the 1px width (the list has no padding to give back) and takes
// the control border colour.
export const list = style({
  listStyle: 'none',
  margin: 0,
  padding: 0,
  maxHeight: '45vh',
  overflowY: 'auto',
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
});

export const item = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[2],
  // A 1px divider under each row; the bottom padding gives that pixel back.
  padding: `${vars.space[1]} ${vars.space[2]} calc(${vars.space[1]} - 1px)`,
  borderBlockEnd: `${vars.border.hair} solid ${vars.color.border}`,
  selectors: {
    '&:last-child': { borderBlockEnd: 'none' },
  },
});
