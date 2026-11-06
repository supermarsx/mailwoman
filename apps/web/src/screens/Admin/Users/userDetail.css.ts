import { style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

/** The header (name, address, status) stays in view above the tabs. */
export const header = style({
  position: 'sticky', insetBlockStart: 0, zIndex: 2, display: 'flex', flexDirection: 'column', gap: vars.space[2],
  paddingBlock: vars.space[2], background: vars.color.bg,
});
export const headline = style({ display: 'flex', alignItems: 'center', flexWrap: 'wrap', gap: vars.space[3], minInlineSize: 0 });
export const avatar = style({
  display: 'inline-flex', alignItems: 'center', justifyContent: 'center', inlineSize: '2.5rem', blockSize: '2.5rem', flex: '0 0 auto',
  borderRadius: vars.radius.pill, background: vars.color.accent, color: vars.color.accentText, fontWeight: vars.weight.bold,
  border: '1px solid transparent',
  '@media': { '(forced-colors: active)': { borderColor: 'CanvasText' } },
});
export const who = style({ minInlineSize: 0, flex: '1 1 12rem', overflowWrap: 'anywhere' });
export const status = style({
  display: 'inline-flex', alignItems: 'center', paddingInline: vars.space[2], border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.pill, fontSize: '0.85em', fontWeight: vars.weight.bold,
});
export const panel = style({ display: 'flex', flexDirection: 'column', gap: vars.space[4], minInlineSize: 0, selectors: { '&[hidden]': { display: 'none' } } });
export const facts = style({ display: 'grid', gridTemplateColumns: 'max-content 1fr', columnGap: vars.space[4], rowGap: vars.space[1], margin: 0, minInlineSize: 0 });
export const list = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2], margin: 0, padding: 0, listStyle: 'none' });
