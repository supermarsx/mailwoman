// The search filter bar under the top-bar search input (t88): one row of chips
// that scrolls sideways at any width (never wraps, never widens the page), and
// an anchored editor popover.

import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

/** The input row keeps its height when the clear button comes and goes. */
export const form = style({ alignItems: 'center' });
export const filtersToggle = style({ display:'inline-flex', alignItems:'center', gap:'0.3rem', flex:'none', whiteSpace:'nowrap', selectors:{'&[aria-expanded="true"]':{background:vars.color.selection}} });
export const guided = style({ padding:'0.6rem', display:'grid', gap:'0.55rem', background:vars.color.bgAlt, borderBlockEnd:`${vars.border.hair} solid ${vars.color.border}` });
export const guidedHint = style({ margin:0, color:vars.color.textDim, fontSize:'0.8rem' });
export const quickFields = style({ display:'flex', flexWrap:'wrap', gap:'0.35rem' });
export const quickField = style({ display:'inline-flex', alignItems:'center', gap:'0.35rem', minBlockSize:'32px' });
export const quickFlags = style({ display:'flex', flexWrap:'wrap', gap:'0.5rem 1rem' });
export const trashToggle = style({ flex:'none', marginInlineStart:'auto' });

/** Opens by growing its one grid row from 0 to its content height; the
 *  duration is the motion token, which is ~0 under prefers-reduced-motion. */
export const reveal = style({
  display: 'grid',
  gridTemplateRows: '0fr',
  minWidth: 0,
  transition: `grid-template-rows ${vars.a11y.motionDuration} ease-out`,
  selectors: { '&[data-open="true"]': { gridTemplateRows: '1fr' } },
});

export const revealInner = style({ minHeight: 0, minWidth: 0, overflow: 'hidden' });

/** Fixed height, so editing a chip never moves the list below. */
export const bar = style({
  display: 'flex',
  alignItems: 'center',
  gap: '0.35rem',
  blockSize: '2.25rem',
  minWidth: 0,
  padding: '0 0.6rem',
  overflowX: 'auto',
  overflowY: 'hidden',
  overscrollBehaviorX: 'contain',
  scrollbarWidth: 'thin',
  whiteSpace: 'nowrap',
  borderBlockEnd: `${vars.border.hair} solid ${vars.color.border}`,
  background: vars.color.bgAlt,
  fontSize: '0.8rem',
});

/** One chip: the label button and its remove button, joined. */
export const chip = style({
  display: 'inline-flex',
  flex: 'none',
  alignItems: 'stretch',
  maxInlineSize: '18rem',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: '999px',
  background: vars.color.surface,
  overflow: 'hidden',
  selectors: { '&[data-negated="true"]': { borderStyle: 'dashed' } },
});

const chipPart = {
  display: 'inline-flex',
  alignItems: 'center',
  gap: '0.3rem',
  minBlockSize: '24px',
  border: 0,
  background: 'transparent',
  color: 'inherit',
  font: 'inherit',
  lineHeight: 1.2,
  cursor: 'pointer',
} as const;

export const chipLabel = style({
  ...chipPart,
  minWidth: 0,
  padding: '0.1rem 0.25rem 0.1rem 0.6rem',
  selectors: { '&:hover': { background: vars.color.selection } },
});

export const chipText = style({ overflow: 'hidden', textOverflow: 'ellipsis' });
export const chipField = style({ color: vars.color.textDim });

export const chipRemove = style({
  ...chipPart,
  justifyContent: 'center',
  minInlineSize: '24px',
  padding: '0 0.3rem',
  color: vars.color.textDim,
  selectors: { '&:hover': { background: vars.color.selection, color: vars.color.text } },
});
globalStyle(`${chipRemove} svg, ${chipLabel} svg`, { inlineSize: '0.85rem', blockSize: '0.85rem' });

/** "Add filter": a quiet chip-shaped button at the end of the row. */
export const add = style({
  ...chipPart,
  flex: 'none',
  padding: '0.1rem 0.6rem',
  border: `${vars.border.rule} dashed ${vars.color.borderStrong}`,
  borderRadius: '999px',
  color: vars.color.textDim,
  selectors: { '&:hover': { color: vars.color.text, background: vars.color.selection } },
});

export const popover = style({
  position: 'fixed',
  zIndex: 60,
  inlineSize: 'min(20rem, calc(100vw - 16px))',
  padding: '0.7rem',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  color: vars.color.text,
  boxShadow: vars.elevation[3],
  overflowY: 'auto',
});

export const popoverForm = style({ display: 'grid', gap: '0.55rem', margin: 0 });
export const popoverTitle = style({ margin: 0, fontSize: '0.85rem', fontWeight: 600 });
export const field = style({ display: 'grid', gap: '0.2rem', fontSize: '0.8rem' });
export const input = style({
  minWidth: 0,
  padding: '0.35rem 0.5rem',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  background: vars.color.bgSink,
  color: 'inherit',
  font: 'inherit',
});
export const check = style({ display: 'flex', alignItems: 'center', gap: '0.4rem', fontSize: '0.8rem' });
export const actions = style({ display: 'flex', justifyContent: 'flex-end', gap: '0.4rem', flexWrap: 'wrap' });
export const removeAction = style({ marginInlineEnd: 'auto' });
export const hint = style({ color: vars.color.textDim, fontSize: '0.75rem' });

/** The reader's "Search related" icon beside the sender line. */
export const readerSearch = style({ verticalAlign: 'middle', marginInlineStart: '0.2rem' });
