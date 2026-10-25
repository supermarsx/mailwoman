import { globalStyle } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

// Checkboxes and radios (t32-h12). One rule for every native checkbox and radio in the
// mail client and the admin console, so no screen is left with the 13 px browser
// default next to another screen's 24 px box:
//   - the box is the 24×24 target (`vars.a11y.touchTarget`, WCAG 2.2 §2.5.8) on every
//     screen; geometric theme glyphs normally, native controls in forced colours;
//   - the tick and fill take the theme accent;
//   - a fixed size, no margin and no shrinking: a flex column cannot stretch it across
//     the row (Chrome then centres a small glyph in the stretched box) and a flex row
//     cannot squeeze it;
//   - the focus ring is the global `:focus-visible` outline (themes.css.ts).
// The `input[type]` selector outranks a single module class, so per-screen checkbox
// classes cannot bring back another size.
const CHECKS = 'input[type="checkbox"], input[type="radio"]';

globalStyle(CHECKS, {
  accentColor: vars.color.accent,
  inlineSize: vars.a11y.touchTarget,
  blockSize: vars.a11y.touchTarget,
  minInlineSize: vars.a11y.touchTarget,
  minBlockSize: vars.a11y.touchTarget,
  margin: 0,
  padding: 0,
  flex: 'none',
  verticalAlign: 'middle',
  cursor: 'pointer',
});

globalStyle(CHECKS, {
  appearance: 'none',
  display: 'inline-grid',
  placeContent: 'center',
  border: `${vars.border.rule} solid ${vars.color.borderStrong}`,
  backgroundColor: vars.color.bgSink,
  color: vars.color.accentText,
  '@media': { '(forced-colors: active)': { appearance: 'auto' } },
});
globalStyle('input[type="checkbox"]', { borderRadius: vars.radius.sm });
globalStyle('input[type="radio"]', { borderRadius: '50%' });
globalStyle('input[type="checkbox"]:checked, input[type="checkbox"]:indeterminate, input[type="radio"]:checked', {
  backgroundColor: vars.color.accent,
  borderColor: vars.color.accent,
});
globalStyle('input[type="checkbox"]::before, input[type="radio"]::before', {
  content: '""',
  inlineSize: '0.75rem',
  blockSize: '0.75rem',
  backgroundColor: 'currentColor',
  visibility: 'hidden',
  '@media': { '(forced-colors: active)': { display: 'none' } },
});
globalStyle('input[type="checkbox"]::before', {
  clipPath: 'polygon(0 45%, 18% 30%, 40% 55%, 82% 5%, 100% 22%, 40% 95%)',
});
globalStyle('input[type="radio"]::before', { borderRadius: '50%' });
globalStyle('input[type="checkbox"]:checked::before, input[type="checkbox"]:indeterminate::before, input[type="radio"]:checked::before', {
  visibility: 'visible',
});
globalStyle('input[type="checkbox"]:indeterminate::before', {
  clipPath: 'polygon(0 35%, 100% 35%, 100% 65%, 0 65%)',
});

globalStyle('input[type="checkbox"]:disabled, input[type="radio"]:disabled', {
  cursor: 'default',
  opacity: 0.6,
  borderStyle: 'dashed',
});

// A `.field` label (app.css: a column with the field name above the input) that holds a
// checkbox or radio is an option, not a field: box and text on one line.
globalStyle('.field:has(> input[type="checkbox"]), .field:has(> input[type="radio"])', {
  flexDirection: 'row',
  alignItems: 'center',
  gap: vars.space[2],
});
