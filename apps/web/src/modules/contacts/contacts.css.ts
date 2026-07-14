// Contacts module styles (plan §3 e7). Token-native scoped classes reusing the
// V2 design-token contract so the module themes with the rest of the shell.

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const layout = style({
  display: 'grid',
  gridTemplateColumns: 'minmax(160px, 220px) minmax(220px, 340px) 1fr',
  gap: vars.space[5],
  height: '100%',
  padding: vars.space[5],
  color: vars.color.text,
  fontFamily: vars.font.ui,
});

export const sidebar = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
  borderInlineEnd: `1px solid ${vars.color.border}`,
  paddingInlineEnd: vars.space[4],
  minWidth: 0,
});

export const heading = style({
  fontSize: '0.72rem',
  fontWeight: 700,
  textTransform: 'uppercase',
  letterSpacing: '0.04em',
  color: vars.color.textDim,
  margin: `${vars.space[3]} 0 ${vars.space[1]}`,
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'center',
});

export const navButton = style({
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'center',
  gap: vars.space[2],
  padding: `${vars.space[2]} ${vars.space[3]}`,
  border: '1px solid transparent',
  borderRadius: vars.radius.md,
  background: 'transparent',
  color: vars.color.text,
  font: 'inherit',
  textAlign: 'start',
  minHeight: vars.a11y.touchTarget,
  cursor: 'pointer',
  width: '100%',
  selectors: {
    '&[aria-current="true"]': {
      background: vars.color.selection,
      borderColor: vars.color.border,
      fontWeight: 600,
    },
    '&:hover': { background: vars.color.bgAlt },
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  },
});

export const count = style({
  fontSize: '0.72rem',
  color: vars.color.textDim,
});

export const listPane = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  minWidth: 0,
  borderInlineEnd: `1px solid ${vars.color.border}`,
  paddingInlineEnd: vars.space[4],
});

export const toolbar = style({
  display: 'flex',
  gap: vars.space[2],
  flexWrap: 'wrap',
  alignItems: 'center',
});

export const input = style({
  flex: '1 1 auto',
  minWidth: 0,
  padding: `${vars.space[2]} ${vars.space[3]}`,
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  color: vars.color.text,
  font: 'inherit',
  minHeight: vars.a11y.touchTarget,
  selectors: { '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing } },
});

export const button = style({
  padding: `${vars.space[2]} ${vars.space[4]}`,
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: vars.color.accent,
  color: vars.color.accentText,
  font: 'inherit',
  minHeight: vars.a11y.touchTarget,
  cursor: 'pointer',
  selectors: { '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing } },
});

export const buttonGhost = style({
  padding: `${vars.space[2]} ${vars.space[3]}`,
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: 'transparent',
  color: vars.color.text,
  font: 'inherit',
  minHeight: vars.a11y.touchTarget,
  cursor: 'pointer',
  selectors: { '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing } },
});

export const contactList = style({
  listStyle: 'none',
  margin: 0,
  padding: 0,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
  overflowY: 'auto',
});

export const contactRow = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[2],
  padding: `${vars.space[2]} ${vars.space[3]}`,
  border: '1px solid transparent',
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  cursor: 'pointer',
  width: '100%',
  textAlign: 'start',
  font: 'inherit',
  color: vars.color.text,
  selectors: {
    '&[aria-current="true"]': { borderColor: vars.color.accent, background: vars.color.selection },
    '&:hover': { background: vars.color.bgAlt },
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  },
});

export const star = style({
  border: 'none',
  background: 'transparent',
  cursor: 'pointer',
  fontSize: '1rem',
  lineHeight: 1,
  padding: vars.space[1],
  minWidth: vars.a11y.touchTarget,
  minHeight: vars.a11y.touchTarget,
  color: vars.color.textDim,
  selectors: {
    '&[aria-pressed="true"]': { color: vars.color.warning },
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  },
});

export const rowBody = style({ flex: '1 1 auto', minWidth: 0, display: 'flex', flexDirection: 'column' });
export const rowName = style({ fontWeight: 600, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' });
export const rowMeta = style({ fontSize: '0.75rem', color: vars.color.textDim, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' });

export const detail = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  minWidth: 0,
  overflowY: 'auto',
});

export const card = style({
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.lg,
  background: vars.color.surface,
  padding: vars.space[5],
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
});

export const cardName = style({ fontSize: '1.4rem', fontWeight: 700, margin: 0 });
export const cardSub = style({ color: vars.color.textDim, margin: 0 });

export const fieldGroup = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2] });
export const fieldLabel = style({ fontSize: '0.72rem', fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.04em', color: vars.color.textDim });
export const fieldRow = style({ display: 'flex', gap: vars.space[2], alignItems: 'center', flexWrap: 'wrap' });

export const empty = style({ color: vars.color.textDim, fontStyle: 'italic', padding: vars.space[4] });

export const dialogBackdrop = style({
  position: 'fixed',
  inset: 0,
  background: 'rgba(0,0,0,0.45)',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  padding: vars.space[5],
  zIndex: 50,
});

export const dialog = style({
  background: vars.color.bg,
  color: vars.color.text,
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.lg,
  padding: vars.space[5],
  maxWidth: '52rem',
  width: '100%',
  maxHeight: '85vh',
  overflowY: 'auto',
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
});

export const table = style({
  borderCollapse: 'collapse',
  width: '100%',
  fontSize: '0.85rem',
});

export const th = style({
  textAlign: 'start',
  padding: vars.space[2],
  borderBottom: `1px solid ${vars.color.border}`,
});

export const td = style({
  padding: vars.space[2],
  borderBottom: `1px solid ${vars.color.border}`,
});

export const select = style({
  padding: `${vars.space[1]} ${vars.space[2]}`,
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.sm,
  background: vars.color.surface,
  color: vars.color.text,
  font: 'inherit',
  selectors: { '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing } },
});

export const actions = style({ display: 'flex', gap: vars.space[2], justifyContent: 'flex-end', flexWrap: 'wrap' });
export const chip = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  padding: `${vars.space[1]} ${vars.space[2]}`,
  borderRadius: vars.radius.pill,
  background: vars.color.bgAlt,
  fontSize: '0.75rem',
});
