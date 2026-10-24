// Admin panel base styles (plan §3 e7; restyled by t32-e8 on the t32 tokens).
//
// Token-native scoped classes over the theme contract (`vars.*`), shared by the
// section screens. The console frame (sidebar, top bar, drawer) and the shared
// components are in `shell/shell.css.ts`. Zero-runtime: vanilla-extract compiles
// these to static CSS at build, inside the lazily loaded admin chunk.
//
// t32 rules (t32.md §2.4): square corners (radius tokens), borders from the
// border tokens, a thicker border takes its width out of the padding (P1), no
// size grows (P2), flat fills and hard shadows only (P3), headings heavy and
// small labels in tracked capitals (P5), state never by colour alone (P6).
// Logical properties throughout, so the layout mirrors under `dir=rtl`.

import { style, globalStyle } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

/** Small-label treatment (P5): tracked capitals, off for scripts it breaks. */
const smallLabel = {
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
  fontWeight: vars.weight.bold,
} as const;

/** Full-page sign-in gate and boot spinner. */
export const gate = style({
  display: 'grid',
  placeItems: 'center',
  minHeight: '100vh',
  background: vars.color.bg,
  color: vars.color.text,
  fontFamily: vars.font.ui,
});

export const section = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  maxWidth: '760px',
});

export const heading = style({
  fontSize: '1.25rem',
  fontWeight: vars.weight.heavy,
  margin: 0,
});

/** A bordered panel. The 2px rule takes 1px from each padding side (P1). */
export const card = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  padding: `calc(${vars.space[4]} - 1px)`,
  borderRadius: vars.radius.lg,
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: vars.color.surface,
});

export const listRow = style({
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'center',
  gap: vars.space[3],
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderRadius: vars.radius.md,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  background: vars.color.bg,
});

export const grid = style({
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
  gap: vars.space[3],
});

/** A status badge: always carries its text (P6), square (P4), tracked (P5). */
export const badge = style({
  ...smallLabel,
  display: 'inline-block',
  padding: `1px ${vars.space[2]}`,
  borderRadius: vars.radius.sm,
  fontSize: '0.7rem',
  background: vars.color.bgAlt,
  color: vars.color.textDim,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  selectors: {
    ':lang(ar) &, :lang(ja) &, :lang(zh) &': { textTransform: 'none', letterSpacing: vars.tracking.none },
  },
});

export const badgeDeferred = style({
  background: 'transparent',
  color: vars.color.textDim,
  fontStyle: 'italic',
});

export const table = style({
  width: '100%',
  borderCollapse: 'collapse',
  fontSize: '0.85rem',
});

globalStyle(`${table} th, ${table} td`, {
  textAlign: 'start',
  padding: `${vars.space[2]} ${vars.space[3]}`,
  borderBottom: `${vars.border.hair} solid ${vars.color.border}`,
  verticalAlign: 'top',
});

// Header row: a 2px rule under it (the 1px it adds comes out of the padding, P1),
// header cells in tracked capitals at a smaller size (P5).
globalStyle(`${table} th`, {
  ...smallLabel,
  color: vars.color.textDim,
  fontSize: '0.75rem',
  whiteSpace: 'nowrap',
  borderBottom: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  paddingBottom: `calc(${vars.space[2]} - 1px)`,
});

globalStyle(`:lang(ar) ${table} th, :lang(ja) ${table} th, :lang(zh) ${table} th`, {
  textTransform: 'none',
  letterSpacing: vars.tracking.none,
});

globalStyle(`${table} tbody tr:hover`, {
  background: vars.color.bgAlt,
});

export const tableWrap = style({
  overflowX: 'auto',
  width: '100%',
});

export const note = style({
  fontSize: '0.8rem',
  color: vars.color.textDim,
});

/** An error line: text plus the danger colour, never colour alone (P6). */
export const error = style({
  color: vars.color.danger,
  fontSize: '0.85rem',
  fontWeight: vars.weight.medium,
});

export const mono = style({
  fontFamily: vars.font.mono,
  fontSize: '0.8rem',
  whiteSpace: 'pre-wrap',
  wordBreak: 'break-all',
});

// ── Sign-in card (t32.md §2.4 "Login / admin login") ───────────────────────────
// surface, 2px rule in borderStrong, the heaviest hard shadow, square, plus the
// decorative Bauhaus mark as a pseudo-element inside the existing padding.
export const loginCard = style([
  card,
  {
    position: 'relative',
    boxShadow: vars.elevation[3],
    selectors: {
      '&::before': {
        content: '""',
        position: 'absolute',
        insetBlockStart: vars.space[3],
        insetInlineEnd: vars.space[3],
        width: '10px',
        height: '10px',
        background: vars.color.signal,
      },
      '&::after': {
        content: '""',
        position: 'absolute',
        insetBlockStart: vars.space[3],
        insetInlineEnd: `calc(${vars.space[3]} + 14px)`,
        width: '10px',
        height: '10px',
        borderRadius: vars.radius.pill,
        background: vars.color.danger,
      },
    },
  },
]);

export const loginActions = style({
  display: 'flex',
  gap: vars.space[3],
  flexWrap: 'wrap',
});

// ── Shared a11y layer (WCAG 2.2) ───────────────────────────────────────────────
// The `.btn` / `.field` controls come from the app stylesheet; the focus outline is
// the global 3px `:focus-visible` rule from themes.css.ts. These scoped rules keep
// the 24×24 minimum target inside the console and its sign-in gate.
globalStyle(`[data-screen="admin"] .btn, ${gate} .btn`, {
  minHeight: vars.a11y.touchTarget,
});

// Field labels in the console: the app's tracked capitals at 0.75rem instead of
// 0.8rem, the same as the shell's FormField. In capitals a label such as "Quota
// messages (0 = unlimited)" wrapped to two lines in the Users form's columns at
// 0.8rem and made the form taller than before t32 (P2).
globalStyle(`[data-screen="admin"] .field > span`, {
  fontSize: '0.75rem',
});

/** A danger button (t32.md §2.4 "Buttons"): danger fill, accentText, no shadow on press. */
globalStyle(`[data-screen="admin"] .btn--danger, ${gate} .btn--danger`, {
  background: vars.color.danger,
  borderColor: vars.color.danger,
  color: vars.color.accentText,
});

// ── former inline styles (t32-h12) ──────────────────────────────────────────
// Static `style={{…}}` objects compile into the template as a literal `style=`
// attribute, which the shell CSP (`style-src 'self'`) drops; these are the same
// declarations as classes.

/** A confirmation dialog box (unattended keys, re-thread). */
export const confirmDialog = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  maxWidth: '480px',
  width: '100%',
  padding: vars.space[5],
  background: vars.color.surface,
  color: vars.color.text,
  border: `${vars.border.hair} solid ${vars.color.border}`,
  borderRadius: vars.radius.lg,
});
/** The buttons at the foot of a confirmation dialog, at the end of the row. */
export const confirmActions = style({ display: 'flex', gap: vars.space[3], justifyContent: 'flex-end' });
/** A card's heading row with an action at the other end. */
export const headingRow = style({ display: 'flex', justifyContent: 'space-between', alignItems: 'center' });
/** A sub-heading inside a card. */
export const subHeading = style({ fontSize: '1rem' });
