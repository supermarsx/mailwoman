// Security-panel styles (plan §3 e3), built entirely on the frozen token
// contract (`theme/contract.css.ts`) so the panel themes with the rest of the
// chrome (light/dark/hc/amoled/grove) — the "new components use vars.* directly"
// path, plan §2.3. Zero-runtime: compiled to static CSS by vanilla-extract.

import { style, styleVariants } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

/** Per-tone colour, shared by the chip, badges and section accents. */
const TONE_COLOR = {
  good: vars.color.success,
  warning: vars.color.warning,
  bad: vars.color.danger,
  neutral: vars.color.textDim,
} as const;

/**
 * Per-tone GLYPH rendered before each badge's text via `::before` (WCAG 1.4.1):
 * a redundant, non-colour shape so pass/fail/warn is legible without colour and
 * under forced-colors. Kept out of the DOM (CSS content) so it never merges into
 * the badge's text node — literal-text assertions ("DKIM passed") stay intact.
 */
const TONE_GLYPH: Record<keyof typeof TONE_COLOR, string> = {
  good: '✓',
  warning: '!',
  bad: '✕',
  neutral: '–',
};

export const root = style({
  fontFamily: vars.font.ui,
  fontSize: vars.fontSize.base,
  color: vars.color.text,
});

// ── Collapsed verdict chip ───────────────────────────────────────────────────

export const chip = style({
  appearance: 'none',
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[2],
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.pill,
  cursor: 'pointer',
  padding: `${vars.space[1]} ${vars.space[3]}`,
  font: 'inherit',
  fontSize: '0.85rem',
  lineHeight: 1.4,
  maxWidth: '100%',
  minHeight: vars.a11y.touchTarget,
  selectors: {
    '&:hover': { borderColor: vars.color.accent },
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
  },
});

export const chipTone = styleVariants(TONE_COLOR, (color) => ({
  borderColor: `color-mix(in srgb, ${color} 45%, ${vars.color.border})`,
}));

export const chipDot = style({
  flex: '0 0 auto',
  width: '0.6rem',
  height: '0.6rem',
  borderRadius: vars.radius.pill,
});

export const chipDotTone = styleVariants(TONE_COLOR, (color) => ({ background: color }));

export const chipLabel = style({
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
});

export const chipCaret = style({
  flex: '0 0 auto',
  color: vars.color.textDim,
  fontSize: '0.7rem',
});

// ── Expanded panel ───────────────────────────────────────────────────────────

export const panel = style({
  marginTop: vars.space[2],
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  boxShadow: vars.elevation[2],
  padding: vars.space[4],
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[4],
  maxWidth: '32rem',
});

export const summary = style({
  margin: 0,
  color: vars.color.text,
  fontSize: '0.95rem',
});

export const section = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  borderTop: `1px solid ${vars.color.border}`,
  paddingTop: vars.space[3],
  selectors: {
    '&:first-of-type': { borderTop: 'none', paddingTop: 0 },
  },
});

export const sectionTitle = style({
  margin: 0,
  fontSize: '0.72rem',
  fontWeight: 700,
  textTransform: 'uppercase',
  letterSpacing: '0.05em',
  color: vars.color.textDim,
});

export const empty = style({
  margin: 0,
  color: vars.color.textDim,
  fontSize: '0.85rem',
});

// ── Tone badge (auth results, signature facets, attachment risk) ─────────────

export const badge = style({
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[1],
  borderRadius: vars.radius.sm,
  padding: `${vars.space[0]} ${vars.space[2]}`,
  fontSize: '0.78rem',
  fontWeight: 600,
  border: '1px solid transparent',
  whiteSpace: 'nowrap',
});

export const badgeTone = styleVariants(TONE_COLOR, (color, key) => ({
  color,
  borderColor: `color-mix(in srgb, ${color} 40%, transparent)`,
  background: `color-mix(in srgb, ${color} 12%, transparent)`,
  '::before': {
    content: `"${TONE_GLYPH[key]}\\00a0"`,
    fontWeight: 700,
  },
}));

// ── Auth grid ────────────────────────────────────────────────────────────────

export const authRow = style({
  display: 'grid',
  gridTemplateColumns: 'auto 1fr',
  alignItems: 'baseline',
  gap: vars.space[3],
});

export const authName = style({
  fontWeight: 600,
  color: vars.color.text,
});

export const authDetail = style({
  color: vars.color.textDim,
  fontSize: '0.8rem',
  fontFamily: vars.font.mono,
  overflowWrap: 'anywhere',
});

// ── Received chain ───────────────────────────────────────────────────────────

export const hopScroll = style({
  overflowX: 'auto',
});

export const hopList = style({
  listStyle: 'none',
  margin: 0,
  padding: 0,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
});

export const hop = style({
  display: 'flex',
  alignItems: 'baseline',
  gap: vars.space[3],
  fontSize: '0.82rem',
});

export const hopIndex = style({
  flex: '0 0 auto',
  width: '1.5rem',
  textAlign: 'right',
  color: vars.color.textDim,
  fontFamily: vars.font.mono,
});

export const hopBody = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[0],
  minWidth: 0,
});

export const hopHost = style({
  color: vars.color.text,
  overflowWrap: 'anywhere',
});

export const hopMeta = style({
  color: vars.color.textDim,
  fontSize: '0.76rem',
  display: 'flex',
  flexWrap: 'wrap',
  gap: vars.space[2],
});

// ── Signature grid ───────────────────────────────────────────────────────────

export const factGrid = style({
  display: 'grid',
  gridTemplateColumns: 'auto 1fr',
  gap: `${vars.space[1]} ${vars.space[3]}`,
  fontSize: '0.82rem',
  margin: 0,
});

export const factKey = style({
  color: vars.color.textDim,
});

export const factVal = style({
  color: vars.color.text,
  fontFamily: vars.font.mono,
  overflowWrap: 'anywhere',
});

// ── Attachments ──────────────────────────────────────────────────────────────

export const list = style({
  listStyle: 'none',
  margin: 0,
  padding: 0,
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
});

export const attachItem = style({
  display: 'flex',
  alignItems: 'center',
  flexWrap: 'wrap',
  gap: vars.space[2],
});

export const attachName = style({
  fontWeight: 600,
  overflowWrap: 'anywhere',
});

export const mismatchNote = style({
  color: vars.color.warning,
  fontSize: '0.76rem',
});

// ── Anomalies ────────────────────────────────────────────────────────────────

export const anomalyItem = style({
  display: 'flex',
  alignItems: 'baseline',
  gap: vars.space[2],
  color: vars.color.text,
  fontSize: '0.84rem',
});

export const anomalyMark = style({
  color: vars.color.warning,
  flex: '0 0 auto',
});

// ── Sender controls ──────────────────────────────────────────────────────────

export const controls = style({
  display: 'flex',
  flexWrap: 'wrap',
  gap: vars.space[2],
});

export const controlBtn = style({
  appearance: 'none',
  border: `1px solid ${vars.color.border}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `${vars.space[1]} ${vars.space[3]}`,
  font: 'inherit',
  fontSize: '0.82rem',
  minHeight: vars.a11y.touchTarget,
  selectors: {
    '&:hover:not(:disabled)': { borderColor: vars.color.accent },
    '&:focus-visible': { outline: 'none', boxShadow: vars.a11y.focusRing },
    '&:disabled': { opacity: 0.6, cursor: 'default' },
  },
});

export const controlBtnDanger = style({
  selectors: {
    '&:hover:not(:disabled)': { borderColor: vars.color.danger, color: vars.color.danger },
  },
});

export const statusLive = style({
  color: vars.color.success,
  fontSize: '0.8rem',
  minHeight: '1.1rem',
});
