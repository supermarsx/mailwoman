// Admin console frame and shared component styles (t32-admin.md §2.1).
//
// Same rules as admin.css.ts (t32.md §2.4 P1–P7): tokens only, square, hard
// shadows, tracked small labels, state never by colour alone, logical
// properties. Nav items, table rows and controls are no taller than the
// pre-t32 admin `navItem`, table rows and `.btn` (density rule).
//
// Phone width is below 761 px, the webmail breakpoint: the sidebar becomes a top
// bar with a menu button that opens the navigation as a drawer.

import { style, styleVariants, globalStyle, keyframes } from '@vanilla-extract/css';
import { registerFieldLayout } from '../../../styles/formLayout.css.ts';
import { vars } from '../../../theme/contract.css.ts';

const PHONE = 'screen and (max-width: 760px)';

const smallLabel = {
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
  fontWeight: vars.weight.bold,
} as const;
const noTrackingScripts = ':lang(ar) &, :lang(ja) &, :lang(zh) &';
const plainLabel = { textTransform: 'none', letterSpacing: vars.tracking.none } as const;
/** The focus ring as an outline (kept under forced colours, takes no layout space).
 *  Each interactive class names its own, so the UX1 gate can check it is there. */
const ringOutline = { outline: `${vars.a11y.focusRingWidth} solid ${vars.a11y.focusRingColor}`, outlineOffset: '2px' } as const;
/** The same ring drawn inside the box, for items in a scrolling or clipped list. */
const ringInset = { ...ringOutline, outlineOffset: `calc(-1 * ${vars.a11y.focusRingWidth})` } as const;

export const srOnly = style({
  position: 'absolute',
  width: '1px',
  height: '1px',
  padding: 0,
  margin: '-1px',
  overflow: 'hidden',
  clip: 'rect(0, 0, 0, 0)',
  whiteSpace: 'nowrap',
  border: 0,
});

globalStyle('[dir="rtl"] .icon--directional', { transform: 'scaleX(-1)' });

// ── Frame ──────────────────────────────────────────────────────────────────────
export const frame = style({
  display: 'grid',
  gridTemplateColumns: 'minmax(180px, min(var(--admin-sidebar-width, 220px), 40vw)) 6px minmax(0, 1fr)',
  minHeight: '100vh',
  background: vars.color.bg,
  color: vars.color.text,
  fontFamily: vars.font.ui,
  '@media': {
    [PHONE]: { gridTemplateColumns: 'minmax(0, 1fr)', gridTemplateRows: 'auto 1fr' },
  },
  selectors: { '&[data-sidebar-collapsed]': { gridTemplateColumns: 'minmax(0, 1fr)' } },
});

export const sidebarResize = style({
  position: 'sticky', top: 0, height: '100vh',
  selectors: { [`${frame}[data-sidebar-collapsed] &`]: { display: 'none' } },
  '@media': { [PHONE]: { display: 'none' } },
});
export const sidebarToggle = style({
  flexShrink: 0,
  '@media': { [PHONE]: { display: 'none' } },
});

/** The desktop sidebar. Its 3px rule takes 2px from the inline-end padding (P1). */
export const sidebar = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  padding: vars.space[4],
  paddingInlineEnd: `calc(${vars.space[4]} - 2px)`,
  borderInlineEnd: `${vars.border.heavy} solid ${vars.color.borderStrong}`,
  background: vars.color.bgAlt,
  maxHeight: '100vh',
  overflowY: 'auto',
  position: 'sticky',
  top: 0,
  selectors: { [`${frame}[data-sidebar-collapsed] &`]: { display: 'none' } },
  '@media': { [PHONE]: { display: 'none' } },
});

export const brand = style({
  fontWeight: vars.weight.heavy,
  fontSize: '1.05rem',
  marginBottom: vars.space[3],
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[2],
});

/** The Bauhaus mark before the brand: yellow square, red circle, blue triangle. */
export const brandMark = style({
  display: 'inline-flex',
  gap: '2px',
  alignItems: 'center',
});
globalStyle(`${brandMark} > span`, { width: '8px', height: '8px', display: 'block' });
globalStyle(`${brandMark} > span:nth-child(1)`, { background: vars.color.signal });
globalStyle(`${brandMark} > span:nth-child(2)`, { background: vars.color.danger, borderRadius: vars.radius.pill });
globalStyle(`${brandMark} > span:nth-child(3)`, {
  background: vars.color.accent,
  clipPath: 'polygon(50% 0, 100% 100%, 0 100%)',
});

export const nav = style({ display: 'flex', flexDirection: 'column', gap: vars.space[3] });

export const group = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1] });

export const groupLabel = style({
  ...smallLabel,
  fontSize: '0.7rem',
  color: vars.color.textDim,
  margin: 0,
  paddingInline: vars.space[3],
  selectors: { [noTrackingScripts]: plainLabel },
});

export const navList = style({ listStyle: 'none', margin: 0, padding: 0, display: 'flex', flexDirection: 'column', gap: vars.space[1] });

/** A sidebar entry: icon + label, height ≤ the pre-t32 admin `navItem`. */
export const navItem = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[3],
  minHeight: vars.a11y.touchTarget,
  padding: `${vars.space[2]} ${vars.space[3]}`,
  color: vars.color.text,
  textDecoration: 'none',
  borderRadius: vars.radius.md,
  transition: `background ${vars.a11y.motionDuration} ease`,
  selectors: {
    '&:hover': { background: vars.color.bg },
    '&:focus-visible': ringInset,
    // Selected: an accent inset bar on the start edge plus bold (P6), no layout change.
    '&[aria-current="page"]': {
      boxShadow: `inset 4px 0 0 ${vars.color.accent}`,
      fontWeight: vars.weight.bold,
      background: vars.color.bg,
    },
    '[dir="rtl"] &[aria-current="page"]': { boxShadow: `inset -4px 0 0 ${vars.color.accent}` },
  },
});
globalStyle(`${navItem} svg`, { flex: 'none' });

export const sidebarFooter = style({
  marginTop: 'auto',
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  paddingTop: vars.space[4],
});

export const iconButton = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[2],
});

// ── Phone top bar + drawer ─────────────────────────────────────────────────────
export const topbar = style({
  display: 'none',
  '@media': {
    [PHONE]: {
      display: 'flex',
      alignItems: 'center',
      gap: vars.space[3],
      padding: `${vars.space[2]} ${vars.space[4]}`,
      paddingBlockEnd: `calc(${vars.space[2]} - 2px)`,
      background: vars.color.bgAlt,
      borderBottom: `${vars.border.heavy} solid ${vars.color.borderStrong}`,
      position: 'sticky',
      top: 0,
      zIndex: 5,
    },
  },
});

export const topbarTitle = style({
  fontWeight: vars.weight.heavy,
  flex: 1,
  minWidth: 0,
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
});

/** Phone touch targets stay at least 40px (narrow.spec.ts). Scoped under the
 *  console root so it outranks admin.css.ts's 24px `.btn` minimum. */
export const phoneTarget = style({
  '@media': {
    [PHONE]: {
      selectors: {
        '[data-screen="admin"] &': { minHeight: '40px', minWidth: '40px', justifyContent: 'center' },
      },
    },
  },
});

export const scrim = style({
  position: 'fixed',
  inset: 0,
  background: vars.color.scrim,
  zIndex: 20,
});

export const drawer = style({
  position: 'fixed',
  insetBlock: 0,
  insetInlineStart: 0,
  width: 'min(85vw, 300px)',
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  padding: vars.space[4],
  background: vars.color.bgAlt,
  borderInlineEnd: `${vars.border.heavy} solid ${vars.color.borderStrong}`,
  boxShadow: vars.elevation[3],
  overflowY: 'auto',
  zIndex: 21,
});
globalStyle(`${drawer} ${navItem}`, { '@media': { [PHONE]: { minHeight: '40px' } } });

export const drawerHead = style({ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: vars.space[3] });

// ── Main column ────────────────────────────────────────────────────────────────
export const main = style({
  position: 'relative',
  padding: vars.space[6],
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[6],
  overflowY: 'auto',
  maxHeight: '100vh',
  minWidth: 0,
  '@media': { [PHONE]: { padding: vars.space[5], maxHeight: 'none' } },
});

export const mainHead = style({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  gap: vars.space[5],
  flexWrap: 'wrap',
});

export const kbd = style({
  fontFamily: vars.font.mono,
  fontSize: '0.75rem',
  color: vars.color.textDim,
});

// ── Breadcrumbs ────────────────────────────────────────────────────────────────
export const crumbs = style({ fontSize: '0.8rem' });
export const crumbList = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[2], listStyle: 'none', margin: 0, padding: 0 });
export const crumbItem = style({ display: 'inline-flex', alignItems: 'center', gap: vars.space[2], color: vars.color.textDim });
globalStyle(`${crumbItem} a`, { color: vars.color.link, textDecoration: 'underline' });
globalStyle(`${crumbItem} [aria-current="page"]`, { color: vars.color.text, fontWeight: vars.weight.bold });

// ── Dialogs (confirm, leave, palette) ──────────────────────────────────────────
export const dialog = style({
  position: 'fixed',
  insetBlockStart: '12vh',
  insetInline: 0,
  marginInline: 'auto',
  width: 'min(calc(100vw - 32px), 480px)',
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: `calc(${vars.space[5]} - 1px)`,
  background: vars.color.surface,
  color: vars.color.text,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.lg,
  boxShadow: vars.elevation[3],
  zIndex: 31,
  maxHeight: '76vh',
  overflowY: 'auto',
});
export const dialogScrim = style([scrim, { zIndex: 30 }]);
export const dialogTitle = style({ margin: 0, fontSize: '1.05rem', fontWeight: vars.weight.heavy });
export const dialogActions = style({ display: 'flex', justifyContent: 'flex-end', gap: vars.space[3], flexWrap: 'wrap' });

// ── Command palette ────────────────────────────────────────────────────────────
export const palette = style([dialog, { insetBlockStart: '10vh', width: 'min(calc(100vw - 32px), 560px)' }]);
export const paletteList = style({ listStyle: 'none', margin: 0, padding: 0, display: 'flex', flexDirection: 'column' });
export const paletteOption = style({
  display: 'flex',
  alignItems: 'center',
  gap: vars.space[3],
  padding: `${vars.space[2]} ${vars.space[3]}`,
  minHeight: vars.a11y.touchTarget,
  cursor: 'pointer',
  selectors: {
    // The input keeps focus (aria-activedescendant); this applies if an option ever takes it.
    '&:focus-visible': ringInset,
    '&[aria-selected="true"]': {
      background: vars.color.bgAlt,
      boxShadow: `inset 4px 0 0 ${vars.color.accent}`,
      fontWeight: vars.weight.bold,
    },
    '[dir="rtl"] &[aria-selected="true"]': { boxShadow: `inset -4px 0 0 ${vars.color.accent}` },
  },
});
export const paletteKind = style({
  ...smallLabel,
  marginInlineStart: 'auto',
  fontSize: '0.7rem',
  color: vars.color.textDim,
  selectors: { [noTrackingScripts]: plainLabel },
});

// ── Form field, help, feedback ─────────────────────────────────────────────────
export const field = style({ display: 'flex', flexDirection: 'column', gap: vars.space[3], minInlineSize: 0 });
export const fieldLabel = style({
  ...smallLabel,
  fontSize: '0.75rem',
  selectors: { [noTrackingScripts]: plainLabel },
});
export const fieldTech = style({ fontWeight: vars.weight.regular, color: vars.color.textDim, textTransform: 'none', letterSpacing: vars.tracking.none });
export const help = style({ margin: 0, fontSize: '0.85rem', lineHeight: 1.5, maxInlineSize: '72ch', color: vars.color.textDim });
export const fieldError = style({
  margin: 0,
  fontSize: '0.8rem',
  color: vars.color.danger,
  fontWeight: vars.weight.medium,
  display: 'flex',
  gap: vars.space[2],
  alignItems: 'center',
});

/** A feedback block: flat surface, 2px rule, an inset bar in the state colour and an icon (P6). */
export const feedback = style({
  display: 'flex',
  alignItems: 'flex-start',
  gap: vars.space[3],
  padding: `calc(${vars.space[3]} - 1px) calc(${vars.space[4]} - 1px)`,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  margin: 0,
});
export const feedbackOk = style({ boxShadow: `inset 4px 0 0 ${vars.color.success}` });
export const feedbackErr = style({ boxShadow: `inset 4px 0 0 ${vars.color.danger}` });
export const feedbackInfo = style({ boxShadow: `inset 4px 0 0 ${vars.color.accent}` });
globalStyle(`${feedbackOk} svg`, { color: vars.color.success, flex: 'none', marginTop: '2px' });
globalStyle(`${feedbackErr} svg`, { color: vars.color.danger, flex: 'none', marginTop: '2px' });
globalStyle(`${feedbackInfo} svg`, { color: vars.color.accent, flex: 'none', marginTop: '2px' });

/** An empty, loading or unavailable state (t32.md §2.4 "Empty / loading / error"). */
export const state = style({
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'flex-start',
  gap: vars.space[3],
  padding: `calc(${vars.space[4]} - 1px)`,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
});
export const stateError = style({ boxShadow: `inset 4px 0 0 ${vars.color.danger}` });

// ── Data table ─────────────────────────────────────────────────────────────────
// Tables are page content, not labelled controls: never inherit field max width.
export const tableContainer = style({ display: 'flex', flexDirection: 'column', gap: vars.space[3], minInlineSize: 0, inlineSize: '100%' });
export const tableToolbar = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[2], alignItems: 'flex-end', justifyContent: 'flex-start' });
globalStyle(`${tableToolbar} > form`, { flex: '0 1 22rem', maxInlineSize: '100%', margin: 0 });
export const chips = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[2] });
export const chip = style({
  appearance: 'none',
  font: 'inherit',
  fontSize: '0.8rem',
  minHeight: vars.a11y.touchTarget,
  padding: `${vars.space[1]} ${vars.space[3]}`,
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  background: 'transparent',
  color: vars.color.text,
  cursor: 'pointer',
  selectors: {
    '&:focus-visible': ringOutline,
    '&[aria-pressed="true"]': {
      background: vars.color.accent,
      color: vars.color.accentText,
      borderColor: vars.color.accent,
      fontWeight: vars.weight.bold,
    },
  },
});
export const sortButton = style({
  appearance: 'none',
  font: 'inherit',
  color: 'inherit',
  letterSpacing: 'inherit',
  textTransform: 'inherit',
  fontWeight: 'inherit',
  background: 'transparent',
  border: 0,
  padding: 0,
  cursor: 'pointer',
  display: 'inline-flex',
  gap: vars.space[1],
  alignItems: 'center',
  selectors: { '&:focus-visible': ringOutline },
});
export const rowFocus = style({
  selectors: { '&:focus-visible': { outlineOffset: '-3px' } },
});
export const pager = style({ display: 'flex', gap: vars.space[3], alignItems: 'center', justifyContent: 'flex-end' });
export const rowActions = style({ display: 'flex', gap: vars.space[2], flexWrap: 'wrap' });

// ── Stat tiles and charts ──────────────────────────────────────────────────────
export const tiles = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fill, minmax(168px, 1fr))',
  gap: vars.space[3],
});
export const tile = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[1],
  padding: `calc(${vars.space[4]} - 1px)`,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  margin: 0,
});
export const tileLabel = style({
  ...smallLabel,
  fontSize: '0.7rem',
  color: vars.color.textDim,
  margin: 0,
  selectors: { [noTrackingScripts]: plainLabel },
});
export const tileValue = style({
  fontSize: '1.5rem',
  fontWeight: vars.weight.heavy,
  margin: 0,
  fontVariantNumeric: 'tabular-nums',
});
export const tileNa = style({ fontSize: '0.9rem', fontWeight: vars.weight.bold, margin: 0 });
export const tileDetail = style({ fontSize: '0.75rem', color: vars.color.textDim, margin: 0 });
globalStyle(`${tile} a`, { color: vars.color.link, textDecoration: 'underline', fontSize: '0.8rem' });

export const chart = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2], minWidth: 0 });
export const chartTitle = style({ margin: 0, fontSize: '0.85rem', fontWeight: vars.weight.bold });
export const chartSvg = style({ display: 'block', width: '100%', height: '48px', overflow: 'visible' });
export const chartLine = style({ fill: 'none', strokeWidth: 2, stroke: vars.color.accent, vectorEffect: 'non-scaling-stroke' });
export const chartLineDanger = style({ stroke: vars.color.danger });
export const chartBase = style({ stroke: vars.color.border, strokeWidth: 1, vectorEffect: 'non-scaling-stroke' });
export const chartHit = style({ fill: 'transparent' });
export const chartDot = style({ fill: vars.color.text });
export const chartSummary = style({ margin: 0, fontSize: '0.75rem', color: vars.color.textDim, fontVariantNumeric: 'tabular-nums' });
export const chartTip = style({
  fontSize: '0.75rem',
  fontVariantNumeric: 'tabular-nums',
  minHeight: '1.2em',
  margin: 0,
});

export const bars = style({ display: 'flex', flexDirection: 'column', gap: vars.space[2], margin: 0, padding: 0, listStyle: 'none' });
export const barRow = style({
  display: 'grid',
  gridTemplateColumns: 'minmax(0, 10rem) minmax(0, 1fr) auto',
  gap: vars.space[3],
  alignItems: 'center',
  fontSize: '0.8rem',
});
export const barLabel = style({ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' });
export const barTrack = style({ height: '10px', background: vars.color.bgAlt, border: `${vars.border.hair} solid ${vars.color.border}` });
export const barFill = style({ height: '100%', background: vars.color.accent });
/** Bar lengths as classes, 0-100 % in whole percent: no style attribute (CSP `style-src 'self'`). */
export const barWidth = styleVariants(
  Object.fromEntries(Array.from({ length: 101 }, (_, i) => [i, { width: `${i}%` }])) as Record<number, { width: string }>,
);
export const barValue = style({ fontVariantNumeric: 'tabular-nums', fontWeight: vars.weight.bold });

// ── Pinned badge, lock switch, help note ───────────────────────────────────────
export const pinned = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[2],
  fontSize: '0.75rem',
  padding: `1px ${vars.space[2]}`,
  border: `${vars.border.hair} dashed ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  color: vars.color.textDim,
});
export const pinnedVar = style({ fontFamily: vars.font.mono, color: vars.color.text });

export const lock = style({
  appearance: 'none',
  font: 'inherit',
  fontSize: '0.8rem',
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[2],
  minHeight: vars.a11y.touchTarget,
  padding: `${vars.space[1]} ${vars.space[2]}`,
  background: 'transparent',
  color: vars.color.text,
  border: `${vars.border.hair} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.sm,
  cursor: 'pointer',
  selectors: {
    '&:focus-visible': ringOutline,
    '&:disabled': { opacity: 0.6, borderStyle: 'dashed', cursor: 'not-allowed' },
  },
});
/** The switch track: the knob's position and fill both change, not only colour (P6). */
export const lockTrack = style({
  position: 'relative',
  width: '28px',
  height: '14px',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  flex: 'none',
  selectors: {
    [`${lock}[aria-checked="true"] &`]: { background: vars.color.accent, borderColor: vars.color.accent },
  },
});
export const lockKnob = style({
  position: 'absolute',
  insetBlock: '1px',
  insetInlineStart: '1px',
  width: '8px',
  background: vars.color.text,
  selectors: {
    [`${lock}[aria-checked="true"] &`]: { insetInlineStart: 'auto', insetInlineEnd: '1px', background: vars.color.accentText },
  },
});

export const helpNote = style({
  margin: 0,
  fontSize: '0.8rem',
  color: vars.color.textDim,
  display: 'flex',
  gap: vars.space[2],
  alignItems: 'flex-start',
});
globalStyle(`${helpNote} svg`, { flex: 'none', marginTop: '2px' });

// ── Wizard ─────────────────────────────────────────────────────────────────────
export const wizard = style({ display: 'flex', flexDirection: 'column', gap: vars.space[4] });
export const steps = style({ display: 'flex', flexWrap: 'wrap', gap: vars.space[2], listStyle: 'none', margin: 0, padding: 0 });
export const stepItem = style({
  ...smallLabel,
  fontSize: '0.7rem',
  padding: `${vars.space[1]} ${vars.space[2]}`,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  color: vars.color.textDim,
  selectors: {
    '&[aria-current="step"]': {
      color: vars.color.text,
      borderColor: vars.color.borderStrong,
      boxShadow: `inset 0 -3px 0 ${vars.color.accent}`,
    },
    [noTrackingScripts]: plainLabel,
  },
});
export const stepDone = style({ color: vars.color.text });
export const reviewList = style({ display: 'grid', gridTemplateColumns: 'minmax(0, 14rem) minmax(0, 1fr)', gap: `${vars.space[2]} ${vars.space[4]}`, margin: 0 });
globalStyle(`${reviewList} dt`, { color: vars.color.textDim });
globalStyle(`${reviewList} dd`, { margin: 0, fontWeight: vars.weight.medium, overflowWrap: 'anywhere' });
export const wizardActions = style({ display: 'flex', gap: vars.space[3], justifyContent: 'space-between', flexWrap: 'wrap' });

const spin = keyframes({ to: { transform: 'rotate(360deg)' } });
export const busy = style({
  display: 'inline-block',
  width: '10px',
  height: '10px',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderInlineEndColor: 'transparent',
  animation: `${spin} 800ms linear infinite`,
  '@media': { '(prefers-reduced-motion: reduce)': { animation: 'none' } },
});

registerFieldLayout(field);

/** Reserve refresh feedback space so retained table rows never jump. */
export const tableRefresh = style({ minBlockSize: '44px' });
globalStyle(`${tableRefresh} > .async-state`, { marginBlock: 0, minBlockSize: '44px' });
