import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const panel = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  margin: `0 0 ${vars.space[2]}`,
  padding: `${vars.space[2]} ${vars.space[3]}`,
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  color: vars.color.text,
  fontSize: '0.88rem',
});
export const row = style({ display: 'flex', alignItems: 'center', flexWrap: 'wrap', gap: vars.space[2], justifyContent: 'space-between' });
export const host = style({ overflowWrap: 'anywhere', minWidth: 0, flex: '1 1 12rem' });
export const segmented = style({ display: 'inline-flex', border: `${vars.border.hair} solid ${vars.color.borderStrong}`, borderRadius: vars.radius.md, overflow: 'hidden' });
export const option = style({
  appearance: 'none',
  border: 0,
  margin: 0,
  padding: `${vars.space[1]} ${vars.space[3]}`,
  font: 'inherit',
  background: 'transparent',
  color: 'inherit',
  cursor: 'pointer',
  selectors: {
    '&[aria-checked="true"]': { background: vars.color.text, color: vars.color.surface },
    '&:disabled': { cursor: 'default', opacity: 0.6 },
    '&:focus-visible': { outline: `2px solid ${vars.color.text}`, outlineOffset: '-2px' },
  },
});
export const hint = style({ margin: 0, opacity: 0.8 });
