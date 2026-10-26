# Settings

The **Settings** section of the admin console (`#/settings`) holds the values of the
settings model: one value for the whole server, optional values per domain and per
user, and whether users may choose their own. The server's registry
(`crates/mw-server/src/settings_registry.rs`) lists every key with its type, bounds,
default and scopes; this page describes each one.

## How a value is chosen

From broad to narrow: the whole server, the domain, a feature profile, a group, an
administrator's value for one user, and the user's own choice (only where users may
change it). The narrowest value that is set applies. A **lock** at any scope wins
over every narrower one: a locked server value applies to everyone, a locked domain
value to every user of the domain. With nothing set, the built-in default applies.

Some keys are tied to an environment variable:

- **pin** — when the variable is set, its value applies and the console cannot change
  the stored value (`409 pinnedByEnv`); the stored value applies once it is unset;
- **fallback** — the stored value applies; the variable stands in for the default
  when nothing is stored.

Changes apply within 30 seconds on every server process. Every change is recorded in
the audit log with the target `setting:<key>`; the History link of a setting opens
those entries.

Locking a setting that has narrower values first shows how many are ignored by the
lock. They are kept, and apply again after unlocking.

The `signin.*` keys are changed in **Sign-in policy** (and the LDAP check in
**Integrations**), which check the effect of a change before it is saved; the
Settings section shows them read-only.

## Settings

<a id="ui.language"></a>
### Language (`ui.language`)

The language of the mail client for users who have not chosen one. "Same as the browser" follows each browser's languages.

<a id="appearance.theme"></a>
### Theme (`appearance.theme`)

The colour theme users see until they pick their own. Locking it holds every user to this theme.

<a id="appearance.accent"></a>
### Accent colour (`appearance.accent`)

The accent colour of the theme. Empty means the theme's own accent.

<a id="appearance.density"></a>
### Density (`appearance.density`)

How much space lists and controls take: compact, cozy or relaxed.

<a id="appearance.brandName"></a>
### Product name (`appearance.brandName`)

The name shown on the sign-in page and in the browser tab.

<a id="login.notice"></a>
### Sign-in notice (`login.notice`)

A short text shown on the sign-in page, for example a support contact. Leave empty for none.

<a id="mail.readReceipts"></a>
### Read receipts (`mail.readReceipts`)

What happens when an opened message asks for a read receipt: ask the user, never send, or always send.

<a id="notifications.enabled"></a>
### New-mail notifications (`notifications.enabled`)

Whether users get notifications for new mail. It does not change their notification rules.

<a id="offline.retentionDays"></a>
### Offline cache age (`offline.retentionDays`)

How many days a device keeps messages for offline use. 0 means no age limit.

<a id="offline.budgetMb"></a>
### Offline cache size (`offline.budgetMb`)

How many megabytes a device may use for offline mail. 0 means no offline cache.

<a id="assistant.enabled"></a>
### AI assistant (`assistant.enabled`)

Whether users see the AI assistant. The deployment switch in AI assistant still applies above this.

<a id="upstream.default"></a>
### Default mail server (`upstream.default`)

The mail server an address signs in to when no domain names one. The domain table wins over this.

<a id="upstream.proxyTimeoutSecs"></a>
### Mail server timeout (`upstream.proxyTimeoutSecs`)

How long one request to the mail server may take, in seconds.

<a id="signin.newUsers"></a>
### New users (`signin.newUsers`)

Who may sign in for the first time. Changed in Sign-in policy.

<a id="signin.domainMode"></a>
### Sign-in domains (`signin.domainMode`)

Whether any domain may sign in or only listed domains. Changed in Sign-in policy.

<a id="signin.refusalDetail"></a>
### Refusal messages (`signin.refusalDetail`)

Whether a refused sign-in says why (disabled account, address not allowed) or gives the same answer as a wrong password. Changed in Sign-in policy.

<a id="signin.ldapBindPrecheck"></a>
### LDAP check before sign-in (`signin.ldapBindPrecheck`)

Check the password against the directory before the mail server. Changed in Integrations, after a test bind.

<a id="twofa.pendingTtlSecs"></a>
### Second-factor time limit (`twofa.pendingTtlSecs`)

How long a user has to enter the second factor after the password, in seconds.

<a id="twofa.maxAttempts"></a>
### Second-factor attempts (`twofa.maxAttempts`)

How many wrong second-factor codes are allowed before the sign-in starts over.

<a id="admin.session.idleSecs"></a>
### Console idle timeout (`admin.session.idleSecs`)

How long an admin console session lasts without a request, in seconds.

<a id="admin.session.absoluteSecs"></a>
### Console session limit (`admin.session.absoluteSecs`)

The longest an admin console session lasts, in seconds, active or not.

<a id="passwd.policy.minLength"></a>
### Minimum password length (`passwd.policy.minLength`)

The fewest characters a new password may have.

<a id="passwd.policy.requireUpper"></a>
### Require an upper-case letter (`passwd.policy.requireUpper`)

A new password must contain an upper-case letter.

<a id="passwd.policy.requireLower"></a>
### Require a lower-case letter (`passwd.policy.requireLower`)

A new password must contain a lower-case letter.

<a id="passwd.policy.requireDigit"></a>
### Require a digit (`passwd.policy.requireDigit`)

A new password must contain a digit.

<a id="passwd.policy.requireSymbol"></a>
### Require a symbol (`passwd.policy.requireSymbol`)

A new password must contain a character that is not a letter or a digit.

<a id="limit.storage_bytes"></a>
### Storage limit (`limit.storage_bytes`)

The most bytes one user may store on this server. 0 means no limit.

<a id="limit.send_per_day"></a>
### Messages sent per day (`limit.send_per_day`)

The most messages one user may send in a day. 0 means no limit.

<a id="limit.ai_calls_per_day"></a>
### AI assistant calls per day (`limit.ai_calls_per_day`)

The most AI assistant requests one user may make in a day. 0 means no limit.

<a id="quota.warnThresholds"></a>
### Usage warning levels (`quota.warnThresholds`)

The percentages of a limit at which users are warned, lowest first.

<a id="quota.warnEnabled"></a>
### Usage warnings (`quota.warnEnabled`)

Whether users are warned when they near a limit.

<a id="usage.retentionDays"></a>
### Usage history (`usage.retentionDays`)

How many days of daily usage counts are kept.

<a id="usage.meterApi"></a>
### Count API use (`usage.meterApi`)

Whether API requests are counted in the usage figures.

<a id="usage.measureIntervalHours"></a>
### Storage measuring interval (`usage.measureIntervalHours`)

How often the stored data of each user is measured, in hours.

<a id="assistant.rateLimitPerMin"></a>
### AI assistant rate limit (`assistant.rateLimitPerMin`)

The most AI assistant requests per minute for the whole server. 0 means no limit.

<a id="plugins.kv.maxKeyBytes"></a>
### Plugin key size (`plugins.kv.maxKeyBytes`)

The longest key a plugin may store, in bytes.

<a id="plugins.kv.maxValueBytes"></a>
### Plugin value size (`plugins.kv.maxValueBytes`)

The largest value a plugin may store under one key, in bytes.

<a id="plugins.kv.maxTotalBytes"></a>
### Plugin storage (`plugins.kv.maxTotalBytes`)

The most bytes one plugin may store in total.

<a id="plugins.kv.maxKeys"></a>
### Plugin keys (`plugins.kv.maxKeys`)

The most keys one plugin may store.

<a id="import.maxMessages"></a>
### Import message count (`import.maxMessages`)

The most messages one import may bring in.

<a id="import.maxMessageBytes"></a>
### Import message size (`import.maxMessageBytes`)

The largest message an import accepts, in bytes.

<a id="webhooks.maxAttempts"></a>
### Webhook attempts (`webhooks.maxAttempts`)

How many times a webhook delivery is tried before it is given up.

<a id="webhooks.timeoutSecs"></a>
### Webhook timeout (`webhooks.timeoutSecs`)

How long one webhook delivery may take, in seconds.

<a id="outbox.maxAttempts"></a>
### Sending attempts (`outbox.maxAttempts`)

How many times a message is handed to the mail server before it is reported as not sent.

<a id="realtime.recheckSecs"></a>
### Live update re-check (`realtime.recheckSecs`)

How often an open live-update connection checks that its session is still valid, in seconds.

<a id="privacy.watermark.enabled"></a>
### Reader watermark (`privacy.watermark.enabled`)

Show a watermark with the reader's address over opened messages.

<a id="privacy.watermark.opacity"></a>
### Watermark strength (`privacy.watermark.opacity`)

How visible the watermark is, from 0.02 (faint) to 0.5.

<a id="privacy.openTracking.retentionDays"></a>
### Open tracking history (`privacy.openTracking.retentionDays`)

How many days read-tracking records are kept.

<a id="privacy.openTracking.unboundTtlHours"></a>
### Unsent tracking tokens (`privacy.openTracking.unboundTtlHours`)

How many hours a tracking token waits for its message to be sent before it is deleted.

<a id="maskedEmail.domain"></a>
### Masked address domain (`maskedEmail.domain`)

The domain masked addresses are created under. Leave empty to turn masked addresses off.

<a id="abuse.reportAddress"></a>
### Abuse report address (`abuse.reportAddress`)

Where spam and abuse reports are sent.

<a id="dlp.adminAddress"></a>
### Data loss notices (`dlp.adminAddress`)

Where data loss prevention notices are sent.

<a id="observability.errorForwardUrl"></a>
### Error forwarding address (`observability.errorForwardUrl`)

An https address that client error reports are forwarded to. Leave empty for none.

<a id="egress.probeUrl"></a>
### Outbound test address (`egress.probeUrl`)

The https address used to test an outbound proxy.

<a id="push.vapidContact"></a>
### Push contact (`push.vapidContact`)

The contact (mailto: or https:) given to push services with each notification.

<a id="push.quietHours"></a>
### Push quiet hours (`push.quietHours`)

Hours (UTC) in which no push notification is sent. Leave empty for none.

<a id="updates.check"></a>
### Check for updates (`updates.check`)

Whether the server checks once a day for a newer release and shows it on the dashboard.

<a id="updates.feedUrl"></a>
### Release feed (`updates.feedUrl`)

The https address the update check reads.

<a id="updates.apply"></a>
### Apply updates (`updates.apply`)

Whether an update can be started from the console through the host update agent.

<a id="backup.retentionCount"></a>
### Backups kept (`backup.retentionCount`)

How many database backups are kept.

<a id="about.securityContact"></a>
### Security contact (`about.securityContact`)

The contact in security.txt (mailto: or https:).

<a id="about.securityPolicyUrl"></a>
### Security policy (`about.securityPolicyUrl`)

The address of your security policy, listed in security.txt.

<a id="about.projectLinks"></a>
### About links (`about.projectLinks`)

Links shown on the About page, each with a label and an https address.

<a id="config.snapshotsKept"></a>
### Configuration snapshots (`config.snapshotsKept`)

How many configuration snapshots are kept.

<a id="jobs.historyDays"></a>
### Job history (`jobs.historyDays`)

How many days of job runs are kept.

<a id="maintenance.announceMinutesDefault"></a>
### Maintenance notice lead time (`maintenance.announceMinutesDefault`)

How many minutes before a maintenance window users are told about it.

<a id="maintenance.defaultMessage"></a>
### Maintenance message (`maintenance.defaultMessage`)

The text users see during maintenance, per language.

<a id="health.diskFreeMinPercent"></a>
### Free disk space floor (`health.diskFreeMinPercent`)

The readiness check fails when free disk space falls below this percentage.

<a id="observability.otlpMetrics"></a>
### Send metrics by OTLP (`observability.otlpMetrics`)

Send server metrics to the OTLP collector set at start.

<a id="metrics.drilldownTopN"></a>
### Metrics detail rows (`metrics.drilldownTopN`)

How many rows a metrics breakdown shows.

<a id="metrics.rollupDays"></a>
### Metrics history (`metrics.rollupDays`)

How many days of hourly metric summaries are kept.

<a id="diagnostics.retentionDays"></a>
### Diagnostics history (`diagnostics.retentionDays`)

How many days of diagnostic runs are kept.

<a id="dataExport.linkHours"></a>
### Data export download window (`dataExport.linkHours`)

How many hours a data export can be downloaded before it is deleted.

<a id="serverText.*"></a>
### Server text (`serverText.*`)

Text the server writes into mail and notifications, per language and domain.

<a id="sieve.maxScriptBytes"></a>
### Maximum filter script size (`sieve.maxScriptBytes`)

Maximum bytes per Sieve script; default 65,536, range 1–1,048,576.

<a id="sieve.maxScripts"></a>
### Maximum saved filter scripts (`sieve.maxScripts`)

Maximum scripts per account; default 5, range 1–100.

<a id="sieve.maxRedirects"></a>
### Maximum redirects (`sieve.maxRedirects`)

Maximum redirect commands per script; default 4, range 0–100.

<a id="sieve.vacationAllowed"></a>
### Vacation actions (`sieve.vacationAllowed`)

Allow vacation commands when the backend supports them. Disabled by default.

<a id="sieve.allowedExtensions"></a>
### Allowed filter extensions (`sieve.allowedExtensions`)

Optional list of lowercase extension names, at most 128 names of 64 bytes each.
Unset adds no policy restriction; backend support and all other limits still apply.

<a id="cap.sieve.raw_edit"></a>
### Raw filter editing (`cap.sieve.raw_edit`)

Allow editing Sieve source. Disabled by default.

<a id="cap.sieve.forward_external"></a>
### External forwarding (`cap.sieve.forward_external`)

Allow scripts to forward outside the account's domain. Disabled by default;
saving with this permission also requires recent verification. These seven
settings support global, domain and administrator-set account values. Mailbox
users cannot override them.

## Session and factor policy

<a id="session.maxConcurrent"></a>
`session.maxConcurrent` limits sessions per account to 1–100 (default 10).
Oldest sessions are revoked when a successful sign-in exceeds the cap.
<a id="session.idleSecs"></a>
`session.idleSecs` is 60–86400 seconds (default 1800).
<a id="session.absoluteSecs"></a>
`session.absoluteSecs` is 300–1296000 seconds (default 1296000: 15 days).
Deployment timeout limits also apply; persisted timestamps survive restarts.
<a id="twofa.rememberDays"></a>
`twofa.rememberDays` is 0–90 days (default 0, disabled).
<a id="twofa.passkeysEnabled"></a>
`twofa.passkeysEnabled` defaults to true.
<a id="twofa.required"></a>
`twofa.required` defaults to false. These six settings support global, domain,
group, profile and administrator-set account policy, without user overrides.

## Reader privacy

<a id="privacy.remoteContent"></a>
`privacy.remoteContent`: `block`, `ask` (default), `proxy`, or `direct`.
<a id="privacy.trackers"></a>
`privacy.trackers` defaults to true.
<a id="privacy.cleanLinks"></a>
`privacy.cleanLinks` defaults to true. These three support global, domain and user settings.
<a id="privacy.maxBytes"></a>
`privacy.maxBytes`: 65536–8388608 bytes, default 8388608.
<a id="privacy.cacheBytes"></a>
`privacy.cacheBytes`: 0–1073741824 bytes, default 33554432.
<a id="privacy.ratePerMin"></a>
`privacy.ratePerMin`: 10–6000 fetches per account per minute, default 120.
<a id="privacy.allowDirect"></a>
`privacy.allowDirect` defaults to false; direct loading also requires the account capability.

<a id="cap.remote_content.direct"></a>
`cap.remote_content.direct` defaults to false and is assigned by administrators.
Direct image loading requires both this permission and `privacy.allowDirect`.
These four resource and deployment limits are global settings.

The reader reports HTML tracker detections separately from remote images that
the server decoded and replaced. The latter counts distinct image URLs for the
current session and message over the signed URL's one-hour lifetime. These
counts are held in memory, can reset after restart or eviction, and retain at
most 256 URLs per message across 1024 recent messages. A saturated count is
shown as a lower bound. They do not count images loaded directly by the browser.

## Feature and integration permissions

These administrator-only settings use the same global, domain, profile, group,
and per-user policy chain. Capability defaults are true except custom CSS and
backgrounds. Broader locks win. The feature catalogue distinguishes enforced,
token-acquisition-only, and pending controls; a defined key alone does not mean
every integration path is governed yet.

Individual UI-plugin keys use SHA-256 of the original plugin ID as their suffix.
Use `integrationPolicyKeys` from `/admin/features`; do not normalize or construct
keys from displayed plugin names. Existing plugin approval and grants still apply.

WASM plugin policy keys also use SHA-256 of the original ID and are returned in
the same catalogue. Loaded plugins recheck permissions for each invocation and
privileged host import, including requests carrying a cached OAuth token.
Credential imports require an instance explicitly bound by the host to that
account; a guest cannot request another account's credentials. Account-aware
spam classification applies the recipient account's domain and profile policy.

<a id="cap.integration.nextcloud"></a>
`cap.integration.nextcloud` — Allow Nextcloud file and share-link operations.

<a id="cap.integration.google"></a>
`cap.integration.google` — Allow acquisition and refresh of Google bridge tokens.

<a id="cap.integration.microsoft"></a>
`cap.integration.microsoft` — Allow acquisition and refresh of Microsoft bridge tokens.

<a id="cap.integration.*"></a>
`cap.integration.*` — Permission for an individual integration identifier; runtime coverage is listed in the feature catalogue.

<a id="cap.ui_plugin.*"></a>
`cap.ui_plugin.*` — Permission for one UI plugin. Use the exact policy key from the integration catalogue.

<a id="cap.ai_assistant"></a>
`cap.ai_assistant` — Allow assistant configuration and invocation.

<a id="cap.plugins"></a>
`cap.plugins` — Allow UI plugins and bridge token acquisition.

<a id="cap.pim.calendar"></a>
`cap.pim.calendar` — Allow calendar methods and subscriptions.

<a id="cap.pim.contacts"></a>
`cap.pim.contacts` — Allow contact and address-book methods.

<a id="cap.pim.tasks"></a>
`cap.pim.tasks` — Allow task methods.

<a id="cap.pim.notes"></a>
`cap.pim.notes` — Allow note methods.

<a id="cap.export"></a>
`cap.export` — Allow mail and PIM exports.

<a id="cap.import"></a>
`cap.import` — Allow mail and PIM imports.

<a id="cap.api_keys"></a>
`cap.api_keys` — API-key policy. Check the feature catalogue for enforcement coverage.

<a id="cap.oauth_apps"></a>
`cap.oauth_apps` — OAuth-application policy. Check the feature catalogue for enforcement coverage.

<a id="cap.rules"></a>
`cap.rules` — Allow managed mail-rule and vacation methods.

<a id="cap.rules.forward"></a>
`cap.rules.forward` — Forwarding policy. Check the feature catalogue for enforcement coverage.

<a id="cap.remote_images"></a>
`cap.remote_images` — Remote-image capability. Existing privacy settings continue to govern requests; check feature coverage.

<a id="cap.user_background"></a>
`cap.user_background` — Custom-background capability, disabled by default. Check feature coverage.

<a id="cap.user_css"></a>
`cap.user_css` — Custom-CSS capability, disabled by default. Check feature coverage.

<a id="limit.attachment_bytes"></a>
`limit.attachment_bytes` — Attachment policy in bytes, default 50000000. Runtime enforcement remains pending.

<a id="limit.send_per_hour"></a>
`limit.send_per_hour` — Submission policy per hour; zero means unlimited. Runtime enforcement remains pending.

<a id="limit.recipients_per_message"></a>
`limit.recipients_per_message` — Recipient-count policy; zero means unlimited. Runtime enforcement remains pending.

<a id="logs.auth.retentionDays"></a>
### Authentication history (`logs.auth.retentionDays`)
Global, hot. Default 7 days; range 1–90. Authentication history is pruned on writes. Durable successful webmail identity is stored separately, so pruning does not change policy identity.

<a id="logs.auth.maxEntries"></a>
### Authentication entry limit (`logs.auth.maxEntries`)
Global, hot. Default 50,000; range 1,000–100,000. Oldest entries are removed on writes. Matching failed attempts in the same minute share one bounded counter; successful authentications remain separate. The privileged authentication log requires unscoped `logs.read` and distinguishes administration from webmail. Unknown transport details remain unavailable; no raw upstream error or credential is stored.

### Message display

<a id="mail.requestReadReceipt"></a>
`mail.requestReadReceipt` defaults to `false` and sets the composing default for
requesting a receipt on supported outgoing mail. It does not compel a recipient
to return a receipt. Incoming receipt responses remain governed separately by
`mail.readReceipts` (`ask`, `never`, `always`).

These keys support global and domain defaults, administrator locks and user overrides. The settings apply inside the isolated message renderer, without changing its content security policy or remote-image permissions.

| Key | Values | Default |
| --- | --- | --- |
| <a id="reader.colorMode"></a>`reader.colorMode` | auto, original, adaptive, invert | auto |
| <a id="reader.fontSize"></a>`reader.fontSize` | 12–24 pixels | 14 |
| <a id="reader.lineHeight"></a>`reader.lineHeight` | 1.2–2.2 | 1.6 |
| <a id="reader.contentWidth"></a>`reader.contentWidth` | comfortable, wide, full | wide |
| <a id="reader.loading"></a>`reader.loading` | skeleton, indicator, none | skeleton |
| <a id="reader.loadingDelayMs"></a>`reader.loadingDelayMs` | 0–1000 milliseconds | 120 |

Automatic colors adapt in dark themes and preserve original colors in light themes. Inversion reverses message colors while counter-inverting images. Adaptive mode removes sender foreground/background overrides while preserving layout and media sources. Loader delay affects only feedback, never delivery.

<a id="mail.quoteStyle"></a>
### `mail.quoteStyle`

Default `classic`; choose `outlook` for structured From, Sent, To, Cc and Subject headers in replies and forwards. Global, domain and user scope. Administrators can lock the default. Only newly opened compositions change; existing drafts retain their content.

<a id="signin.retryCooldownSecs"></a>
### `signin.retryCooldownSecs`
Global initial password sign-in cooldown, 0–300 seconds, default 3. Applies to mailbox/native and administrator login using the trusted client IP (IPv6 /64). MFA continuation and administrator session expiry are unchanged. Refusals return 429 with Retry-After. Bounded process-local state resets on restart and is independent per replica. Existing failure bans remain in force.

<a id="mail.externalRecipientWarning"></a>
### `mail.externalRecipientWarning`
Global/domain/user composer safeguard: `off`, `warn` (default), or `confirm`. Checks To, Cc and Bcc. Confirmation is invalidated when the external recipient set or sending identity changes. This is not a server-side delivery block and does not restrict other mail clients or direct API submissions.

<a id="mail.trustedRecipientDomains"></a>
### `mail.trustedRecipientDomains`
Global/domain/user list of up to 100 exact domains, each at most 253 characters. DNS case and IDNA are normalized; invalid entries grant no trust. Subdomains are not implicitly trusted. The selected sending identity's domain is trusted automatically. Administrators can lock both settings to prevent user overrides; an unlocked user list replaces the inherited list.

<a id="login.showDomainHint"></a>
### `login.showDomainHint`

Global boolean, default `false`. Shows the preferred-domain hint below Email or username on the sign-in page when enabled. Hiding this hint does not change username routing or authentication.

<a id="mail.quickReply"></a>
`mail.quickReply` defaults to `false`. This checkpoint registers the preference only; the compact reply composer is not wired yet.
It supports global/domain defaults, administrator locks and user overrides.
