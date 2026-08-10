# Theme gallery + per-account appearance sync (t19 e13, SPEC §17.1/§17.3).
#
# Its own catalog module rather than more keys in settings.ftl: that file belongs
# to the account-settings surface (t16 e15) and this is a separate feature area.
# Settings.tsx loads both.
#
# Theme NAMES and descriptions are not here — they come from the theme registry
# (apps/web/src/theme/registry.ts) and are English there.

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

## Admin › Appearance

appearance-admin-default-note = This is the default for users who have not chosen an appearance of their own. Users who have keep theirs, and changing this does not move them.
