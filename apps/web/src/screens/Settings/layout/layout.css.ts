// Settings layout kit: card grid, compact rows, sticky sub-navigation and the
// collapse / search states the SettingsToolbar drives through data attributes.
// Tokens only (vars.*), logical properties, forced-colours safe.
import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';
import { actions as legacyActions, field as legacyField, ghost as legacyGhost } from '../styles.css.ts';

const ring = { outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`, outlineOffset: '2px' } as const;

export const toolbar = style({
  position: 'sticky', insetBlockStart: 0, zIndex: 2, gridColumn: '1 / -1',
  display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: vars.space[2],
  padding: `${vars.space[2]} 0`, background: vars.color.bg,
  borderBlockEnd: `1px solid ${vars.color.border}`,
  selectors: { '&[hidden]': { display: 'none' } },
});
export const searchBox = style({
  flex: '1 1 14rem', minInlineSize: 0, maxInlineSize: '24rem', font: 'inherit', fontSize: '0.875rem',
  padding: `${vars.space[1]} ${vars.space[3]}`, minBlockSize: '2rem', color: vars.color.text,
  background: vars.color.bgSink, border: `${vars.border.rule} solid ${vars.color.borderStrong}`, borderRadius: vars.radius.md,
  selectors: { '&:focus-visible': ring },
  '@media': { '(pointer: coarse)': { minBlockSize: vars.a11y.touchTarget } },
});
export const chips = style({
  display: 'flex', gap: vars.space[1], flex: '1 1 100%', order: 3, overflowX: 'auto', flexWrap: 'nowrap',
  scrollbarWidth: 'thin', paddingBlockEnd: '2px', margin: 0, padding: 0, listStyle: 'none',
});
export const chip = style({
  flex: '0 0 auto', font: 'inherit', fontSize: '0.8125rem', cursor: 'pointer', whiteSpace: 'nowrap',
  padding: `${vars.space[1]} ${vars.space[3]}`, borderRadius: vars.radius.md, color: vars.color.text,
  background: 'transparent', border: `1px solid ${vars.color.border}`,
  selectors: {
    '&:hover': { background: vars.color.surface },
    '&[aria-current="true"]': { background: vars.color.surface, borderColor: vars.color.accent, fontWeight: vars.weight.bold },
    '&:focus-visible': ring,
  },
  '@media': { '(forced-colors: active)': { selectors: { '&[aria-current="true"]': { outline: '2px solid Highlight' } } } },
});
export const toolButton = style({
  font: 'inherit', fontSize: '0.8125rem', cursor: 'pointer', padding: `${vars.space[1]} ${vars.space[2]}`,
  background: 'transparent', color: vars.color.textDim, border: 0, textDecoration: 'underline',
  selectors: { '&:hover': { color: vars.color.text }, '&:focus-visible': ring },
});
export const empty = style({ gridColumn: '1 / -1', margin: 0, padding: vars.space[3], color: vars.color.textDim, display: 'flex', flexWrap: 'wrap', gap: vars.space[2], alignItems: 'center' });

// ---- cards and rows (used by SettingsCard / SettingRow) ----
export const card = style({
  display: 'flex', flexDirection: 'column', gap: vars.space[2], minInlineSize: 0, containerType: 'inline-size',
  padding: vars.space[4], borderRadius: vars.radius.lg, background: vars.color.surface,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
});
export const cardHead = style({ display: 'flex', flexDirection: 'column', gap: '2px' });
export const cardTitle = style({ fontSize: '1rem', fontWeight: vars.weight.heavy, margin: 0 });
export const cardDesc = style({ fontSize: '0.8rem', color: vars.color.textDim, margin: 0 });
export const rows = style({ display: 'flex', flexDirection: 'column' });

export const row = style({
  display: 'grid', gridTemplateColumns: 'minmax(0,1fr) auto', columnGap: vars.space[3], rowGap: '2px',
  alignItems: 'center', paddingBlock: vars.space[2], minBlockSize: '2.75rem',
  borderBlockStart: `1px solid ${vars.color.border}`,
  selectors: { '&:first-child': { borderBlockStart: 0 } },
  '@container': { '(max-width: 22rem)': { gridTemplateColumns: 'minmax(0,1fr)' } },
});
export const rowLabel = style({ fontSize: '0.9rem', fontWeight: vars.weight.medium, cursor: 'pointer', gridColumn: 1 });
export const rowHelp = style({ fontSize: '0.78rem', color: vars.color.textDim, margin: 0, gridColumn: 1, gridRow: 2 });
export const rowControl = style({
  gridColumn: 2, gridRow: '1 / span 2', display: 'flex', flexDirection: 'column', alignItems: 'flex-end', justifyContent: 'center', gap: '2px',
  '@container': { '(max-width: 22rem)': { gridColumn: 1, gridRow: 'auto', alignItems: 'flex-start' } },
});
export const rowReset = style({
  font: 'inherit', fontSize: '0.75rem', cursor: 'pointer', background: 'transparent', color: vars.color.textDim, border: 0, textDecoration: 'underline', padding: vars.space[1],
  selectors: { '&:disabled': { opacity: 0.5, cursor: 'not-allowed' }, '&:focus-visible': ring },
});
export const rowNote = style({ gridColumn: '1 / -1', margin: 0 });
globalStyle(`${rowControl} select, ${rowControl} input:not([type="checkbox"])`, { maxInlineSize: '14rem', minInlineSize: '6rem' });
globalStyle(`${rowControl} input[type="checkbox"]`, { inlineSize: '1.25rem', blockSize: '1.25rem', accentColor: vars.color.accent, cursor: 'pointer' });

// ---- DOM-driven states (set by SettingsToolbar on any section's cards) ----
// Doubled attribute: must out-rank the section container's flex column rule.
globalStyle('[data-kit-grid][data-kit-grid]:not([hidden])', {
  display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(min(22rem, 100%), 1fr))', gap: vars.space[4], alignItems: 'start',
});
globalStyle('[data-kit-grid] > :not([data-kit-card])', { gridColumn: '1 / -1' });
globalStyle('[data-kit-card]', { scrollMarginBlockStart: '7rem', minInlineSize: 0 });
// The heading row is the toggle: an invisible button laid over it, so the
// heading's own text nodes stay where their reactive bindings expect them.
globalStyle('[data-kit-head]', { position: 'relative', paddingInlineEnd: '1.5rem' });
globalStyle('[data-kit-toggle]', {
  position: 'absolute', inset: 0, inlineSize: '100%', blockSize: '100%', maxInlineSize: 'none', background: 'transparent', border: 0, padding: 0, margin: 0, cursor: 'pointer', color: 'inherit',
});
globalStyle('[data-kit-toggle]:focus-visible', ring);
globalStyle('[data-kit-toggle]::after', {
  content: '""', position: 'absolute', insetInlineEnd: '0.25rem', insetBlockStart: '0.55rem', inlineSize: '0.5rem', blockSize: '0.5rem',
  borderInlineEnd: '2px solid currentColor', borderBlockEnd: '2px solid currentColor', transform: 'rotate(45deg)', transition: 'transform 120ms',
});
globalStyle('[data-kit-collapsed="true"] [data-kit-toggle]::after', { transform: 'rotate(-45deg)' });
globalStyle('[data-kit-collapsed="true"] > :not([data-kit-head])', { display: 'none' });
globalStyle('[data-kit-searching] [data-kit-collapsed="true"] > :not([data-kit-head])', { display: 'revert' });
globalStyle('[data-kit-hidden]', { display: 'none !important' });
globalStyle('[data-kit-match]', { boxShadow: `inset 3px 0 0 ${vars.color.accent}` });

// Existing (not yet row-migrated) fields inside a card: the trailing "Use
// administrator default" reset reads as a quiet link, not a full-height button.
globalStyle(`[data-kit-card] .${legacyField} > .${legacyGhost}:last-child:not(:only-child), [data-kit-card] .${legacyField} > .${legacyActions} > .${legacyGhost}:only-child`, {
  minBlockSize: 'auto', padding: '2px 6px', fontSize: '0.75rem', borderColor: 'transparent', color: vars.color.textDim, textDecoration: 'underline', alignSelf: 'flex-start',
});
