import { style } from '@vanilla-extract/css';
export const setupKey = style({ overflowWrap: 'anywhere', minInlineSize: 0 });
export const formGroup = style({ border: 0, padding: 0, margin: 0, minInlineSize: 0 });

export const totpQr = style({ inlineSize: 'min(100%, 18rem)', blockSize: 'auto', alignSelf: 'flex-start' });
