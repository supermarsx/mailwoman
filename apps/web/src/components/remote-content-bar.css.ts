// Remote-images banner styles (t16 §S8/S9; single line since t55), on the frozen
// token contract so it themes with the rest of the reader chrome. One row of a
// fixed block size: the text ellipsises and the actions turn into icon buttons
// in a narrow container, so the banner never wraps and the body below it never
// moves when its text changes. Zero-runtime vanilla-extract.

import { style, globalStyle } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

const NARROW = 'mw-remote-bar (max-width: 34rem)';

/** Flat notice: hair border, 3px warning rule on the leading edge. */
export const root = style({
  containerType: 'inline-size',
  containerName: 'mw-remote-bar',
  marginBlock: vars.space[1],
  marginInline: 0,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  borderInlineStart: `${vars.border.heavy} solid ${vars.color.warning}`,
  borderRadius: vars.radius.md,
  background: vars.color.bgAlt,
  color: vars.color.text,
  fontFamily: vars.font.ui,
  fontSize: '0.8125rem',
  lineHeight: '1.25',
  selectors: {
    '&[data-state="shown"], &[data-state="proxy"], &[data-state="direct"]': { borderInlineStartColor: vars.color.border },
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
  color: vars.color.warning,
  selectors: { [`${root}[data-state="shown"] &, ${root}[data-state="proxy"] &, ${root}[data-state="direct"] &`]: { color: vars.color.textDim } },
});

/** Headline + tracker count; the part that gives way. */
export const summary = style({
  flex: '1 1 auto',
  minInlineSize: 0,
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
  outlineOffset: '1px',
});

export const headline = style({ fontWeight: vars.weight.bold });

export const trackers = style({ color: vars.color.textDim });

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

globalStyle(`${btn} svg, ${mark} svg`, { inlineSize: '14px', blockSize: '14px', flex: '0 0 auto' });
