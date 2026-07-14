// Directory/GAL UI styles (plan §3 e7). Token-native — design tokens unchanged.

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const wrap = style({
  position: 'relative',
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  fontFamily: vars.font.ui,
  color: vars.color.text,
});

export const input = style({
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderRadius: vars.radius.md,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  font: 'inherit',
  fontSize: '0.9rem',
});

export const listbox = style({
  listStyle: 'none',
  margin: 0,
  padding: vars.space[1],
  display: 'flex',
  flexDirection: 'column',
  gap: '2px',
  borderRadius: vars.radius.md,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.surface,
  maxHeight: '18rem',
  overflowY: 'auto',
});

export const option = style({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  gap: vars.space[3],
  padding: `${vars.space[2]} ${vars.space[3]}`,
  minHeight: vars.a11y.touchTarget,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  transition: `background ${vars.a11y.motionDuration}`,
  selectors: {
    '&[aria-selected="true"]': { background: vars.color.bgAlt },
    '&:hover': { background: vars.color.bgAlt },
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  },
});

export const optName = style({ fontSize: '0.9rem', fontWeight: 600 });
export const optMail = style({ fontSize: '0.78rem', color: vars.color.textDim });

export const badge = style({
  fontSize: '0.68rem',
  fontWeight: 700,
  textTransform: 'uppercase',
  letterSpacing: '0.04em',
  padding: `2px ${vars.space[2]}`,
  borderRadius: vars.radius.pill,
  background: vars.color.bgSink,
  color: vars.color.textDim,
});

export const meta = style({ fontSize: '0.78rem', color: vars.color.textDim, margin: 0 });
export const error = style({ fontSize: '0.85rem', color: vars.color.danger, margin: 0 });

export const button = style({
  appearance: 'none',
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[1]} ${vars.space[3]}`,
  minHeight: vars.a11y.touchTarget,
  font: 'inherit',
  fontSize: '0.82rem',
  fontWeight: 600,
  transition: `background ${vars.a11y.motionDuration}`,
  selectors: {
    '&:disabled': { opacity: 0.5, cursor: 'not-allowed' },
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  },
});

// ── expand-group ("who is actually in this?") ─────────────────────────────────

export const expandPanel = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  padding: vars.space[4],
  borderRadius: vars.radius.lg,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.surface,
});

export const memberList = style({
  listStyle: 'none',
  margin: 0,
  padding: 0,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
});

export const member = style({
  display: 'flex',
  flexDirection: 'column',
  padding: `${vars.space[1]} ${vars.space[2]}`,
  borderRadius: vars.radius.md,
  selectors: { '&:nth-child(odd)': { background: vars.color.bgAlt } },
});

// ── per-contact security tab ───────────────────────────────────────────────────

export const secTab = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: vars.space[4],
  borderRadius: vars.radius.lg,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.surface,
});

export const secRow = style({
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'center',
  gap: vars.space[3],
  padding: vars.space[3],
  borderRadius: vars.radius.md,
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  flexWrap: 'wrap',
});

export const mono = style({ fontFamily: vars.font.mono, fontSize: '0.8rem', wordBreak: 'break-all' });

export const verified = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: '0.25rem',
  color: vars.color.success,
  fontWeight: 600,
  fontSize: '0.8rem',
});
export const unverified = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: '0.25rem',
  color: vars.color.warning,
  fontWeight: 600,
  fontSize: '0.8rem',
});
// Redundant (non-colour) status glyph so verified/expired isn't colour-only
// (WCAG 1.4.1); `aria-hidden` since the adjacent word carries the meaning.
export const statusIcon = style({ fontWeight: 700 });

export const photo = style({
  width: '48px',
  height: '48px',
  borderRadius: vars.radius.pill,
  objectFit: 'cover',
  border: `1px solid ${vars.color.border}`,
});

export const heading = style({ fontSize: '0.95rem', fontWeight: 600, margin: 0 });
export const subHeading = style({
  fontSize: '0.74rem',
  fontWeight: 600,
  color: vars.color.textDim,
  textTransform: 'uppercase',
  letterSpacing: '0.04em',
});
