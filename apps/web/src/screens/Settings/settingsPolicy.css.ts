// Styles for the settings-model parts of Settings (worker f01): the
// "Set by your administrator" note and the language picker. Tokens only,
// logical properties only, so every theme, forced colours and RTL hold.

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const lockNote = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  margin: 0,
  fontSize: '0.8rem',
  color: vars.color.textDim,
});

export const lockIcon = style({
  inlineSize: '0.9rem',
  blockSize: '0.9rem',
  flex: 'none',
});
