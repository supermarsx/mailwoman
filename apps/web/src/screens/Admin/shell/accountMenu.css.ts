// The signed-in admin's avatar and its menu (sidebar footer, collapsed-sidebar
// header, phone top bar). The avatar reuses the webmail's `.person-avatar`
// palette; this file only sizes the box and lays out the popover.
import { style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

const ring = { outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`, outlineOffset: '2px' } as const;

export const root = style({ position: 'relative', minInlineSize: 0, display: 'flex' });

export const trigger = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[2],
  inlineSize: '100%',
  minInlineSize: 0,
  minBlockSize: '2.25rem',
  padding: vars.space[1],
  font: 'inherit',
  textAlign: 'start',
  color: vars.color.text,
  background: 'transparent',
  border: `${vars.border.rule} solid transparent`,
  cursor: 'pointer',
  selectors: {
    '&:hover': { borderColor: vars.color.borderStrong, background: vars.color.surface },
    '&[aria-expanded="true"]': { borderColor: vars.color.borderStrong, background: vars.color.surface },
    '&:focus-visible': ring,
  },
});

export const triggerIcon = style({ inlineSize: 'auto', flex: 'none' });

export const avatar = style({
  margin: 0,
  flex: '0 0 auto',
  inlineSize: '2rem',
  blockSize: '2rem',
  fontSize: '.75rem',
});

export const name = style({
  flex: '1 1 auto',
  minInlineSize: 0,
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
  fontWeight: vars.weight.bold,
  fontSize: '.875rem',
});

export const popover = style({
  position: 'fixed',
  zIndex: 40,
  inlineSize: 'min(18rem, calc(100vw - 1rem))',
  maxBlockSize: 'calc(100vh - 1rem)',
  overflowY: 'auto',
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  padding: vars.space[3],
  color: vars.color.text,
  background: vars.color.surface,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
});

export const identity = style({ display: 'flex', alignItems: 'center', gap: vars.space[3], minInlineSize: 0 });

export const identityText = style({
  minInlineSize: 0,
  fontSize: '.85rem',
  lineHeight: 1.4,
  overflowWrap: 'anywhere',
});

export const picker = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1] });
