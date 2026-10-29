// t86: the mailbox avatar editor in Settings › Linked mailboxes. The avatar in
// each row is a button that opens a compact anchored popover (same frame as the
// colour picker's): a live preview, Letter/Icon, the icon grid or the letter
// field, the two colours and the contrast note, then Reset / Cancel / Save.
import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

const coarse = '(pointer: coarse)';

/** The avatar in a table row, as a button: the box does not change size or place. */
export const avatarButton = style({
  gridArea: 'avatar',
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  padding: '2px',
  margin: '-2px',
  border: 0,
  borderRadius: vars.radius.md,
  background: 'transparent',
  cursor: 'pointer',
  selectors: {
    '&:hover:not(:disabled)': { boxShadow: `0 0 0 2px ${vars.color.borderStrong}` },
    '&[aria-expanded="true"]': { boxShadow: `0 0 0 2px ${vars.color.accent}` },
    '&:focus-visible': { outline: `2px solid ${vars.color.focus}`, outlineOffset: '1px' },
    '&:disabled': { cursor: 'default' },
  },
});

export const wrap = style({ gridArea: 'avatar', position: 'relative', display: 'inline-flex' });

export const popover = style({
  position: 'fixed',
  zIndex: 10000,
  inset: 'auto',
  margin: 0,
  boxSizing: 'border-box',
  inlineSize: 'min(20.5rem, calc(100vw - 16px))',
  overflowY: 'auto',
  overscrollBehavior: 'contain',
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: vars.space[3],
  color: vars.color.text,
  background: vars.color.surface,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  boxShadow: vars.elevation[3],
  fontFamily: vars.font.ui,
  fontSize: '0.875rem',
  selectors: {
    '&[hidden]': { display: 'none' },
    '&:popover-open': { inset: 'auto' },
  },
});

export const head = style({ display: 'flex', alignItems: 'center', gap: vars.space[3], minInlineSize: 0 });
export const headText = style({ display: 'flex', flexDirection: 'column', gap: '2px', minInlineSize: 0 });
export const title = style({ margin: 0, fontSize: '0.875rem', fontWeight: vars.weight.bold });
export const subtitle = style({ margin: 0, fontSize: '0.8rem', color: vars.color.textDim, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' });

/** 6 columns of 36px cells (44px on touch): 26 icons in five rows. */
export const grid = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fill, minmax(36px, 1fr))',
  gap: vars.space[1],
  margin: 0,
  padding: 0,
  border: 0,
  '@media': { [coarse]: { gridTemplateColumns: 'repeat(auto-fill, minmax(44px, 1fr))' } },
});

export const cell = style({
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  blockSize: '36px',
  padding: 0,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  borderRadius: vars.radius.sm,
  background: vars.color.surface,
  color: vars.color.text,
  cursor: 'pointer',
  selectors: {
    '&:hover': { background: vars.color.bgAlt },
    '&[aria-checked="true"]': { borderColor: vars.color.accent, background: vars.color.accent, color: vars.color.accentText },
    '&:focus-visible': { outline: `2px solid ${vars.color.focus}`, outlineOffset: '1px' },
  },
  '@media': { [coarse]: { blockSize: '44px' } },
});

export const field = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1] });
export const label = style({ fontSize: '0.8rem', fontWeight: vars.weight.bold });
export const letter = style({ inlineSize: '5rem' });
export const colours = style({ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: vars.space[2], '@media': { 'screen and (max-width: 360px)': { gridTemplateColumns: '1fr' } } });
export const note = style({ margin: 0, fontSize: '0.8rem', color: vars.color.textDim });
export const warn = style({ margin: 0, fontSize: '0.8rem', color: vars.color.danger, display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: vars.space[2] });
export const footer = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'center', justifyContent: 'flex-end', gap: vars.space[2] });
export const spacer = style({ flex: '1 1 auto' });
