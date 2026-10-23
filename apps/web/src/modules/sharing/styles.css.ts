// Mailbox ACL editor styles (t13 e8). Token-native — no design-token changes.
//
// t32 restyle (plan §2.4): square corners, 2px control and panel borders in
// `borderStrong` with 1px less padding so no box grows (P1), focus is the
// outline ring, headings heavy, labels and chips small tracked capitals.

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

/** The outline ring (forced colours keep it; they drop a box-shadow ring). */
const focusOutline = {
  outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`,
  outlineOffset: '2px',
} as const;

/** `base` less the 1px a 2px rule adds over the old 1px border (P1). */
const less1 = (base: string): string => `calc(${base} - 1px)`;

export const wrap = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  fontFamily: vars.font.ui,
  color: vars.color.text,
});

export const heading = style({ fontSize: '1rem', fontWeight: vars.weight.heavy, margin: 0 });
export const subHeading = style({
  fontSize: '0.74rem',
  fontWeight: vars.weight.bold,
  color: vars.color.textDim,
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
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
  border: `${vars.border.hair} solid ${vars.color.border}`,
  background: vars.color.bgAlt,
  margin: 0,
});

/** One ACL entry (an identifier + its rights checkbox grid) as a fieldset card. */
export const entryCard = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: less1(vars.space[4]),
  borderRadius: vars.radius.lg,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
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
  fontWeight: vars.weight.heavy,
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
  marginBlockStart: '0.15rem',
  flexShrink: 0,
  accentColor: vars.color.accent,
  cursor: 'pointer',
  selectors: {
    '&:disabled': { cursor: 'not-allowed', opacity: 0.55 },
    '&:focus-visible': focusOutline,
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
  fontWeight: vars.weight.bold,
  padding: `2px ${vars.space[2]}`,
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  color: vars.color.text,
});

export const input = style({
  padding: `${less1(vars.space[2])} ${less1(vars.space[3])}`,
  borderRadius: vars.radius.md,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: vars.color.bgSink,
  color: vars.color.text,
  font: 'inherit',
  fontSize: '0.9rem',
  minHeight: vars.a11y.touchTarget,
  selectors: {
    // A light page puts fields on the brighter surface instead of the sink.
    ':root[data-appearance="light"] &': { background: vars.color.surface },
    '&:focus-visible': focusOutline,
  },
});

// Ghost button: inverts on hover instead of casting a shadow.
export const button = style({
  appearance: 'none',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: 'transparent',
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${less1(vars.space[1])} ${less1(vars.space[3])}`,
  minHeight: vars.a11y.touchTarget,
  font: 'inherit',
  fontSize: '0.82rem',
  fontWeight: vars.weight.medium,
  transition: `background ${vars.a11y.motionDuration}, color ${vars.a11y.motionDuration}`,
  selectors: {
    // Not colour alone: a disabled button also has a dashed border.
    '&:disabled': { opacity: 0.5, cursor: 'not-allowed', borderStyle: 'dashed' },
    '&:hover:not(:disabled)': { background: vars.color.text, color: vars.color.bg },
    '&:focus-visible': focusOutline,
  },
});

export const dangerButton = style([
  button,
  {
    color: vars.color.accentText,
    background: vars.color.danger,
    borderColor: vars.color.danger,
    selectors: {
      '&:hover:not(:disabled)': {
        background: vars.color.danger,
        color: vars.color.accentText,
        boxShadow: vars.elevation[1],
      },
      '&:active:not(:disabled)': { boxShadow: 'none' },
    },
  },
]);

/** The add-grant form row. */
export const addForm = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: vars.space[4],
  borderRadius: vars.radius.lg,
  border: `${vars.border.hair} dashed ${vars.color.borderStrong}`,
  background: vars.color.bg,
});

export const formRow = style({
  display: 'flex',
  gap: vars.space[2],
  alignItems: 'center',
  flexWrap: 'wrap',
});
