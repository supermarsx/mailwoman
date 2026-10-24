// Styles of the theme and accent picker in Settings → Appearance (t32 V).
//
// The gallery cards keep the size they had as inline styles (9.5rem wide,
// 0.5rem padding); the classes double their own selector (`&&`) so they win over
// the shared `option` button class whatever order the two stylesheets load in.
// Only colours that depend on the pack or the accent stay inline, set through
// the CSSOM by Solid, which the CSP allows.

import { style } from '@vanilla-extract/css';
import { vars } from './contract.css.ts';

export const card = style({
  selectors: {
    '&&': {
      display: 'flex',
      flexDirection: 'column',
      gap: '6px',
      alignItems: 'stretch',
      textAlign: 'start',
      width: '9.5rem',
      padding: '0.5rem',
    },
  },
});

export const cardName = style({ fontWeight: 600 });

export const cardDescription = style({ fontSize: '0.72rem', opacity: 0.85 });

export const cardMeta = style({ fontSize: '0.7rem', opacity: 0.7 });

/** The miniature of a pack; its colours are set inline from the pack palette. */
export const preview = style({
  display: 'block',
  height: '3rem',
  padding: '0.3rem',
  borderRadius: vars.radius.sm,
  borderWidth: vars.border.hair,
  borderStyle: 'solid',
});

export const previewPanel = style({
  display: 'block',
  height: '0.85rem',
  marginBottom: '0.3rem',
});

export const previewText = style({
  display: 'block',
  height: '0.35rem',
  width: '70%',
  marginBottom: '0.25rem',
});

export const previewAccent = style({
  display: 'block',
  height: '0.35rem',
  width: '35%',
});

/**
 * An accent swatch: a 28px circle filled with the active theme's shade of the
 * accent. The chosen one carries a check mark in the accent's ink and a ring,
 * so the choice is not shown by colour alone.
 */
export const swatch = style({
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  width: '28px',
  height: '28px',
  padding: 0,
  borderRadius: vars.radius.pill,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  cursor: 'pointer',
  // At or under the 13.33px a bare button had, so the swatch text is no bigger.
  font: 'inherit',
  fontSize: '0.8rem',
  fontWeight: vars.weight.bold,
  lineHeight: 1,
  selectors: {
    '&[aria-pressed="true"]': {
      boxShadow: `0 0 0 2px ${vars.color.surface}, 0 0 0 4px ${vars.color.text}`,
    },
  },
});
