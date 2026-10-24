// Third-party list (t32-e75, about/ThirdPartyList.tsx).
//
// Tokens only (t32.md §2.4): square, hard edges, state never by colour alone,
// logical properties; focus uses the global `:focus-visible` outline (themes.css.ts).
// Entries are cards in one column, so the list needs no
// horizontal scroll at phone width; a long licence line wraps inside its own
// scroll box.

import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const root = style({ display: 'flex', flexDirection: 'column', gap: vars.space[4], minInlineSize: 0 });

export const toolbar = style({ display: 'flex', flexDirection: 'column', gap: vars.space[3] });

export const search = style({ maxInlineSize: '480px' });

export const summary = style({ margin: 0, fontSize: '0.85rem', color: vars.color.textDim });

export const list = style({
  listStyle: 'none',
  margin: 0,
  padding: 0,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
});

export const item = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
  padding: vars.space[3],
  border: `${vars.border.hair} solid ${vars.color.border}`,
  background: vars.color.surface,
  minInlineSize: 0,
});

export const head = style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'baseline',
  gap: vars.space[2],
  margin: 0,
  fontSize: '0.95rem',
  fontWeight: vars.weight.bold,
  overflowWrap: 'anywhere',
});

export const version = style({ fontWeight: vars.weight.regular, color: vars.color.textDim, fontFamily: vars.font.mono, fontSize: '0.85rem' });

export const kind = style({
  fontSize: '0.7rem',
  fontWeight: vars.weight.bold,
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
  padding: `0 ${vars.space[1]}`,
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  color: vars.color.textDim,
  selectors: { ':lang(ar) &, :lang(ja) &, :lang(zh) &': { textTransform: 'none', letterSpacing: vars.tracking.none } },
});

export const meta = style({ margin: 0, fontSize: '0.85rem', color: vars.color.textDim, overflowWrap: 'anywhere' });

export const licence = style({ color: vars.color.text, fontFamily: vars.font.mono });

export const links = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[3], margin: 0, padding: 0, listStyle: 'none' });

export const link = style({
  display: 'inline-flex',
  alignItems: 'center',
  minHeight: vars.a11y.touchTarget,
  color: vars.color.link,
  fontSize: '0.85rem',
  textDecoration: 'underline',
});

export const details = style({ fontSize: '0.85rem' });

export const detailsSummary = style({
  cursor: 'pointer',
  // stays `list-item` so the browser keeps the open/closed marker
  minHeight: vars.a11y.touchTarget,
  paddingBlock: vars.space[1],
  color: vars.color.text,
});

export const textBlock = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1], marginBlockStart: vars.space[2] });

export const textTitle = style({ margin: 0, fontSize: '0.8rem', fontWeight: vars.weight.bold });

export const textNote = style({ fontWeight: vars.weight.regular, color: vars.color.textDim });

export const pre = style({
  margin: 0,
  padding: vars.space[2],
  maxBlockSize: '320px',
  overflow: 'auto',
  whiteSpace: 'pre-wrap',
  overflowWrap: 'anywhere',
  fontFamily: vars.font.mono,
  fontSize: '0.75rem',
  background: vars.color.bgSink,
  color: vars.color.text,
  border: `${vars.border.hair} solid ${vars.color.border}`,
});

export const more = style({ alignSelf: 'flex-start' });
