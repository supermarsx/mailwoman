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

export const root = style({
  fontFamily: vars.font.ui,
  fontSize: vars.fontSize.base,
  color: vars.color.text,
});

// ── Collapsed verdict chip ───────────────────────────────────────────────────

/** Square chip with the 2px control rule (1px back from each padding); the
 *  border takes the verdict's tone colour, the dot and the label repeat it, and
 *  hover lifts a hard shadow. Focus is the global outline. */
export const chip = style({
  appearance: 'none',
  display: 'inline-flex',
  alignItems: 'center',
  gap: vars.space[2],
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.sm,
  cursor: 'pointer',
  padding: `calc(${vars.space[1]} - 1px) calc(${vars.space[3]} - 1px)`,
  font: 'inherit',
  fontSize: '0.85rem',
  lineHeight: 1.4,
  maxWidth: '100%',
  minHeight: vars.a11y.touchTarget,
  selectors: {
    '&:hover': { boxShadow: vars.elevation[1] },
  },
});

export const chipTone = styleVariants(TONE_COLOR, (color) => ({
  borderColor: color,
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
  marginBlockStart: vars.space[2],
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  borderRadius: vars.radius.md,
  background: vars.color.surface,
  boxShadow: vars.elevation[2],
  padding: `calc(${vars.space[4]} - 1px)`,
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
  borderBlockStart: `${vars.border.hair} solid ${vars.color.border}`,
  paddingBlockStart: vars.space[3],
  selectors: {
    '&:first-of-type': { borderBlockStart: 'none', paddingBlockStart: 0 },
  },
});

export const sectionTitle = style({
  margin: 0,
  fontSize: '0.72rem',
  fontWeight: vars.weight.bold,
  textTransform: 'uppercase',
  letterSpacing: vars.tracking.label,
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
  fontWeight: vars.weight.bold,
  border: `${vars.border.hair} solid transparent`,
  whiteSpace: 'nowrap',
});

// Flat: the tone colour for the text, the glyph and a hair border, no tinted
// (translucent) fill.
export const badgeTone = styleVariants(TONE_COLOR, (color) => ({ color, borderColor: color }));

// ── Auth grid ────────────────────────────────────────────────────────────────

export const authRow = style({
  display: 'grid',
  gridTemplateColumns: 'auto 1fr',
  alignItems: 'baseline',
  gap: vars.space[3],
});

export const authName = style({
  fontWeight: vars.weight.bold,
  color: vars.color.text,
});

export const authDetail = style({
  color: vars.color.textDim,
  fontSize: '0.8rem',
  fontFamily: vars.font.mono,
  overflowWrap: 'anywhere',
});

// ── Received chain ───────────────────────────────────────────────────────────

// Long hops scroll sideways. `overflow-x: auto` also makes the block axis a
// scroll axis, and the 1.2 line height leaves the font's content area 2px past
// the last line box, which made the chain a keyboard-unreachable scroll region
// (axe scrollable-region-focusable). Glyph ink stays inside the line box.
export const hopScroll = style({
  overflowX: 'auto',
  overflowY: 'hidden',
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
  textAlign: 'end',
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
  fontWeight: vars.weight.bold,
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
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  background: vars.color.bg,
  color: vars.color.text,
  borderRadius: vars.radius.md,
  cursor: 'pointer',
  padding: `calc(${vars.space[1]} - 1px) calc(${vars.space[3]} - 1px)`,
  font: 'inherit',
  fontSize: '0.82rem',
  minHeight: vars.a11y.touchTarget,
  selectors: {
    '&:hover:not(:disabled)': { boxShadow: vars.elevation[1] },
    '&:disabled': { opacity: 0.6, cursor: 'default', borderStyle: 'dashed' },
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
