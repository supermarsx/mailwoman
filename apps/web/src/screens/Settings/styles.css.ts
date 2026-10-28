// Settings account-surface styles (t16 e15). Token-native — no new design tokens.
// Layout uses logical properties (inline-start/end, margin-inline) so the surface
// mirrors correctly under `dir="rtl"` (W20) with no per-direction overrides.
//
// t32 restyle (plan §2.4): square corners, flat fields, 2px control and panel
// borders in `borderStrong` with 1px less padding so no box grows (P1), focus is
// the outline ring, headings heavy, field labels small tracked capitals, and a
// selected option is filled with the accent (plus `aria-pressed` and weight).

import { style, globalStyle } from '@vanilla-extract/css';
import { registerFieldLayout } from '../../styles/formLayout.css.ts';
import { vars } from '../../theme/contract.css.ts';

/** The outline ring (forced colours keep it; they drop a box-shadow ring). */
const focusOutline = {
  outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`,
  outlineOffset: '2px',
} as const;

/** `base` less the 1px a 2px rule adds over the old 1px border (P1). */
const less1 = (base: string): string => `calc(${base} - 1px)`;

export const stack = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  fontFamily: vars.font.ui,
  color: vars.color.text,
});

export const section = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: less1(vars.space[5]),
  borderRadius: vars.radius.lg,
  background: vars.color.surface,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
});

export const heading = style({ fontSize: '1rem', fontWeight: vars.weight.heavy, margin: 0 });
export const subheading = style({ fontSize: '0.9rem', fontWeight: vars.weight.heavy, margin: 0 });
export const prose = style({ fontSize: '0.9rem', lineHeight: 1.5, margin: 0 });
export const meta = style({ fontSize: '0.8rem', color: vars.color.textDim, margin: 0 });

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
  borderRadius: vars.radius.md,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: vars.color.bgSink,
  color: vars.color.text,
  font: 'inherit',
  fontSize: '0.9rem',
  minHeight: vars.a11y.touchTarget,
  transitionProperty: 'border-color',
  transitionDuration: vars.a11y.motionDuration,
  selectors: {
    // A light page puts fields on the brighter surface instead of the sink.
    ':root[data-appearance="light"] &': { background: vars.color.surface },
    '&:focus-visible': focusOutline,
  },
});

export const textarea = style([input, { minHeight: '5rem', resize: 'vertical', lineHeight: 1.5 }]);

export const select = style([input, { cursor: 'pointer' }]);

export const row = style({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  gap: vars.space[3],
  flexWrap: 'wrap',
});

export const grow = style({ flex: 1, minWidth: '12rem' });

export const actions = style({ display: 'flex', gap: vars.space[3], flexWrap: 'wrap', alignItems: 'center' });

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
  // Content-sized in any flex column or grid (which would stretch `auto`).
  inlineSize: 'fit-content',
  maxInlineSize: '100%',
  transitionProperty: 'box-shadow, background, color',
  transitionDuration: vars.a11y.motionDuration,
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
  padding: `${less1(vars.space[2])} ${less1(vars.space[4])}`,
  font: 'inherit',
  fontSize: '0.9rem',
  minHeight: vars.a11y.touchTarget,
  // Content-sized in any flex column or grid (which would stretch `auto`).
  inlineSize: 'fit-content',
  maxInlineSize: '100%',
  transitionProperty: 'box-shadow, background, color',
  transitionDuration: vars.a11y.motionDuration,
  selectors: {
    '&:hover:not(:disabled)': { background: vars.color.text, color: vars.color.bg },
    '&:disabled': { opacity: 0.5, cursor: 'not-allowed', borderStyle: 'dashed' },
    '&:focus-visible': focusOutline,
  },
});

export const danger = style([
  ghost,
  {
    borderColor: vars.color.danger,
    background: vars.color.danger,
    color: vars.color.accentText,
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

// A pressable option in a segmented control (presets / direction). A 1px rule
// keeps every option, and the theme gallery cards built on it, the same size.
export const option = style({
  appearance: 'none',
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  background: 'transparent',
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[2]} ${vars.space[3]}`,
  font: 'inherit',
  fontSize: '0.85rem',
  minHeight: vars.a11y.touchTarget,
  selectors: {
    '&:hover:not([aria-pressed="true"])': { background: vars.color.bgAlt },
    '&[aria-pressed="true"]': {
      borderColor: vars.color.accent,
      background: vars.color.accent,
      color: vars.color.accentText,
      fontWeight: 600,
    },
    '&:focus-visible': focusOutline,
  },
});

export const options = style({ display: 'flex', gap: vars.space[2], flexWrap: 'wrap' });

export const list = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  margin: 0,
  padding: 0,
  listStyle: 'none',
});

export const item = style({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  gap: vars.space[3],
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  background: vars.color.bg,
  flexWrap: 'wrap',
});

export const itemMain = style({ display: 'flex', flexDirection: 'column', gap: '2px', minWidth: '10rem' });
export const itemName = style({ fontSize: '0.9rem', fontWeight: 600 });

// Status tags and key bindings (`<kbd>`), so the text case is left alone.
export const badge = style({
  fontSize: '0.72rem',
  fontWeight: vars.weight.bold,
  padding: `1px ${vars.space[2]}`,
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  color: vars.color.textDim,
});

export const check = style({
  display: 'flex',
  alignItems: 'flex-start',
  gap: vars.space[2],
  fontSize: '0.88rem',
  cursor: 'pointer',
  lineHeight: 1.4,
});

export const checkbox = style({
  width: vars.a11y.touchTarget,
  height: vars.a11y.touchTarget,
  minWidth: vars.a11y.touchTarget,
  minHeight: vars.a11y.touchTarget,
  margin: 0,
  cursor: 'pointer',
  accentColor: vars.color.accent,
  selectors: {
    '&:focus-visible': focusOutline,
  },
});

// The one-time recovery-code panel — dashed warning border, monospace grid.
export const codesPanel = style({
  padding: vars.space[4],
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  border: `${vars.border.hair} dashed ${vars.color.warning}`,
});

export const codesGrid = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fill, minmax(9rem, 1fr))',
  gap: vars.space[2],
  fontFamily: vars.font.mono,
  fontSize: '0.95rem',
  marginBlock: vars.space[3],
  marginInline: 0,
  userSelect: 'all',
});

export const secret = style({
  fontFamily: vars.font.mono,
  fontSize: '1rem',
  letterSpacing: '0.05em',
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  wordBreak: 'break-all',
  userSelect: 'all',
});

export const warn = style({
  fontSize: '0.88rem',
  lineHeight: 1.5,
  margin: 0,
  padding: vars.space[4],
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  borderInlineStart: `${vars.border.heavy} solid ${vars.color.warning}`,
});

export const success = style({ fontSize: '0.9rem', color: vars.color.success, margin: 0, fontWeight: 600 });
export const error = style({ fontSize: '0.85rem', color: vars.color.danger, margin: 0 });

// Direction-preview frame (W20) — respects its own `dir` attribute.
export const previewFrame = style({
  padding: vars.space[4],
  borderRadius: vars.radius.md,
  border: `${vars.border.hair} dashed ${vars.color.border}`,
  background: vars.color.bg,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
});

// ── Settings layout primitive (t47, the Settings half of t39 L15) ─────────────
// One layout for every user Settings section:
//   section card  = `section` + `heading` + optional `prose`/`meta` description
//   setting row   = `field`: label, control, help text (`meta`/`hint`), actions
//   group         = `group` + `subheading`: related rows under a subheading
//   actions       = `actions` (start) or `actionsEnd` (end of the row/group)
//   destructive   = `dangerZone`: separated by a rule, below the other rows
// Buttons are content-sized everywhere in Settings: the flex columns and grids
// that hold them stretch any item whose inline size is `auto`, which is what
// made actions span the whole card (see the zero-specificity rule below).

/** Related settings under a subheading, ruled off from the rows above. */
export const group = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  minInlineSize: 0,
  margin: 0,
  padding: 0,
  border: 0,
});

/** Actions aligned at the end of their row or group. */
export const actionsEnd = style([actions, { justifyContent: 'flex-end' }]);

/** Destructive actions: kept apart from the ordinary rows by a rule and space. */
export const dangerZone = style({
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'flex-start',
  gap: vars.space[3],
  marginBlockStart: vars.space[3],
  paddingBlockStart: vars.space[4],
  borderBlockStart: `${vars.border.hair} solid ${vars.color.danger}`,
});

registerFieldLayout(field);
// One setting per line: a multi-column auto-fit grid packed unrelated settings
// (and their help and reset buttons) side by side into a wall of controls.
globalStyle(`${section}:has(> ${field})`, { display: 'flex', flexDirection: 'column' });
// The row spans the card (so its rule does); its control and help keep a
// readable measure.
globalStyle(`${section} > ${field}`, { maxInlineSize: 'none', marginBlockEnd: 0 });
globalStyle(`${section} > ${field} > :is(input:not([type='checkbox'], [type='radio']), select)`, { maxInlineSize: '32rem' });
globalStyle(`${section} > ${field} > :is(p, ${meta})`, { maxInlineSize: '70ch' });
// Each setting row, and each group, is ruled off from the one before it.
globalStyle(`${section} > :is(${field}, ${group}) + :is(${field}, ${group})`, {
  paddingBlockStart: vars.space[4],
  borderBlockStart: `${vars.border.hair} solid ${vars.color.border}`,
});
globalStyle(`${section} > ${field}:has(> :is(h3, h4, ${subheading}))`, { gap: vars.space[3] });

// Column sections stretch content surfaces, but actions retain their natural size.
globalStyle(`${section} > button, ${section} > form > button`, { alignSelf: 'flex-start', inlineSize: 'fit-content', maxInlineSize: '100%' });
globalStyle(`${section} > form`, { display: 'flex', flexDirection: 'column', gap: vars.space[4] });
// Stacked cards in one Settings section, nested cards, and headings.
globalStyle(`${section} ${section}`, { background: vars.color.bg, borderWidth: vars.border.hair });
globalStyle(`${actions} > *, ${options} > *`, { flex: '0 0 auto' });

globalStyle(`${row}:has(> ${field})`, { alignItems: 'end' });
globalStyle(`${row} > label:has(> input[type="checkbox"]), ${row} > label:has(> input[type="radio"])`, { alignSelf: 'end', alignItems: 'center', minBlockSize: '2.5rem', marginBlockEnd: 0 });

// Every Settings section (`RetainedSettingsSection`) is a column with the same
// rhythm between its cards, whatever module renders them.
const settingsSection = '[data-settings-section]:not([hidden])';
globalStyle(settingsSection, { display: 'flex', flexDirection: 'column', gap: vars.space[5], minInlineSize: 0 });
// Zero specificity: any component class that sizes a button on purpose (theme
// cards, table actions, segmented options) still wins; only the `auto` inline
// size that flex/grid parents stretch is replaced.
globalStyle(':where([data-settings-section]) :where(button, input[type="submit"], input[type="button"], input[type="reset"])', {
  inlineSize: 'fit-content',
  maxInlineSize: '100%',
});
// Long labels wrap inside their button instead of overflowing the card.
globalStyle(':where([data-settings-section]) :where(button)', { overflowWrap: 'anywhere' });
// Radio and checkbox choices laid out in a row keep a gap between options.
globalStyle(':where([data-settings-section]) :where(fieldset, [role="radiogroup"])', { minInlineSize: 0 });
globalStyle(':where([data-settings-section]) :where(fieldset, [role="radiogroup"]) > label + label', { marginInlineStart: vars.space[4] });
