// Composer header: two explicit row densities and the themed Cc/Bcc toggle.
// The density classes only publish custom properties (--mw-head-*); the grid
// rules in compose-chrome.css read them, so the rhythm lives in one place and
// uses the space tokens. Comfortable is the default; compact is used when the
// header is not pinned open (auto / collapsed modes) and stays tighter, but
// rows still never touch.

import { createVar, style } from '@vanilla-extract/css';
import { linkColor } from '../../theme/accentLink.ts';
import { vars } from '../../theme/contract.css.ts';

export const headRowGap = createVar();
export const headSubjectGap = createVar();
export const headRowMin = createVar();

export const densityComfortable = style({
  vars: {
    [headRowGap]: vars.space[2],
    [headSubjectGap]: vars.space[3],
    [headRowMin]: '2.5rem',
  },
});

export const densityCompact = style({
  vars: {
    [headRowGap]: vars.space[1],
    [headSubjectGap]: vars.space[2],
    [headRowMin]: '2rem',
  },
});

export const ccbccGroup = style({
  display: 'inline-flex',
  alignItems: 'center',
  alignSelf: 'center',
  gap: vars.space[1],
  paddingInlineStart: vars.space[2],
  borderInlineStart: `${vars.border.hair} solid ${vars.color.border}`,
});

export const ccbccToggle = style({
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  gap: vars.space[1],
  boxSizing: 'border-box',
  minBlockSize: headRowMin,
  minInlineSize: vars.a11y.touchTarget,
  paddingInline: vars.space[2],
  border: `${vars.border.hair} solid transparent`,
  borderRadius: vars.radius.md,
  background: 'transparent',
  color: linkColor,
  font: 'inherit',
  fontSize: '.8125rem',
  fontWeight: 500,
  whiteSpace: 'nowrap',
  cursor: 'pointer',
  transition: `background-color ${vars.a11y.motionDuration}, border-color ${vars.a11y.motionDuration}`,
  selectors: {
    '&:hover': {
      background: `color-mix(in srgb, ${vars.color.accent} 12%, transparent)`,
      borderColor: vars.color.border,
    },
    '&:active': {
      background: `color-mix(in srgb, ${vars.color.accent} 22%, transparent)`,
    },
    '&[aria-pressed="true"]': {
      background: `color-mix(in srgb, ${vars.color.accent} 16%, ${vars.color.bgAlt})`,
      borderColor: vars.color.borderStrong,
      color: vars.color.text,
    },
    '&:focus-visible': {
      outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`,
      outlineOffset: '1px',
    },
  },
  '@media': {
    '(forced-colors: active)': {
      color: 'LinkText',
      borderColor: 'ButtonBorder',
      selectors: {
        '&[aria-pressed="true"]': { background: 'Highlight', color: 'HighlightText', forcedColorAdjust: 'none' },
        '&:focus-visible': { outlineColor: 'Highlight' },
      },
    },
  },
});
