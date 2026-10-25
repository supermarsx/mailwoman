// Admin Settings section styles (worker f01). Tokens only, logical properties
// only (UX1); the layout comes from the UX seam (`shell/ux.css.ts`).

import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

/** A setting's name over its key in the list. */
export const nameCell = style({ display: 'flex', flexDirection: 'column', gap: '2px', minInlineSize: '16ch', whiteSpace: 'normal' });
export const valueCell = style({ display: 'inline', whiteSpace: 'normal', overflowWrap: 'anywhere', verticalAlign: 'middle' });

globalStyle('[data-testid="admin-settings"] table', { tableLayout: 'auto', inlineSize: '100%' });
globalStyle('[data-testid="admin-settings"] table td:last-child, [data-testid="admin-settings"] table th:last-child', {
  position: 'sticky', insetInlineEnd: 0, background: vars.color.surface, zIndex: 1,
  borderInlineStart: `${vars.border.hair} solid ${vars.color.border}`,
});

/** The technical key, secondary to the plain name. */
export const keyName = style({
  margin: 0,
  fontFamily: vars.font.mono,
  fontSize: '0.75rem',
  color: vars.color.textDim,
  overflowWrap: 'anywhere',
});

/** The row of the two switches under the value. */
export const switches = style({
  display: 'grid',
  gap: vars.space[5],
  paddingBlock: vars.space[4],
  borderBlock: `${vars.border.hair} solid ${vars.color.border}`,
});
export const switchOption = style({ display: 'flex', flexDirection: 'column', gap: vars.space[3], minInlineSize: 0 });

/** An add or edit form for a narrower value. */
export const inlineForm = style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'flex-end',
  gap: vars.space[3],
  minWidth: 0,
});
