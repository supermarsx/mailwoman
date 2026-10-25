import { globalStyle } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

// Low-specificity defaults cover native controls without overriding the layout,
// semantics or selected/danger states of specialised components.
const fields = ':where(input:not([type="checkbox"], [type="radio"], [type="range"], [type="hidden"], [type="submit"], [type="reset"], [type="button"]), textarea, select)';
globalStyle(fields, {
  font: 'inherit',
  color: vars.color.text,
  backgroundColor: vars.color.bgSink,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  paddingBlock: vars.space[2],
  paddingInline: vars.space[3],
  maxInlineSize: '100%',
  minInlineSize: 0,
  accentColor: vars.color.accent,
});
globalStyle(':where(textarea)', { resize: 'vertical' });
globalStyle(':where(select)', { cursor: 'pointer' });
globalStyle(':where(select[multiple], select[size])', { padding: vars.space[1] });
globalStyle(':where(select option, select optgroup)', {
  backgroundColor: vars.color.bgSink,
  color: vars.color.text,
  padding: vars.space[2],
});
globalStyle(':where(select option:checked)', {
  backgroundColor: vars.color.accent,
  color: vars.color.accentText,
  '@media': {
    '(forced-colors: active)': {
      backgroundColor: 'Highlight',
      color: 'HighlightText',
      forcedColorAdjust: 'none',
    },
  },
});
globalStyle(':where(button, input[type="button"], input[type="submit"], input[type="reset"]), input::file-selector-button', {
  font: 'inherit',
  fontWeight: vars.weight.bold,
  color: vars.color.text,
  backgroundColor: vars.color.surface,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  paddingBlock: vars.space[2],
  paddingInline: vars.space[3],
  cursor: 'pointer',
});
globalStyle('input::file-selector-button', { marginInlineEnd: vars.space[3] });
globalStyle(':where(button, input[type="button"], input[type="submit"], input[type="reset"]):hover:enabled', {
  boxShadow: vars.elevation[1],
});
globalStyle(':where(button, input[type="button"], input[type="submit"], input[type="reset"]):active:enabled', {
  boxShadow: 'none',
});
globalStyle(':root body :is(input:not([type="checkbox"], [type="radio"]), select, textarea, button):disabled', {
  opacity: 0.6,
  borderStyle: 'dashed',
  cursor: 'not-allowed',
});
globalStyle(':root body input[readonly], :root body textarea[readonly]', {
  backgroundColor: vars.color.bgAlt,
  borderStyle: 'dashed',
});
globalStyle(':where(input, textarea)::placeholder', { color: vars.color.textDim, opacity: 1 });
globalStyle(':root body :is(input, select, textarea)[aria-invalid="true"]', { borderColor: vars.color.danger });
globalStyle(':where(input[type="range"], progress, meter)', { accentColor: vars.color.accent });
globalStyle('input[type="range"]', {
  appearance: 'none',
  background: 'transparent',
  blockSize: vars.a11y.touchTarget,
  maxInlineSize: '100%',
  cursor: 'pointer',
});
// Separate vendor rules: an unsupported pseudo-element must not invalidate
// the other engine's selector. No gradients or rounded platform slider skin.
for (const track of ['::-webkit-slider-runnable-track', '::-moz-range-track']) {
  globalStyle(`input[type="range"]${track}`, {
    blockSize: '6px',
    backgroundColor: vars.color.bgSink,
    border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
    borderRadius: vars.radius.sm,
    '@media': { '(forced-colors: active)': { borderColor: 'CanvasText', backgroundColor: 'Canvas' } },
  });
}
for (const thumb of ['::-webkit-slider-thumb', '::-moz-range-thumb']) {
  globalStyle(`input[type="range"]${thumb}`, {
    appearance: 'none',
    inlineSize: vars.a11y.touchTarget,
    blockSize: vars.a11y.touchTarget,
    backgroundColor: vars.color.accent,
    border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
    borderRadius: vars.radius.sm,
    '@media': { '(forced-colors: active)': { backgroundColor: 'Highlight', borderColor: 'CanvasText', forcedColorAdjust: 'none' } },
  });
}
globalStyle('input[type="range"]::-webkit-slider-thumb', { marginBlockStart: `calc((6px - ${vars.border.rule} * 2 - ${vars.a11y.touchTarget}) / 2)` });
globalStyle('input[type="color"]', { minBlockSize: '2.5rem', inlineSize: '4rem', padding: vars.space[1] });
for (const swatch of ['::-webkit-color-swatch', '::-moz-color-swatch']) {
  globalStyle(`input[type="color"]${swatch}`, { border: `${vars.border.rule} solid ${vars.color.borderStrong}`, borderRadius: vars.radius.sm });
}
globalStyle(':where(summary)', { cursor: 'pointer', fontWeight: vars.weight.bold });
globalStyle(':where(dialog, [popover], [role="listbox"], [role="menu"])', {
  backgroundColor: vars.color.surface,
  color: vars.color.text,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  boxShadow: vars.elevation[2],
});
globalStyle(':where([role="option"][aria-selected="true"], [role="menuitemradio"][aria-checked="true"], [role="menuitemcheckbox"][aria-checked="true"])', {
  backgroundColor: vars.color.accent,
  color: vars.color.accentText,
  '@media': { '(forced-colors: active)': { backgroundColor: 'Highlight', color: 'HighlightText', forcedColorAdjust: 'none' } },
});
globalStyle(':where(button[role="switch"][aria-checked="true"], button[aria-pressed="true"])', {
  backgroundColor: vars.color.accent,
  color: vars.color.accentText,
  borderColor: vars.color.accent,
  '@media': { '(forced-colors: active)': { backgroundColor: 'Highlight', color: 'HighlightText', borderColor: 'Highlight', forcedColorAdjust: 'none' } },
});
