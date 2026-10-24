import { globalStyle } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

// Checkboxes and radios (t32-h12). One rule for every native checkbox and radio in the
// mail client and the admin console, so no screen is left with the 13 px browser
// default next to another screen's 24 px box:
//   - the box is the 24×24 target (`vars.a11y.touchTarget`, WCAG 2.2 §2.5.8) on every
//     screen; the browser draws it, so forced colours and the platform look still apply;
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

globalStyle('input[type="checkbox"]:disabled, input[type="radio"]:disabled', {
  cursor: 'default',
});

// A `.field` label (app.css: a column with the field name above the input) that holds a
// checkbox or radio is an option, not a field: box and text on one line.
globalStyle('.field:has(> input[type="checkbox"]), .field:has(> input[type="radio"])', {
  flexDirection: 'row',
  alignItems: 'center',
  gap: vars.space[2],
});
