import { vars } from '../../theme/contract.css.ts';
/** Shared admin group surface; pages supply the outside gutters. */
export const subsectionSurface = {
  boxSizing: 'border-box', minInlineSize: 0, margin: 0,
  padding: `calc(${vars.space[6]} - ${vars.border.rule})`,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md, background: vars.color.surface,
  '@media': { 'screen and (max-width: 760px)': { padding: `calc(${vars.space[5]} - ${vars.border.rule})` } },
} as const;
