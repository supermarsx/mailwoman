import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';
import { fieldLayout, fieldsLayout, registerFieldLayout } from '../../styles/formLayout.css.ts';
import { subsectionSurface } from './subsectionLayout.css.ts';

export const controls = style({ border: 0, padding: 0, margin: 0, minInlineSize: 0 });

export const field = style(fieldLayout);
registerFieldLayout(field);
export const fields = style(fieldsLayout);
export const editor = style({ display: 'grid', gap: vars.space[5], minInlineSize: 0 });
export const group = style({ ...subsectionSurface, display: 'grid', gap: vars.space[3] });
export const choices = style({ ...fieldsLayout, alignItems: 'start' });
export const check = style({ display: 'flex', alignItems: 'center', gap: vars.space[3], minInlineSize: 0 });
globalStyle(`${check} > input`, { flexShrink: 0, margin: 0 });
export const actions = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: vars.space[3] });
globalStyle(`${actions} > button`, { inlineSize: 'auto', flex: '0 0 auto' });
