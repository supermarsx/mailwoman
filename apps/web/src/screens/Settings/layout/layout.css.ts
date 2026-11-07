// Settings layout kit: stacked cards and compact rows. Navigation is the section
// list (SettingsNav) and, where a section has them, its own tab row.
// Tokens only (vars.*), logical properties, forced-colours safe.
import { globalStyle, keyframes, style } from '@vanilla-extract/css';
import { vars } from '../../../theme/contract.css.ts';

const ring = { outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`, outlineOffset: '2px' } as const;

// ---- cards and rows (used by SettingsCard / SettingRow) ----
export const card = style({
  display: 'flex', flexDirection: 'column', gap: vars.space[2], minInlineSize: 0, containerType: 'inline-size',
  paddingBlock: vars.space[3], scrollMarginBlockStart: vars.space[2],
  borderBlockStart: `1px solid ${vars.color.border}`,
  selectors: { '&:first-child': { borderBlockStart: 0, paddingBlockStart: 0 } },
});
export const cardHead = style({ display: 'flex', flexDirection: 'column', gap: '2px' });
export const cardTitle = style({ fontSize: '1rem', fontWeight: vars.weight.heavy, margin: 0 });
export const cardDesc = style({ fontSize: '0.8rem', color: vars.color.textDim, margin: 0 });
// Collapsible card: the title row is the summary, with the browser's own marker.
export const disclosure = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2] });
export const disclosureSummary = style({
  cursor: 'pointer', display: 'list-item', minBlockSize: '2rem',
  selectors: { '&:focus-visible': ring },
});
globalStyle(`${disclosureSummary} > h4`, { display: 'inline' });
globalStyle(`${disclosureSummary} > p`, { marginBlockStart: '2px' });
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
// A checkbox's size, accent and cursor come from styles/checks.css.ts alone.

// Deep link target (SettingsRoute.card): a short outline, no layout change.
const flash = keyframes({ '0%': { outlineColor: vars.color.accent }, '100%': { outlineColor: 'transparent' } });
globalStyle('[data-card-flash]', { outline: '2px solid transparent', outlineOffset: '4px', animation: `${flash} 1.6s ease-out`, '@media': { '(prefers-reduced-motion: reduce)': { animation: 'none', outlineColor: vars.color.accent } } });

// A row whose control needs the full width under its label.
export const rowStacked = style({
  gridTemplateColumns: 'minmax(0,1fr)',
});
globalStyle(`${rowStacked} > ${rowControl}`, { gridColumn: 1, gridRow: 'auto', alignItems: 'stretch' });
globalStyle(`${rowStacked} > ${rowControl} textarea`, { inlineSize: '100%', maxInlineSize: 'none' });

// A toggle is small: keep it beside its label even in a narrow card.
globalStyle(`${row}:has(> ${rowControl} > input[type="checkbox"])`, { '@container': { '(max-width: 22rem)': { gridTemplateColumns: 'minmax(0,1fr) auto' } } });
globalStyle(`${row} > ${rowControl}:has(> input[type="checkbox"])`, { '@container': { '(max-width: 22rem)': { gridColumn: 2, gridRow: '1 / span 2', alignItems: 'flex-end' } } });

// A toolbar line above a list or table: a search field and a few buttons.
export const toolRow = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: `${vars.space[1]} ${vars.space[3]}` });
export const inlineField = style({ display: 'inline-flex', alignItems: 'center', gap: vars.space[2], flex: '1 1 14rem', minInlineSize: 0, fontSize: '0.85rem' });
globalStyle(`${inlineField} > input`, { flex: '1 1 8rem', minInlineSize: 0, maxInlineSize: '24rem' });
