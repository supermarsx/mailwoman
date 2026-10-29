// The storage meter under the folder list and its Settings detail (t39-e7).
import { globalStyle, style, styleVariants } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const meter = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
  padding: `${vars.space[2]} ${vars.space[3]}`,
  fontSize: '0.8125rem',
  color: vars.color.textDim,
  minWidth: 0,
});

export const label = style({
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
});

export const track = style({
  position: 'relative',
  height: '6px',
  borderRadius: vars.radius.pill,
  background: vars.color.bgSink,
  border: `1px solid ${vars.color.border}`,
  overflow: 'hidden',
});

const fillBase = style({
  position: 'absolute',
  insetBlock: 0,
  insetInlineStart: 0,
  borderRadius: vars.radius.pill,
});

export const fill = styleVariants({
  ok: [fillBase, { background: vars.color.accent }],
  warn: [fillBase, { background: vars.color.warning }],
  critical: [fillBase, { background: vars.color.danger }],
});

export const warning = style({
  color: vars.color.text,
  fontWeight: vars.weight.bold,
});

export const table = style({
  width: '100%',
  borderCollapse: 'collapse',
});

globalStyle(`${table} th, ${table} td`, {
  textAlign: 'start',
  padding: vars.space[2],
  borderBottom: `1px solid ${vars.color.border}`,
  verticalAlign: 'top',
});

export const detailTrack = style([track, { marginBlockStart: vars.space[1], maxWidth: '16rem' }]);
