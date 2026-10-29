// Settings › Linked mailboxes as a management table (t48 k). One row per
// mailbox: a slim drag grip tight against the avatar, the nickname with the
// address under it, the primary star, an expired-status icon, and the icon
// actions at the end. Rows share one height and are ruled off by hairlines. On
// a phone each row becomes a two-line card (who on the first, actions on the
// second) with 44px targets.
import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

const phone = 'screen and (max-width: 600px)';
const coarse = '(pointer: coarse)';

export const header = style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'flex-start',
  justifyContent: 'space-between',
  gap: vars.space[3],
});
export const headerText = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2], minInlineSize: 0, flex: '1 1 20rem' });
export const addButton = style({ flex: '0 0 auto', alignSelf: 'flex-start' });

export const table = style({
  listStyle: 'none',
  margin: 0,
  padding: 0,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  minInlineSize: 0,
});

export const row = style({
  display: 'grid',
  gridTemplateColumns: 'auto auto minmax(0, 1fr) auto',
  gridTemplateAreas: '"grip avatar who actions"',
  alignItems: 'center',
  columnGap: vars.space[3],
  minBlockSize: '3.5rem',
  paddingBlock: vars.space[2],
  paddingInline: `${vars.space[1]} ${vars.space[3]}`,
  background: vars.color.surface,
  selectors: {
    '& + &': { borderBlockStart: `${vars.border.hair} solid ${vars.color.border}` },
    '&:hover': { background: vars.color.bgAlt },
    '&:focus-within': { background: vars.color.bgAlt },
    '&[data-dragging="true"]': { opacity: 0.6 },
    '&[data-drop="before"]': { boxShadow: `inset 0 2px 0 ${vars.color.accent}` },
    '&[data-drop="after"]': { boxShadow: `inset 0 -2px 0 ${vars.color.accent}` },
  },
  '@media': {
    [phone]: {
      gridTemplateColumns: 'auto auto minmax(0, 1fr)',
      gridTemplateAreas: '"grip avatar who" ". actions actions"',
      rowGap: vars.space[2],
    },
  },
});

/** A slim dotted grip: about 10px drawn; the padding widens what can be grabbed. */
export const grip = style({
  gridArea: 'grip',
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  inlineSize: '10px',
  blockSize: '1.5rem',
  paddingInline: '4px',
  boxSizing: 'content-box',
  cursor: 'grab',
  touchAction: 'none',
  background: 'transparent',
  border: 0,
  color: vars.color.textDim,
  selectors: {
    '&::before': {
      content: '""',
      inlineSize: '6px',
      blockSize: '14px',
      backgroundImage: `radial-gradient(circle, currentColor 1.2px, transparent 1.4px)`,
      backgroundSize: '3px 4px',
      backgroundRepeat: 'repeat',
    },
    '&:hover, &:focus-visible': { color: vars.color.text },
    '&:focus-visible': { outline: `2px solid ${vars.color.focus}`, outlineOffset: 0 },
  },
});

export const avatar = style({ gridArea: 'avatar' });

export const who = style({
  gridArea: 'who',
  display: 'flex',
  flexDirection: 'column',
  gap: '2px',
  minInlineSize: 0,
});
const ellipsis = { overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', minInlineSize: 0 } as const;
export const name = style({ ...ellipsis, fontSize: '0.9rem', fontWeight: vars.weight.bold });
export const address = style({ ...ellipsis, fontSize: '0.8rem', color: vars.color.textDim });

export const actions = style({
  gridArea: 'actions',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'flex-end',
  gap: vars.space[1],
  '@media': { [phone]: { justifyContent: 'flex-start' } },
});
/** The admin row-icon size (ux.iconAction), raised to 44px on phones and touch. */
globalStyle(`${actions} > :is(button, span[role="img"])`, {
  '@media': { [`${phone}, ${coarse}`]: { inlineSize: '44px', blockSize: '44px', minInlineSize: '44px' } },
});

export const status = style({
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  inlineSize: vars.a11y.touchTarget,
  blockSize: vars.a11y.touchTarget,
  color: vars.color.warning,
});

export const rename = style({ display: 'flex', flexDirection: 'column', gap: '2px', minInlineSize: 0 });

export const empty = style({
  margin: 0,
  padding: vars.space[4],
  border: `${vars.border.hair} dashed ${vars.color.border}`,
  color: vars.color.textDim,
  fontSize: '0.9rem',
});
