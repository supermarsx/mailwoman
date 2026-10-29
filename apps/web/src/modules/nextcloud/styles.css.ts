// Nextcloud UI styles (plan §3 e7). Token-native — design tokens unchanged.
//
// t32 restyle (plan §2.4): square corners, 2px control and panel borders in
// `borderStrong` with 1px less padding so no box grows (P1), focus is the
// outline ring, a picked row carries an inset start bar (not colour alone),
// headings heavy, field labels small tracked capitals.

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

/** The outline ring (forced colours keep it; they drop a box-shadow ring). */
const focusOutline = {
  outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`,
  outlineOffset: '2px',
} as const;

/** `base` less the 1px a 2px rule adds over the old 1px border (P1). */
const less1 = (base: string): string => `calc(${base} - 1px)`;

export const panel = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  fontFamily: vars.font.ui,
  color: vars.color.text,
});

export const bar = style({ display: 'flex', alignItems: 'center', gap: vars.space[2], flexWrap: 'wrap' });
export const crumb = style({ fontSize: '0.82rem', color: vars.color.textDim, fontFamily: vars.font.mono });

// The list keeps its 1px frame: it has no padding to give back, and a 2px frame
// would make it 2px taller.
export const list = style({
  listStyle: 'none',
  margin: 0,
  padding: 0,
  display: 'flex',
  flexDirection: 'column',
  gap: '2px',
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  maxHeight: '22rem',
  overflowY: 'auto',
});

// A single list row (the `<li>`) is a plain layout wrapper; the interactive
// affordance lives on the inner `row` button (keyboard-operable + focus ring).
export const item = style({ display: 'block' });

// Interactive file/folder row: a real <button> so it is natively focusable and
// operable with Enter/Space, with a visible focus ring (WCAG 2.4.11/2.4.13) and
// a 24px minimum target (WCAG 2.5.8). Selection is shown via aria-pressed (not
// colour alone — WCAG 1.4.1) plus the ✓ glyph and an inset start bar.
export const row = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[3],
  width: '100%',
  minHeight: vars.a11y.touchTarget,
  padding: `${vars.space[2]} ${vars.space[3]}`,
  appearance: 'none',
  border: 'none',
  background: 'transparent',
  color: vars.color.text,
  font: 'inherit',
  textAlign: 'start',
  cursor: 'pointer',
  borderRadius: vars.radius.sm,
  transition: `background ${vars.a11y.motionDuration}`,
  selectors: {
    '&:hover': { background: vars.color.bgAlt },
    '&[aria-pressed="true"]': {
      background: vars.color.bgAlt,
      boxShadow: `inset 4px 0 0 ${vars.color.accent}`,
    },
    '&[aria-pressed="true"]:dir(rtl)': { boxShadow: `inset -4px 0 0 ${vars.color.accent}` },
    // Inside the scrolling list, so the ring is drawn inward and not clipped.
    '&:focus-visible': { ...focusOutline, outlineOffset: '-3px' },
  },
});

// Non-interactive row (e.g. a file shown while picking a destination folder).
export const rowStatic = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[3],
  width: '100%',
  minHeight: vars.a11y.touchTarget,
  padding: `${vars.space[2]} ${vars.space[3]}`,
  color: vars.color.textDim,
});

export const name = style({ fontSize: '0.9rem', flex: 1 });
export const size = style({ fontSize: '0.76rem', color: vars.color.textDim });
export const dirIcon = style({ fontSize: '0.9rem' });

export const field = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2] });
// Field label: small tracked capitals (the tracking token is 0 for ar/ja/zh).
export const label = style({
  fontSize: '0.8rem',
  fontWeight: vars.weight.bold,
  letterSpacing: vars.tracking.label,
  textTransform: 'uppercase',
  color: vars.color.textDim,
});

export const input = style({
  padding: `${less1(vars.space[2])} ${less1(vars.space[3])}`,
  minHeight: vars.a11y.touchTarget,
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

// The checkbox inside carries its own outline ring (the global `:focus-visible`).
export const check = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[2],
  minHeight: vars.a11y.touchTarget,
  fontSize: '0.88rem',
  cursor: 'pointer',
});

export const button = style({
  appearance: 'none',
  border: `${vars.border.rule} solid ${vars.color.accent}`,
  background: vars.color.accent,
  color: vars.color.accentText,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  minHeight: vars.a11y.touchTarget,
  padding: `${less1(vars.space[2])} ${less1(vars.space[4])}`,
  font: 'inherit',
  fontSize: '0.9rem',
  fontWeight: vars.weight.medium,
  transition: `box-shadow ${vars.a11y.motionDuration}`,
  selectors: {
    '&:hover:not(:disabled)': { boxShadow: vars.elevation[1] },
    '&:active:not(:disabled)': { boxShadow: 'none' },
    // Not colour alone: a disabled button also has a dashed border.
    '&:disabled': { opacity: 0.5, cursor: 'not-allowed', borderStyle: 'dashed' },
    '&:focus-visible': focusOutline,
  },
});

// Ghost buttons invert on hover instead of casting a shadow.
export const ghost = style({
  appearance: 'none',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: 'transparent',
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  minWidth: vars.a11y.touchTarget,
  minHeight: vars.a11y.touchTarget,
  padding: `${less1(vars.space[1])} ${less1(vars.space[3])}`,
  font: 'inherit',
  fontSize: '0.82rem',
  transition: `background ${vars.a11y.motionDuration}, color ${vars.a11y.motionDuration}`,
  selectors: {
    '&:hover:not(:disabled)': { background: vars.color.text, color: vars.color.bg },
    '&:disabled': { opacity: 0.5, cursor: 'not-allowed', borderStyle: 'dashed' },
    '&:focus-visible': focusOutline,
  },
});

export const linkBox = style({
  fontFamily: vars.font.mono,
  fontSize: '0.85rem',
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  color: vars.color.text,
  wordBreak: 'break-all',
  userSelect: 'all',
});

export const meta = style({ fontSize: '0.8rem', color: vars.color.textDim, margin: 0 });
export const error = style({ fontSize: '0.85rem', color: vars.color.danger, margin: 0 });
export const heading = style({ fontSize: '0.95rem', fontWeight: vars.weight.heavy, margin: 0 });
export const actions = style({ display: 'flex', gap: vars.space[2], flexWrap: 'wrap', alignItems: 'center' });
