import { style, globalStyle } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

/** The custom-tag warning: an icon and text, with a warning rule, never colour alone. */
export const warning = style({
  display: 'flex',
  alignItems: 'flex-start',
  gap: vars.space[3],
  margin: 0,
  paddingInlineStart: vars.space[3],
  borderInlineStart: `${vars.border.heavy} solid ${vars.color.warning}`,
  lineHeight: 1.5,
  maxInlineSize: '72ch',
});
globalStyle(`${warning} > svg`, { flexShrink: 0, marginBlockStart: '0.15em', color: vars.color.warning });

/** Buttons keep their own width and an 8px gap (t47). */
export const actions = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: vars.space[3] });
globalStyle(`${actions} > .btn`, { inlineSize: 'auto' });

export const directives = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2], margin: 0, padding: 0, listStyle: 'none' });
export const directive = style({ fontFamily: vars.font.mono, fontSize: '0.8rem', overflowWrap: 'anywhere' });
export const directiveName = style({ fontWeight: vars.weight.bold });

export const results = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2], margin: 0, padding: 0, listStyle: 'none' });
export const result = style({ display: 'flex', alignItems: 'flex-start', gap: vars.space[2], overflowWrap: 'anywhere' });
globalStyle(`${result} > svg`, { flexShrink: 0, marginBlockStart: '0.15em' });

export const scope = style({ margin: 0, paddingInlineStart: vars.space[5], display: 'flex', flexDirection: 'column', gap: vars.space[1], lineHeight: 1.5 });

export const snippet = style({ fontFamily: vars.font.mono, fontSize: '0.8rem', minBlockSize: '10rem', resize: 'vertical' });
export const origins = style({ fontFamily: vars.font.mono, fontSize: '0.8rem', minBlockSize: '4.5rem', resize: 'vertical' });
