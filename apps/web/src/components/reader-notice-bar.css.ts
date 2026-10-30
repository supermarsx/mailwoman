// Reader notice bars (t110; the remote-images banner of t16 §S8/S9, single line
// since t55, is the model). One family for every notice above the message body:
// mailing list, remote images, linked files and attachments. Each is one row of
// a fixed block size on the frozen token contract: the text ellipsises and the
// actions turn into icon buttons in a narrow container, so a bar never wraps and
// the body below never moves when a bar's text changes. Zero-runtime
// vanilla-extract.

import { style, globalStyle } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

const NARROW = 'mw-notice-bar (max-width: 34rem)';

/** The stack the bars live in, in a fixed order (Reader.tsx): one gap between
 *  bars, one margin around the stack, so two bars sit exactly as far apart as a
 *  bar sits from the header and the body. */
export const stack = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
  marginBlock: vars.space[1],
  minInlineSize: 0,
  selectors: { '&:empty': { display: 'none' } },
});

/** Flat notice: hair border; a warning notice adds a 3px rule on the leading edge. */
export const root = style({
  containerType: 'inline-size',
  containerName: 'mw-notice-bar',
  margin: 0,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  borderInlineStart: `${vars.border.heavy} solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  color: vars.color.text,
  fontFamily: vars.font.ui,
  fontSize: '0.8125rem',
  lineHeight: '1.25',
  minInlineSize: 0,
  selectors: {
    '&[data-tone="warning"]': { borderInlineStartColor: vars.color.warning },
  },
});

/** The single row; its block size is fixed so state changes never resize it. */
export const row = style({
  display: 'flex',
  flexWrap: 'nowrap',
  alignItems: 'center',
  gap: vars.space[2],
  blockSize: '2rem',
  paddingInline: `calc(${vars.space[2]} - 2px) ${vars.space[1]}`,
  minInlineSize: 0,
  overflow: 'hidden',
});

export const mark = style({
  flex: '0 0 auto',
  display: 'inline-flex',
  color: vars.color.textDim,
  selectors: { [`${root}[data-tone="warning"] &`]: { color: vars.color.warning } },
});

/** Headline + detail; the part that gives way. */
export const summary = style({
  flex: '1 1 auto',
  minInlineSize: 0,
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
  outlineOffset: '1px',
});

export const headline = style({ fontWeight: vars.weight.bold });

/** Secondary text after the headline (" · 3 trackers removed"). */
export const detail = style({ color: vars.color.textDim });

export const actions = style({
  flex: '0 0 auto',
  display: 'inline-flex',
  flexWrap: 'nowrap',
  alignItems: 'center',
  gap: vars.space[1],
  marginInlineStart: 'auto',
});

export const btn = style({
  appearance: 'none',
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.sm,
  cursor: 'pointer',
  paddingBlock: 0,
  paddingInline: vars.space[2],
  font: 'inherit',
  fontSize: '0.8125rem',
  whiteSpace: 'nowrap',
  textDecoration: 'none',
  blockSize: '1.5rem',
  minInlineSize: vars.a11y.touchTarget,
  minBlockSize: vars.a11y.touchTarget,
  justifyContent: 'center',
  selectors: {
    '&:hover:not(:disabled)': { background: vars.color.bgAlt, borderColor: vars.color.accent },
    '&:disabled': { opacity: 0.5, cursor: 'default' },
  },
  '@container': { [NARROW]: { paddingInline: 0, inlineSize: '1.75rem' } },
});

export const btnPrimary = style({ borderColor: vars.color.accent });

export const iconBtn = style({ paddingInline: 0, inlineSize: '1.5rem' });

/** The visible action label; visually hidden (still the accessible name and the
 *  tooltip text) in a narrow container. */
export const label = style({
  '@container': {
    [NARROW]: {
      position: 'absolute',
      width: '1px',
      height: '1px',
      overflow: 'hidden',
      clip: 'rect(0 0 0 0)',
      whiteSpace: 'nowrap',
    },
  },
});

/** The menu chevron points down. */
export const chevron = style({ display: 'inline-flex', transform: 'rotate(90deg)' });

/** The expandable region of a collapsible bar (attachments): below the row, inside
 *  the same frame. No height animation, so nothing slides under reduced motion
 *  or otherwise; the content below moves once, by the region's height. */
export const panel = style({
  borderBlockStart: `${vars.border.hair} solid ${vars.color.border}`,
  padding: vars.space[2],
  fontSize: '0.875rem',
  minInlineSize: 0,
  selectors: { '&[hidden]': { display: 'none' } },
});

globalStyle(`${btn} svg, ${mark} svg`, { inlineSize: '14px', blockSize: '14px', flex: '0 0 auto' });
