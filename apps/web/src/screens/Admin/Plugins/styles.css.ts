// Admin → Plugins styles (plan §3 e7). Token-native — design tokens unchanged.
// t32 restyle: square, flat, hard shadows only. Wherever a border went from 1px to the
// 2px rule, the padding on that side gives back 1px, so no box grows (t32 §2.4 P1).

import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

/** Padding shorthand minus the extra 1px of a 2px border (P1). */
const lessRule = (block: string, inline: string): string => `calc(${block} - 1px) calc(${inline} - 1px)`;

export const screen = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  fontFamily: vars.font.ui,
  color: vars.color.text,
});

export const heading = style({ fontSize: '1.1rem', fontWeight: vars.weight.heavy, margin: 0 });
export const prose = style({ fontSize: '0.9rem', lineHeight: 1.5, margin: 0 });
export const meta = style({ fontSize: '0.8rem', color: vars.color.textDim, margin: 0 });

/** The PERMANENT unsigned-plugin banner (non-dismissible while an unsigned plugin runs). */
export const unsignedBanner = style({
  fontSize: '0.9rem',
  lineHeight: 1.5,
  margin: 0,
  padding: vars.space[4],
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  borderInlineStart: `4px solid ${vars.color.danger}`,
  fontWeight: vars.weight.bold,
});

export const list = style({ listStyle: 'none', margin: 0, padding: 0, display: 'flex', flexDirection: 'column', gap: vars.space[3] });

export const card = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: `calc(${vars.space[4]} - 1px)`,
  borderRadius: vars.radius.lg,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: vars.color.surface,
});

export const cardHead = style({ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', gap: vars.space[3], flexWrap: 'wrap' });
export const title = style({ fontSize: '0.98rem', fontWeight: vars.weight.heavy, margin: 0 });
export const row = style({ display: 'flex', gap: vars.space[2], alignItems: 'center', flexWrap: 'wrap' });

// A square badge with a hairline in its own text colour; the 1px border comes out of the
// padding. Every chip carries a text label, so the colour is never the only signal.
export const chip = style({
  fontSize: '0.7rem',
  fontWeight: vars.weight.bold,
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
  padding: `1px calc(${vars.space[2]} - 1px)`,
  border: `${vars.border.hair} solid currentColor`,
  borderRadius: vars.radius.sm,
  background: vars.color.bgSink,
  color: vars.color.textDim,
});

export const signedChip = style({ background: vars.color.bgSink, color: vars.color.success });
export const unsignedChip = style({ background: vars.color.bgSink, color: vars.color.danger });
export const capChip = style({ fontFamily: vars.font.mono, background: vars.color.bgAlt, color: vars.color.text });

const buttonBase = {
  appearance: 'none',
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: lessRule(vars.space[1], vars.space[3]),
  font: 'inherit',
  fontSize: '0.82rem',
} as const;

// The keyboard focus ring of the theme (the global :focus-visible ring is not relied on).
const focusRing = { outline: 'none', boxShadow: vars.a11y.focusRing } as const;

// Hover lifts a hard shadow (no box changes); a disabled button is dimmed and dashed.
const buttonStates = {
  '&:focus-visible': focusRing,
  '&:hover:not(:disabled)': { boxShadow: vars.elevation[1] },
  '&:active:not(:disabled)': { boxShadow: 'none' },
  '&:disabled': { opacity: 0.6, cursor: 'not-allowed', borderStyle: 'dashed' },
} as const;

export const button = style({
  ...buttonBase,
  border: `${vars.border.rule} solid ${vars.color.accent}`,
  background: vars.color.accent,
  color: vars.color.accentText,
  fontWeight: 600,
  selectors: buttonStates,
});

// Ghost buttons invert on hover instead of casting a shadow.
export const ghost = style({
  ...buttonBase,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: 'transparent',
  color: vars.color.text,
  selectors: {
    '&:focus-visible': focusRing,
    '&:hover:not(:disabled)': { background: vars.color.text, color: vars.color.bg },
    '&:disabled': buttonStates['&:disabled'],
  },
});

export const danger = style({
  ...buttonBase,
  border: `${vars.border.rule} solid ${vars.color.danger}`,
  background: vars.color.danger,
  color: vars.color.accentText,
  fontWeight: 600,
  selectors: buttonStates,
});

// The label row of a checkbox: the box inside takes the focus, so the row shows it with :focus-within.
export const check = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[2],
  fontSize: '0.84rem',
  cursor: 'pointer',
  selectors: { '&:focus-within': { outline: `${vars.border.hair} solid ${vars.color.accent}`, outlineOffset: '2px' } },
});
globalStyle(`${check} input`, { accentColor: vars.color.accent });
export const limits = style({ fontSize: '0.78rem', color: vars.color.textDim, fontFamily: vars.font.mono });
export const error = style({ fontSize: '0.85rem', color: vars.color.danger, margin: 0 });

// -- Third-party allowlist panel (t15 26.15) --------------------------------

/** The computed SHA-256 the admin approves — full, legible, wraps rather than truncates
 *  (the admin must be able to read the exact bytes they are trusting). */
export const digest = style({
  fontFamily: vars.font.mono,
  fontSize: '0.8rem',
  wordBreak: 'break-all',
  margin: 0,
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  color: vars.color.text,
});

/** A small label for a labelled value (e.g. "Computed SHA-256"). */
export const fieldLabel = style({
  fontSize: '0.72rem',
  fontWeight: vars.weight.bold,
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
  color: vars.color.textDim,
  margin: 0,
});

/** A neutral informational note (unsigned-but-pinned, high-power caps). NOT an alarm —
 *  no danger colour, no heavy weight. */
export const infoNote = style({
  fontSize: '0.8rem',
  lineHeight: 1.5,
  color: vars.color.textDim,
  margin: 0,
});

/** A revoked pin, visually de-emphasised in the oversight list. */
export const revokedRow = style({ opacity: 0.6 });

// -- Registration form, grants, status (26.20) --------------------------------

export const form = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: `calc(${vars.space[4]} - 1px)`,
  borderRadius: vars.radius.lg,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: vars.color.surface,
});

export const subheading = style({ fontSize: '0.86rem', fontWeight: vars.weight.heavy, margin: 0 });

export const field = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1], fontSize: '0.84rem' });
// The field's own label (its first span, or a fieldset legend that is not a subheading)
// is a small tracked capital; the hint span after the control keeps its sentence case.
// Capitals run about 45% wider, so in the narrow layout (the app's 760px breakpoint) the
// label keeps its case: there the admin column is too narrow and the label would wrap
// onto an extra line.
globalStyle(`${field} > span:first-child, ${field} > legend:not(${subheading})`, {
  fontSize: '0.8rem',
  fontWeight: vars.weight.bold,
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
  '@media': { '(max-width: 760px)': { textTransform: 'none', letterSpacing: vars.tracking.none } },
});
// A fieldset used as a field: a square, solid divider instead of the UA's groove. It keeps
// the UA's 2px width (the rule token), so the rows inside keep their width.
globalStyle(`fieldset${field}`, {
  border: `${vars.border.rule} solid ${vars.color.border}`,
  borderRadius: vars.radius.sm,
});

export const input = style({
  font: 'inherit',
  fontSize: '0.86rem',
  padding: lessRule(vars.space[1], vars.space[2]),
  borderRadius: vars.radius.md,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: vars.color.bgSink,
  color: vars.color.text,
  minWidth: 0,
  selectors: {
    // A light page puts fields on the brighter surface instead of the sink.
    ':root[data-appearance="light"] &': { background: vars.color.surface },
  },
});

/** The loaded / not loaded / restart required chip. */
export const loadedChip = style({ background: vars.color.bgSink, color: vars.color.success });
export const restartChip = style({ background: vars.color.bgSink, color: vars.color.danger });

/** The line that says why a plugin is not running, or that a restart is needed. */
export const statusLine = style({ fontSize: '0.84rem', lineHeight: 1.5, margin: 0 });

/** A component's own answer (the classifier test), shown verbatim. */
export const detail = style({
  fontFamily: vars.font.mono,
  fontSize: '0.76rem',
  wordBreak: 'break-word',
  whiteSpace: 'pre-wrap',
  margin: 0,
  padding: vars.space[2],
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
});

// The allowlist confirmation dialog and its button row (t32-h12): these were a static
// `style={{…}}` object, which compiles to a literal `style=` attribute that the shell
// CSP (`style-src 'self'`) drops.
export const confirmDialog = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  maxWidth: '520px',
  width: '100%',
  padding: vars.space[5],
  background: vars.color.surface,
  color: vars.color.text,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  borderRadius: vars.radius.lg,
});
export const confirmActions = style({ display: 'flex', gap: vars.space[3], justifyContent: 'flex-end' });
