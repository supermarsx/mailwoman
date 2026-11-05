import {globalStyle, style} from '@vanilla-extract/css';

// Fixed box per size, so nothing shifts while the lazy path chunk loads.
export const logo = style({
  display: 'inline-block', flex: 'none', verticalAlign: 'middle', boxSizing: 'border-box',
  selectors: {
    '&[data-size="16"]': {inlineSize: '16px', blockSize: '16px'},
    '&[data-size="20"]': {inlineSize: '20px', blockSize: '20px'},
    '&[data-size="24"]': {inlineSize: '24px', blockSize: '24px'},
    '&[data-size="32"]': {inlineSize: '32px', blockSize: '32px'},
  },
});

// Brand colours and tile hues are replaced by system colours.
globalStyle(`${logo} path`, {'@media': {'(forced-colors: active)': {fill: 'CanvasText'}}});
globalStyle(`${logo} rect`, {'@media': {'(forced-colors: active)': {fill: 'Canvas', stroke: 'CanvasText'}}});
globalStyle(`${logo} text`, {'@media': {'(forced-colors: active)': {fill: 'CanvasText'}}});

export const label = style({display: 'inline-flex', alignItems: 'center', gap: '0.5em', minInlineSize: 0});

export const cell = style({display: 'flex', alignItems: 'center', gap: '0.5em', minInlineSize: 0});
export const cellText = style({minInlineSize: 0, overflowWrap: 'anywhere'});
