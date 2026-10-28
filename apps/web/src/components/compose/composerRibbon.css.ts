// Composer ribbon helpers (t38-e5): the editor's slot in a ribbon tab and the
// one-line popovers that hold what does not fit three rows (link entry, find
// and replace, paragraph options, security details, Nextcloud, Assist).

import { style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

/** A slot (and the portal container inside it) lays its groups out as if they were the track's own. */
export const slot = style({ display: 'contents' });

export const popoverRoot = style({ display: 'flex', alignItems: 'center', minInlineSize: 0, blockSize: '100%' });

// Fixed, so the ribbon's `overflow: hidden` does not clip it; above the bar
// (900) and a folded group's popover (1400), under context menus (1500).
export const popover = style({
  position: 'fixed',
  zIndex: 1450,
  boxSizing: 'border-box',
  minInlineSize: '14rem',
  maxInlineSize: 'calc(100vw - 16px)',
  maxBlockSize: 'min(24rem, calc(100dvh - 16px))',
  overflow: 'auto',
  padding: vars.space[2],
  background: vars.color.surface,
  color: vars.color.text,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  boxShadow: vars.elevation[3],
  whiteSpace: 'normal',
  selectors: { '&[hidden]': { display: 'none' } },
});
