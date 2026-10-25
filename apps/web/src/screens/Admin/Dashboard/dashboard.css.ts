// Dashboard layout (t32-admin.md §2.2): panels in one column, tiles in a strict
// grid (shell.css.ts), the three traffic charts side by side on wide screens.
// Same t32 rules as admin.css.ts.

import { style, styleVariants, globalStyle } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

export const page = style({ display: 'flex', flexDirection: 'column', gap: vars.space[5], maxWidth: '1100px' });

export const head = style({ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: vars.space[3], flexWrap: 'wrap' });

export const panel = style({ display: 'flex', flexDirection: 'column', gap: vars.space[3], minWidth: 0 });

/** The blocked-accounts panel: a danger inset bar plus its heading text (P6). */
export const alertPanel = style({
  padding: `calc(${vars.space[4]} - 1px)`,
  paddingInlineStart: `calc(${vars.space[4]} + 3px)`,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: vars.color.surface,
  boxShadow: `inset 4px 0 0 ${vars.color.danger}`,
  selectors: { '[dir="rtl"] &': { boxShadow: `inset -4px 0 0 ${vars.color.danger}` } },
});

export const panelTitle = style({
  margin: 0,
  fontSize: '0.75rem',
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
  fontWeight: vars.weight.bold,
  color: vars.color.textDim,
  selectors: {
    ':lang(ar) &, :lang(ja) &, :lang(zh) &': { textTransform: 'none', letterSpacing: vars.tracking.none },
  },
});

export const list = style({ listStyle: 'none', margin: 0, padding: 0, display: 'flex', flexDirection: 'column' });

const row = {
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[3],
  flexWrap: 'wrap',
  padding: `${vars.space[2]} 0`,
  borderBottom: `${vars.border.hair} solid ${vars.color.border}`,
} as const;

export const warning = style(row);
globalStyle(`${warning} > svg`, { color: vars.color.warning, flex: 'none' });

export const link = style({ color: vars.color.link, textDecoration: 'underline' });
globalStyle(`${warning} a, ${list} a:not(.btn)`, { color: vars.color.link, textDecoration: 'underline' });

export const blockedRow = style({ ...row, justifyContent: 'space-between' });

export const checkRow = style(row);
export const checkLabel = style({ flex: 1, minWidth: '12rem' });
const status = { display: 'inline-flex', alignItems: 'center', gap: vars.space[1], fontWeight: vars.weight.bold, minWidth: '5.5rem' } as const;
export const checkDone = style({ ...status, color: vars.color.success });
export const checkOpen = style({ ...status, color: vars.color.warning });

export const charts = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
  gap: vars.space[5],
});

export const facts = style({
  display: 'grid',
  gridTemplateColumns: 'minmax(0, 14rem) minmax(0, 1fr)',
  gap: `${vars.space[2]} ${vars.space[4]}`,
  margin: 0,
  fontSize: '0.85rem',
  '@media': { 'screen and (max-width: 760px)': { gridTemplateColumns: 'minmax(0, 1fr)' } },
});
globalStyle(`${facts} dt`, { color: vars.color.textDim });
globalStyle(`${facts} dd`, { margin: 0, overflowWrap: 'anywhere' });

export const healthValue = style({ display: 'flex', gap: vars.space[2], alignItems: 'flex-start' });
export const healthOk = style({ color: vars.color.success, display: 'inline-flex', flex: 'none', marginTop: '1px' });
export const healthBad = style({ color: vars.color.warning, display: 'inline-flex', flex: 'none', marginTop: '1px' });

export const widgets = style({ display: 'flex', flexDirection: 'column', gap: vars.space[5], minInlineSize: 0 });

export const widgetOrder = styleVariants({ 0: { order: 1 }, 1: { order: 2 }, 2: { order: 3 }, 3: { order: 4 }, 4: { order: 5 }, 5: { order: 6 }, 6: { order: 7 }, 7: { order: 8 } });
