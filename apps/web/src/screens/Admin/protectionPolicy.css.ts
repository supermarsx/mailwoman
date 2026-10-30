// Adaptive protection policy editor (t97). One column on a phone; rule rows wrap
// their controls instead of scrolling sideways.

import { style, globalStyle } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const stack = style({ display: 'flex', flexDirection: 'column', gap: vars.space[4], minInlineSize: 0 });
export const card = style({
  display: 'flex', flexDirection: 'column', gap: vars.space[3], minInlineSize: 0,
  padding: `calc(${vars.space[4]} - 1px)`,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
});
export const title = style({ margin: 0, fontSize: '1rem', fontWeight: vars.weight.bold });
export const note = style({ margin: 0, fontSize: '0.85rem', color: vars.color.textDim, lineHeight: 1.5 });
export const error = style({ margin: 0, fontSize: '0.85rem', color: vars.color.danger });
export const actions = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[2], alignItems: 'center' });

export const summary = style({ margin: 0, paddingInlineStart: vars.space[4], display: 'flex', flexDirection: 'column', gap: vars.space[2], lineHeight: 1.5 });

export const presets = style({ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(min(100%, 13rem), 1fr))', gap: vars.space[3] });
export const preset = style({
  display: 'grid', gridTemplateColumns: 'auto 1fr', columnGap: vars.space[2], rowGap: vars.space[1], alignItems: 'start',
  padding: vars.space[3], border: `${vars.border.rule} solid ${vars.color.border}`, borderRadius: vars.radius.md, cursor: 'pointer',
  selectors: {
    '&[data-selected="true"]': { borderColor: vars.color.accent, boxShadow: `inset 0 0 0 1px ${vars.color.accent}` },
    '&:has(:focus-visible)': { outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`, outlineOffset: '2px' },
  },
});
globalStyle(`${preset} > input`, { marginBlockStart: '0.2rem' });
globalStyle(`${preset} > span:last-child`, { gridColumn: '2' });
export const presetName = style({ fontWeight: vars.weight.bold });

export const diff = style({ inlineSize: '100%', borderCollapse: 'collapse', fontSize: '0.85rem', fontVariantNumeric: 'tabular-nums' });
globalStyle(`${diff} caption`, { textAlign: 'start', fontWeight: vars.weight.bold, paddingBlockEnd: vars.space[2] });
globalStyle(`${diff} th, ${diff} td`, { textAlign: 'start', padding: `${vars.space[1]} ${vars.space[2]}`, borderBlockEnd: `${vars.border.hair} solid ${vars.color.border}`, verticalAlign: 'top', overflowWrap: 'anywhere' });
globalStyle(`${diff} tr[data-fired="true"]`, { background: vars.color.bgAlt });

export const rules = style({ listStyle: 'none', margin: 0, padding: 0, display: 'flex', flexDirection: 'column' });
export const rule = style({
  display: 'flex', flexDirection: 'column', gap: vars.space[2], paddingBlock: vars.space[3],
  borderBlockStart: `${vars.border.hair} solid ${vars.color.border}`,
  selectors: { '&[data-changed="true"]': { boxShadow: `inset 3px 0 0 ${vars.color.accent}`, paddingInlineStart: vars.space[3] } },
});
export const ruleHead = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'baseline', gap: vars.space[2] });
export const ruleLabel = style({ fontWeight: vars.weight.bold });
export const key = style({ fontFamily: vars.font.mono, fontSize: '0.75rem', color: vars.color.textDim, overflowWrap: 'anywhere' });
export const help = style({ margin: 0, lineHeight: 1.5 });
export const example = style({ margin: 0, fontSize: '0.85rem', color: vars.color.textDim, lineHeight: 1.5 });
export const exampleTag = style({ fontWeight: vars.weight.bold, color: vars.color.text });
export const ruleControls = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: vars.space[3], minInlineSize: 0 });
globalStyle(`${ruleControls} textarea`, { inlineSize: 'min(100%, 28rem)' });
export const number = style({ display: 'inline-flex', alignItems: 'center', gap: vars.space[2], flexWrap: 'wrap' });
globalStyle(`${number} input`, { inlineSize: '9rem' });
export const signals = style({ display: 'flex', flexWrap: 'wrap', gap: `${vars.space[1]} ${vars.space[4]}`, border: 0, margin: 0, padding: 0, minInlineSize: 0 });
export const signal = style({ display: 'inline-flex', gap: vars.space[2], alignItems: 'center' });
export const srLegend = style({ position: 'absolute', inlineSize: '1px', blockSize: '1px', overflow: 'hidden', clip: 'rect(0 0 0 0)', whiteSpace: 'nowrap' });

export const simForm = style({ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(min(100%, 12rem), 1fr))', gap: vars.space[3], alignItems: 'end' });
globalStyle(`${simForm} input:not([type="checkbox"]), ${simForm} select`, { inlineSize: '100%' });
export const check = style({ display: 'inline-flex', gap: vars.space[2], alignItems: 'center' });
export const simResult = style({
  display: 'flex', flexDirection: 'column', gap: vars.space[2], padding: vars.space[3],
  border: `${vars.border.rule} solid ${vars.color.border}`, borderRadius: vars.radius.md,
  selectors: {
    '&[data-action="block"]': { boxShadow: `inset 4px 0 0 ${vars.color.danger}` },
    '&[data-action="throttle"]': { boxShadow: `inset 4px 0 0 ${vars.color.warning}` },
    '&[data-action="allow"]': { boxShadow: `inset 4px 0 0 ${vars.color.success}` },
  },
});
export const verdict = style({ margin: 0, fontWeight: vars.weight.bold, lineHeight: 1.5 });
