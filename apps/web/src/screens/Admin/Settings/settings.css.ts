// Admin Settings section styles (worker f01). Tokens only, logical properties
// only (UX1); the layout comes from the UX seam (`shell/ux.css.ts`).

import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

/** A setting's name over its key in the list. */
export const nameCell = style({ display: 'flex', flexDirection: 'column', gap: '2px', minInlineSize: '16ch', maxInlineSize: '24ch' });
export const valueCell = style({ display: 'inline-block', maxInlineSize: '24ch', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', verticalAlign: 'middle' });
globalStyle('[data-testid="admin-settings"] table td:last-child, [data-testid="admin-settings"] table th:last-child', {
  position: 'sticky', insetInlineEnd: 0, background: vars.color.surface, zIndex: 1,
  borderInlineStart: `${vars.border.hair} solid ${vars.color.border}`,
});

/** The technical key, secondary to the plain name. */
export const keyName = style({
  fontFamily: vars.font.mono,
  fontSize: '0.75rem',
  color: vars.color.textDim,
  overflowWrap: 'anywhere',
});

/** The row of the two switches under the value. */
export const switches = style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'center',
  gap: `${vars.space[2]} ${vars.space[4]}`,
});

/** An add or edit form for a narrower value. */
export const inlineForm = style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'flex-end',
  gap: vars.space[3],
  minWidth: 0,
});
