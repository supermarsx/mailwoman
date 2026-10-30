// Security centre Overview (t97): tiles come from shell.css.ts; this lays out the
// detail panels under them (one column on a phone, two or three on wide screens).

import { style, globalStyle } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const page = style({ display: 'flex', flexDirection: 'column', gap: vars.space[4], minInlineSize: 0 });
export const head = style({ display: 'flex', alignItems: 'center', flexWrap: 'wrap', gap: vars.space[3] });
export const lede = style({ margin: 0, flex: '1 1 20rem', color: vars.color.textDim, lineHeight: 1.5 });
export const updated = style({ margin: 0, fontSize: '0.8rem', color: vars.color.textDim, fontVariantNumeric: 'tabular-nums' });

export const panels = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fit, minmax(min(100%, 20rem), 1fr))',
  gap: vars.space[4],
  alignItems: 'start',
});
export const panel = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  minInlineSize: 0,
  padding: `calc(${vars.space[4]} - 1px)`,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
});
export const panelTitle = style({ margin: 0, fontSize: '0.95rem', fontWeight: vars.weight.bold });
export const subTitle = style({ margin: 0, fontSize: '0.8rem', fontWeight: vars.weight.bold, color: vars.color.textDim });
export const note = style({ margin: 0, fontSize: '0.8rem', color: vars.color.textDim, lineHeight: 1.5 });
export const warn = style([note, { display: 'flex', gap: vars.space[2], alignItems: 'flex-start', color: vars.color.text }]);
export const link = style({ color: vars.color.link, textDecoration: 'underline', fontSize: '0.85rem' });
export const linkRow = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[4] });
export const mono = style({ fontFamily: vars.font.mono, fontSize: '0.8rem', overflowWrap: 'anywhere' });
export const dim = style({ color: vars.color.textDim, fontSize: '0.8rem' });

const listBase = { listStyle: 'none', margin: 0, padding: 0, display: 'flex', flexDirection: 'column', gap: vars.space[2] } as const;
export const facts = style(listBase);
globalStyle(`${facts} > li`, { display: 'flex', justifyContent: 'space-between', gap: vars.space[3], fontVariantNumeric: 'tabular-nums' });
export const ranked = style(listBase);
globalStyle(`${ranked} > li`, { display: 'flex', alignItems: 'baseline', gap: vars.space[3], minInlineSize: 0, fontVariantNumeric: 'tabular-nums' });
globalStyle(`${ranked} > li > strong`, { marginInlineStart: 'auto' });
export const rows = style(listBase);
globalStyle(`${rows} > li`, { display: 'flex', flexWrap: 'wrap', alignItems: 'baseline', columnGap: vars.space[3], rowGap: vars.space[1], minInlineSize: 0 });

export const checks = style(listBase);
export const check = style({ display: 'flex', alignItems: 'center', gap: vars.space[2], minInlineSize: 0 });
const mark = { display: 'inline-flex', flex: 'none' } as const;
export const checkPass = style({ ...mark, color: vars.color.success });
export const checkWarn = style({ ...mark, color: vars.color.warning });
export const checkInfo = style({ ...mark, color: vars.color.textDim });
