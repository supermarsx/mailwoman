// OAuth consent screen styles (plan §3 e8). Token-native — design tokens unchanged.
// t32 restyle: square, flat, hard shadows only. Wherever a border went from 1px to the
// 2px rule, the padding on that side gives back 1px, so no box grows (t32 §2.4 P1).

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const backdrop = style({
  minHeight: '100vh',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  padding: vars.space[5],
  background: vars.color.bg,
  fontFamily: vars.font.ui,
  color: vars.color.text,
});

export const card = style({
  width: '100%',
  maxWidth: '480px',
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  padding: `calc(${vars.space[6]} - 1px)`,
  borderRadius: vars.radius.lg,
  background: vars.color.surface,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  boxShadow: vars.elevation[3],
});

export const heading = style({ fontSize: '1.1rem', fontWeight: vars.weight.heavy, margin: 0 });
export const client = style({ fontSize: '1rem', fontWeight: vars.weight.bold });
export const prose = style({ fontSize: '0.9rem', lineHeight: 1.5, margin: 0, color: vars.color.text });
export const subHeading = style({
  fontSize: '0.78rem',
  fontWeight: vars.weight.bold,
  color: vars.color.textDim,
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
});

export const scopeList = style({
  margin: 0,
  paddingInlineStart: vars.space[5],
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  fontSize: '0.9rem',
  lineHeight: 1.45,
});

// Status badges: square, bold, and the status is always in the text. They keep sentence
// case: the unapproved one is a full sentence and in capitals it wraps onto a second line
// in the phone-width card.
export const approved = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  fontSize: '0.75rem',
  fontWeight: vars.weight.bold,
  padding: `${vars.space[1]} ${vars.space[3]}`,
  borderRadius: vars.radius.sm,
  background: vars.color.success,
  color: vars.color.accentText,
  width: 'fit-content',
});

export const unapproved = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  fontSize: '0.75rem',
  fontWeight: vars.weight.bold,
  padding: `${vars.space[1]} ${vars.space[3]}`,
  borderRadius: vars.radius.sm,
  background: vars.color.bgAlt,
  color: vars.color.danger,
  border: `${vars.border.hair} solid ${vars.color.danger}`,
  width: 'fit-content',
});

export const meta = style({ fontSize: '0.78rem', color: vars.color.textDim, wordBreak: 'break-all' });

export const actions = style({ display: 'flex', gap: vars.space[3], justifyContent: 'flex-end' });

// WCAG 2.2 §2.4.11/§2.4.13 — visible focus indicator. An outline, not a box-shadow ring,
// because box-shadow is dropped under forced colours.
const focusOutline = {
  outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`,
  outlineOffset: '2px',
} as const;

export const grant = style({
  appearance: 'none',
  border: `${vars.border.rule} solid ${vars.color.accent}`,
  background: vars.color.accent,
  color: vars.color.accentText,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  // WCAG 2.2 §2.5.8 — minimum 24×24 CSS-px target.
  minHeight: vars.a11y.touchTarget,
  padding: `calc(${vars.space[2]} - 1px) calc(${vars.space[5]} - 1px)`,
  font: 'inherit',
  fontSize: '0.9rem',
  fontWeight: 600,
  selectors: {
    // Hover lifts a hard shadow (no box changes); disabled is dimmed and dashed.
    '&:hover:not(:disabled)': { boxShadow: vars.elevation[1] },
    '&:active:not(:disabled)': { boxShadow: 'none' },
    '&:disabled': { opacity: 0.6, cursor: 'not-allowed', borderStyle: 'dashed' },
    '&:focus-visible': focusOutline,
  },
});

// Ghost button: transparent with the strong border, inverted on hover.
export const deny = style({
  appearance: 'none',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: 'transparent',
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  minHeight: vars.a11y.touchTarget,
  padding: `calc(${vars.space[2]} - 1px) calc(${vars.space[5]} - 1px)`,
  font: 'inherit',
  fontSize: '0.9rem',
  selectors: {
    '&:hover:not(:disabled)': { background: vars.color.text, color: vars.color.bg },
    '&:focus-visible': focusOutline,
  },
});

export const error = style({ fontSize: '0.85rem', color: vars.color.danger, margin: 0 });
