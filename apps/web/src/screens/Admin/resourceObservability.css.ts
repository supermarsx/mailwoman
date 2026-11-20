import {globalStyle, style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';
import * as admin from './admin.css.ts';

export const grid = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fit, minmax(min(100%, 320px), 1fr))',
  gap: vars.space[5],
  minInlineSize: 0,
});

export const card = style([admin.card, {
  gap: vars.space[4],
  overflowWrap: 'anywhere',
}]);

export const cpuCard = style([card, {gridColumn: '1 / -1'}]);

export const heading = style({
  margin: 0,
  fontSize: '1rem',
  lineHeight: 1.4,
  fontWeight: vars.weight.bold,
});

export const table = style([admin.table, {tableLayout: 'fixed'}]);
globalStyle(`${table} th, ${table} td`, {
  paddingInline: vars.space[2],
  overflowWrap: 'anywhere',
  whiteSpace: 'normal',
  lineHeight: 1.5,
});
globalStyle(`${table} thead th`, {whiteSpace: 'normal', overflowWrap: 'anywhere'});
globalStyle(`${table} tbody th`, {fontWeight: vars.weight.medium, minInlineSize: 0});
globalStyle(`${table} td`, {fontVariantNumeric: 'tabular-nums'});
globalStyle(`${card} p`, {margin: 0, lineHeight: 1.6});
