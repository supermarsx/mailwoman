// V7 Assist UI styles (plan §3 e6). Token-native — the frozen design tokens
// (theme/contract.css.ts) are UNCHANGED; this file only references `vars.*`.

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

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
  padding: vars.space[4],
  borderRadius: vars.radius.lg,
  background: vars.color.surface,
  border: `1px solid ${vars.color.border}`,
});

export const heading = style({ fontSize: '1rem', fontWeight: 600, margin: 0 });
export const subHeading = style({
  fontSize: '0.75rem',
  fontWeight: 600,
  color: vars.color.textDim,
  textTransform: 'uppercase',
  letterSpacing: '0.04em',
});
export const prose = style({ fontSize: '0.9rem', lineHeight: 1.5, margin: 0 });
export const meta = style({ fontSize: '0.78rem', color: vars.color.textDim });

export const row = style({ display: 'flex', gap: vars.space[3], alignItems: 'center', flexWrap: 'wrap' });
export const field = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2] });
export const toolbar = style({ display: 'flex', gap: vars.space[2], flexWrap: 'wrap' });

export const input = style({
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderRadius: vars.radius.md,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  font: 'inherit',
  fontSize: '0.9rem',
  minWidth: 0,
});

export const button = style({
  appearance: 'none',
  border: `1px solid ${vars.color.accent}`,
  background: vars.color.accent,
  color: vars.color.accentText,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[2]} ${vars.space[4]}`,
  font: 'inherit',
  fontSize: '0.9rem',
  fontWeight: 600,
  selectors: { '&:disabled': { opacity: 0.5, cursor: 'not-allowed' } },
});

export const ghost = style({
  appearance: 'none',
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[2]} ${vars.space[3]}`,
  font: 'inherit',
  fontSize: '0.85rem',
  selectors: { '&:disabled': { opacity: 0.5, cursor: 'not-allowed' } },
});

// The push-to-talk dictation trigger; `active` is toggled while recording.
export const mic = style({
  appearance: 'none',
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.pill,
  cursor: 'pointer',
  padding: `${vars.space[2]} ${vars.space[4]}`,
  font: 'inherit',
  fontSize: '0.85rem',
  fontWeight: 600,
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
  border: `1px solid ${vars.color.border}`,
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
  border: `1px solid ${vars.color.border}`,
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
  border: `1px solid ${vars.color.border}`,
  borderLeft: `3px solid ${vars.color.warning}`,
  background: vars.color.bgAlt,
});

// Suggestion output (composer transform / summary) the user chooses to apply.
export const suggestion = style({
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  border: `1px dashed ${vars.color.border}`,
  fontSize: '0.9rem',
  lineHeight: 1.5,
  whiteSpace: 'pre-wrap',
});

// Auto-tag suggestion chip.
export const badge = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[2],
  padding: `${vars.space[1]} ${vars.space[3]}`,
  borderRadius: vars.radius.pill,
  border: `1px solid ${vars.color.border}`,
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
  borderLeft: `3px solid ${vars.color.accent}`,
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
