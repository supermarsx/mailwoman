import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

export const list = style({ display: 'flex', gap: vars.space[2], overflowX: 'auto', paddingBlockEnd: vars.space[2], borderBottom: `1px solid ${vars.color.border}` });
globalStyle(`${list} button`, { flex: '0 0 auto', padding: `${vars.space[2]} ${vars.space[3]}`, minHeight: vars.a11y.touchTarget, background: 'transparent', color: vars.color.text, border: `1px solid ${vars.color.border}`, font: 'inherit', cursor: 'pointer' });
globalStyle(`${list} button[aria-selected="true"]`, { background: vars.color.accent, color: vars.color.accentText, borderColor: vars.color.accent });
globalStyle(`${list} button:focus-visible`, { outline: `2px solid ${vars.a11y.focusRingColor}`, outlineOffset: 2 });
export const panel = style({ minWidth: 0, selectors: { '&:not([hidden])': { display: 'flex', flexDirection: 'column', gap: vars.space[4] } } });
