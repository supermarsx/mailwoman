// The workspace tab strip as a collapsible bar (t38 §2.1, `ui.bar.workspaceTabs`).
//
// The strip's chrome (tabs, close buttons) stays in styles/workspace.css.ts.
// Here: the bar's rule moves from the strip to the panel so it runs under the
// pin as well, and the strip scrolls only sideways (a thin scrollbar, no
// vertical overflow); the mouse wheel drives it through `horizontalWheel`.

import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const root = style({});

export const panel = style({
  alignItems: 'center',
  background: vars.color.surface,
  borderBlockEnd: `${vars.border.hair} solid ${vars.color.border}`,
});

export const strip = style({
  minInlineSize: 0,
  overflowY: 'hidden',
  scrollbarWidth: 'thin',
});

// Beats `.mail-workspace .subtab-strip` (two classes) in workspace.css.ts.
globalStyle(`.mail-workspace ${panel} ${strip}`, {
  borderBlockEnd: 'none',
  overflowY: 'hidden',
});
