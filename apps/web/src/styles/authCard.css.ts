// Sign-in card size (t32-h14), shared by the mail client's Login and the admin
// console's AdminLogin so both screens show the same card.
//
// The card fills the page's width up to AUTH_CARD_MAX_WIDTH and the page keeps a
// 16px gutter around it, so on a phone the card stops 16px short of each edge.
// Without a width of its own a card in a `place-items: center` grid shrinks to
// its longest line: the admin card was as wide as its one-line session note
// (769px at 1440) and changed width at the TOTP step.
//
// The theme contract has no size tokens (it is frozen), so the width is this one
// named value; e2e/t32-h14-login-size.spec.ts reads it from this file.

import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

/** The widest a sign-in card gets, on both sign-in screens. */
export const AUTH_CARD_MAX_WIDTH = '360px';

/** The page behind a sign-in card: the 16px gutter on every side. */
export const authPage = style({
  padding: vars.space[5],
});

/** A sign-in card: the page's width, at most AUTH_CARD_MAX_WIDTH. */
export const authCard = style({
  width: '100%',
  maxWidth: AUTH_CARD_MAX_WIDTH,
});
