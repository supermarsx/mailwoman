// Layout of the feature profile editor (t98): grouped feature rows that stay one
// column on a phone and two (label | controls) from 720px.

import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

export const toolbar = style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'flex-end',
  gap: vars.space[3],
  minInlineSize: 0,
});

export const search = style({ flex: '1 1 16rem', minInlineSize: 0, maxInlineSize: '28rem' });

export const summary = style({ margin: 0, fontSize: '0.85rem', color: vars.color.textDim });

export const group = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  margin: 0,
  padding: vars.space[4],
  border: `${vars.border.hair} solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  minInlineSize: 0,
});

export const groupHead = style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'center',
  gap: vars.space[2],
  justifyContent: 'space-between',
});

export const groupTitle = style({ margin: 0, fontSize: '1rem', fontWeight: vars.weight.heavy });

export const groupActions = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[2] });

export const row = style({
  display: 'grid',
  gridTemplateColumns: 'minmax(0, 1fr)',
  gap: vars.space[2],
  paddingBlock: vars.space[3],
  borderBlockStart: `${vars.border.hair} solid ${vars.color.border}`,
  minInlineSize: 0,
  '@media': {
    '(min-width: 720px)': { gridTemplateColumns: 'minmax(0, 1fr) minmax(0, 20rem)', alignItems: 'start' },
  },
  selectors: { '&[data-differs="true"]': { boxShadow: `inset 3px 0 0 ${vars.color.accent}` } },
});
globalStyle(`[dir="rtl"] ${row}[data-differs="true"]`, { boxShadow: `inset -3px 0 0 ${vars.color.accent}` });

export const rowText = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1], minInlineSize: 0, paddingInlineStart: vars.space[2] });

export const rowName = style({ fontWeight: vars.weight.bold, overflowWrap: 'anywhere' });

export const rowHelp = style({ margin: 0, fontSize: '0.8rem', lineHeight: 1.45, color: vars.color.textDim });

export const controls = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2], minInlineSize: 0 });

export const controlLine = style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'center',
  gap: vars.space[2],
  minInlineSize: 0,
});
globalStyle(`${controlLine} > select, ${controlLine} > input:not([type="checkbox"])`, { flex: '1 1 9rem', minInlineSize: 0 });

export const keyLabel = style({ fontSize: '0.78rem', color: vars.color.textDim, flexBasis: '100%' });

export const other = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2] });

export const otherSummary = style({
  cursor: 'pointer',
  fontWeight: vars.weight.bold,
  paddingBlock: vars.space[2],
  selectors: { '&:focus-visible': { outline: `2px solid ${vars.color.accent}`, outlineOffset: '2px' } },
});

export const preview = style({ display: 'flex', flexDirection: 'column', gap: vars.space[3], minInlineSize: 0 });

export const off = style({ color: vars.color.danger, fontWeight: vars.weight.bold });
export const onText = style({ color: vars.color.success, fontWeight: vars.weight.bold });

export const hiddenInput = style({ display: 'none' });
