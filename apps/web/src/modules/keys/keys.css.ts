// Key-management module styles (plan §3 e2). Token-native scoped classes reusing
// the V2 design-token contract so the module themes with the rest of the shell
// (same posture as the V3 contacts module styles).
//
// t32 restyle (plan §2.4): square corners, 2px control and panel borders in
// `borderStrong` with 1px less padding so no box grows (P1), focus is the
// outline ring, the current key carries an inset start bar (not colour alone),
// titles heavy, group headers, labels and badges small tracked capitals, and
// the dialog is a flat surface with the hard `elevation[3]` shadow over the
// theme's scrim.

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
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
} as const;

/** Fields sit on the sink in dark packs and on the brighter surface in light ones. */
const field = {
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  color: vars.color.text,
  selectors: {
    ':root[data-appearance="light"] &': { background: vars.color.surface },
    '&:focus-visible': focusOutline,
  },
} as const;

export const layout = style({
  display: 'grid',
  gridTemplateColumns: 'minmax(240px, 360px) 1fr',
  gap: vars.space[5],
  height: '100%',
  padding: vars.space[5],
  color: vars.color.text,
  fontFamily: vars.font.ui,
});

export const listPane = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  minWidth: 0,
  borderInlineEnd: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  paddingInlineEnd: vars.space[4],
  overflowY: 'auto',
});

export const head = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1] });
export const title = style({ fontSize: '1.1rem', fontWeight: vars.weight.heavy, margin: 0 });
export const subtitle = style({ fontSize: '0.8rem', color: vars.color.textDim, margin: 0 });

export const toolbar = style({ display: 'flex', gap: vars.space[2], flexWrap: 'wrap', alignItems: 'center' });

export const heading = style({
  ...capsLabel,
  fontSize: '0.72rem',
  color: vars.color.textDim,
  marginBlock: `${vars.space[3]} ${vars.space[1]}`,
  marginInline: 0,
});

export const button = style({
  padding: `${less1(vars.space[2])} ${less1(vars.space[4])}`,
  border: `${vars.border.rule} solid ${vars.color.accent}`,
  borderRadius: vars.radius.md,
  background: vars.color.accent,
  color: vars.color.accentText,
  font: 'inherit',
  cursor: 'pointer',
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
export const buttonGhost = style({
  padding: `${less1(vars.space[2])} ${less1(vars.space[3])}`,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  background: 'transparent',
  color: vars.color.text,
  font: 'inherit',
  cursor: 'pointer',
  minHeight: vars.a11y.touchTarget,
  selectors: {
    '&:hover:not(:disabled)': { background: vars.color.text, color: vars.color.bg },
    '&:disabled': { opacity: 0.5, cursor: 'not-allowed', borderStyle: 'dashed' },
    '&:focus-visible': focusOutline,
  },
});

export const keyList = style({
  listStyle: 'none',
  margin: 0,
  padding: 0,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
});

export const keyRow = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[2],
  padding: `${vars.space[2]} ${vars.space[3]}`,
  border: `${vars.border.hair} solid transparent`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  cursor: 'pointer',
  width: '100%',
  textAlign: 'start',
  font: 'inherit',
  color: vars.color.text,
  selectors: {
    '&:hover': { background: vars.color.bgAlt },
    '&[aria-current="true"]': {
      borderColor: vars.color.accent,
      background: vars.color.selection,
      boxShadow: `inset 4px 0 0 ${vars.color.accent}`,
    },
    '&[aria-current="true"]:dir(rtl)': { boxShadow: `inset -4px 0 0 ${vars.color.accent}` },
    // Inside the scrolling list pane, so the ring is drawn inward and not clipped.
    '&:focus-visible': { ...focusOutline, outlineOffset: '-3px' },
  },
});

export const rowBody = style({ flex: '1 1 auto', minWidth: 0, display: 'flex', flexDirection: 'column' });
export const rowName = style({ fontWeight: 600, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' });
export const rowMeta = style({ fontSize: '0.72rem', color: vars.color.textDim, fontFamily: vars.font.mono });

export const badge = style({
  ...capsLabel,
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  padding: `${vars.space[1]} ${vars.space[2]}`,
  borderRadius: vars.radius.md,
  fontSize: '0.68rem',
  background: vars.color.bgAlt,
  color: vars.color.textDim,
});

export const badgeVerified = style({ background: vars.color.success, color: vars.color.accentText });
export const badgeRevoked = style({ background: vars.color.danger, color: vars.color.accentText });

export const detail = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  minWidth: 0,
  overflowY: 'auto',
});

export const card = style({
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.lg,
  background: vars.color.surface,
  padding: less1(vars.space[5]),
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
});

export const cardName = style({ fontSize: '1.3rem', fontWeight: vars.weight.heavy, margin: 0 });
export const cardSub = style({ color: vars.color.textDim, margin: 0, fontSize: '0.85rem' });

export const fieldGroup = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2] });
export const fieldLabel = style({
  ...capsLabel,
  fontSize: '0.72rem',
  color: vars.color.textDim,
});
export const fieldRow = style({ display: 'flex', gap: vars.space[2], alignItems: 'center', flexWrap: 'wrap' });

export const fingerprint = style({
  fontFamily: vars.font.mono,
  fontSize: '0.85rem',
  wordBreak: 'break-all',
  margin: 0,
});

export const words = style({
  display: 'flex',
  flexWrap: 'wrap',
  gap: vars.space[1],
  listStyle: 'none',
  margin: 0,
  padding: 0,
});

export const word = style({
  fontFamily: vars.font.mono,
  fontSize: '0.8rem',
  padding: `${vars.space[1]} ${vars.space[2]}`,
  borderRadius: vars.radius.sm,
  background: vars.color.bgAlt,
});

// A QR code needs dark modules on a light field to scan, whatever the theme, so
// the frame stays white (the one colour literal here, on purpose).
export const qr = style({
  width: '160px',
  height: '160px',
  border: `${vars.border.hair} solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: '#ffffff',
  padding: vars.space[2],
});

export const input = style({
  ...field,
  flex: '1 1 auto',
  minWidth: 0,
  minHeight: vars.a11y.touchTarget,
  padding: `${less1(vars.space[2])} ${less1(vars.space[3])}`,
  font: 'inherit',
  // Not the bold of the capitals label that wraps it.
  fontWeight: vars.weight.regular,
});

export const textarea = style({
  ...field,
  width: '100%',
  minHeight: '7rem',
  padding: less1(vars.space[3]),
  fontFamily: vars.font.mono,
  fontSize: '0.8rem',
  resize: 'vertical',
});

export const select = style({
  ...field,
  minHeight: vars.a11y.touchTarget,
  padding: `${less1(vars.space[2])} ${less1(vars.space[3])}`,
  font: 'inherit',
  fontWeight: vars.weight.regular,
});

// Small (native) checkbox controls — enlarge the interactive target to the WCAG
// 2.2 §2.5.8 minimum and give them a visible focus ring.
export const checkbox = style({
  minWidth: vars.a11y.touchTarget,
  minHeight: vars.a11y.touchTarget,
  accentColor: vars.color.accent,
  selectors: {
    '&:focus-visible': focusOutline,
  },
});

export const empty = style({ color: vars.color.textDim, fontStyle: 'italic', padding: vars.space[4] });

export const consent = style({
  display: 'flex',
  gap: vars.space[2],
  alignItems: 'flex-start',
  padding: vars.space[3],
  border: `${vars.border.hair} solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  fontSize: '0.8rem',
});

export const actions = style({ display: 'flex', gap: vars.space[2], flexWrap: 'wrap' });

// The page behind the dialog is covered by the theme's modal scrim.
export const dialogBackdrop = style({
  position: 'fixed',
  inset: 0,
  background: vars.color.scrim,
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  padding: vars.space[5],
  zIndex: 50,
});

export const dialog = style({
  background: vars.color.surface,
  color: vars.color.text,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.lg,
  boxShadow: vars.elevation[3],
  padding: less1(vars.space[5]),
  maxWidth: '40rem',
  width: '100%',
  maxHeight: '85vh',
  overflowY: 'auto',
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
});

export const fieldStack = style({ display: 'flex', flexDirection: 'column', gap: vars.space[3] });
// A field label wraps its control; the control keeps its own case and spacing
// (form controls reset text-transform and letter-spacing).
export const label = style({
  ...capsLabel,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
  fontSize: '0.8rem',
  color: vars.color.textDim,
  selectors: {
    // A checkbox label is an option name in a row of options, not a field
    // label: capitals would widen the row until it wraps. Box and name share
    // one line (this was an inline style the shell CSP drops).
    '&:has(> input[type="checkbox"])': {
      flexDirection: 'row',
      alignItems: 'center',
      fontWeight: vars.weight.regular,
      textTransform: 'none',
      letterSpacing: vars.tracking.none,
      color: vars.color.text,
    },
  },
});
export const preview = style({
  padding: vars.space[3],
  border: `${vars.border.hair} solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
});
