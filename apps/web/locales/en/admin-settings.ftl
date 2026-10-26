# Mailwoman — admin console: Settings (settings model C; t32 worker f01).
# Source locale: en. Plain words, no marketing (UX17).

admin-settings-title = Settings
admin-settings-description = Values for the whole server, per domain and per user. A locked value cannot be changed by users or by narrower scopes.
admin-settings-list-caption = Settings
admin-settings-search = Search settings
admin-settings-empty = No setting matches the search.
admin-settings-unavailable = This server does not offer the settings model yet.
admin-settings-col-setting = Setting
admin-settings-col-value = Value
admin-settings-col-source = Source
admin-settings-col-overrides = Narrower values
admin-settings-open = Open
admin-settings-overrides-none = None
admin-settings-overrides = { $domains } domains, { $users } users
admin-settings-not-found = There is no setting named { $key }.

# Value display
admin-settings-value-on = On
admin-settings-value-off = Off
admin-settings-value-empty = Not set
admin-settings-value-list = { $count ->
    [one] { $count } item
   *[other] { $count } items
}

# Detail
admin-settings-detail-key = Key: { $key }
admin-settings-global = For the whole server
admin-settings-global-help = This value applies to every user unless a domain or a user has a narrower value, or the user chooses their own where that is allowed.
admin-settings-value-label = Value
admin-settings-json-hint = JSON. The server checks the value and points at the part it refuses.
admin-settings-json-invalid = This is not valid JSON.
admin-settings-user-override = Users may change it
admin-settings-user-override-help = When on, users can choose their own value in their Settings. When off, the value set here or for their domain applies to them.
admin-settings-lock-help = When locked, domain values and user choices are ignored and this value applies to everyone.
admin-settings-save = Save
admin-settings-saved = Saved. It applies within 30 seconds on every server process.
admin-settings-saved-restart = Saved. It applies after a restart.
admin-settings-dedicated = This setting is changed in { $section }, which checks its effect first.
admin-settings-dedicated-link = Open { $section }
admin-settings-pinned = Set by the environment variable { $name }. The value stored here applies when the variable is unset.
admin-settings-lock-title = Lock { $setting }?
admin-settings-lock-body = Locking ignores the narrower values listed below. They are kept, and apply again when you unlock.
admin-settings-lock-confirm = Lock
admin-settings-impact-domain = { $count ->
    [one] { $count } domain value is ignored
   *[other] { $count } domain values are ignored
}
admin-settings-impact-user = { $count ->
    [one] { $count } user's own choice is ignored
   *[other] { $count } users' own choices are ignored
}
admin-settings-impact-admin-user = { $count ->
    [one] { $count } per-user value is ignored
   *[other] { $count } per-user values are ignored
}
admin-settings-impact-profile = { $count ->
    [one] { $count } feature profile value is ignored
   *[other] { $count } feature profile values are ignored
}
admin-settings-impact-group = { $count ->
    [one] { $count } group value is ignored
   *[other] { $count } group values are ignored
}

# Domain values
admin-settings-domains = Domain values
admin-settings-domains-help = A domain value applies to the users of that domain instead of the value for the whole server.
admin-settings-domains-caption = Domain values of { $setting }
admin-settings-domains-empty = No domain has its own value.
admin-settings-domain = Domain
admin-settings-domain-value = Value for this domain
admin-settings-domain-locked = Locked for the domain's users
admin-settings-domain-add = Add a domain value
admin-settings-domain-save = Save domain value
admin-settings-domain-remove = Remove
admin-settings-domain-remove-title = Remove the value of { $domain }?
admin-settings-domain-remove-body = The users of { $domain } get the value for the whole server again.
admin-settings-domain-required = Enter a domain name.
admin-settings-locked-yes = Locked
admin-settings-locked-no = Not locked

# Per-user values
admin-settings-users = Per-user values
admin-settings-users-help = A value an administrator set for one user. For limits the user is named by address.
admin-settings-users-caption = Per-user values of { $setting }
admin-settings-users-empty = No user has a value set by an administrator.
admin-settings-user = User
admin-settings-user-hint = Account id, or the address for limits.
admin-settings-user-value = Value for this user
admin-settings-user-add = Add a per-user value
admin-settings-user-save = Save user value
admin-settings-user-required = Enter a user.
admin-settings-user-remove-title = Remove the value of { $user }?
admin-settings-user-remove-body = The user gets their domain's value, or the value for the whole server.

# Sources (what decides the value now)
admin-settings-source-default = Built-in default
admin-settings-source-envFallback = Environment variable (no stored value)
admin-settings-source-envPin = Environment variable
admin-settings-source-global = Stored for the whole server

# Categories
admin-settings-cat-interface = Interface
admin-settings-cat-appearance = Appearance
admin-settings-cat-mail = Mail
admin-settings-cat-notifications = Notifications
admin-settings-cat-offline = Offline use
admin-settings-cat-assistant = AI assistant
admin-settings-cat-upstream = Mail server
admin-settings-cat-signin = Sign-in
admin-settings-cat-admin = Admin console
admin-settings-cat-passwords = Passwords
admin-settings-cat-limits = Limits and usage
admin-settings-cat-plugins = Plugins
admin-settings-cat-features = Feature permissions
admin-settings-cat-integrations = Integration permissions
admin-settings-key-cap-integration-nextcloud = Nextcloud
admin-settings-help-cap-integration-nextcloud = Allow account-linked Nextcloud files, sharing and calendar/contact/task operations.
admin-settings-key-cap-integration-google = Google bridge
admin-settings-help-cap-integration-google = Allow Google bridge operations, including token acquisition and refresh.
admin-settings-key-cap-integration-microsoft = Microsoft bridge
admin-settings-help-cap-integration-microsoft = Allow Microsoft bridge operations, including token acquisition and refresh.
admin-settings-key-cap-integration-family = Individual integration
admin-settings-help-cap-integration-family = Permission for an individual integration identifier; runtime coverage is listed in the feature catalogue.
admin-settings-key-cap-ui-plugin-family = Individual UI plugin
admin-settings-help-cap-ui-plugin-family = Permission for one UI plugin. Use the exact policy key from the integration catalogue.
admin-settings-key-cap-ai-assistant = AI assistant
admin-settings-help-cap-ai-assistant = Allow assistant configuration and invocation.
admin-settings-key-cap-plugins = Plugins
admin-settings-help-cap-plugins = Allow UI and engine plugin operations.
admin-settings-key-cap-pim-calendar = Calendar
admin-settings-help-cap-pim-calendar = Allow calendar methods and subscriptions.
admin-settings-key-cap-pim-contacts = Contacts
admin-settings-help-cap-pim-contacts = Allow contact and address-book methods.
admin-settings-key-cap-pim-tasks = Tasks
admin-settings-help-cap-pim-tasks = Allow task methods.
admin-settings-key-cap-pim-notes = Notes
admin-settings-help-cap-pim-notes = Allow note methods.
admin-settings-key-cap-export = Export
admin-settings-help-cap-export = Allow mail and PIM exports.
admin-settings-key-cap-import = Import
admin-settings-help-cap-import = Allow mail and PIM imports.
admin-settings-key-cap-api-keys = API keys
admin-settings-help-cap-api-keys = API-key policy. Check the feature catalogue for enforcement coverage.
admin-settings-key-cap-oauth-apps = OAuth applications
admin-settings-help-cap-oauth-apps = OAuth-application policy. Check the feature catalogue for enforcement coverage.
admin-settings-key-cap-rules = Mail rules
admin-settings-help-cap-rules = Allow managed mail-rule and vacation methods.
admin-settings-key-cap-rules-forward = Rule forwarding
admin-settings-help-cap-rules-forward = Forwarding policy. Check the feature catalogue for enforcement coverage.
admin-settings-key-cap-remote-images = Remote images
admin-settings-help-cap-remote-images = Remote-image capability. Existing privacy settings continue to govern requests; check feature coverage.
admin-settings-key-cap-user-background = Custom background
admin-settings-help-cap-user-background = Custom-background capability, disabled by default. Check feature coverage.
admin-settings-key-cap-user-css = Custom CSS
admin-settings-help-cap-user-css = Custom-CSS capability, disabled by default. Check feature coverage.
admin-settings-key-limit-attachment-bytes = Attachment size limit
admin-settings-help-limit-attachment-bytes = Attachment policy in bytes, default 50000000. Runtime enforcement remains pending.
admin-settings-key-limit-send-per-hour = Hourly sending limit
admin-settings-help-limit-send-per-hour = Submission policy per hour; zero means unlimited. Runtime enforcement remains pending.
admin-settings-key-limit-recipients-per-message = Recipients per message
admin-settings-help-limit-recipients-per-message = Recipient-count policy; zero means unlimited. Runtime enforcement remains pending.
admin-settings-cat-import = Import
admin-settings-cat-webhooks = Webhooks
admin-settings-cat-realtime = Live updates
admin-settings-cat-privacy = Privacy
admin-settings-cat-observability = Observability
admin-settings-cat-egress = Outbound connections
admin-settings-cat-updates = Updates
admin-settings-cat-backups = Backups
admin-settings-cat-about = About
admin-settings-cat-config = Configuration
admin-settings-cat-jobs = Jobs
admin-settings-cat-maintenance = Maintenance
admin-settings-cat-health = Health checks
admin-settings-cat-diagnostics = Diagnostics
admin-settings-cat-users = Users
admin-settings-cat-server-text = Server text

# One label and one help sentence per setting (UX4). Keys: crates/mw-server/src/settings_registry.rs
admin-settings-key-ui-language = Language
admin-settings-help-ui-language = The language of the mail client for users who have not chosen one. "Same as the browser" follows each browser's languages.
admin-settings-key-appearance-theme = Theme
admin-settings-help-appearance-theme = The colour theme users see until they pick their own. Locking it holds every user to this theme.
admin-settings-key-appearance-accent = Accent colour
admin-settings-help-appearance-accent = The accent colour of the theme. Empty means the theme's own accent.
admin-settings-key-appearance-density = Density
admin-settings-help-appearance-density = How much space lists and controls take: compact, cozy or relaxed.
admin-settings-key-appearance-font = Interface font
admin-settings-key-packages-allowed = Package tracking allowed
admin-settings-help-packages-allowed = Permit package tracking for the affected accounts. The provider must also be enabled in Package tracking, and each user must opt in. Disabling this setting immediately prevents new provider calls.
admin-settings-help-appearance-font = Choose a bundled font for accounts without a font preference, or lock the choice for all affected accounts. Fonts load from this server; no Google Fonts request is made. Default uses the theme font.
admin-settings-key-appearance-brand-name = Product name
admin-settings-help-appearance-brand-name = The name shown on the sign-in page and in the browser tab.
admin-settings-key-login-notice = Sign-in notice
admin-settings-help-login-notice = A short text shown on the sign-in page, for example a support contact. Leave empty for none.
admin-settings-key-mail-read-receipts = Read receipts
admin-settings-key-receipts-dsn-allowed = Delivery status requests
admin-settings-help-receipts-dsn-allowed = Allow accounts to request SMTP delivery status notifications. This does not request or prove that a message was read.
admin-settings-help-mail-read-receipts = What happens when an opened message asks for a read receipt: ask the user, never send, or always send.
admin-settings-key-notifications-enabled = New-mail notifications
admin-settings-help-notifications-enabled = Whether users get notifications for new mail. It does not change their notification rules.
admin-settings-key-offline-retention-days = Offline cache age
admin-settings-help-offline-retention-days = How many days a device keeps messages for offline use. 0 means no age limit.
admin-settings-key-offline-budget-mb = Offline cache size
admin-settings-help-offline-budget-mb = How many megabytes a device may use for offline mail. 0 means no offline cache.
admin-settings-key-assistant-enabled = AI assistant
admin-settings-help-assistant-enabled = Whether users see the AI assistant. The deployment switch in AI assistant still applies above this.
admin-settings-key-upstream-default = Default mail server
admin-settings-help-upstream-default = The mail server an address signs in to when no domain names one. The domain table wins over this.
admin-settings-key-upstream-proxy-timeout-secs = Mail server timeout
admin-settings-help-upstream-proxy-timeout-secs = How long one request to the mail server may take, in seconds.
admin-settings-key-signin-new-users = New users
admin-settings-help-signin-new-users = Who may sign in for the first time. Changed in Sign-in policy.
admin-settings-key-signin-domain-mode = Sign-in domains
admin-settings-help-signin-domain-mode = Whether any domain may sign in or only listed domains. Changed in Sign-in policy.
admin-settings-key-signin-refusal-detail = Refusal messages
admin-settings-help-signin-refusal-detail = Whether a refused sign-in says why (disabled account, address not allowed) or gives the same answer as a wrong password. Changed in Sign-in policy.
admin-settings-key-signin-ldap-bind-precheck = LDAP check before sign-in
admin-settings-help-signin-ldap-bind-precheck = Check the password against the directory before the mail server. Changed in Integrations, after a test bind.
admin-settings-key-twofa-pending-ttl-secs = Second-factor time limit
admin-settings-help-twofa-pending-ttl-secs = How long a user has to enter the second factor after the password, in seconds.
admin-settings-key-twofa-max-attempts = Second-factor attempts
admin-settings-help-twofa-max-attempts = How many wrong second-factor codes are allowed before the sign-in starts over.
admin-settings-key-admin-session-idle-secs = Console idle timeout
admin-settings-help-admin-session-idle-secs = How long an admin console session lasts without a request, in seconds.
admin-settings-key-admin-session-absolute-secs = Console session limit
admin-settings-help-admin-session-absolute-secs = The longest an admin console session lasts, in seconds, active or not.
admin-settings-key-passwd-policy-min-length = Minimum password length
admin-settings-help-passwd-policy-min-length = The fewest characters a new password may have.
admin-settings-key-passwd-policy-require-upper = Require an upper-case letter
admin-settings-help-passwd-policy-require-upper = A new password must contain an upper-case letter.
admin-settings-key-passwd-policy-require-lower = Require a lower-case letter
admin-settings-help-passwd-policy-require-lower = A new password must contain a lower-case letter.
admin-settings-key-passwd-policy-require-digit = Require a digit
admin-settings-help-passwd-policy-require-digit = A new password must contain a digit.
admin-settings-key-passwd-policy-require-symbol = Require a symbol
admin-settings-help-passwd-policy-require-symbol = A new password must contain a character that is not a letter or a digit.
admin-settings-key-limit-storage-bytes = Storage limit
admin-settings-help-limit-storage-bytes = The most bytes one user may store on this server. 0 means no limit.
admin-settings-key-limit-send-per-day = Messages sent per day
admin-settings-help-limit-send-per-day = The most messages one user may send in a day. 0 means no limit.
admin-settings-key-limit-ai-calls-per-day = AI assistant calls per day
admin-settings-help-limit-ai-calls-per-day = The most AI assistant requests one user may make in a day. 0 means no limit.
admin-settings-key-quota-warn-thresholds = Usage warning levels
admin-settings-help-quota-warn-thresholds = The percentages of a limit at which users are warned, lowest first.
admin-settings-key-quota-warn-enabled = Usage warnings
admin-settings-help-quota-warn-enabled = Whether users are warned when they near a limit.
admin-settings-key-usage-retention-days = Usage history
admin-settings-help-usage-retention-days = How many days of daily usage counts are kept.
admin-settings-key-usage-meter-api = Count API use
admin-settings-help-usage-meter-api = Whether API requests are counted in the usage figures.
admin-settings-key-usage-measure-interval-hours = Storage measuring interval
admin-settings-help-usage-measure-interval-hours = How often the stored data of each user is measured, in hours.
admin-settings-key-assistant-rate-limit-per-min = AI assistant rate limit
admin-settings-help-assistant-rate-limit-per-min = The most AI assistant requests per minute for the whole server. 0 means no limit.
admin-settings-key-plugins-kv-max-key-bytes = Plugin key size
admin-settings-help-plugins-kv-max-key-bytes = The longest key a plugin may store, in bytes.
admin-settings-key-plugins-kv-max-value-bytes = Plugin value size
admin-settings-help-plugins-kv-max-value-bytes = The largest value a plugin may store under one key, in bytes.
admin-settings-key-plugins-kv-max-total-bytes = Plugin storage
admin-settings-help-plugins-kv-max-total-bytes = The most bytes one plugin may store in total.
admin-settings-key-plugins-kv-max-keys = Plugin keys
admin-settings-help-plugins-kv-max-keys = The most keys one plugin may store.
admin-settings-key-import-max-messages = Import message count
admin-settings-help-import-max-messages = The most messages one import may bring in.
admin-settings-key-import-max-message-bytes = Import message size
admin-settings-help-import-max-message-bytes = The largest message an import accepts, in bytes.
admin-settings-key-webhooks-max-attempts = Webhook attempts
admin-settings-help-webhooks-max-attempts = How many times a webhook delivery is tried before it is given up.
admin-settings-key-webhooks-timeout-secs = Webhook timeout
admin-settings-help-webhooks-timeout-secs = How long one webhook delivery may take, in seconds.
admin-settings-key-outbox-max-attempts = Sending attempts
admin-settings-help-outbox-max-attempts = How many times a message is handed to the mail server before it is reported as not sent.
admin-settings-key-realtime-recheck-secs = Live update re-check
admin-settings-help-realtime-recheck-secs = How often an open live-update connection checks that its session is still valid, in seconds.
admin-settings-key-privacy-watermark-enabled = Reader watermark
admin-settings-help-privacy-watermark-enabled = Show a watermark with the reader's address over opened messages.
admin-settings-key-privacy-watermark-opacity = Watermark strength
admin-settings-help-privacy-watermark-opacity = How visible the watermark is, from 0.02 (faint) to 0.5.
admin-settings-key-privacy-open-tracking-retention-days = Open tracking history
admin-settings-help-privacy-open-tracking-retention-days = How many days read-tracking records are kept.
admin-settings-key-privacy-open-tracking-unbound-ttl-hours = Unsent tracking tokens
admin-settings-help-privacy-open-tracking-unbound-ttl-hours = How many hours a tracking token waits for its message to be sent before it is deleted.
admin-settings-key-masked-email-domain = Masked address domain
admin-settings-help-masked-email-domain = The domain masked addresses are created under. Leave empty to turn masked addresses off.
admin-settings-key-abuse-report-address = Abuse report address
admin-settings-help-abuse-report-address = Where spam and abuse reports are sent.
admin-settings-key-dlp-admin-address = Data loss notices
admin-settings-help-dlp-admin-address = Where data loss prevention notices are sent.
admin-settings-key-observability-error-forward-url = Error forwarding address
admin-settings-help-observability-error-forward-url = An https address that client error reports are forwarded to. Leave empty for none.
admin-settings-key-egress-probe-url = Outbound test address
admin-settings-help-egress-probe-url = The https address used to test an outbound proxy.
admin-settings-key-push-vapid-contact = Push contact
admin-settings-help-push-vapid-contact = The contact (mailto: or https:) given to push services with each notification.
admin-settings-key-push-quiet-hours = Push quiet hours
admin-settings-help-push-quiet-hours = Hours (UTC) in which no push notification is sent. Leave empty for none.
admin-settings-key-updates-check = Check for updates
admin-settings-help-updates-check = Whether the server checks once a day for a newer release and shows it on the dashboard.
admin-settings-key-updates-feed-url = Release feed
admin-settings-help-updates-feed-url = The https address the update check reads.
admin-settings-key-updates-apply = Apply updates
admin-settings-help-updates-apply = Whether an update can be started from the console through the host update agent.
admin-settings-key-backup-retention-count = Backups kept
admin-settings-help-backup-retention-count = How many database backups are kept.
admin-settings-key-about-security-contact = Security contact
admin-settings-help-about-security-contact = The contact in security.txt (mailto: or https:).
admin-settings-key-about-security-policy-url = Security policy
admin-settings-help-about-security-policy-url = The address of your security policy, listed in security.txt.
admin-settings-key-about-project-links = About links
admin-settings-help-about-project-links = Links shown on the About page, each with a label and an https address.
admin-settings-key-config-snapshots-kept = Configuration snapshots
admin-settings-help-config-snapshots-kept = How many configuration snapshots are kept.
admin-settings-key-jobs-history-days = Job history
admin-settings-help-jobs-history-days = How many days of job runs are kept.
admin-settings-key-maintenance-announce-minutes-default = Maintenance notice lead time
admin-settings-help-maintenance-announce-minutes-default = How many minutes before a maintenance window users are told about it.
admin-settings-key-maintenance-default-message = Maintenance message
admin-settings-help-maintenance-default-message = The text users see during maintenance, per language.
admin-settings-key-health-disk-free-min-percent = Free disk space floor
admin-settings-help-health-disk-free-min-percent = The readiness check fails when free disk space falls below this percentage.
admin-settings-key-observability-otlp-metrics = Send metrics by OTLP
admin-settings-help-observability-otlp-metrics = Send server metrics to the OTLP collector set at start.
admin-settings-key-metrics-drilldown-top-n = Metrics detail rows
admin-settings-help-metrics-drilldown-top-n = How many rows a metrics breakdown shows.
admin-settings-key-metrics-rollup-days = Metrics history
admin-settings-help-metrics-rollup-days = How many days of hourly metric summaries are kept.
admin-settings-key-diagnostics-retention-days = Diagnostics history
admin-settings-help-diagnostics-retention-days = How many days of diagnostic runs are kept.
admin-settings-key-data-export-link-hours = Data export download window
admin-settings-help-data-export-link-hours = How many hours a data export can be downloaded before it is deleted.
admin-settings-key-server-text-family = Server text
admin-settings-help-server-text-family = Text the server writes into mail and notifications, per language and domain.
admin-settings-cat-sieve = Mail filters
admin-settings-key-sieve-max-script-bytes = Maximum filter script size
admin-settings-help-sieve-max-script-bytes = Maximum bytes in a single Sieve filter script.
admin-settings-key-sieve-max-scripts = Maximum filter scripts
admin-settings-help-sieve-max-scripts = Maximum number of saved scripts per account.
admin-settings-key-sieve-max-redirects = Maximum redirects
admin-settings-help-sieve-max-redirects = Maximum redirect commands in one script.
admin-settings-key-sieve-vacation-allowed = Vacation filter actions
admin-settings-help-sieve-vacation-allowed = Allow scripts to send vacation responses when the backend supports them.
admin-settings-key-sieve-allowed-extensions = Allowed filter extensions
admin-settings-help-sieve-allowed-extensions = An optional list of lowercase Sieve extension names. The backend must also support each extension.
admin-settings-key-cap-sieve-raw-edit = Edit filter script source
admin-settings-help-cap-sieve-raw-edit = Allow raw Sieve script edits for the selected accounts.
admin-settings-key-cap-sieve-forward-external = Forward outside the account domain
admin-settings-help-cap-sieve-forward-external = Permit external forwarding in scripts. Saving these scripts also requires recent verification.
admin-settings-key-session-max-concurrent = Concurrent sessions
admin-settings-help-session-max-concurrent = Maximum sessions per account. The oldest sessions are revoked when a new sign-in exceeds this limit.
admin-settings-key-session-idle-secs = Session idle timeout
admin-settings-help-session-idle-secs = End idle sessions after this many seconds, up to the deployment limit.
admin-settings-key-session-absolute-secs = Session lifetime
admin-settings-help-session-absolute-secs = End sessions after this many seconds, up to the deployment limit.
admin-settings-key-twofa-remember-days = Remembered device lifetime
admin-settings-help-twofa-remember-days = Days before a remembered device must verify a second factor again. Zero disables this feature.
admin-settings-key-twofa-passkeys-enabled = Allow passkeys
admin-settings-help-twofa-passkeys-enabled = Allow passkeys as an authentication factor.
admin-settings-key-twofa-required = Require two-factor authentication
admin-settings-help-twofa-required = Require an enrolled second factor before issuing a session.
admin-settings-key-privacy-remote-content = Remote content
admin-settings-help-privacy-remote-content = Block images, ask first, use the proxy, or load directly when permitted.
admin-settings-key-privacy-trackers = Block trackers
admin-settings-help-privacy-trackers = Block known email tracking images.
admin-settings-key-privacy-clean-links = Clean links
admin-settings-help-privacy-clean-links = Remove tracking parameters from message links.
admin-settings-key-privacy-max-bytes = Remote image limit
admin-settings-help-privacy-max-bytes = Maximum bytes fetched for one remote image.
admin-settings-key-privacy-cache-bytes = Remote image cache
admin-settings-help-privacy-cache-bytes = Maximum bytes retained in the remote image cache.
admin-settings-key-privacy-rate-per-min = Remote image request rate
admin-settings-help-privacy-rate-per-min = Maximum remote image fetches per account per minute.
admin-settings-key-privacy-allow-direct = Allow direct images
admin-settings-key-cap-remote-content-direct = Direct image permission
admin-settings-help-cap-remote-content-direct = Allow this account to select direct images when the server permits it.
admin-settings-help-privacy-allow-direct = Permit direct image loading for accounts with the required capability.
admin-settings-key-logs-auth-retention-days = Authentication history retention
admin-settings-help-logs-auth-retention-days = Days of authentication metadata retained, independently of durable account identity.
admin-settings-key-logs-auth-max-entries = Authentication history entry limit
admin-settings-help-logs-auth-max-entries = Maximum retained entries. Matching repeated failures within one minute share an entry.

admin-settings-key-reader-color-mode = Message colors
admin-settings-cat-reader = Message display
admin-settings-key-mail-quick-reply = Quick reply
admin-settings-help-mail-quick-reply = Offer a compact reply tab with recipients already filled in. Off by default; users may enable it unless locked by an administrator. Normal send protections still apply.
admin-settings-key-mail-request-read-receipt = Request read receipts by default
admin-settings-help-mail-request-read-receipt = Start supported outgoing messages with a receipt request selected. This is a composing default, not a guarantee that the recipient will send a receipt.
admin-settings-help-reader-color-mode = Automatic follows dark themes. Choose original, adaptive or inverted rendering inside the isolated message frame.
admin-settings-key-reader-font-size = Message font size
admin-settings-help-reader-font-size = Default reading size in pixels, from 12 to 24.
admin-settings-key-reader-line-height = Message line height
admin-settings-help-reader-line-height = Reading line spacing from 1.2 to 2.2.
admin-settings-key-reader-content-width = Message reading width
admin-settings-help-reader-content-width = Comfortable, wide or full-pane message content.
admin-settings-key-reader-loading = Message loading appearance
admin-settings-help-reader-loading = Placeholder lines, a text indicator or no visual indicator. Accessibility announcements remain available.
admin-settings-key-reader-loading-delay-ms = Message loading delay
admin-settings-help-reader-loading-delay-ms = Delay only the loading indicator, between 0 and 1000 milliseconds. Message delivery is never delayed.

admin-settings-key-login-show-domain-hint = Show preferred domain on sign-in
admin-settings-help-login-show-domain-hint = Show which domain accepts a username without an email suffix. Hidden by default. This only changes the hint; signing in with a username still works.

admin-settings-key-signin-retry-cooldown-secs = Sign-in retry cooldown
admin-settings-help-signin-retry-cooldown-secs = Minimum seconds between password sign-in attempts from one client network. Default 3; 0 disables the cooldown. Existing failure bans still apply.

admin-settings-key-mail-quote-style = Reply and forward header style
admin-settings-key-mail-external-recipient-warning = External recipient warning
admin-settings-help-mail-external-recipient-warning = Off, warn, or require confirmation in the composer. Covers To, Cc and Bcc. This is a composer safeguard, not a server delivery restriction.
admin-settings-key-mail-trusted-recipient-domains = Trusted recipient domains
admin-settings-help-mail-trusted-recipient-domains = Up to 100 exact recipient domains. The selected sending identity's domain is also trusted. Invalid entries never grant trust. Lock this setting to prevent user overrides.
admin-settings-help-mail-quote-style = Use classic quoted replies or Outlook-style message headers. This sets the user default; lock it to enforce the style. Existing drafts are unchanged.

admin-settings-key-signin-anti-bot-mode = Sign-in bot protection
admin-settings-help-signin-anti-bot-mode = Self-hosted proof-of-work verification: off, mailbox, admin, or all. No external verification service is contacted.
admin-settings-key-signin-anti-bot-difficulty = Sign-in verification work
admin-settings-help-signin-anti-bot-difficulty = Maximum solver counter, 100–20000. Higher values require more browser work; default 2000.
admin-settings-key-signin-anti-bot-expiry-secs = Sign-in verification lifetime
admin-settings-help-signin-anti-bot-expiry-secs = Seconds a single-use verification remains valid, 60–600; default 180. A fresh verification is needed after each credential attempt.

admin-settings-key-attachments-preview-allowed = Allow attachment previews
admin-settings-help-attachments-preview-allowed = Disable to prevent attachment preview rendering; original downloads have their own control.

admin-settings-key-attachments-download-allowed = Allow original attachment delivery
admin-settings-help-attachments-download-allowed = Disabling also prevents previews that require original bytes. Browser previews are not copy protection.

admin-settings-key-attachments-max-bytes = Maximum attachment bytes
admin-settings-help-attachments-max-bytes = Zero is unlimited. Applies to original delivered bytes, including range requests.

admin-settings-key-attachments-allowed-extensions = Allowed filename extensions
admin-settings-help-attachments-allowed-extensions = JSON array of lowercase extensions without a dot, such as ["pdf","tar.gz"]. Empty allows any extension; blocked rules win.

admin-settings-key-attachments-blocked-extensions = Blocked filename extensions
admin-settings-help-attachments-blocked-extensions = JSON array of lowercase extensions. Evaluated against stored MIME filenames, never the download URL name.

admin-settings-key-attachments-warn-extensions = Warn for filename extensions
admin-settings-help-attachments-warn-extensions = JSON array of extensions requiring explicit acknowledgement before original delivery.

admin-settings-key-attachments-allowed-mime-types = Allowed MIME types
admin-settings-help-attachments-allowed-mime-types = JSON array of lowercase types or top-level wildcards such as ["image/*","application/pdf"]. Empty allows any type.

admin-settings-key-attachments-blocked-mime-types = Blocked MIME types
admin-settings-help-attachments-blocked-mime-types = JSON array of lowercase types or top-level wildcards. Blocked rules take precedence.

admin-settings-key-attachments-warn-mime-types = Warn for MIME types
admin-settings-help-attachments-warn-mime-types = JSON array of lowercase types or top-level wildcards requiring acknowledgement. MIME metadata is not malware detection.

admin-settings-cat-shortcuts = Keyboard shortcuts
admin-settings-cat-time = Time zones
admin-settings-key-time-display-zone = Display time zone
admin-settings-help-time-display-zone = IANA time zone such as Europe/Lisbon, or local to follow the current device. Message timestamps remain stored as their original instants.
admin-settings-key-time-comparison-zone = Comparison time zone
admin-settings-help-time-comparison-zone = Optional second IANA time zone for message and calendar helpers. Leave empty to hide the comparison; no location is inferred from an email address.
admin-settings-key-time-show-helpers = Show time zone helpers
admin-settings-help-time-show-helpers = Show local and selected-zone comparisons for message timestamps and calendar events. Admin locks can enforce this choice.
admin-settings-key-shortcuts-enabled = Enable keyboard shortcuts
admin-settings-help-shortcuts-enabled = Allow application shortcuts. Locks can disable them globally or for a domain. Typing fields and modal dialogs retain their normal keyboard handling.
admin-settings-key-shortcuts-compose = Compose shortcut
admin-settings-help-shortcuts-compose = Use inherit for the selected preset, disabled to remove this action, or a chord such as Ctrl+Shift+K. Conflicting or invalid chords do not run.
admin-settings-key-shortcuts-archive = Archive shortcut
admin-settings-help-shortcuts-archive = Use inherit for the selected preset, disabled to remove this action, or a chord such as Ctrl+Shift+K. Conflicting or invalid chords do not run.
admin-settings-key-shortcuts-reply = Reply shortcut
admin-settings-help-shortcuts-reply = Use inherit for the selected preset, disabled to remove this action, or a chord such as Ctrl+Shift+K. Conflicting or invalid chords do not run.
admin-settings-key-shortcuts-reply-all = Reply all shortcut
admin-settings-help-shortcuts-reply-all = Use inherit for the selected preset, disabled to remove this action, or a chord such as Ctrl+Shift+K. Conflicting or invalid chords do not run.
admin-settings-key-shortcuts-forward = Forward shortcut
admin-settings-help-shortcuts-forward = Use inherit for the selected preset, disabled to remove this action, or a chord such as Ctrl+Shift+K. Conflicting or invalid chords do not run.
admin-settings-key-shortcuts-next = Next message shortcut
admin-settings-help-shortcuts-next = Use inherit for the selected preset, disabled to remove this action, or a chord such as Ctrl+Shift+K. Conflicting or invalid chords do not run.
admin-settings-key-shortcuts-previous = Previous message shortcut
admin-settings-help-shortcuts-previous = Use inherit for the selected preset, disabled to remove this action, or a chord such as Ctrl+Shift+K. Conflicting or invalid chords do not run.
admin-settings-key-shortcuts-search = Focus search shortcut
admin-settings-help-shortcuts-search = Use inherit for the selected preset, disabled to remove this action, or a chord such as Ctrl+Shift+K. Conflicting or invalid chords do not run.
admin-settings-key-shortcuts-refresh = Refresh messages shortcut
admin-settings-help-shortcuts-refresh = Use inherit for the selected preset, disabled to remove this action, or a chord such as Ctrl+Shift+K. Conflicting or invalid chords do not run.
admin-settings-key-shortcuts-settings = Open settings shortcut
admin-settings-help-shortcuts-settings = Use inherit for the selected preset, disabled to remove this action, or a chord such as Ctrl+Shift+K. Conflicting or invalid chords do not run.
admin-settings-key-shortcuts-trash = Move to trash shortcut
admin-settings-help-shortcuts-trash = Use inherit for the selected preset, disabled to remove this action, or a chord such as Ctrl+Shift+K. Conflicting or invalid chords do not run.
admin-settings-key-shortcuts-mark-read = Mark read shortcut
admin-settings-help-shortcuts-mark-read = Use inherit for the selected preset, disabled to remove this action, or a chord such as Ctrl+Shift+K. Conflicting or invalid chords do not run.
admin-settings-key-shortcuts-mark-unread = Mark unread shortcut
admin-settings-help-shortcuts-mark-unread = Use inherit for the selected preset, disabled to remove this action, or a chord such as Ctrl+Shift+K. Conflicting or invalid chords do not run.
admin-settings-key-shortcuts-spam = Mark as spam shortcut
admin-settings-help-shortcuts-spam = Use inherit for the selected preset, disabled to remove this action, or a chord such as Ctrl+Shift+K. Conflicting or invalid chords do not run.

admin-settings-key-notifications-push-allowed = Background push notifications allowed
admin-settings-help-notifications-push-allowed = Permit this account's enabled notifications to register and receive background push. Browser permission is still required.
admin-settings-key-offline-persistent-storage-allowed = Persistent browser storage requests allowed
admin-settings-help-offline-persistent-storage-allowed = Allow an explicit request to protect local cached data from automatic browser eviction. The browser may refuse; this does not grant permission automatically.

admin-settings-key-security-signin-mailbox-failure-limit = Mailbox failed-credential limit
admin-settings-help-security-signin-mailbox-failure-limit = Rejected credentials from one source IP within the counting window trigger a temporary mailbox sign-in lock. Network and configuration failures do not count.
admin-settings-key-security-signin-admin-failure-limit = Administrator failed-credential limit
admin-settings-help-security-signin-admin-failure-limit = Rejected administrator credentials from one source IP trigger a separate temporary admin sign-in lock. Short admin session and fresh-proof requirements stay unchanged.
admin-settings-key-security-signin-failure-window-secs = Failed-credential counting window
admin-settings-help-security-signin-failure-window-secs = Rolling window in seconds for credential and MFA rejections. Successful completed sign-in resets that source and surface.
admin-settings-key-security-signin-lock-secs = Temporary sign-in lock duration
admin-settings-help-security-signin-lock-secs = Lock duration in seconds. Repeated attempts while locked do not extend its original expiry. Manual global IP bans are separate.

admin-settings-key-security-signin-geo-scope = Sign-in country policy scope
admin-settings-help-security-signin-geo-scope = Off by default. Apply only to new mailbox, administrator, or all sign-ins. Existing sessions are not revoked. Check the current-source diagnostic before enabling; an unavailable local GeoIP database refuses sign-in.
admin-settings-key-security-signin-geo-mode = Country allow or deny list
admin-settings-help-security-signin-geo-mode = Allow permits only listed countries; deny refuses listed countries. Unknown addresses use their own explicit policy.
admin-settings-key-security-signin-geo-countries = Sign-in country codes
admin-settings-help-security-signin-geo-countries = JSON array of uppercase two-letter country codes, for example ["PT", "GB"]. Uses only the operator-supplied MW_GEOIP_DB; no external location service is contacted.
admin-settings-key-security-signin-geo-unknown = Unmapped source address policy
admin-settings-help-security-signin-geo-unknown = Allow or deny addresses absent from a readable local database, including private addresses. Missing or unreadable databases always refuse active-policy sign-ins, even when unknown addresses are allowed.

admin-settings-key-contacts-avatar-mode = Contact pictures and generated avatars
admin-settings-help-contacts-avatar-mode = Generate initials locally, hide avatars, or use embedded photos from each account own address books. No remote photo URLs or external avatar service is contacted. User and domain policy can be held by an administrator.
admin-settings-cat-contacts = Contact pictures

admin-settings-key-reader-action-style = Message action ribbon
admin-settings-help-reader-action-style = Compact icons are the default. Choose labels for icons with visible text; both retain action names and themed tooltips. Supports global, domain and user policy locks.
admin-settings-key-nextcloud-sync-interval-secs = Nextcloud automatic refresh interval
admin-settings-help-nextcloud-sync-interval-secs = Seconds between background collection refreshes for connected native-engine accounts. Zero is manual only; positive values have a sixty-second minimum. Failures back off. No background sign-in is attempted.
admin-settings-key-mail-folder-role-inbox = Inbox folder mapping
admin-settings-help-mail-folder-role-inbox = Null uses automatic discovery. Use an exact path object for shared defaults or an owned mailbox ID object for one account. Missing or conflicting targets disable this role; administrator locks apply.
admin-settings-key-mail-folder-role-drafts = Drafts folder mapping
admin-settings-help-mail-folder-role-drafts = Null uses automatic discovery. Use an exact path object for shared defaults or an owned mailbox ID object for one account. Missing or conflicting targets disable this role; administrator locks apply.
admin-settings-key-mail-folder-role-sent = Sent folder mapping
admin-settings-help-mail-folder-role-sent = Null uses automatic discovery. Use an exact path object for shared defaults or an owned mailbox ID object for one account. Missing or conflicting targets disable this role; administrator locks apply.
admin-settings-key-mail-folder-role-trash = Trash folder mapping
admin-settings-help-mail-folder-role-trash = Null uses automatic discovery. Use an exact path object for shared defaults or an owned mailbox ID object for one account. Missing or conflicting targets disable this role; administrator locks apply.
admin-settings-key-mail-folder-role-junk = Junk folder mapping
admin-settings-help-mail-folder-role-junk = Null uses automatic discovery. Use an exact path object for shared defaults or an owned mailbox ID object for one account. Missing or conflicting targets disable this role; administrator locks apply.
admin-settings-key-mail-folder-role-archive = Archive folder mapping
admin-settings-help-mail-folder-role-archive = Null uses automatic discovery. Use an exact path object for shared defaults or an owned mailbox ID object for one account. Missing or conflicting targets disable this role; administrator locks apply.

admin-settings-key-mail-request-delivery-receipt = Request delivery notifications by default
admin-settings-help-mail-request-delivery-receipt = Select delivery notifications for new messages when the account supports SMTP DSN. Locking controls this saved default; receipts.dsnAllowed separately governs whether requests are permitted. Delivery does not prove reading.
