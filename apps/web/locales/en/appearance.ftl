# Theme gallery + per-account appearance sync (t19 e13, SPEC §17.1/§17.3).
#
# Its own catalog module rather than more keys in settings.ftl: that file belongs
# to the account-settings surface (t16 e15) and this is a separate feature area.
# Settings.tsx loads both.
#
# Theme names and descriptions, family names and accent names are here too; the
# theme registry (apps/web/src/theme/registry.ts) holds only their ids.

## Theme mode (fixed / system / schedule)

appearance-mode = When to switch
appearance-mode-fixed = One theme
appearance-mode-system = Follow the system
appearance-mode-schedule = By time of day

appearance-mode-hint-fixed = The theme you pick stays on until you change it.
appearance-mode-hint-system = Switches with your operating system's light and dark setting.
appearance-mode-hint-schedule = Switches to the dark theme during the hours set below.

## Gallery

appearance-gallery = Theme
appearance-gallery-hint = Choosing a theme here switches to one fixed theme.
appearance-active = In use
appearance-light = Light
appearance-dark = Dark

## The light/dark pair used by the two automatic modes

appearance-pair = Themes to switch between
appearance-pair-light = Light theme
appearance-pair-dark = Dark theme

## Schedule window

appearance-schedule = Dark hours
appearance-schedule-start = From
appearance-schedule-end = Until

## Server sync

appearance-sync = Saving
appearance-sync-idle = Not started.
appearance-sync-loading = Checking your account…
appearance-sync-synced = Saved to your account. Other devices pick this up when they start.
appearance-sync-local-only = Saved on this device. Sign in to save these settings to your account.
appearance-sync-error = Could not reach the server. Saved on this device; it will be sent again on your next change.
appearance-sync-reset = Forget the appearance saved for my account
appearance-sync-reset-done = Removed from your account. This device keeps what it is showing now.
appearance-sync-deployment = This server's default is { $theme }.

## Theme names and descriptions (theme/tokens.ts THEME_META)

appearance-theme-dark = Dark
appearance-theme-dark-desc = Near-black page, off-white text, blue actions, red alerts, yellow markers.
appearance-theme-light = Light
appearance-theme-light-desc = Off-white page, black text, blue actions, red alerts, yellow markers.
appearance-theme-bauhaus-black = Black
appearance-theme-bauhaus-black-desc = Black page, white text and controls, blue actions, red alerts, yellow markers.
appearance-theme-paper = Paper
appearance-theme-paper-desc = Warm off-white page, black ink, muted blue, red and ochre.
appearance-theme-blueprint = Blueprint
appearance-theme-blueprint-desc = Deep blue page, white text and lines, cyan actions, yellow markers.
appearance-theme-terminal = Terminal
appearance-theme-terminal-desc = Near-black page, green text, amber markers. Colours only; the font is unchanged.
appearance-theme-primary = Primary blocks
appearance-theme-primary-desc = White page, black text, flat blue, red and yellow blocks.
appearance-theme-slate-light = Slate Light
appearance-theme-slate-light-desc = Cool blue-grey surfaces on a tinted page, muted navy accent.
appearance-theme-slate-dark = Slate Dark
appearance-theme-slate-dark-desc = Cool blue-grey dark theme with a muted blue accent.
appearance-theme-ocean-light = Ocean Light
appearance-theme-ocean-light-desc = Blue-green light theme with a deep teal accent.
appearance-theme-ocean-dark = Ocean Dark
appearance-theme-ocean-dark-desc = Blue-green dark theme with a cyan accent.
appearance-theme-plum-light = Plum Light
appearance-theme-plum-light-desc = Muted purple light theme with a violet accent.
appearance-theme-plum-dark = Plum Dark
appearance-theme-plum-dark-desc = Muted purple dark theme with a violet accent.
appearance-theme-grove-light = Grove Light
appearance-theme-grove-light-desc = Warm paper tones with wood-grain texture and a moss accent.
appearance-theme-grove-dark = Grove Dark
appearance-theme-grove-dark-desc = Warm walnut tones with wood-grain texture and a moss accent.
appearance-theme-amoled = AMOLED
appearance-theme-amoled-desc = Pure-black dark theme; unlit pixels on OLED screens.
appearance-theme-hc-light = High contrast (light)
appearance-theme-hc-light-desc = Black on white with black borders and no tinted surfaces.
appearance-theme-hc-dark = High contrast (dark)
appearance-theme-hc-dark-desc = White on black with white borders and no tinted surfaces.

## Gallery groups (theme/registry.ts)

appearance-family-core = Default
appearance-family-bauhaus = Bauhaus
appearance-family-slate = Slate
appearance-family-ocean = Ocean
appearance-family-plum = Plum
appearance-family-grove = Grove
appearance-family-amoled = AMOLED
appearance-family-contrast = High contrast

## Accents (theme/accentMath.ts)

appearance-accent-default = Theme default
appearance-accent-red = Red
appearance-accent-orange = Orange
appearance-accent-yellow = Yellow
appearance-accent-green = Green
appearance-accent-teal = Teal
appearance-accent-blue = Blue
appearance-accent-indigo = Indigo
appearance-accent-violet = Violet
appearance-accent-magenta = Magenta
appearance-accent-graphite = Graphite

appearance-custom-title = Personal appearance customization
appearance-custom-description = Saved only for this signed-in workspace. Administrator switches control execution; drafts can be saved while disabled.
appearance-custom-loading = Loading customization…
appearance-custom-error = Appearance customization is unavailable.
appearance-custom-css-on = Custom CSS enabled
appearance-custom-css-off = Custom CSS disabled by administrator
appearance-custom-js-on = Appearance scripts enabled
appearance-custom-js-off = Appearance scripts disabled by administrator
appearance-custom-locked = Administrator appearance locks pause custom CSS. Scripts cannot change locked fields.
appearance-custom-css = Restricted CSS
appearance-custom-css-help = Up to 16 KiB. Visual properties and simple selectors only; no URLs, imports, attributes, positioning, or !important. Selectors stay inside your workspace.
appearance-custom-js = Isolated appearance script
appearance-custom-js-help = Up to 16 KiB and 500 ms. Call appearance.set with an object containing accent '#3366cc' or density 'compact'. Supported fields: accent, density, font and mode (light or dark). context provides dark and reducedMotion booleans. Changes are temporary; no mail, account data, network, or application DOM is available.
appearance-custom-save = Save customization
appearance-custom-reset = Reset customization
appearance-custom-reload = Discard draft and reload saved customization
appearance-custom-discard = Discard this unsaved customization draft and reload?
appearance-custom-saved = Customization saved.
appearance-custom-safe = Recovery: add appearanceSafe=1 to the page query to open this workspace without custom CSS or scripts.

appearance-custom-accent = Custom accent colour
appearance-custom-accent-reset = Use named or theme accent
