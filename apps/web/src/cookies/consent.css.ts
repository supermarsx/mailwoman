// Styles of the cookie consent control. Tokens only (`vars.*`), so every theme and
// accent applies. Banner, card and modal share one panel; the layout and position
// classes place it. Forced-colors mode keeps borders and focus outlines.

import { style, styleVariants } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const backdrop = style({
  position: 'fixed',
  inset: 0,
  zIndex: 9990,
  background: vars.color.scrim,
  '@media': { '(forced-colors: active)': { background: 'transparent' } },
});

export const panel = style({
  position: 'fixed',
  zIndex: 9991,
  boxSizing: 'border-box',
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: vars.space[4],
  maxHeight: 'calc(100vh - 16px)',
  overflowY: 'auto',
  color: vars.color.text,
  background: vars.color.surface,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  boxShadow: vars.elevation[3],
  fontFamily: vars.font.ui,
  fontSize: '0.875rem',
  lineHeight: 1.5,
  selectors: { '&:focus': { outline: 'none' } },
  '@media': { '(forced-colors: active)': { border: '2px solid CanvasText', boxShadow: 'none' } },
});

export const layout = styleVariants({
  banner: { insetInline: vars.space[2], width: 'auto', maxWidth: '72rem', marginInline: 'auto' },
  card: { width: 'min(26rem, calc(100vw - 16px))' },
  modal: { width: 'min(34rem, calc(100vw - 16px))' },
});

export const place = styleVariants({
  bottom: { insetBlockEnd: vars.space[2] },
  top: { insetBlockStart: vars.space[2] },
  start: { insetBlockEnd: vars.space[2], insetInlineStart: vars.space[2] },
  end: { insetBlockEnd: vars.space[2], insetInlineEnd: vars.space[2] },
  center: { insetBlockStart: '50%', insetInlineStart: '50%', transform: 'translate(-50%, -50%)' },
});

/** A banner or card in a corner must not stretch across the page. */
export const cornerWidth = style({ insetInline: 'auto' });

export const title = style({ margin: 0, fontSize: '1rem', fontWeight: vars.weight.bold });
export const body = style({ margin: 0, color: vars.color.text });
export const links = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[3], margin: 0, padding: 0, listStyle: 'none' });
export const link = style({ color: vars.color.link });

export const actions = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[2], alignItems: 'center' });
/** Reject all and Accept all take the same width when the policy asks for equal prominence. */
export const equal = style({ flex: '1 1 8rem', minHeight: '2.5rem', justifyContent: 'center' });
export const quiet = style({ background: 'transparent', border: 'none', color: vars.color.link, textDecoration: 'underline', cursor: 'pointer', font: 'inherit', padding: vars.space[1] });

export const categories = style({ display: 'flex', flexDirection: 'column', gap: vars.space[3], margin: 0, padding: 0, listStyle: 'none' });
export const category = style({
  display: 'grid',
  gridTemplateColumns: 'auto 1fr',
  columnGap: vars.space[3],
  rowGap: vars.space[1],
  padding: vars.space[3],
  border: `${vars.border.hair} solid ${vars.color.border}`,
  borderRadius: vars.radius.sm,
  '@media': { '(forced-colors: active)': { borderColor: 'CanvasText' } },
});
export const categoryName = style({ fontWeight: vars.weight.bold });
export const categoryText = style({ gridColumn: '2', margin: 0, color: vars.color.textDim });
export const note = style({ margin: 0, color: vars.color.textDim, fontSize: '0.8125rem' });
export const detailsList = style({ margin: 0, paddingInlineStart: vars.space[4], color: vars.color.textDim, fontSize: '0.8125rem' });
export const mono = style({ fontFamily: vars.font.mono });

export const check = style({
  width: '1.125rem',
  height: '1.125rem',
  accentColor: vars.color.accent,
  selectors: { '&:focus-visible': { outline: `2px solid ${vars.color.focus}`, outlineOffset: '2px' } },
});

/** Buttons inherit the app's `.btn` focus ring; the quiet text button needs its own. */
export const quietFocus = style({
  selectors: { '&:focus-visible': { outline: `2px solid ${vars.color.focus}`, outlineOffset: '2px' } },
});
