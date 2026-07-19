# Mailwoman — user Settings (appearance) strings (source locale: en).
# The dismissible appearance dialog: theme / density / accent / font / layout.
# (Theme and accent preset LABELS come from theme/tokens.ts, not this catalog.)

settings-title = Settings
settings-appearance = Appearance
settings-close = Close settings

settings-theme = Theme
settings-density = Density
settings-accent = Accent
settings-font = Interface font
settings-layout = Layout

# Density options
settings-density-compact = Compact
settings-density-cozy = Cozy
settings-density-relaxed = Relaxed

# Interface-font options
settings-font-default = Default
settings-font-system = System
settings-font-serif = Serif
settings-font-mono = Mono

# Layout options
settings-layout-default = Default
settings-layout-ribbon = Ribbon

## Account settings surface (t16 e15) — 2FA, sessions, signatures, identities,
## notification rules, saved searches, device preferences. Shared action labels
## first, then one block per screen.

settings-cancel = Cancel
settings-edit = Edit
settings-delete = Delete
settings-save = Save
settings-saved = Saved

# Two-factor authentication (S1)
settings-2fa-title = Two-factor authentication
settings-2fa-intro = Add a second factor to require more than a password at sign-in.
settings-2fa-policy-required = Your organization requires a second factor on this account.
settings-2fa-enabled = Enabled
settings-2fa-recovery-title = Recovery codes
settings-2fa-recovery-once = Save these now. They are shown once and cannot be retrieved again. Each code works one time.
settings-2fa-recovery-ack = I have saved these codes
settings-2fa-totp-title = Authenticator app
settings-2fa-totp-enrol = Set up an authenticator app
settings-2fa-totp-scan = Add this secret to your authenticator app, then enter the code it shows.
settings-2fa-totp-uri-link = Open in an authenticator app
settings-2fa-totp-code-label = Authenticator code
settings-2fa-totp-confirm = Confirm
settings-2fa-totp-disable = Remove authenticator
settings-2fa-passkey-title = Passkeys
settings-2fa-passkey-remove = Remove
settings-2fa-passkey-unsupported = This browser does not support passkeys.
settings-2fa-passkey-label-placeholder = Passkey name (optional)
settings-2fa-passkey-add = Add a passkey
settings-2fa-recovery-remaining = { $count } recovery { $count ->
        [one] code
       *[other] codes
    } remaining
settings-2fa-recovery-regenerate = Generate new recovery codes
settings-2fa-recovery-code-label = Recovery code
settings-2fa-error-generic = That did not work. Try again.
settings-2fa-error-code = That code did not verify.
settings-2fa-error-passkey = The passkey could not be registered.

# Two-factor challenge at sign-in (S1 web half)
settings-2fa-challenge-title = Two-factor authentication
settings-2fa-challenge-intro = Confirm your second factor to finish signing in.
settings-2fa-challenge-failed = That did not verify. Try again.
settings-2fa-challenge-use-passkey = Use a passkey
settings-2fa-challenge-method = Choose a method
settings-2fa-challenge-totp-tab = Authenticator code
settings-2fa-challenge-recovery-tab = Recovery code
settings-2fa-challenge-verify = Verify

# Active sessions (S11)
settings-sessions-title = Active sessions
settings-sessions-intro = Sessions currently signed in to this account.
settings-sessions-last-seen = Last active { $when }
settings-sessions-revoke = Sign out
settings-sessions-current = This session
settings-sessions-revoke-others = Sign out { $count } other { $count ->
        [one] session
       *[other] sessions
    }
settings-sessions-error = The session could not be changed.

# Signatures (W12)
settings-sig-title = Signatures
settings-sig-intro = Text appended to messages you send. One signature may be the default.
settings-sig-default = Default
settings-sig-new = New signature
settings-sig-name-label = Name
settings-sig-body-label = Signature
settings-sig-default-label = Use as default
settings-sig-error-name = Give the signature a name.
settings-sig-error-generic = The signature could not be saved.

# Identities
settings-ident-title = Identities
settings-ident-intro = The name and address you send as. Each identity can use its own signature.
settings-ident-new = New identity
settings-ident-name-label = Display name
settings-ident-email-label = Email address
settings-ident-replyto-label = Reply-to address (optional)
settings-ident-signature-label = Default signature
settings-ident-signature-none = None
settings-ident-error-fields = Enter a name and a valid email address.
settings-ident-error-generic = The identity could not be saved.

# Notification rules + quiet hours (W15)
settings-notif-title = Notifications
settings-notif-intro = Choose which new messages notify you, and when to stay quiet.
settings-notif-enabled-label = Show notifications for new mail
settings-notif-quiet-title = Quiet hours
settings-notif-quiet-enabled-label = Silence notifications during a set time
settings-notif-quiet-start = From
settings-notif-quiet-end = To
settings-notif-rules-title = Rules
settings-notif-rule-match-placeholder = Sender, mailbox, or subject contains…
settings-notif-rule-action-label = Action
settings-notif-rule-notify = Notify
settings-notif-rule-mute = Mute
settings-notif-rule-add = Add a rule
settings-notif-error = The notification settings could not be saved.

# Saved searches → search folders (W13)
settings-search-title = Saved searches
settings-search-intro = Show a saved search as a folder in your mailbox list.
settings-search-empty = You have no saved searches yet.
settings-search-as-folder = Show { $name } as a folder
settings-search-as-folder-label = Show as folder
settings-search-error = The saved search could not be changed.

# Keyboard shortcuts (W14)
settings-kbd-title = Keyboard shortcuts
settings-kbd-intro = Choose a shortcut set. Changes apply on this device.
settings-kbd-default = Mailwoman
settings-kbd-gmail = Gmail
settings-kbd-outlook = Outlook
settings-kbd-vim = Vim
settings-kbd-action-compose = Compose
settings-kbd-action-archive = Archive
settings-kbd-action-reply = Reply
settings-kbd-action-next = Next message
settings-kbd-action-previous = Previous message
settings-kbd-action-search = Search

# Offline cache (W16)
settings-offline-title = Offline cache
settings-offline-intro = How much mail to keep on this device for offline use, and how to reclaim space.
settings-offline-budget-label = Cache budget (MB)
settings-offline-retention-label = Keep for (days)
settings-offline-strategy-label = When the budget is reached
settings-offline-lru = Remove least recently used
settings-offline-oldest = Remove oldest first
settings-offline-manual = Only when I clear it
settings-offline-purge = Clear the offline cache now

# Interface direction (W20)
settings-dir-title = Interface direction
settings-dir-intro = Follow the language, or force a direction. Right-to-left mirrors the layout.
settings-dir-auto = Automatic
settings-dir-ltr = Left to right
settings-dir-rtl = Right to left
settings-dir-preview-title = Preview
settings-dir-preview-body = This text mirrors when the direction is right to left.
