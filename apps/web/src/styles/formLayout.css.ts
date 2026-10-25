import { globalStyle } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

/** Shared form geometry only; control appearance belongs to the theme. */
export const fieldLayout = {
  display: 'flex', flexDirection: 'column', gap: vars.space[2],
  minInlineSize: 0, maxInlineSize: '32rem',
} as const;
export const fieldsLayout = {
  display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(min(100%, 16rem), 1fr))',
  gap: vars.space[3], alignItems: 'start', minInlineSize: 0,
  '@media': { 'screen and (max-width: 760px)': { gridTemplateColumns: 'minmax(0, 1fr)' } },
} as const;

/** Apply to labelled field wrappers, never to unrelated login/compose controls. */
export function registerFieldLayout(selector: string): void {
  globalStyle(selector, fieldLayout);
  globalStyle(`${selector} > input:not([type='checkbox'], [type='radio'], [type='range'], [type='color']), ${selector} > select`, {
    boxSizing: 'border-box', inlineSize: '100%', minInlineSize: 0, maxInlineSize: '100%',
  });
  globalStyle(`${selector} > input[type='number']`, { inlineSize: '10rem' });
  globalStyle(`${selector} > select`, { inlineSize: 'fit-content' });
  globalStyle(`${selector} > button`, { alignSelf: 'flex-start', maxInlineSize: '100%' });
  globalStyle(`${selector} > textarea`, { boxSizing: 'border-box', inlineSize: '100%', minInlineSize: 0 });
  globalStyle(`${selector}:has(textarea), ${selector}:has([contenteditable='true']), ${selector}[data-field-span='full']`, {
    maxInlineSize: 'none', gridColumn: '1 / -1',
  });
  globalStyle(`${selector}:has(> input[type='checkbox']), ${selector}:has(> input[type='radio'])`, {
    flexDirection: 'row', alignItems: 'flex-start',
  });
}

/** Related fields share a row; prose, editors and action groups span the group. */
export function registerFieldGrid(selector: string, fields = '.field, [data-form-field]'): void {
  globalStyle(selector, fieldsLayout);
  globalStyle(`${selector} > *`, { gridColumn: '1 / -1', minInlineSize: 0 });
  for (const field of fields.split(',').map(value => value.trim())) {
    globalStyle(`${selector} > ${field}`, { gridColumn: 'auto', marginBlockEnd: 0 });
    globalStyle(`${selector} > ${field}:has(textarea), ${selector} > ${field}:has([contenteditable='true']), ${selector} > ${field}[data-field-span='full']`, { gridColumn: '1 / -1', maxInlineSize: 'none' });
  }
  globalStyle(`${selector} > button`, { justifySelf: 'start', maxInlineSize: '100%' });
}
