// Directory/GAL UI styles (plan §3 e7). Token-native — design tokens unchanged.
//
// t32 restyle (plan §2.4): square corners, 2px control and panel borders in
// `borderStrong` with 1px less padding so no box grows (P1), focus is the
// outline ring, a selected option carries an inset start bar (not colour
// alone), headings heavy, labels and badges small tracked capitals.

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
  position: 'relative',
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  fontFamily: vars.font.ui,
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
  selectors: {
    // A light page puts fields on the brighter surface instead of the sink.
    ':root[data-appearance="light"] &': { background: vars.color.surface },
    '&:focus-visible': focusOutline,
  },
});

export const listbox = style({
  listStyle: 'none',
  margin: 0,
  padding: less1(vars.space[1]),
  display: 'flex',
  flexDirection: 'column',
  gap: '2px',
  borderRadius: vars.radius.md,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: vars.color.surface,
  maxHeight: '18rem',
  overflowY: 'auto',
});

export const option = style({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  gap: vars.space[3],
  padding: `${vars.space[2]} ${vars.space[3]}`,
  minHeight: vars.a11y.touchTarget,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  transition: `background ${vars.a11y.motionDuration}`,
  selectors: {
    '&[aria-selected="true"]': {
      background: vars.color.bgAlt,
      boxShadow: `inset 4px 0 0 ${vars.color.accent}`,
    },
    '&[aria-selected="true"]:dir(rtl)': { boxShadow: `inset -4px 0 0 ${vars.color.accent}` },
    '&:hover': { background: vars.color.bgAlt },
    '&:focus-visible': { ...focusOutline, outlineOffset: '-3px' },
  },
});

export const optName = style({ fontSize: '0.9rem', fontWeight: 600 });
export const optMail = style({ fontSize: '0.78rem', color: vars.color.textDim });

export const badge = style({
  fontSize: '0.68rem',
  fontWeight: vars.weight.bold,
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
  padding: `2px ${vars.space[2]}`,
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  color: vars.color.textDim,
});

export const meta = style({ fontSize: '0.78rem', color: vars.color.textDim, margin: 0 });
export const error = style({ fontSize: '0.85rem', color: vars.color.danger, margin: 0 });

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
    '&:hover:not(:disabled)': { background: vars.color.text, color: vars.color.bg },
    // Not colour alone: a disabled button also has a dashed border.
    '&:disabled': { opacity: 0.5, cursor: 'not-allowed', borderStyle: 'dashed' },
    '&:focus-visible': focusOutline,
  },
});

// ── expand-group ("who is actually in this?") ─────────────────────────────────

export const expandPanel = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  padding: less1(vars.space[4]),
  borderRadius: vars.radius.lg,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: vars.color.surface,
});

export const memberList = style({
  listStyle: 'none',
  margin: 0,
  padding: 0,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
});

export const member = style({
  display: 'flex',
  flexDirection: 'column',
  padding: `${vars.space[1]} ${vars.space[2]}`,
  borderRadius: vars.radius.md,
  selectors: { '&:nth-child(odd)': { background: vars.color.bgAlt } },
});

// ── per-contact security tab ───────────────────────────────────────────────────

export const secTab = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: less1(vars.space[4]),
  borderRadius: vars.radius.lg,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: vars.color.surface,
});

export const secRow = style({
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'center',
  gap: vars.space[3],
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  background: vars.color.bg,
  flexWrap: 'wrap',
});

export const mono = style({ fontFamily: vars.font.mono, fontSize: '0.8rem', wordBreak: 'break-all' });

export const verified = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: '0.25rem',
  color: vars.color.success,
  fontWeight: 600,
  fontSize: '0.8rem',
});
export const unverified = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: '0.25rem',
  color: vars.color.warning,
  fontWeight: 600,
  fontSize: '0.8rem',
});
// Redundant (non-colour) status glyph so verified/expired isn't colour-only
// (WCAG 1.4.1); `aria-hidden` since the adjacent word carries the meaning.
export const statusIcon = style({ fontWeight: 700 });

// The contact photo stays round (a true circle).
export const photo = style({
  width: '48px',
  height: '48px',
  borderRadius: vars.radius.pill,
  objectFit: 'cover',
  border: `${vars.border.hair} solid ${vars.color.border}`,
});

export const heading = style({ fontSize: '0.95rem', fontWeight: vars.weight.heavy, margin: 0 });
export const subHeading = style({
  fontSize: '0.74rem',
  fontWeight: vars.weight.bold,
  color: vars.color.textDim,
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
});
