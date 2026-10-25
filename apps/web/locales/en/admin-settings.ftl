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
admin-settings-key-appearance-brand-name = Product name
admin-settings-help-appearance-brand-name = The name shown on the sign-in page and in the browser tab.
admin-settings-key-login-notice = Sign-in notice
admin-settings-help-login-notice = A short text shown on the sign-in page, for example a support contact. Leave empty for none.
admin-settings-key-mail-read-receipts = Read receipts
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
