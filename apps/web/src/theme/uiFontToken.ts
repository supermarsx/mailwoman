// The interface font setting (t101).
//
// Each font setting (the webmail's Settings > Appearance, the console's My
// preferences) writes its stack to the inline `--mw-ui-font` custom property on
// `:root`, or removes it for the theme's own face. The packs bind `vars.font.ui`
// to `var(--mw-ui-font, <pack face>)`, so every rule that uses the token follows
// the setting: the body, and the surfaces that restate the token on their own
// root (the console frame and sign-in gate, Settings, the modules) and so would
// otherwise pin the pack's face over the inherited one.
import type { ThemeTokens } from './tokens.ts';
import { vars } from './contract.css.ts';

/** The inline override the font settings write on `:root`. */
export const UI_FONT_PROPERTY = '--mw-ui-font';
/** A selected UI surface keeps normal text readable; text selection uses ink. */
export const UI_SELECTION_ACCENT_PERCENT = 16;

/** The token values a pack binds: its own, with the UI face behind the override. */
export function emittedThemeTokens(tokens: ThemeTokens): ThemeTokens {
  return { ...tokens,
    color: {...tokens.color,
      text: `color-mix(in srgb, var(--mw-ui-text-color, ${tokens.color.text}) var(--mw-ui-text-mix, 100%), ${tokens.color.bg})`,
      focus: vars.color.accent,
      selection: `color-mix(in srgb, ${vars.color.accent} ${UI_SELECTION_ACCENT_PERCENT}%, ${vars.color.bg})`,
    },
    a11y: {...tokens.a11y, focusRingColor: vars.color.accent, focusRing: `0 0 0 2px ${vars.color.bg}, 0 0 0 5px ${vars.color.accent}`},
    weight: {...tokens.weight, regular: 'var(--mw-ui-font-weight, '+tokens.weight.regular+')', medium: `var(--mw-ui-medium-weight, ${tokens.weight.medium})`, bold: `var(--mw-ui-bold-weight, ${tokens.weight.bold})`, heavy: `var(--mw-ui-heavy-weight, ${tokens.weight.heavy})`},
    font: { ...tokens.font, ui: `var(${UI_FONT_PROPERTY}, ${tokens.font.ui})` } };
}
