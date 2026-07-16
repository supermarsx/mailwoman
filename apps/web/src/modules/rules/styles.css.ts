// Rules (mail-filter) UI styles (audit #1). Token-native — design tokens unchanged.

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const panel = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  fontFamily: vars.font.ui,
  color: vars.color.text,
});

export const heading = style({ fontSize: '1rem', fontWeight: 600, margin: 0 });

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

export const tabs = style({ display: 'flex', gap: vars.space[2], borderBottom: `1px solid ${vars.color.border}` });

export const tab = style({
  appearance: 'none',
  background: 'transparent',
  border: 'none',
  borderBottom: '2px solid transparent',
  color: vars.color.textDim,
  padding: `${vars.space[2]} ${vars.space[3]}`,
  minHeight: vars.a11y.touchTarget,
  cursor: 'pointer',
  font: 'inherit',
  selectors: {
    '&[aria-selected="true"]': { color: vars.color.text, borderBottomColor: vars.color.accent },
  },
});

export const list = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2], listStyle: 'none', margin: 0, padding: 0 });

export const ruleRow = style({
  display: 'flex',
  gap: vars.space[3],
  alignItems: 'center',
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.surface,
  flexWrap: 'wrap',
});

export const ruleName = style({ fontWeight: 600, flex: 1, minWidth: '8rem' });

export const badge = style({
  fontSize: '0.72rem',
  fontWeight: 600,
  textTransform: 'uppercase',
  letterSpacing: '0.03em',
  padding: `2px ${vars.space[2]}`,
  borderRadius: vars.radius.pill,
  border: `1px solid ${vars.color.border}`,
  color: vars.color.textDim,
  whiteSpace: 'nowrap',
});

export const badgeServer = style({ color: vars.color.accent, borderColor: vars.color.accent });

export const builder = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: vars.space[4],
  borderRadius: vars.radius.lg,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bgAlt,
});

export const row = style({ display: 'flex', gap: vars.space[2], alignItems: 'center', flexWrap: 'wrap' });

export const field = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1] });

export const label = style({ fontSize: '0.8rem', fontWeight: 600, color: vars.color.textDim });

export const input = style({
  font: 'inherit',
  color: vars.color.text,
  background: vars.color.surface,
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.sm,
  padding: `${vars.space[2]} ${vars.space[2]}`,
  minHeight: vars.a11y.touchTarget,
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

export const btn = style({
  appearance: 'none',
  font: 'inherit',
  cursor: 'pointer',
  minHeight: vars.a11y.touchTarget,
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderRadius: vars.radius.sm,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.surface,
  color: vars.color.text,
});

export const btnPrimary = style([btn, { background: vars.color.accent, color: vars.color.accentText, borderColor: vars.color.accent }]);

export const btnDanger = style([btn, { color: vars.color.danger, borderColor: vars.color.danger }]);

export const iconBtn = style([btn, { minWidth: vars.a11y.touchTarget, padding: vars.space[1] }]);

// ── raw editor ────────────────────────────────────────────────────────────

export const editorWrap = style({ position: 'relative', display: 'flex', flexDirection: 'column', gap: vars.space[2] });

export const textarea = style({
  font: vars.font.mono,
  fontSize: '0.85rem',
  lineHeight: 1.5,
  color: vars.color.text,
  background: vars.color.bgSink,
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  padding: vars.space[3],
  minHeight: '10rem',
  width: '100%',
  resize: 'vertical',
  whiteSpace: 'pre',
  overflowX: 'auto',
});

export const highlight = style({
  font: vars.font.mono,
  fontSize: '0.85rem',
  lineHeight: 1.5,
  margin: 0,
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  border: `1px solid ${vars.color.border}`,
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
  border: `1px solid ${vars.color.danger}`,
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
  border: `1px solid ${vars.color.border}`,
  flexWrap: 'wrap',
});

export const matchDot = style({
  width: '0.6rem',
  height: '0.6rem',
  borderRadius: vars.radius.pill,
  flexShrink: 0,
});

export const matchYes = style([matchDot, { background: vars.color.success }]);
export const matchNo = style([matchDot, { background: vars.color.border }]);
