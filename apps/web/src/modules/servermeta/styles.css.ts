// Server / mailbox METADATA view styles (t13 e8). Token-native.
//
// t32 restyle (plan §2.4): square corners, 2px control borders in `borderStrong`
// with 1px less padding so no box grows (P1), focus is the outline ring,
// headings heavy, labels small tracked capitals.

import { globalStyle, style } from '@vanilla-extract/css';
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

// In the user's Settings the view is a section card like its neighbours (the
// Settings `section` primitive); the admin console already frames it in a panel.
globalStyle(`[data-settings-section] ${wrap}`, {
  padding: less1(vars.space[5]),
  borderRadius: vars.radius.lg,
  background: vars.color.surface,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
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

export const notice = style({
  fontSize: '0.82rem',
  color: vars.color.textDim,
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderRadius: vars.radius.md,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  background: vars.color.bgAlt,
  margin: 0,
});

export const list = style({
  listStyle: 'none',
  margin: 0,
  padding: 0,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
});

export const row = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  padding: vars.space[3],
  borderRadius: vars.radius.lg,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  background: vars.color.surface,
});

export const rowHeader = style({
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'center',
  gap: vars.space[3],
  flexWrap: 'wrap',
});

export const valueText = style({
  fontFamily: vars.font.mono,
  fontSize: '0.82rem',
  color: vars.color.text,
  whiteSpace: 'pre-wrap',
  wordBreak: 'break-word',
  margin: 0,
});

export const unset = style({ fontSize: '0.8rem', color: vars.color.textDim, fontStyle: 'italic' });

export const input = style({
  padding: `${less1(vars.space[2])} ${less1(vars.space[3])}`,
  borderRadius: vars.radius.md,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: vars.color.bgSink,
  color: vars.color.text,
  font: 'inherit',
  fontSize: '0.9rem',
  minHeight: vars.a11y.touchTarget,
  flex: 1,
  minWidth: '10rem',
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

export const editRow = style({
  display: 'flex',
  gap: vars.space[2],
  alignItems: 'center',
  flexWrap: 'wrap',
});

export const addForm = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: vars.space[4],
  borderRadius: vars.radius.lg,
  border: `${vars.border.hair} dashed ${vars.color.borderStrong}`,
  background: vars.color.bg,
});
