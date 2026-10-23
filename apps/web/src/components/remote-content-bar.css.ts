// Remote-content (image-grant) bar styles (t16 §S8/S9, e14b), built on the frozen
// token contract (`theme/contract.css.ts`) so it themes with the rest of the
// reader chrome. Zero-runtime vanilla-extract.

import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

/** A flat notice: hair border, with a 3px warning bar on the leading edge whose
 *  extra width comes out of the leading padding. */
export const root = style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'center',
  gap: vars.space[2],
  paddingBlock: vars.space[2],
  paddingInline: `calc(${vars.space[3]} - 2px) ${vars.space[3]}`,
  marginBlock: vars.space[2],
  marginInline: 0,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  borderInlineStart: `${vars.border.heavy} solid ${vars.color.warning}`,
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
  fontWeight: vars.weight.bold,
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
  border: `${vars.border.hair} solid ${vars.color.border}`,
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

/** Square, 2px control rule (1px back from each padding), hard shadow on hover;
 *  focus is the global outline. */
export const btn = style({
  appearance: 'none',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.sm,
  cursor: 'pointer',
  padding: `calc(${vars.space[1]} - 1px) calc(${vars.space[2]} - 1px)`,
  font: 'inherit',
  fontSize: '0.8rem',
  minHeight: vars.a11y.touchTarget,
  selectors: {
    '&:hover:not(:disabled)': { boxShadow: vars.elevation[1] },
    '&:disabled': { opacity: 0.5, cursor: 'default', borderStyle: 'dashed' },
  },
});

/** Primary "load once" action: accent border. */
export const btnPrimary = style({
  borderColor: vars.color.accent,
});

/** Revoke ("turn off") action: danger border. */
export const btnDanger = style({
  borderColor: vars.color.danger,
});

/** The polite live region echoing the last action. */
export const status = style({
  flexBasis: '100%',
  color: vars.color.textDim,
  fontSize: '0.8rem',
  minHeight: '1.2em',
});
