// The notice shown while processing of the signed-in person's data is restricted.
import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const banner = style({
  position: 'fixed',
  insetBlockEnd: 0,
  insetInline: 0,
  zIndex: 50,
  display: 'flex',
  flexWrap: 'wrap',
  gap: vars.space[3],
  alignItems: 'center',
  padding: `${vars.space[2]} ${vars.space[4]}`,
  background: vars.color.bgAlt,
  color: vars.color.text,
  borderBlockStart: `${vars.border.heavy} solid ${vars.color.warning}`,
  fontSize: '0.85rem',
});
