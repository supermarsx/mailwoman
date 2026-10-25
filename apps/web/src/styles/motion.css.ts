import { globalStyle, keyframes, style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

// Opacity only: content never changes position, size or scroll geometry.
const enter = keyframes({ from: { opacity: 0 }, to: { opacity: 1 } });
const controls = ':where(button, a, input, select, textarea, [role="button"], [role="tab"], [role="checkbox"], [role="switch"])';
const surfaces = ':where([role="dialog"], [role="alertdialog"], dialog[open], [role="menu"], [role="tabpanel"], [data-mw-panel], details[open] > :not(summary))';

globalStyle(controls, {
  transitionProperty: 'background-color, border-color, color, opacity, box-shadow',
  transitionDuration: vars.a11y.motionDuration,
  transitionTimingFunction: 'ease-out',
});
globalStyle(surfaces, {
  animationName: enter,
  animationDuration: vars.a11y.motionDurationSlow,
  animationTimingFunction: 'ease-out',
});

export const route = style({
  display: 'flex', flexDirection: 'column', flex: '1 1 auto', minInlineSize: 0,
  selectors: {
    '&[data-mw-entering="true"]': { animation: `${enter} ${vars.a11y.motionDurationSlow} ease-out` },
    ':root[data-reduced-motion] &': { animation: 'none' },
  },
  '@media': { '(prefers-reduced-motion: reduce)': {
    animation: 'none',
    selectors: { '&[data-mw-entering="true"]': { animation: 'none' } },
  } },
});

globalStyle(`${controls}, ${surfaces}`, {
  '@media': { '(prefers-reduced-motion: reduce)': { transition: 'none', animation: 'none' } },
});
globalStyle(`:root[data-reduced-motion] :is(${controls}, ${surfaces})`, {
  transition: 'none', animation: 'none',
});
