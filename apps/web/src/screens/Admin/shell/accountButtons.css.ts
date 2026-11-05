// The signed-in admin's user and Sign out squares (sidebar footer, collapsed
// header, phone top bar). The avatar reuses the webmail's `.person-avatar`
// palette; this file only sizes the boxes.
import { style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

const SIZE = '2.5rem';

export const row = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: vars.space[2], minInlineSize: 0 });

export const square = style({
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  flex: 'none',
  inlineSize: SIZE,
  blockSize: SIZE,
  padding: 0,
  font: 'inherit',
  color: vars.color.text,
  background: 'transparent',
  border: `${vars.border.rule} solid ${vars.color.border}`,
  textDecoration: 'none',
  cursor: 'pointer',
  selectors: {
    '&:hover': { borderColor: vars.color.accent, background: vars.color.surface },
    '&:focus-visible': { outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`, outlineOffset: '2px' },
  },
  '@media': {
    '(forced-colors: active)': {
      borderColor: 'ButtonText',
      selectors: { '&:hover': { borderColor: 'Highlight' }, '&:focus-visible': { outlineColor: 'Highlight' } },
    },
  },
});

export const avatar = style({ margin: 0, flex: '0 0 auto', inlineSize: '1.75rem', blockSize: '1.75rem', fontSize: '.75rem' });

export const picker = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1] });
