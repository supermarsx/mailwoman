// Mailbox ACL editor styles (t13 e8). Token-native — no design-token changes.

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const wrap = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  fontFamily: vars.font.ui,
  color: vars.color.text,
});

export const heading = style({ fontSize: '1rem', fontWeight: 600, margin: 0 });
export const subHeading = style({
  fontSize: '0.74rem',
  fontWeight: 600,
  color: vars.color.textDim,
  textTransform: 'uppercase',
  letterSpacing: '0.04em',
  margin: 0,
});
export const meta = style({ fontSize: '0.8rem', color: vars.color.textDim, margin: 0 });
export const error = style({ fontSize: '0.85rem', color: vars.color.danger, margin: 0 });
export const mono = style({ fontFamily: vars.font.mono, fontSize: '0.85rem', wordBreak: 'break-all' });

/** The read-only banner shown when the current user lacks the `a` right. */
export const notice = style({
  fontSize: '0.82rem',
  color: vars.color.textDim,
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderRadius: vars.radius.md,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bgAlt,
  margin: 0,
});

/** One ACL entry (an identifier + its rights checkbox grid) as a fieldset card. */
export const entryCard = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: vars.space[4],
  borderRadius: vars.radius.lg,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.surface,
  margin: 0,
});

export const entryHeader = style({
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'center',
  gap: vars.space[3],
  flexWrap: 'wrap',
});

export const legend = style({
  fontWeight: 600,
  fontSize: '0.9rem',
  padding: 0,
});

/** The grid of the eleven RFC 4314 rights checkboxes. */
export const rightsGrid = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fill, minmax(14rem, 1fr))',
  gap: vars.space[2],
  border: 0,
  padding: 0,
  margin: 0,
});

export const rightRow = style({
  display: 'flex',
  alignItems: 'flex-start',
  gap: vars.space[2],
  padding: `${vars.space[1]} ${vars.space[2]}`,
  borderRadius: vars.radius.md,
  selectors: { '&:hover': { background: vars.color.bgAlt } },
});

export const checkbox = style({
  width: '1.1rem',
  height: '1.1rem',
  marginTop: '0.15rem',
  flexShrink: 0,
  accentColor: vars.color.accent,
  cursor: 'pointer',
  selectors: {
    '&:disabled': { cursor: 'not-allowed', opacity: 0.55 },
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  },
});

export const rightLabel = style({
  display: 'flex',
  flexDirection: 'column',
  gap: '1px',
  cursor: 'pointer',
});
export const rightName = style({ fontSize: '0.85rem', fontWeight: 600 });
export const rightDesc = style({ fontSize: '0.74rem', color: vars.color.textDim });

/** MYRIGHTS summary chips for the current user. */
export const myRightsRow = style({
  display: 'flex',
  flexWrap: 'wrap',
  gap: vars.space[2],
  alignItems: 'center',
});
export const chip = style({
  fontSize: '0.72rem',
  fontWeight: 600,
  padding: `2px ${vars.space[2]}`,
  borderRadius: vars.radius.pill,
  background: vars.color.bgSink,
  color: vars.color.text,
});

export const input = style({
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderRadius: vars.radius.md,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  font: 'inherit',
  fontSize: '0.9rem',
  minHeight: vars.a11y.touchTarget,
});

export const button = style({
  appearance: 'none',
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[1]} ${vars.space[3]}`,
  minHeight: vars.a11y.touchTarget,
  font: 'inherit',
  fontSize: '0.82rem',
  fontWeight: 600,
  transition: `background ${vars.a11y.motionDuration}`,
  selectors: {
    '&:disabled': { opacity: 0.5, cursor: 'not-allowed' },
    '&:hover:not(:disabled)': { background: vars.color.bgAlt },
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  },
});

export const dangerButton = style([
  button,
  { color: vars.color.danger, borderColor: vars.color.danger },
]);

/** The add-grant form row. */
export const addForm = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: vars.space[4],
  borderRadius: vars.radius.lg,
  border: `1px dashed ${vars.color.border}`,
  background: vars.color.bg,
});

export const formRow = style({
  display: 'flex',
  gap: vars.space[2],
  alignItems: 'center',
  flexWrap: 'wrap',
});
