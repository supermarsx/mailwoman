// Remote-content (image-grant) bar styles (t16 §S8/S9, e14b), built on the frozen
// token contract (`theme/contract.css.ts`) so it themes with the rest of the
// reader chrome. Zero-runtime vanilla-extract.

import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const root = style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'center',
  gap: vars.space[2],
  padding: `${vars.space[2]} ${vars.space[3]}`,
  margin: `${vars.space[2]} 0`,
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  color: vars.color.text,
  fontFamily: vars.font.ui,
  fontSize: '0.85rem',
});

/** The "N remote images blocked" / "N trackers" summary text. */
export const summary = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  fontWeight: 600,
});

export const mark = style({
  flex: '0 0 auto',
  color: vars.color.warning,
});

/** The scrollable list of blocked hosts (untrusted — each host gets dir="auto"). */
export const hosts = style({
  display: 'inline-flex',
  flexWrap: 'wrap',
  gap: vars.space[1],
  maxWidth: '100%',
  color: vars.color.textDim,
});

export const host = style({
  padding: `0 ${vars.space[1]}`,
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.sm,
  background: vars.color.bg,
  maxWidth: '18rem',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
});

/** The grant-action button row, pushed to the trailing edge. */
export const actions = style({
  display: 'inline-flex',
  flexWrap: 'wrap',
  gap: vars.space[1],
  marginInlineStart: 'auto',
});

export const btn = style({
  appearance: 'none',
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.sm,
  cursor: 'pointer',
  padding: `${vars.space[1]} ${vars.space[2]}`,
  font: 'inherit',
  fontSize: '0.8rem',
  minHeight: vars.a11y.touchTarget,
  selectors: {
    '&:hover': { borderColor: vars.color.accent },
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
    '&:disabled': { opacity: 0.5, cursor: 'default' },
  },
});

/** Primary "load once" action, tinted with the accent. */
export const btnPrimary = style({
  borderColor: `color-mix(in srgb, ${vars.color.accent} 55%, ${vars.color.border})`,
});

/** Revoke ("turn off") action, tinted with the danger tone. */
export const btnDanger = style({
  borderColor: `color-mix(in srgb, ${vars.color.danger} 55%, ${vars.color.border})`,
});

/** The polite live region echoing the last action. */
export const status = style({
  flexBasis: '100%',
  color: vars.color.textDim,
  fontSize: '0.8rem',
  minHeight: '1.2em',
});
