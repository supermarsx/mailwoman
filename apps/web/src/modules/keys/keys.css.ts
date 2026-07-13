// Key-management module styles (plan §3 e2). Token-native scoped classes reusing
// the V2 design-token contract so the module themes with the rest of the shell
// (same posture as the V3 contacts module styles).

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

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
  borderRight: `1px solid ${vars.color.border}`,
  paddingRight: vars.space[4],
  overflowY: 'auto',
});

export const head = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1] });
export const title = style({ fontSize: '1.1rem', fontWeight: 700, margin: 0 });
export const subtitle = style({ fontSize: '0.8rem', color: vars.color.textDim, margin: 0 });

export const toolbar = style({ display: 'flex', gap: vars.space[2], flexWrap: 'wrap', alignItems: 'center' });

export const heading = style({
  fontSize: '0.72rem',
  fontWeight: 700,
  textTransform: 'uppercase',
  letterSpacing: '0.04em',
  color: vars.color.textDim,
  margin: `${vars.space[3]} 0 ${vars.space[1]}`,
});

export const button = style({
  padding: `${vars.space[2]} ${vars.space[4]}`,
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: vars.color.accent,
  color: vars.color.accentText,
  font: 'inherit',
  cursor: 'pointer',
});

export const buttonGhost = style({
  padding: `${vars.space[2]} ${vars.space[3]}`,
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: 'transparent',
  color: vars.color.text,
  font: 'inherit',
  cursor: 'pointer',
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
  border: '1px solid transparent',
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  cursor: 'pointer',
  width: '100%',
  textAlign: 'left',
  font: 'inherit',
  color: vars.color.text,
  selectors: {
    '&[aria-current="true"]': { borderColor: vars.color.accent, background: vars.color.selection },
    '&:hover': { background: vars.color.bgAlt },
  },
});

export const rowBody = style({ flex: '1 1 auto', minWidth: 0, display: 'flex', flexDirection: 'column' });
export const rowName = style({ fontWeight: 600, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' });
export const rowMeta = style({ fontSize: '0.72rem', color: vars.color.textDim, fontFamily: vars.font.mono });

export const badge = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  padding: `${vars.space[1]} ${vars.space[2]}`,
  borderRadius: vars.radius.pill,
  fontSize: '0.68rem',
  fontWeight: 600,
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
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.lg,
  background: vars.color.surface,
  padding: vars.space[5],
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
});

export const cardName = style({ fontSize: '1.3rem', fontWeight: 700, margin: 0 });
export const cardSub = style({ color: vars.color.textDim, margin: 0, fontSize: '0.85rem' });

export const fieldGroup = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2] });
export const fieldLabel = style({
  fontSize: '0.72rem',
  fontWeight: 700,
  textTransform: 'uppercase',
  letterSpacing: '0.04em',
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

export const qr = style({
  width: '160px',
  height: '160px',
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: '#ffffff',
  padding: vars.space[2],
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
});

export const textarea = style({
  width: '100%',
  minHeight: '7rem',
  padding: vars.space[3],
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  color: vars.color.text,
  fontFamily: vars.font.mono,
  fontSize: '0.8rem',
  resize: 'vertical',
});

export const select = style({
  padding: `${vars.space[2]} ${vars.space[3]}`,
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  color: vars.color.text,
  font: 'inherit',
});

export const empty = style({ color: vars.color.textDim, fontStyle: 'italic', padding: vars.space[4] });

export const consent = style({
  display: 'flex',
  gap: vars.space[2],
  alignItems: 'flex-start',
  padding: vars.space[3],
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  fontSize: '0.8rem',
});

export const actions = style({ display: 'flex', gap: vars.space[2], flexWrap: 'wrap' });

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
  maxWidth: '40rem',
  width: '100%',
  maxHeight: '85vh',
  overflowY: 'auto',
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
});

export const fieldStack = style({ display: 'flex', flexDirection: 'column', gap: vars.space[3] });
export const label = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1], fontSize: '0.8rem' });
export const preview = style({
  padding: vars.space[3],
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
});
