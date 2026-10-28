// The active mailbox indicator and picker (t38 §2.3, t40): the address as text
// when there is one mailbox; otherwise a trigger (full or one-letter icon) that
// opens a menu of mailboxes. Rows are 44px tall on coarse pointers.
import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

const coarse = '(pointer: coarse)';

export const address = style({
  display: 'block',
  minWidth: 0,
  maxWidth: '100%',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
});

export const plain = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[2],
  minWidth: 0,
  maxWidth: '100%',
});

export const plainText = style([address, { flex: '1 1 auto' }]);

export const picker = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
  flex: '1 1 auto',
  minWidth: 0,
  maxWidth: '100%',
});

export const trigger = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[2],
  inlineSize: '100%',
  minWidth: 0,
  minBlockSize: '2rem',
  padding: '.2rem .4rem .2rem .25rem',
  font: 'inherit',
  fontSize: '.8125rem',
  textAlign: 'start',
  color: vars.color.text,
  background: vars.color.surface,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  cursor: 'pointer',
  selectors: {
    '&:hover:not(:disabled)': { background: vars.color.bgAlt },
    '&[aria-expanded="true"]': { borderColor: vars.color.accent },
    '&[data-linked="true"]': { boxShadow: `inset 4px 0 0 ${vars.color.accent}` },
    '&:disabled': { cursor: 'progress', opacity: 0.7 },
  },
  '@media': { [coarse]: { minBlockSize: '44px' } },
});

export const triggerText = style({ display: 'flex', flexDirection: 'column', flex: '1 1 auto', minWidth: 0 });
export const triggerAddress = style([address, { fontWeight: vars.weight.bold }]);
export const triggerSecondary = style([address, { fontSize: '.6875rem', color: vars.color.textDim }]);

export const chevron = style({
  flex: 'none',
  display: 'inline-flex',
  color: vars.color.textDim,
  selectors: { [`${trigger}[aria-expanded="true"] &`]: { transform: 'rotate(180deg)' } },
});

export const avatar = style({
  margin: 0,
  borderRadius: vars.radius.sm,
  selectors: {
    '&[data-size="sm"]': { flexBasis: '1.35rem', inlineSize: '1.35rem', blockSize: '1.35rem', fontSize: '.6rem' },
    '&[data-size="md"]': { flexBasis: '2rem', inlineSize: '2rem', blockSize: '2rem', fontSize: '.72rem' },
  },
});

export const unread = style({ flex: 'none' });

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
  '@media': { [coarse]: { inlineSize: '44px', blockSize: '44px' } },
});

/** The one-letter indicator with nothing to open: a label, not a button. */
export const iconStatic = style({
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  color: vars.color.text,
  background: vars.color.surface,
  cursor: 'default',
});

export const popover = style({
  position: 'fixed',
  zIndex: 1400,
  display: 'flex',
  flexDirection: 'column',
  gap: 0,
  minInlineSize: '16rem',
  maxInlineSize: 'min(24rem, calc(100vw - 16px))',
  maxBlockSize: 'min(70vh, 32rem)',
  overflowY: 'auto',
  padding: 0,
  background: vars.color.surface,
  color: vars.color.text,
  border: `${vars.border.heavy} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  boxShadow: vars.elevation[2],
});

export const row = style({
  position: 'relative',
  display: 'flex',
  alignItems: 'stretch',
  minWidth: 0,
  background: vars.color.surface,
  borderBlockEnd: `${vars.border.hair} solid ${vars.color.border}`,
  userSelect: 'none',
  WebkitTouchCallout: 'none',
  selectors: {
    '&:last-child': { borderBlockEnd: 'none' },
    '&[data-active="true"]': { boxShadow: `inset 4px 0 0 ${vars.color.accent}` },
    // The dragged row lifts and follows the pointer (transform set while dragging).
    '&[data-dragging="true"]': {
      zIndex: 2,
      opacity: 0.85,
      outline: `${vars.border.rule} solid ${vars.color.borderStrong}`,
      boxShadow: vars.elevation[3],
      cursor: 'grabbing',
    },
    // The drop line: where the dragged mailbox will land.
    '&[data-drop="before"]::before, &[data-drop="after"]::after': {
      content: '""',
      position: 'absolute',
      insetInline: 0,
      blockSize: '3px',
      background: vars.color.accent,
      zIndex: 3,
      pointerEvents: 'none',
    },
    '&[data-drop="before"]::before': { insetBlockStart: '-2px' },
    '&[data-drop="after"]::after': { insetBlockEnd: '-2px' },
  },
  '@media': { '(prefers-reduced-motion: no-preference)': { transition: 'box-shadow 120ms ease' } },
});

export const grip = style({
  flex: 'none',
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  inlineSize: '1.5rem',
  color: vars.color.textDim,
  cursor: 'grab',
  touchAction: 'none',
  selectors: { '&:hover': { color: vars.color.text } },
  '@media': { [coarse]: { inlineSize: '2rem' } },
});

export const rename = style({ display: 'flex', flexDirection: 'column', gap: '.2rem', flex: '1 1 auto', minWidth: 0, padding: '.35rem .5rem' });
export const renameInput = style({
  font: 'inherit',
  fontSize: '.8125rem',
  minBlockSize: '2rem',
  minWidth: 0,
  padding: '.25rem .4rem',
  color: vars.color.text,
  background: vars.color.bg,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  selectors: { '&:focus-visible': { outline: `2px solid ${vars.color.focus}`, outlineOffset: 1 } },
  '@media': { [coarse]: { minBlockSize: '44px' } },
});

export const item = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[2],
  flex: '1 1 auto',
  minWidth: 0,
  minBlockSize: '2.75rem',
  padding: '.35rem .5rem .35rem .65rem',
  font: 'inherit',
  fontSize: '.8125rem',
  textAlign: 'start',
  color: vars.color.text,
  background: 'transparent',
  border: 'none',
  cursor: 'pointer',
  selectors: {
    '&:hover, &:focus-visible': { background: vars.color.bgAlt },
    '&:focus-visible': { outline: `2px solid ${vars.color.focus}`, outlineOffset: -2 },
    '&[aria-disabled="true"]': { cursor: 'not-allowed', color: vars.color.textDim },
  },
  '@media': { [coarse]: { minBlockSize: '44px' } },
});

export const itemText = style({ display: 'flex', flexDirection: 'column', flex: '1 1 auto', minWidth: 0, gap: '.05rem' });
export const itemName = style([address, { fontWeight: vars.weight.bold }]);
export const itemAddress = style([address, { fontSize: '.75rem', color: vars.color.textDim }]);
export const itemMeta = style({
  display: 'flex',
  flexWrap: 'wrap',
  gap: vars.space[2],
  fontSize: '.6875rem',
  color: vars.color.textDim,
  selectors: { '&:empty': { display: 'none' } },
});

export const firstMark = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: '.2rem',
  fontWeight: vars.weight.bold,
  letterSpacing: vars.tracking.label,
  textTransform: 'uppercase',
  color: vars.color.text,
});

export const check = style({ flex: 'none', display: 'inline-flex', inlineSize: '1rem', color: vars.color.text });

export const rowAction = style({
  flex: 'none',
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  inlineSize: '2.25rem',
  padding: 0,
  color: vars.color.textDim,
  background: 'transparent',
  border: 'none',
  borderInlineStart: `${vars.border.hair} solid ${vars.color.border}`,
  cursor: 'pointer',
  selectors: {
    '&:hover:not(:disabled), &:focus-visible': { background: vars.color.bgAlt, color: vars.color.text },
    '&:focus-visible': { outline: `2px solid ${vars.color.focus}`, outlineOffset: -2 },
  },
  '@media': { [coarse]: { inlineSize: '44px' } },
});

export const error = style({ fontSize: '.8125rem', color: vars.color.danger, overflowWrap: 'anywhere' });

export const control = style({ display: 'flex', alignItems: 'center', gap: vars.space[1], minWidth: 0, maxWidth: '100%' });
export const refreshHelp = style({
  flex: 'none',
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  inlineSize: '1.75rem',
  minBlockSize: '1.75rem',
  color: vars.color.text,
  '@media': { [coarse]: { inlineSize: '44px', minBlockSize: '44px' } },
});
