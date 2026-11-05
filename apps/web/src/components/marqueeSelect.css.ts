import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

// Marquee (rubber-band) selection: an accent-tinted band with an accent
// outline, theme tokens only. The band is fixed to the viewport and clipped to
// the scroll area by the controller; pointer events pass through it.
export const band = style({
  position: 'fixed', zIndex: 30, pointerEvents: 'none', boxSizing: 'border-box',
  backgroundColor: `color-mix(in srgb, ${vars.color.accent} 18%, transparent)`,
  border: `1px solid ${vars.color.accent}`,
  '@media': { '(forced-colors: active)': { backgroundColor: 'Highlight', opacity: 0.3, borderColor: 'Highlight' } },
});

/** Set on the container while a drag runs: no text selection, no native drag images. */
export const dragging = style({ userSelect: 'none', WebkitUserSelect: 'none', cursor: 'default' });

/** An item that belongs to the marquee selection; the owner sets `data-marquee-item` and `data-selected="true"`. */
globalStyle('[data-marquee-item][data-selected="true"]:not(.sidebar__box--active)', {
  backgroundColor: `color-mix(in srgb, ${vars.color.accent} 22%, ${vars.color.bg})`,
  boxShadow: `inset 0 0 0 1px ${vars.color.accent}`,
});
globalStyle('[data-marquee-item][data-selected="true"].sidebar__box--active', { boxShadow: `inset 0 0 0 2px ${vars.color.text}` });
