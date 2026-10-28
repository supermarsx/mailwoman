// Rules (mail-filter) UI styles (audit #1). Token-native — design tokens unchanged.
//
// t32 restyle (plan §2.4): square corners, 2px control and panel borders in
// `borderStrong` with 1px less padding so no box grows (P1), focus is the
// outline ring, headings heavy, labels and badges small tracked capitals, and
// the active tab has a 3px bar plus heavier text (not colour alone).

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

/** The outline ring (forced colours keep it; they drop a box-shadow ring). */
const focusOutline = {
  outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`,
  outlineOffset: '2px',
} as const;

/** `base` less the 1px a 2px rule adds over the old 1px border (P1). */
const less1 = (base: string): string => `calc(${base} - 1px)`;

/** Small tracked capitals (the tracking token is 0 for ar/ja/zh). */
const capsLabel = {
  fontWeight: vars.weight.bold,
  letterSpacing: vars.tracking.label,
  textTransform: 'uppercase',
} as const;

export const panel = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  fontFamily: vars.font.ui,
  color: vars.color.text,
});

/** The Rules & filters section as a Settings card (matches the Settings
 * `section` primitive); the sub-panels inside it stay plain `panel`s. */
export const card = style([panel, {
  minInlineSize: 0,
  padding: less1(vars.space[5]),
  borderRadius: vars.radius.lg,
  background: vars.color.surface,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
}]);

export const heading = style({ fontSize: '1rem', fontWeight: vars.weight.heavy, margin: 0 });

/** Visually hidden but exposed to assistive tech (the `.sr-only` pattern). */
export const srOnly = style({
  position: 'absolute',
  width: '1px',
  height: '1px',
  padding: 0,
  margin: '-1px',
  overflow: 'hidden',
  clip: 'rect(0 0 0 0)',
  whiteSpace: 'nowrap',
  border: 0,
});

export const prose = style({ fontSize: '0.9rem', lineHeight: 1.5, margin: 0, color: vars.color.textDim });

export const tabs = style({
  display: 'flex',
  gap: vars.space[2],
  borderBlockEnd: `${vars.border.hair} solid ${vars.color.borderStrong}`,
});

// The 2px bottom border turns accent on the active tab; a 1px inset shadow on
// top of it makes the 3px bar without changing the box. Tab names keep their
// case and the active one is only medium weight: capitals or bold would widen
// the tabs past their old size.
export const tab = style({
  appearance: 'none',
  background: 'transparent',
  border: 'none',
  borderBlockEnd: `${vars.border.rule} solid transparent`,
  borderRadius: vars.radius.sm,
  color: vars.color.textDim,
  padding: `${vars.space[2]} ${vars.space[3]}`,
  minHeight: vars.a11y.touchTarget,
  cursor: 'pointer',
  font: 'inherit',
  selectors: {
    '&:hover': { color: vars.color.text },
    '&[aria-selected="true"]': {
      color: vars.color.text,
      fontWeight: vars.weight.medium,
      borderBlockEndColor: vars.color.accent,
      boxShadow: `inset 0 -1px 0 ${vars.color.accent}`,
    },
    '&:focus-visible': focusOutline,
  },
});

export const list = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2], listStyle: 'none', margin: 0, padding: 0 });

export const ruleRow = style({
  display: 'flex',
  gap: vars.space[3],
  alignItems: 'center',
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  background: vars.color.surface,
  flexWrap: 'wrap',
});

export const ruleName = style({ fontWeight: 600, flex: 1, minWidth: '8rem' });

export const badge = style({
  ...capsLabel,
  fontSize: '0.72rem',
  padding: `2px ${vars.space[2]}`,
  borderRadius: vars.radius.md,
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  color: vars.color.textDim,
  whiteSpace: 'nowrap',
});

export const badgeServer = style({ color: vars.color.accent, borderColor: vars.color.accent });

export const builder = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: less1(vars.space[4]),
  borderRadius: vars.radius.lg,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: vars.color.bgAlt,
});

export const row = style({ display: 'flex', gap: vars.space[2], alignItems: 'center', flexWrap: 'wrap' });

export const field = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1] });

export const label = style({ ...capsLabel, fontSize: '0.8rem', color: vars.color.textDim });

export const input = style({
  font: 'inherit',
  color: vars.color.text,
  background: vars.color.bgSink,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  padding: `${less1(vars.space[2])} ${less1(vars.space[2])}`,
  minHeight: vars.a11y.touchTarget,
  selectors: {
    // A light page puts fields on the brighter surface instead of the sink.
    ':root[data-appearance="light"] &': { background: vars.color.surface },
    '&:focus-visible': focusOutline,
  },
});

export const select = style([input, { cursor: 'pointer' }]);

export const clause = style({
  display: 'flex',
  gap: vars.space[2],
  alignItems: 'center',
  flexWrap: 'wrap',
  padding: vars.space[2],
  borderRadius: vars.radius.sm,
  background: vars.color.surface,
});

// Ghost button: inverts on hover instead of casting a shadow.
export const btn = style({
  appearance: 'none',
  font: 'inherit',
  cursor: 'pointer',
  minHeight: vars.a11y.touchTarget,
  padding: `${less1(vars.space[2])} ${less1(vars.space[3])}`,
  borderRadius: vars.radius.sm,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: 'transparent',
  color: vars.color.text,
  selectors: {
    '&:hover:not(:disabled)': { background: vars.color.text, color: vars.color.bg },
    // Not colour alone: a disabled button also has a dashed border.
    '&:disabled': { opacity: 0.5, cursor: 'not-allowed', borderStyle: 'dashed' },
    '&:focus-visible': focusOutline,
  },
});

export const btnPrimary = style([
  btn,
  {
    background: vars.color.accent,
    color: vars.color.accentText,
    borderColor: vars.color.accent,
    selectors: {
      '&:hover:not(:disabled)': {
        background: vars.color.accent,
        color: vars.color.accentText,
        boxShadow: vars.elevation[1],
      },
      '&:active:not(:disabled)': { boxShadow: 'none' },
    },
  },
]);

export const btnDanger = style([
  btn,
  {
    background: vars.color.danger,
    color: vars.color.accentText,
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

export const iconBtn = style([btn, { minWidth: vars.a11y.touchTarget, padding: less1(vars.space[1]) }]);

// ── raw editor ────────────────────────────────────────────────────────────

export const editorWrap = style({ position: 'relative', display: 'flex', flexDirection: 'column', gap: vars.space[2] });

export const textarea = style({
  font: vars.font.mono,
  fontSize: '0.85rem',
  lineHeight: 1.5,
  color: vars.color.text,
  background: vars.color.bgSink,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  padding: less1(vars.space[3]),
  minHeight: '10rem',
  width: '100%',
  resize: 'vertical',
  whiteSpace: 'pre',
  overflowX: 'auto',
  selectors: {
    ':root[data-appearance="light"] &': { background: vars.color.surface },
    '&:focus-visible': focusOutline,
  },
});

export const highlight = style({
  font: vars.font.mono,
  fontSize: '0.85rem',
  lineHeight: 1.5,
  margin: 0,
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  overflowX: 'auto',
  whiteSpace: 'pre',
});

export const tokKeyword = style({ color: vars.color.accent, fontWeight: 600 });
export const tokString = style({ color: vars.color.success });
export const tokTag = style({ color: vars.color.link });
export const tokNumber = style({ color: vars.color.warning });
export const tokComment = style({ color: vars.color.textDim, fontStyle: 'italic' });

export const diagList = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
  listStyle: 'none',
  margin: 0,
  padding: vars.space[2],
  borderRadius: vars.radius.sm,
  border: `${vars.border.hair} solid ${vars.color.danger}`,
  color: vars.color.danger,
  fontSize: '0.82rem',
});

export const okNote = style({ color: vars.color.success, fontSize: '0.82rem' });

// ── dry-run ───────────────────────────────────────────────────────────────

export const dryGrid = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2] });

export const dryResult = style({
  display: 'flex',
  gap: vars.space[2],
  alignItems: 'center',
  padding: vars.space[2],
  borderRadius: vars.radius.sm,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  flexWrap: 'wrap',
});

// Match markers stay round (true circles).
export const matchDot = style({
  width: '0.6rem',
  height: '0.6rem',
  borderRadius: vars.radius.pill,
  flexShrink: 0,
});

export const matchYes = style([matchDot, { background: vars.color.success }]);
export const matchNo = style([matchDot, { background: vars.color.border }]);
