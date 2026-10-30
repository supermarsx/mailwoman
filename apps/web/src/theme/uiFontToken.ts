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

/** The inline override the font settings write on `:root`. */
export const UI_FONT_PROPERTY = '--mw-ui-font';

/** The token values a pack binds: its own, with the UI face behind the override. */
export function emittedThemeTokens(tokens: ThemeTokens): ThemeTokens {
  return { ...tokens, font: { ...tokens.font, ui: `var(${UI_FONT_PROPERTY}, ${tokens.font.ui})` } };
}
