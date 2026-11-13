import {style} from '@vanilla-extract/css';
import {vars} from '../../theme/contract.css.ts';

/** One container treatment for settings cards, editors and management panels. */
export const panelSurface=style({
  minInlineSize:0,boxSizing:'border-box',
  padding:'clamp(.75rem, 2vw, 1.25rem)',
  border:`${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius:vars.radius.lg,background:vars.color.surface,
});
export const sectionStack=style({
  minInlineSize:0,
  selectors:{'&:not([hidden])':{display:'flex',flexDirection:'column',gap:vars.space[4]}},
});
