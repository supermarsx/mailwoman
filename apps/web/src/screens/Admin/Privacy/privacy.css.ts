import { style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

export const controls = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: vars.space[3] });
export const tiles = style({ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(min(100%, 11rem), 1fr))', gap: vars.space[3] });
