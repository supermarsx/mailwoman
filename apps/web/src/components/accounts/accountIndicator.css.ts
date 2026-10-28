// The active mailbox account indicator (t38 §2.3): the address as text, or in
// compact folder mode a one-letter button that opens the same choices.
import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const address = style({
  display: 'block',
  minWidth: 0,
  maxWidth: '100%',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
});

export const iconButton = style({
  flex: 'none',
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  inlineSize: '28px',
  blockSize: '28px',
  minWidth: '24px',
  minHeight: '24px',
  padding: 0,
  borderRadius: '50%',
  fontWeight: vars.weight.bold,
  textTransform: 'uppercase',
  selectors: {
    '&[data-linked="true"]': { boxShadow: `inset 0 0 0 2px ${vars.color.accent}` },
  },
  '@media': { '(pointer: coarse)': { inlineSize: '44px', blockSize: '44px' } },
});

export const popover = style({
  position: 'fixed',
  zIndex: 1400,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  minInlineSize: '14rem',
  maxInlineSize: 'min(24rem, calc(100vw - 16px))',
  padding: vars.space[2],
  background: vars.color.surface,
  color: vars.color.text,
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  boxShadow: vars.elevation[2],
});

export const popoverTitle = style({
  margin: 0,
  fontSize: '.8rem',
  color: vars.color.textDim,
  overflowWrap: 'anywhere',
});
