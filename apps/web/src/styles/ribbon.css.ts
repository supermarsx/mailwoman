// Ribbon layout-preset styles (plan §3 e4: optional collapsible Outlook-style
// ribbon). Vanilla-extract scoped classes built entirely on the token contract
// — this is the "new components use vars.* directly" path (vs the legacy
// bridge that keeps app.css working).
//
// t32: square tabs and buttons, the 2px control rule (1px back from each
// padding), the 3px region rule under the ribbon (2px back from the tab row's
// top padding), tab labels in small tracked capitals, and the active tab marked
// by a 3px accent bar and a surface field rather than by colour alone (every
// tab label is already bold, so the active one does not change width).

import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const ribbon = style({
  display: 'flex',
  flexDirection: 'column',
  background: vars.color.bgAlt,
  borderBlockEnd: `${vars.border.heavy} solid ${vars.color.borderStrong}`,
  backgroundImage: vars.texture.grain,
  fontFamily: vars.font.ui,
});

export const tabs = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[2],
  paddingBlock: `calc(${vars.space[2]} - 2px) 0`,
  paddingInline: vars.space[4],
});

export const tab = style({
  appearance: 'none',
  border: 'none',
  background: 'transparent',
  color: vars.color.textDim,
  font: 'inherit',
  fontSize: '0.8rem',
  fontWeight: vars.weight.bold,
  letterSpacing: vars.tracking.label,
  textTransform: 'uppercase',
  cursor: 'pointer',
  padding: `${vars.space[2]} ${vars.space[4]}`,
  borderRadius: vars.radius.sm,
});

export const tabActive = style({
  color: vars.color.text,
  background: vars.color.surface,
  boxShadow: `inset 0 -3px 0 ${vars.color.accent}`,
});

export const collapseBtn = style({
  marginInlineStart: 'auto',
  appearance: 'none',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: 'transparent',
  color: vars.color.textDim,
  borderRadius: vars.radius.sm,
  cursor: 'pointer',
  padding: `calc(${vars.space[1]} - 1px) calc(${vars.space[3]} - 1px)`,
  font: 'inherit',
});

export const body = style({
  display: 'flex',
  alignItems: 'stretch',
  gap: vars.space[4],
  padding: vars.space[4],
  background: vars.color.surface,
  overflowX: 'auto',
});

export const group = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  paddingInlineEnd: vars.space[4],
  borderInlineEnd: `${vars.border.hair} solid ${vars.color.border}`,
  selectors: {
    '&:last-child': { borderInlineEnd: 'none' },
  },
});

export const groupRow = style({
  display: 'flex',
  gap: vars.space[2],
  flexWrap: 'wrap',
});

export const groupLabel = style({
  fontSize: '0.7rem',
  color: vars.color.textDim,
  fontWeight: vars.weight.bold,
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
});

export const btn = style({
  appearance: 'none',
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[2],
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `calc(${vars.space[2]} - 1px) calc(${vars.space[3]} - 1px)`,
  font: 'inherit',
  fontSize: '0.85rem',
  selectors: {
    '&:hover': { boxShadow: vars.elevation[1] },
    '&[aria-pressed="true"]': {
      background: vars.color.accent,
      color: vars.color.accentText,
      borderColor: vars.color.accent,
    },
  },
});
