import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

export const actor = style({ display: 'block', minInlineSize: 0, overflowWrap: 'anywhere' });
export const identifier = style({ display: 'block', maxInlineSize: '100%', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', fontFamily: vars.font.mono });
export const auditTable = style({ tableLayout: 'fixed', inlineSize: '100%' });
globalStyle(`${auditTable} td`, { overflowWrap: 'anywhere', verticalAlign: 'top' });
globalStyle(`${auditTable} th:first-child`, { inlineSize: '22%' });
globalStyle(`${auditTable} th:nth-child(2)`, { inlineSize: '26%' });
globalStyle(`${auditTable} th:nth-child(3)`, { inlineSize: '24%' });
const narrow = '(max-width: 760px)';
globalStyle(`${auditTable} thead`, { '@media': { [narrow]: { position: 'absolute', inlineSize: '1px', blockSize: '1px', overflow: 'hidden', clipPath: 'inset(50%)' } } });
globalStyle(`${auditTable}, ${auditTable} tbody`, { '@media': { [narrow]: { display: 'block', inlineSize: '100%' } } });
globalStyle(`${auditTable} tbody tr`, { '@media': { [narrow]: { display: 'grid', gridTemplateColumns: '1fr', gap: vars.space[1], paddingBlock: vars.space[3], borderBlockEnd: `${vars.border.rule} solid ${vars.color.borderStrong}` } } });
globalStyle(`${auditTable} td`, { '@media': { [narrow]: { display: 'grid', gridTemplateColumns: '5rem minmax(0, 1fr)', gap: vars.space[2], padding: 0, border: 'none' } } });
globalStyle(`${auditTable} td::before`, { '@media': { [narrow]: { content: 'attr(data-label)', color: vars.color.textDim, fontFamily: 'inherit', fontWeight: vars.weight.medium } } });
