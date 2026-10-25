// Styles of the admin UX seam (t32-e71a): page templates, source badges, impact
// preview, field meta row, stream view and the console's appearance control.
//
// Same rules as shell.css.ts (t32.md §2.4 P1–P7) and the UX1 gate
// (`adminStyle.rules.test.ts`): `vars.*` tokens only, logical properties only,
// every interactive class has its own `:focus-visible` rule, no fixed width
// over 480 px without a max-width. State is never shown by colour alone: every
// badge has an icon and text, the selected card a check and a heavier rule.
// Under `forced-colors: active` borders are kept and the highlight is an outline
// in the system `Highlight` colour.

import { globalStyle, style, styleVariants } from '@vanilla-extract/css';
import { registerFieldLayout, registerFieldGrid } from '../../../styles/formLayout.css.ts';
import { vars } from '../../../theme/contract.css.ts';
import { ACCENT_LIST, THEME_LIST, accentShade } from '../../../theme/registry.ts';
import type { ThemeName } from '../../../theme/contract.css.ts';
import type { AccentId } from '../../../theme/accentMath.ts';

const PHONE = 'screen and (max-width: 760px)';
/** `FIELD_HIGHLIGHT_ATTR` of fieldAnchors.ts (not imported: that module collects the
 *  anchor files at build time, which a style module must not pull in). */
const FIELD_HIGHLIGHT_ATTR = 'data-field-highlight';
const FORCED = '(forced-colors: active)';

const smallLabel = {
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
  fontWeight: vars.weight.bold,
} as const;
const noTrackingScripts = ':lang(ar) &, :lang(ja) &, :lang(zh) &';
const plainLabel = { textTransform: 'none', letterSpacing: vars.tracking.none } as const;
const focusRing = { outline: 'none', boxShadow: vars.a11y.focusRing } as const;

// ── Page frame (every template) ───────────────────────────────────────────────
export const page = style({ display: 'flex', flexDirection: 'column', gap: vars.space[4], minWidth: 0 });
export const pageHead = style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'flex-start',
  justifyContent: 'space-between',
  gap: vars.space[3],
});
export const pageTitleBlock = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1], minWidth: 0 });
export const pageTitle = style({ margin: 0, fontSize: '1.25rem', fontWeight: vars.weight.heavy });
export const pageDescription = style({ margin: 0, color: vars.color.textDim, maxWidth: '72ch' });
export const pageActions = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[2] });

/** Loading / error / empty / not-available block inside the final layout (UX11). */
export const pageState = style({
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'flex-start',
  gap: vars.space[2],
  padding: `calc(${vars.space[4]} - 1px)`,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
});
export const pageStateText = style({ margin: 0 });
export const pageStateCode = style({ fontFamily: vars.font.mono, fontSize: '0.8rem', color: vars.color.textDim });
export const stateRow = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: vars.space[2] });

// ── List + detail ─────────────────────────────────────────────────────────────
export const listDetail = style({
  display: 'grid',
  gridTemplateColumns: 'minmax(0, 1fr)',
  gap: vars.space[4],
  '@media': {
    'screen and (min-width: 1200px)': { gridTemplateColumns: 'minmax(0, 3fr) minmax(0, 2fr)' },
  },
});
export const listPane = style({
  minWidth: 0,
  selectors: { [`${listDetail}[data-detail-open] &`]: { '@media': { [PHONE]: { display: 'none' } } } },
});
export const detailPane = style({
  minWidth: 0,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: `calc(${vars.space[4]} - 1px)`,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  selectors: { '&:focus-visible': focusRing },
});
export const detailHead = style({ display: 'flex', alignItems: 'center', gap: vars.space[2], flexWrap: 'wrap' });
export const detailTitle = style({ margin: 0, fontSize: '1.05rem', fontWeight: vars.weight.heavy, overflowWrap: 'anywhere' });
/** "Back to the list" link: shown on phones only, where list and detail are one at a time. */
export const backLink = style({
  display: 'none',
  alignItems: 'center',
  gap: vars.space[2],
  minHeight: vars.a11y.touchTarget,
  color: vars.color.link,
  '@media': { [PHONE]: { display: 'inline-flex' } },
  selectors: { '&:focus-visible': focusRing, '&:hover': { textDecoration: 'underline' } },
});
/** The chevron turned to point back (it is mirrored again under RTL by `icon--directional`). */
export const backIcon = style({ rotate: '180deg' });

// ── Settings form ─────────────────────────────────────────────────────────────
export const form = style({ display: 'flex', flexDirection: 'column', gap: vars.space[4], minWidth: 0 });
export const fieldset = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  margin: 0,
  padding: `calc(${vars.space[4]} - 1px)`,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  minWidth: 0,
});
export const legend = style({ padding: 0, fontSize: '1rem', fontWeight: vars.weight.heavy, float: 'none' });
export const fieldsetHelp = style({ margin: 0, fontSize: '0.85rem', color: vars.color.textDim });
export const fieldBlock = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2], minWidth: 0 });
/** Source badge, propagation note, history and revert under a field. */
export const fieldMeta = style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'center',
  gap: `${vars.space[1]} ${vars.space[3]}`,
  fontSize: '0.8rem',
  color: vars.color.textDim,
});
/** A form control (input, select, textarea) in the house field look of `.field input`
 *  (app.css): the template's controls sit outside a `.field`, so without this a select
 *  stayed at the browser's ~20 px (under the 24 px target) and inputs had no frame. */
export const control = style({
  maxWidth: '100%',
  font: 'inherit',
  padding: 'calc(0.5rem - 1px) calc(0.6rem - 1px)',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  color: vars.color.text,
  selectors: { ':root[data-appearance="light"] &': { background: vars.color.surface } },
  '@media': { [PHONE]: { paddingBlock: '0.5rem' } },
});
export const checkboxRow = style({ display: 'flex', alignItems: 'center', gap: vars.space[2] });
export const learnMore = style({
  color: vars.color.link,
  selectors: { '&:focus-visible': focusRing, '&:hover': { textDecoration: 'underline' } },
});
export const linkButton = style({
  appearance: 'none',
  font: 'inherit',
  fontSize: '0.8rem',
  background: 'transparent',
  border: 0,
  padding: 0,
  minHeight: vars.a11y.touchTarget,
  color: vars.color.link,
  textDecoration: 'underline',
  cursor: 'pointer',
  selectors: { '&:focus-visible': focusRing, '&:disabled': { color: vars.color.textDim, cursor: 'default' } },
});

/** Sticky save bar: count of changed fields, Discard, Save. */
export const saveBar = style({
  position: 'sticky',
  insetBlockEnd: 0,
  zIndex: 4,
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'center',
  justifyContent: 'space-between',
  gap: vars.space[3],
  padding: `${vars.space[3]} ${vars.space[4]}`,
  background: vars.color.bgAlt,
  borderBlockStart: `${vars.border.heavy} solid ${vars.color.borderStrong}`,
});
export const saveCount = style({ margin: 0, fontWeight: vars.weight.bold });

// ── Source badges ─────────────────────────────────────────────────────────────
export const badges = style({ display: 'inline-flex', flexWrap: 'wrap', gap: vars.space[1] });
export const badge = style({
  ...smallLabel,
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  fontSize: '0.68rem',
  padding: `0 ${vars.space[2]}`,
  minHeight: '20px',
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  color: vars.color.text,
  background: vars.color.bg,
  selectors: {
    [noTrackingScripts]: plainLabel,
    '&[data-kind="env-pin"], &[data-kind="locked"], &[data-kind="restart"]': {
      boxShadow: `inset 3px 0 0 ${vars.color.warning}`,
    },
    '[dir="rtl"] &[data-kind="env-pin"], [dir="rtl"] &[data-kind="locked"], [dir="rtl"] &[data-kind="restart"]': {
      boxShadow: `inset -3px 0 0 ${vars.color.warning}`,
    },
    '&[data-kind="stored"]': { boxShadow: `inset 3px 0 0 ${vars.color.accent}` },
    '[dir="rtl"] &[data-kind="stored"]': { boxShadow: `inset -3px 0 0 ${vars.color.accent}` },
  },
  '@media': { [FORCED]: { border: `${vars.border.hair} solid CanvasText` } },
});
globalStyle(`${badge} svg`, { flex: 'none', width: '12px', height: '12px' });

// ── Impact preview ────────────────────────────────────────────────────────────
/** A form inside a dialog keeps the dialog's column layout. */
export const dialogForm = style({ display: 'flex', flexDirection: 'column', gap: vars.space[3] });
export const impact = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2] });
export const impactHeading = style({
  ...smallLabel,
  margin: 0,
  fontSize: '0.72rem',
  color: vars.color.textDim,
  selectors: { [noTrackingScripts]: plainLabel },
});
export const impactList = style({ listStyle: 'none', margin: 0, padding: 0, display: 'flex', flexDirection: 'column', gap: vars.space[2] });
export const impactItem = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
  paddingInlineStart: vars.space[3],
  borderInlineStart: `${vars.border.heavy} solid ${vars.color.warning}`,
});
export const impactCount = style({ fontWeight: vars.weight.bold });
export const impactNames = style({ fontSize: '0.8rem', color: vars.color.textDim, overflowWrap: 'anywhere' });

// ── Stale write notice ────────────────────────────────────────────────────────
export const stale = style({ flexDirection: 'column', alignItems: 'flex-start' });
export const staleText = style({ display: 'flex', gap: vars.space[2], margin: 0, alignItems: 'flex-start' });

// ── Dashboard tiles ───────────────────────────────────────────────────────────
export const tileGrid = style({
  listStyle: 'none',
  margin: 0,
  padding: 0,
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fill, minmax(min(100%, 240px), 1fr))',
  gap: vars.space[3],
});
export const tileCell = style({ minWidth: 0 });
export const tileWide = style({ gridColumn: '1 / -1' });

// ── Stream (logs, jobs, live runs) ────────────────────────────────────────────
export const streamToolbar = style({ display: 'flex', flexWrap: 'wrap', alignItems: 'flex-end', gap: vars.space[3] });
export const streamStatus = style({
  ...smallLabel,
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[2],
  fontSize: '0.72rem',
  margin: 0,
  selectors: { [noTrackingScripts]: plainLabel },
});
export const streamLog = style({
  maxHeight: '60vh',
  overflowY: 'auto',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  background: vars.color.bgSink,
  fontFamily: vars.font.mono,
  fontSize: '0.8rem',
  selectors: { '&:focus-visible': focusRing },
});
export const streamList = style({ listStyle: 'none', margin: 0, padding: 0 });
export const streamEntry = style({
  padding: `${vars.space[1]} ${vars.space[3]}`,
  borderBlockEnd: `${vars.border.hair} solid ${vars.color.border}`,
  overflowWrap: 'anywhere',
  whiteSpace: 'pre-wrap',
});

// ── Field deep-link highlight ─────────────────────────────────────────────────
globalStyle(`[${FIELD_HIGHLIGHT_ATTR}]`, {
  outline: `${vars.a11y.focusRingWidth} solid ${vars.color.focus}`,
  outlineOffset: '4px',
  borderRadius: vars.radius.sm,
});
globalStyle(`[${FIELD_HIGHLIGHT_ATTR}]`, {
  '@media': { [FORCED]: { outline: `${vars.a11y.focusRingWidth} solid Highlight` } },
});

// ── Console appearance control (DC5) ──────────────────────────────────────────
export const appearanceDialog = style({ width: 'min(calc(100vw - 32px), 720px)' });
export const appearanceSection = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  margin: 0,
  padding: 0,
  border: 0,
  minWidth: 0,
});
export const appearanceLegend = style({
  ...smallLabel,
  padding: 0,
  fontSize: '0.75rem',
  marginBlockEnd: vars.space[2],
  selectors: { [noTrackingScripts]: plainLabel },
});
export const modeGroup = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[2] });
export const modeOption = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[2],
  minHeight: vars.a11y.touchTarget,
  padding: `${vars.space[1]} ${vars.space[3]}`,
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  cursor: 'pointer',
  selectors: {
    '&:has(input:checked)': { boxShadow: `inset 0 0 0 1px ${vars.color.accent}`, fontWeight: vars.weight.bold },
    '&:focus-within': focusRing,
    '&:focus-visible': focusRing,
  },
});
export const familyLabel = style({ margin: 0, fontSize: '0.8rem', color: vars.color.textDim });
export const themeGrid = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fill, minmax(min(100%, 150px), 1fr))',
  gap: vars.space[2],
});
export const themeCard = style({
  appearance: 'none',
  font: 'inherit',
  textAlign: 'start',
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
  padding: vars.space[2],
  minHeight: vars.a11y.touchTarget,
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  color: vars.color.text,
  cursor: 'pointer',
  selectors: {
    '&:hover': { background: vars.color.bgAlt },
    '&:focus-visible': focusRing,
    '&[aria-pressed="true"]': {
      borderColor: vars.color.accent,
      boxShadow: `inset 0 0 0 2px ${vars.color.accent}`,
      fontWeight: vars.weight.bold,
    },
  },
  '@media': { [FORCED]: { border: `${vars.border.rule} solid ButtonText` } },
});
export const themeName = style({ display: 'flex', alignItems: 'center', gap: vars.space[1], fontSize: '0.85rem' });
export const themeMeta = style({ fontSize: '0.72rem', color: vars.color.textDim });

/** The miniature: page, panel, a text bar and the accent, in the pack's own colours. */
export const preview = style({
  display: 'grid',
  gridTemplateColumns: '1fr 2fr',
  gridTemplateRows: '1fr 1fr',
  gap: '3px',
  height: '36px',
  padding: '3px',
  border: `${vars.border.hair} solid`,
  borderRadius: vars.radius.sm,
  '@media': { [FORCED]: { forcedColorAdjust: 'none' } },
});
export const previewPanel = style({ gridRow: '1 / 3' });
export const previewText = style({ alignSelf: 'center', height: '4px' });
export const previewAccent = style({ alignSelf: 'center', height: '8px', width: '60%' });

const paletteOf = (id: ThemeName) => THEME_LIST.find((e) => e.id === id)!.palette;
const themeIds = THEME_LIST.map((e) => e.id);
const byTheme = <T>(fn: (id: ThemeName) => T): Record<ThemeName, T> =>
  Object.fromEntries(themeIds.map((id) => [id, fn(id)])) as Record<ThemeName, T>;

/** Per-theme preview colours (values from the theme registry, never written by hand). */
export const previewBg = styleVariants(byTheme((id) => ({ background: paletteOf(id).bg, borderColor: paletteOf(id).borderStrong })));
export const previewPanelBg = styleVariants(byTheme((id) => ({ background: paletteOf(id).surface })));
export const previewTextBg = styleVariants(byTheme((id) => ({ background: paletteOf(id).text })));
export const previewAccentBg = styleVariants(byTheme((id) => ({ background: paletteOf(id).accent })));

export const accentRow = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[2], alignItems: 'center' });
export const accentDefault = style({
  appearance: 'none',
  font: 'inherit',
  fontSize: '0.8rem',
  minHeight: '28px',
  padding: `0 ${vars.space[3]}`,
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  background: vars.color.surface,
  color: vars.color.text,
  cursor: 'pointer',
  selectors: {
    '&:hover': { background: vars.color.bgAlt },
    '&:focus-visible': focusRing,
    '&[aria-pressed="true"]': { boxShadow: `inset 0 0 0 2px ${vars.color.accent}`, fontWeight: vars.weight.bold },
  },
});
export const swatch = style({
  appearance: 'none',
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  width: '28px',
  height: '28px',
  padding: 0,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.pill,
  cursor: 'pointer',
  selectors: {
    '&:focus-visible': focusRing,
    '&:hover': { borderColor: vars.color.text },
    '&[aria-pressed="true"]': { borderColor: vars.color.text, borderWidth: vars.border.heavy },
  },
  '@media': { [FORCED]: { forcedColorAdjust: 'none', border: `${vars.border.rule} solid ButtonText` } },
});
globalStyle(`${swatch} svg`, { width: '14px', height: '14px' });

/** One class per accent; each theme paints its own shade of it (as `data-accent` would). */
export const swatchAccent = styleVariants(
  Object.fromEntries(ACCENT_LIST.map((a) => [a.id, {}])) as Record<AccentId, Record<string, never>>,
);
for (const theme of themeIds) {
  for (const a of ACCENT_LIST) {
    const shade = accentShade(theme, a.id);
    globalStyle(`:root[data-theme="${theme}"] ${swatchAccent[a.id]}`, { background: shade.accent, color: shade.accentText });
  }
}
// Before any theme attribute (first paint) the default Dark shade applies.
for (const a of ACCENT_LIST) {
  const shade = accentShade('dark', a.id);
  globalStyle(`:root:not([data-theme]) ${swatchAccent[a.id]}`, { background: shade.accent, color: shade.accentText });
}

/** Pushes the header's appearance button to the inline end, next to Search. On a phone
 *  the top bar has its own button, so this one is not shown there. */
export const headPush = style({ marginInlineStart: 'auto', '@media': { [PHONE]: { display: 'none' } } });

/** The header button that opens the appearance dialog. */
export const appearanceButton = style({
  selectors: { '&:focus-visible': focusRing },
});

registerFieldLayout(fieldBlock);
registerFieldGrid(fieldset);
