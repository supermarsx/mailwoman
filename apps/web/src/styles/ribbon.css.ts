// Ribbon layout-preset styles (plan §3 e4: optional collapsible Outlook-style
// ribbon; t38: groups are bars/RibbonGroup, three rows each). Vanilla-extract scoped classes built entirely on the token contract
// — this is the "new components use vars.* directly" path (vs the legacy
// bridge that keeps app.css working).
//
// t32: square tabs and buttons, the 2px control rule (1px back from each
// padding), the 3px region rule under the ribbon (2px back from the tab row's
// top padding), tab labels in small tracked capitals, and the active tab marked
// by a 3px accent bar and a surface field rather than by colour alone (every
// tab label is already bold, so the active one does not change width).

import { globalStyle, style } from '@vanilla-extract/css';
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
  minInlineSize: 0,
  userSelect:'none',
  WebkitUserSelect:'none',
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[2],
  paddingBlock: `calc(${vars.space[2]} - 2px) 0`,
  paddingInline: vars.space[2],
});

// The tab buttons in a row (was an inline style the shell CSP drops).
export const tablist = style({ display: 'flex', gap: 0, minInlineSize: 0, overflowX: 'auto', scrollbarWidth: 'thin' });

export const tab = style({
  flex: 'none',
  whiteSpace: 'nowrap',
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
  padding: `${vars.space[1]} ${vars.space[3]}`,
  borderRadius: vars.radius.sm,
});

export const tabActive = style({
  color: vars.color.text,
  background: vars.color.surface,
  boxShadow: `inset 0 -3px 0 ${vars.color.accent}`,
});

// The revealed panel (pinned: in the flow; auto-hide: over the content).
export const panel = style({ background: vars.color.surface });

// The active tab's groups. No vertical padding: the RibbonPanel inside has
// a fixed block size, and nothing here may add a fourth row or scroll.
export const body = style({
  minInlineSize: 0,
  paddingInline: vars.space[2],
  overflow: 'hidden',
});

export const btn = style({
  minInlineSize:0, maxInlineSize:'14rem', whiteSpace:'nowrap',
  selectors:{'&[aria-pressed="true"]':{background:vars.color.accent,color:vars.color.accentText,borderColor:vars.color.accent}},
});

// A `large` ribbon item: icon over the label, all three rows high (Office's
// big Paste button).
export const btnLarge = style({
  flexDirection: 'column',
  justifyContent: 'center',
  gap: vars.space[1],
  blockSize: '100%',
  minInlineSize: '3.5rem',
  maxInlineSize: '7rem',
  paddingBlock: 0,
});
globalStyle(`${btnLarge} svg`, { inlineSize: '20px', blockSize: '20px' });

// A `small` select fills its row and its item's explicit inline size.
export const select = style({ inlineSize: '100%', maxInlineSize: 'none' });

// A label clips with an ellipsis rather than growing the item (long locales).
export const label = style({
  display: 'block',
  // A line box taller than the glyphs (0.85rem text needs 18px), so the clip
  // never cuts a descender and the span never scrolls; it fits a 28px row.
  lineHeight: '20px',
  minInlineSize: 0,
  maxInlineSize: '100%',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
});

// A folded group's dropdown text: its 16px line clips the 0.75rem glyphs by
// 1px in Chromium (17 > 16), which reads as vertical overflow. A 18px line
// still fits the 88px button.
globalStyle(`${ribbon} [data-ribbon-more] > span`, { blockSize: '18px', lineHeight: '18px' });

export const search=style({minInlineSize:0,maxInlineSize:'64rem',paddingInline:vars.space[2],paddingBlock:vars.space[1]});
export const commands=style({display:'flex',alignItems:'center',flexWrap:'wrap',alignContent:'center',gap:vars.space[1],blockSize:'100%',minInlineSize:0});
globalStyle(`.${commands} > .workspace-bar`,{display:'contents'});

globalStyle(`.${search} .mail-search-area > div`,{maxBlockSize:'min(calc(40dvh / var(--mw-ui-zoom, 1)), 22rem)',overflow:'auto'});

globalStyle(`${ribbon} [data-ribbon-item=small] > button`,{inlineSize:"100%",justifyContent:"flex-start"});
globalStyle(`${ribbon} [data-ribbon-item] svg`,{flexShrink:0});
globalStyle(`${ribbon} [data-ribbon-item] button > span`,{minInlineSize:0,overflow:"hidden",textOverflow:"ellipsis"});

export const previewTitle=style({minInlineSize:0,overflow:"hidden",textOverflow:"ellipsis",whiteSpace:"nowrap",fontSize:".8rem",paddingInline:vars.space[2]});
