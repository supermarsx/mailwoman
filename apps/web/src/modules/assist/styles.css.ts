// V7 Assist UI styles (plan §3 e6). Token-native — the frozen design tokens
// (theme/contract.css.ts) are UNCHANGED; this file only references `vars.*`.
//
// t32 restyle (plan §2.4): square corners, 2px control and panel borders in
// `borderStrong` with 1px less padding so no box grows (P1), focus is the
// outline ring, headings heavy, labels small tracked capitals.

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
  gap: vars.space[4],
  fontFamily: vars.font.ui,
  color: vars.color.text,
  minWidth: 0,
});

export const section = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: less1(vars.space[4]),
  borderRadius: vars.radius.lg,
  background: vars.color.surface,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
});

export const heading = style({ fontSize: '1rem', fontWeight: vars.weight.heavy, margin: 0 });
export const subHeading = style({
  fontSize: '0.75rem',
  fontWeight: vars.weight.bold,
  color: vars.color.textDim,
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
});
export const prose = style({ fontSize: '0.9rem', lineHeight: 1.5, margin: 0 });
export const meta = style({ fontSize: '0.78rem', color: vars.color.textDim });

export const row = style({ display: 'flex', gap: vars.space[3], alignItems: 'center', flexWrap: 'wrap' });
export const field = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2] });
export const toolbar = style({ display: 'flex', gap: vars.space[2], flexWrap: 'wrap' });

/** The ask box: {@link input} that grows to fill its flex {@link row}. A class
 *  rather than an inline `style={{ flex: '1 1 auto' }}` — a fully-static style
 *  object is hoisted by Solid's compiler into the compiled template as a literal
 *  `style="…"` attribute, which the shell's `style-src 'self'` blocks (t24-e13). */
export const askInput = style({ flex: '1 1 auto' });

export const input = style({
  padding: `${less1(vars.space[2])} ${less1(vars.space[3])}`,
  borderRadius: vars.radius.md,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: vars.color.bgSink,
  color: vars.color.text,
  font: 'inherit',
  fontSize: '0.9rem',
  minWidth: 0,
  minHeight: vars.a11y.touchTarget,
  selectors: {
    // A light page puts fields on the brighter surface instead of the sink.
    ':root[data-appearance="light"] &': { background: vars.color.surface },
    '&:focus-visible': focusOutline,
  },
});

export const button = style({
  appearance: 'none',
  border: `${vars.border.rule} solid ${vars.color.accent}`,
  background: vars.color.accent,
  color: vars.color.accentText,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${less1(vars.space[2])} ${less1(vars.space[4])}`,
  font: 'inherit',
  fontSize: '0.9rem',
  fontWeight: vars.weight.medium,
  minHeight: vars.a11y.touchTarget,
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
  padding: `${less1(vars.space[2])} ${less1(vars.space[3])}`,
  font: 'inherit',
  fontSize: '0.85rem',
  minHeight: vars.a11y.touchTarget,
  selectors: {
    '&:hover:not(:disabled)': { background: vars.color.text, color: vars.color.bg },
    '&:disabled': { opacity: 0.5, cursor: 'not-allowed', borderStyle: 'dashed' },
    '&:focus-visible': focusOutline,
  },
});

// The push-to-talk dictation trigger; `active` is toggled while recording. It
// is a text button, so it is square like the others (round is kept for circles).
export const mic = style({
  appearance: 'none',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: 'transparent',
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${less1(vars.space[2])} ${less1(vars.space[4])}`,
  font: 'inherit',
  fontSize: '0.85rem',
  fontWeight: vars.weight.medium,
  minHeight: vars.a11y.touchTarget,
  selectors: {
    '&:focus-visible': focusOutline,
  },
});
export const micActive = style({
  background: vars.color.danger,
  color: vars.color.accentText,
  borderColor: vars.color.danger,
});

export const check = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[2],
  fontSize: '0.9rem',
  cursor: 'pointer',
});

// Chat transcript.
export const transcript = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  maxHeight: '18rem',
  overflowY: 'auto',
  padding: vars.space[2],
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  border: `${vars.border.hair} solid ${vars.color.border}`,
});
export const bubbleUser = style({
  alignSelf: 'flex-end',
  maxWidth: '85%',
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderRadius: vars.radius.md,
  background: vars.color.accent,
  color: vars.color.accentText,
  fontSize: '0.9rem',
});
export const bubbleAssistant = style({
  alignSelf: 'flex-start',
  maxWidth: '85%',
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  color: vars.color.text,
  fontSize: '0.9rem',
});

// A proposed tool action awaiting HUMAN review (never auto-executed).
export const proposal = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  borderInlineStart: `${vars.border.heavy} solid ${vars.color.warning}`,
  background: vars.color.bgAlt,
});

// Suggestion output (composer transform / summary) the user chooses to apply.
export const suggestion = style({
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  border: `${vars.border.hair} dashed ${vars.color.border}`,
  fontSize: '0.9rem',
  lineHeight: 1.5,
  whiteSpace: 'pre-wrap',
});

// Auto-tag suggestion chip: square with a hair rule. The tag is user content,
// so its case is left alone.
export const badge = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[2],
  padding: `${vars.space[1]} ${vars.space[3]}`,
  borderRadius: vars.radius.md,
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  background: vars.color.bg,
  fontSize: '0.82rem',
});

export const disclosure = style({
  fontSize: '0.82rem',
  lineHeight: 1.5,
  margin: 0,
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  borderInlineStart: `${vars.border.heavy} solid ${vars.color.accent}`,
  color: vars.color.text,
});

export const error = style({ fontSize: '0.85rem', color: vars.color.danger, margin: 0 });

export const auditList = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
  listStyle: 'none',
  margin: 0,
  padding: 0,
  fontSize: '0.78rem',
  color: vars.color.textDim,
});
