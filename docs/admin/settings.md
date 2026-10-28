# Settings

The **Settings** section of the admin console (`#/settings`) holds the values of the
settings model: one value for the whole server, optional values per domain and per
user, and whether users may choose their own. The server's registry
(`crates/mw-server/src/settings_registry.rs`) lists every key with its type, bounds,
default and scopes; this page describes each one.

## Linked mailbox accounts

<a id="accounts.linkingEnabled"></a>
`accounts.linkingEnabled` is an administrative boolean, disabled by default.
Global `false` is a hard ceiling. When globally enabled, domain and per-user
administrative restrictions still apply to both the primary and child account.

<a id="accounts.useLinked"></a>
`accounts.useLinked` defaults to `true` and lets a user opt out of linked mailbox
access. It never overrides administrative disablement. Disabling access does not
delete a linked mailbox, its messages, identities, signatures or preferences.

<a id="accounts.maxLinked"></a>
`accounts.maxLinked` is the global maximum number of verified child mailboxes per
primary identity: default `5`, permitted range `1`–`16`.

This initial release supports native IMAP, POP3 and EWS accounts. Proxy JMAP,
SSO-only and header-only contexts are refused: the existing proxy second-factor
storage still needs verified-realm isolation before it can safely support links.
Adding an account requires a fresh primary proof and the child's actual password
and any enrolled second factor. A forced first-factor enrollment or password
change must be completed through a separate sign-in before linking. Existing
metadata-only legacy links are not adopted. Child authentication expires at its
original deadline; unlink and link again to authenticate afresh. Switching never
changes the primary identity used for identity-provider consent or account
management. This initial runtime supports explicit mail and personal-settings
routes; linked realtime, external integrations and security-factor management
are unavailable, rather than silently executed as the primary account.

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

<a id="packages.allowed"></a>
### Package tracking allowed (`packages.allowed`)

Restricts package tracking globally, by domain or for an individual account. Default `true` only permits availability: the separately configured provider and explicit personal opt-in both remain off by default. A false effective value stops provider calls; local removal and opting out remain available. See [Package tracking](package-tracking.md).

<a id="ui.language"></a>
### Language (`ui.language`)

The language of the mail client for users who have not chosen one. "Same as the browser" follows each browser's languages.

<a id="ui.bar.mailTopBar"></a>
### Mail toolbar (`ui.bar.mailTopBar`)

`pinned` (default) keeps the mail toolbar, with the folder, list and reader layout controls, always shown. `auto` hides it behind a slim handle; it slides out when the user points at, taps or tabs to the handle. Users change it under **Settings → Toolbars** unless the value is locked.

<a id="ui.bar.workspaceTabs"></a>
### Workspace tabs (`ui.bar.workspaceTabs`)

`pinned` (default) keeps the workspace tab strip always shown. `auto` shows only a slim handle with the active tab's name until the user points at, taps or tabs to it.

<a id="ui.bar.mailRibbon"></a>
### Mail ribbon (`ui.bar.mailRibbon`)

Applies to the ribbon layout. `pinned` (default) keeps the ribbon open. `auto` shows only its tab names; a ribbon tab opens over the message list when the user points at, taps or tabs to it.

<a id="ui.bar.composerRibbon"></a>
### Editor ribbon (`ui.bar.composerRibbon`)

Applies to every composer, embedded or in its own window. `pinned` (default) keeps the editor ribbon open. `auto` shows only its tab names; opening a tab pushes the message body down instead of covering it.

All four bar keys take `pinned` or `auto` and have global and domain defaults, administrator locks and user values. A locked bar shows its pin button disabled with "Set by your administrator".

<a id="appearance.theme"></a>
### Theme (`appearance.theme`)

The colour theme users see until they pick their own. Locking it holds every user to this theme.

<a id="appearance.accent"></a>
### Accent colour (`appearance.accent`)

The accent colour of the theme. Empty means the theme's own accent.

<a id="appearance.badgeColor"></a>
### Unread badge colour (`appearance.badgeColor`)

The colour of the unread-count badges on folders, inbox tabs and the mailbox picker. `default` keeps the theme's signal colour; `accent` follows the account's accent colour; the ten named colours (`red`, `orange`, `yellow`, `green`, `teal`, `blue`, `indigo`, `violet`, `magenta`, `graphite`) use the same per-theme shades as the named accents, so the count text keeps at least 4.5:1 contrast in every theme. Sets the colour for accounts without a saved choice; a lock overrides the account choice and disables the badge colour buttons in user Appearance settings.

<a id="appearance.density"></a>
### Density (`appearance.density`)

How much space lists and controls take: compact, cozy or relaxed.

<a id="appearance.font"></a>
### Interface font (`appearance.font`)

Sets the font for accounts without a saved choice. A lock overrides the account choice and disables the font buttons in user Appearance settings. Available values are `default` (theme font), `system`, `serif` (Newsreader), `mono` (JetBrains Mono), `inter`, `roboto`, `open-sans`, and `jost`. Font assets are bundled and served locally; the app does not contact Google Fonts. Account font choices synchronize through the account appearance endpoint.

<a id="appearance.brandName"></a>
### Product name (`appearance.brandName`)

The name shown on the sign-in page and in the browser tab.

<a id="login.notice"></a>
### Sign-in notice (`login.notice`)

A short text shown on the sign-in page, for example a support contact. Leave empty for none.

<a id="receipts.dsnAllowed"></a>
### Delivery status requests (`receipts.dsnAllowed`)

Allow SMTP delivery-status requests, globally, by domain, or by address. The
default is true. This administrator-only permission is checked before queueing
and at transmission; it does not request or establish that a person read mail.

<a id="mail.readReceipts"></a>
### Read receipts (`mail.readReceipts`)

What happens when a displayed message requests a read receipt: ask, never, automatic when safe, contacts only, organisation only, or automatic inside the organisation and ask elsewhere. Administrator-held modes override personal sender exceptions.

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
`cap.integration.nextcloud` — Allow account-linked Nextcloud files, sharing and calendar/contact/task operations.

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

<a id="mail.quickReply"></a>
`mail.quickReply` defaults to `false`. When enabled, the reader offers a compact reply tab with prefilled recipients and subject. Users can expand those fields without losing the draft. The same composer handles signatures, threading, encryption, DLP, external-recipient confirmation and submission.
It supports global/domain defaults, administrator locks and user overrides.

<a id="mail.requestReadReceipt"></a>
`mail.requestReadReceipt` defaults to `false` and sets the composing default for
requesting a receipt on supported outgoing mail. It does not compel a recipient
to return a receipt. Incoming receipt responses remain governed separately by
`mail.readReceipts` (`ask`, `never`, `always`, `contactsOnly`, `orgOnly`, `orgAlwaysAskOthers`).

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

<a id="login.showDomainHint"></a>
### `login.showDomainHint`

Global boolean, default `false`. Shows the preferred-domain hint below Email or username on the sign-in page when enabled. Hiding this hint does not change username routing or authentication.

<a id="login.rememberSignins"></a>
### `login.rememberSignins`

Global boolean, default `true`. Allows this browser to remember successful sign-in addresses for later selection. Only addresses are remembered, never passwords, tokens or other credentials. The public sign-in configuration exposes `rememberSignins`; when it is false, the client clears remembered addresses and stops recording them. This does not change session lifetime or authentication requirements.

<a id="login.showRememberedHint"></a>
### `login.showRememberedHint`

Global boolean, default `false`. Controls explanatory text beside remembered sign-in addresses. The public sign-in configuration exposes `showRememberedHint`. Enabling this hint does not enable remembering when `login.rememberSignins` is false.

<a id="signin.retryCooldownSecs"></a>
### `signin.retryCooldownSecs`
Global password sign-in retry cooldown, 0–300 seconds, default 3. Applies to mailbox/native and administrator login using the trusted client IP (IPv6 /64). Only an attempt that does not end in a signed-in session starts the wait: a wrong password, unknown user, disabled account, backend error, or a password that only reaches an MFA challenge. Each attempt holds its slot while it runs, so parallel attempts from one source are still refused; a successful sign-in gives its slot back, so the next user behind the same address is not delayed. MFA continuation and administrator session expiry are unchanged. Refusals return 429 with Retry-After. Bounded process-local state resets on restart and is independent per replica. Existing failure bans remain in force.

<a id="mail.quoteStyle"></a>
### `mail.quoteStyle`

Default `classic`; choose `outlook` for structured From, Sent, To, Cc and Subject headers in replies and forwards. Global, domain and user scope. Administrators can lock the default. Only newly opened compositions change; existing drafts retain their content.

<a id="mail.externalRecipientWarning"></a>
### `mail.externalRecipientWarning`
Global/domain/user composer safeguard: `off`, `warn` (default), or `confirm`. Checks To, Cc and Bcc. Confirmation is invalidated when the external recipient set or sending identity changes. This is not a server-side delivery block and does not restrict other mail clients or direct API submissions.

<a id="mail.trustedRecipientDomains"></a>
### `mail.trustedRecipientDomains`
Global/domain/user list of up to 100 exact domains, each at most 253 characters. DNS case and IDNA are normalized; invalid entries grant no trust. Subdomains are not implicitly trusted. The selected sending identity's domain is trusted automatically. Administrators can lock both settings to prevent user overrides; an unlocked user list replaces the inherited list.

<a id="signin.antiBotMode"></a>
### `signin.antiBotMode`
Global self-hosted ALTCHA proof-of-work policy: `off` (default), `mailbox`, `admin`, or `all`. Applies to initial password login, including native clients; MFA continuation remains protected by existing factor attempt limits. The widget and worker are served locally with no external verification service. Passkey ceremonies retain their challenge/signature protections.
<a id="signin.antiBotDifficulty"></a>
### `signin.antiBotDifficulty`
Global solver counter ceiling, 100–20000, default2000. Higher values increase client CPU work; PBKDF2/SHA-256 cost is fixed and server verification uses the signed derived-key fast path.
<a id="signin.antiBotExpirySecs"></a>
### `signin.antiBotExpirySecs`
Global issued challenge lifetime, 60–600seconds, default180. Every proof is consumed atomically once before credential processing and bound to its login surface and trusted client network. At most10000 outstanding challenges and8 per network are retained; issuance also has a bounded process-local rate limit. Expired rows are removed during issuance. Issued proofs are deliberately omitted by migrate-store; users obtain fresh verification after migration. Replicas require the same persistent server key and database.


## Attachment access policy

Global and domain administrators can restrict original attachment delivery and previews. Policy uses stored message MIME metadata; it does not detect malware or prevent copying bytes already delivered to a client. Under restrictive rules, whole-message and nested-message exports are refused rather than bypassing attachment controls. Proxy downloads require an owned email ID to resolve the blob metadata. Unnamed uploaded blobs cannot satisfy filename rules.

<a id="attachments.previewAllowed"></a>
### `attachments.previewAllowed`

Disable to prevent attachment preview rendering; original downloads have their own control.

<a id="attachments.downloadAllowed"></a>
### `attachments.downloadAllowed`

Disabling also prevents previews that require original bytes. Browser previews are not copy protection.

<a id="attachments.maxBytes"></a>
### `attachments.maxBytes`

Zero is unlimited. Applies to original delivered bytes, including range requests.

<a id="attachments.allowedExtensions"></a>
### `attachments.allowedExtensions`

JSON array of lowercase extensions without a dot, such as ["pdf","tar.gz"]. Empty allows any extension; blocked rules win.

<a id="attachments.blockedExtensions"></a>
### `attachments.blockedExtensions`

JSON array of lowercase extensions. Evaluated against stored MIME filenames, never the download URL name.

<a id="attachments.warnExtensions"></a>
### `attachments.warnExtensions`

JSON array of extensions requiring explicit acknowledgement before original delivery.

<a id="attachments.allowedMimeTypes"></a>
### `attachments.allowedMimeTypes`

JSON array of lowercase types or top-level wildcards such as ["image/*","application/pdf"]. Empty allows any type.

<a id="attachments.blockedMimeTypes"></a>
### `attachments.blockedMimeTypes`

JSON array of lowercase types or top-level wildcards. Blocked rules take precedence.

<a id="attachments.warnMimeTypes"></a>
### `attachments.warnMimeTypes`

JSON array of lowercase types or top-level wildcards requiring acknowledgement. MIME metadata is not malware detection.

## Time zone helpers

<a id="time.displayZone"></a>`time.displayZone` is global/domain/user text, default `local`. Use a supported IANA zone such as `Europe/Lisbon`, or `local` for the device zone. This changes display, not stored message instants.

<a id="time.comparisonZone"></a>`time.comparisonZone` is global/domain/user text, default empty. A second explicit IANA zone supports sender/recipient scheduling comparisons; no person's location is inferred from their address.

<a id="time.showHelpers"></a>`time.showHelpers` is global/domain/user boolean, default true. It controls time comparison helpers. Calendar event zones remain part of each event's actual timezone-aware data.

## Keyboard shortcuts

<a id="shortcuts.enabled"></a>
`shortcuts.enabled` is a global/domain/user boolean, default true. False disables all application shortcuts. The user's Off preset also always disables them. Admin locks use the normal settings policy model.

Each binding below is global/domain/user text (maximum 64 characters): `inherit` uses the selected preset; `disabled` removes that action; otherwise enter one key with optional Ctrl/Cmd, Alt and Shift modifiers. Duplicate chords disable all conflicting actions. Invalid bindings do not execute. Inputs, editors, IME and modal dialogs retain their usual keyboard behavior. Browser-reserved shortcuts may be unavailable on some platforms.

- <a id="shortcuts.compose"></a>`shortcuts.compose` — Compose; default `inherit`.
- <a id="shortcuts.archive"></a>`shortcuts.archive` — Archive; default `inherit`.
- <a id="shortcuts.reply"></a>`shortcuts.reply` — Reply; default `inherit`.
- <a id="shortcuts.replyAll"></a>`shortcuts.replyAll` — Reply all; default `inherit`.
- <a id="shortcuts.forward"></a>`shortcuts.forward` — Forward; default `inherit`.
- <a id="shortcuts.next"></a>`shortcuts.next` — Next message; default `inherit`.
- <a id="shortcuts.previous"></a>`shortcuts.previous` — Previous message; default `inherit`.
- <a id="shortcuts.search"></a>`shortcuts.search` — Focus search; default `inherit`.
- <a id="shortcuts.refresh"></a>`shortcuts.refresh` — Refresh messages; default `inherit`.
- <a id="shortcuts.settings"></a>`shortcuts.settings` — Open settings; default `inherit`.
- <a id="shortcuts.trash"></a>`shortcuts.trash` — Move to trash; default `inherit`.
- <a id="shortcuts.markRead"></a>`shortcuts.markRead` — Mark read; default `inherit`.
- <a id="shortcuts.markUnread"></a>`shortcuts.markUnread` — Mark unread; default `inherit`.
- <a id="shortcuts.spam"></a>`shortcuts.spam` — Mark as spam; default `inherit`.

<a id="notifications.pushAllowed"></a>
### `notifications.pushAllowed`
Administrator ceiling (global/domain, default true) for account-authorized background push registration and delivery. Account notifications must also be enabled. Browser permissions remain device-local.

<a id="offline.persistentStorageAllowed"></a>
### `offline.persistentStorageAllowed`
Administrator ceiling (global/domain, default false) for an explicit browser persistent-storage request. The browser decides whether to grant the request; this setting cannot silently grant permission.

<a id="security.signin.mailboxFailureLimit"></a>
<a id="security.signin.adminFailureLimit"></a>
<a id="security.signin.failureWindowSecs"></a>
<a id="security.signin.lockSecs"></a>
### Temporary credential-failure locks

Global limits default to 10 rejected mailbox credentials and 5 rejected admin credentials per
trusted source IP in a rolling 900-second window. `security.signin.mailboxFailureLimit` and
`security.signin.adminFailureLimit` accept 1–100. `security.signin.failureWindowSecs` and
`security.signin.lockSecs` accept 30–86400 seconds and default to 900. These locks are distinct
from `signin.retryCooldownSecs` (failed-attempt pacing) and manual global IP bans. Source counters
are durable across replicas; rejection while locked does not extend expiry. Admin and mailbox
counters are separate. A successful completed login resets its matching source/surface counter;
issuing an MFA challenge is not success. Typed network, configuration and unknown-challenge errors
do not count as rejected credentials. Existing permanent bans remain until an operator removes them.

<a id="security.signin.geoScope"></a>
<a id="security.signin.geoMode"></a>
<a id="security.signin.geoCountries"></a>
<a id="security.signin.geoUnknown"></a>
### Sign-in country policy

`security.signin.geoScope` is off by default; mailbox, admin, and all apply to new sign-ins only.
`security.signin.geoMode` selects allow (default) or deny for the uppercase two-letter country
list `security.signin.geoCountries` (default empty). `security.signin.geoUnknown` explicitly
allows or denies unmapped addresses (default deny). An active policy with missing or unreadable
`MW_GEOIP_DB` refuses admission, including when unknown addresses are allowed. The local database
is operator supplied and cached until restart; no external lookup occurs. Inspect the current-source
diagnostic under Observability before enabling. Existing authenticated requests and sessions are
not governed by this initial sign-in policy; broader session geofencing is not implemented here.

<a id="contacts.avatarMode"></a>
### Contact pictures (`contacts.avatarMode`)

Generated initials are the default and stay on the device. `off` hides avatars; `contacts` uses only embedded PNG/JPEG photos from the signed-in account address books, with initials as fallback. This global/domain/user setting supports administrator locks. No remote PHOTO URL, external avatar service, or inferred profile lookup is requested.

The server requires an authenticated account and exact contact email, reads at most 16 cards of 256 KiB each, and accepts embedded photos up to 128 KiB and 512 by 512 pixels. Images are re-encoded in the existing media jail and returned privately without caching. Unavailable or malformed pictures use initials. Imported or synchronized vCards can supply embedded photos; editing/uploading a profile photograph is a separate capability.

<a id="nextcloud.syncIntervalSecs"></a>
### Nextcloud refresh interval (`nextcloud.syncIntervalSecs`)
Global/domain/user integer, default 300 seconds, range 0–86400. Zero disables periodic refresh. Positive values have a 60-second runtime minimum. Only already connected native-engine accounts with active collection bindings are considered; the scheduler never logs into mailbox accounts. Two account refreshes run concurrently with a 30-second deadline and exponential failure backoff capped at one day. Manual and scheduled refreshes share the same account lock. Current capability, source route and linked credentials are checked again for each DAV operation.

<a id="reader.actionStyle"></a>
### Message action ribbon (`reader.actionStyle`)
Global/domain/user enum, default `icons`; `labels` adds visible text beside each native icon. Both styles preserve accessible action names, keyboard activation and themed explanatory tooltips. Mobile controls keep 44-pixel targets. User configuration is in Message display and administrator locks apply.

### Special-folder mappings

<a id="mail.folderRole.inbox"></a>
`mail.folderRole.inbox`: nullable owned mailbox target, global/domain/user, default null (automatic).

<a id="mail.folderRole.drafts"></a>
`mail.folderRole.drafts`: nullable owned mailbox target, global/domain/user, default null (automatic).

<a id="mail.folderRole.sent"></a>
`mail.folderRole.sent`: nullable owned mailbox target, global/domain/user, default null (automatic).

<a id="mail.folderRole.trash"></a>
`mail.folderRole.trash`: nullable owned mailbox target, global/domain/user, default null (automatic).

<a id="mail.folderRole.junk"></a>
`mail.folderRole.junk`: nullable owned mailbox target, global/domain/user, default null (automatic).

<a id="mail.folderRole.archive"></a>
`mail.folderRole.archive`: nullable owned mailbox target, global/domain/user, default null (automatic).

Use `{"path":"INBOX/Trash"}` for an exact upstream path or `{"id":"mailbox-id"}` for one account. Each role inherits and locks independently. Native engine overrides do not rewrite backend discovery data; resetting restores automatic roles. Missing targets and conflicting role assignments do not silently fall back. Proxy accounts retain their upstream role contract.

<a id="security.signin.geoEnforceSessions"></a>
`security.signin.geoEnforceSessions` defaults to false. When enabled, the same country scope,
list and unknown-address policy checks incoming authenticated mailbox/admin requests,
native bearer requests and signed reader content. It does not silently revoke stored sessions;
access resumes from an allowed source. Public pages and logout remain available. New sign-ins
and recovery sign-ins retain their existing country checks. Long-lived streams already open
before a policy change are checked on reconnect, not terminated retroactively.

## Aggregate and adaptive request protection

State is bounded to 20,000 subjects and 10,000 decision events. Expired state is excluded from enforcement and views immediately. A lifecycle-bound maintenance task removes at most 128 expired subjects and 128 expired events per minute, with additional cleanup during administrative reads and new-subject admission; physical removal can therefore lag logical expiry during a backlog. Empty OFF deployments perform no counter or maintenance writes. Decision events expire logically after 30 days.

<a id="security.protection.rateMode"></a>

`security.protection.rateMode` (default `"off"`): Off by default. Observe records budget exhaustion without refusal; enforce returns a temporary 429. Existing protocol concurrency limits remain independent.

<a id="security.protection.adaptiveMode"></a>

`security.protection.adaptiveMode` (default `"off"`): Off by default. Observe records explainable typed anomalies and proposed actions; enforce applies bounded temporary throttles or blocks. This is a deterministic counter policy, not an AI classifier.

<a id="security.protection.clientRequests"></a>

`security.protection.clientRequests` (default `600`): Aggregate request allowance across replicas for this verified subject dimension. Zero disables this dimension; source clients use only the trusted proxy address resolver.

<a id="security.protection.accountRequests"></a>

`security.protection.accountRequests` (default `300`): Aggregate request allowance across replicas for this verified subject dimension. Zero disables this dimension; source clients use only the trusted proxy address resolver.

<a id="security.protection.adminRequests"></a>

`security.protection.adminRequests` (default `120`): Aggregate request allowance across replicas for this verified subject dimension. Zero disables this dimension; source clients use only the trusted proxy address resolver.

<a id="security.protection.rateWindowSecs"></a>

`security.protection.rateWindowSecs` (default `60`): Fixed request-budget interval. Capacity load shedding and ordinary not-found responses are not adaptive evidence.

<a id="security.protection.adaptiveClients"></a>

`security.protection.adaptiveClients` (default `true`): Enable this subject dimension when adaptive mode is active. Accounts and administrators are counted only after their identity is verified.

<a id="security.protection.adaptiveAccounts"></a>

`security.protection.adaptiveAccounts` (default `true`): Enable this subject dimension when adaptive mode is active. Accounts and administrators are counted only after their identity is verified.

<a id="security.protection.adaptiveAdmins"></a>

`security.protection.adaptiveAdmins` (default `true`): Enable this subject dimension when adaptive mode is active. Accounts and administrators are counted only after their identity is verified.

<a id="security.protection.shortWindowSecs"></a>

`security.protection.shortWindowSecs` (default `300`): Rolling observation window. Long windows support hours through thirty days; bucket rounding can retain at most one extra sixty-fourth of a window.

<a id="security.protection.shortMinimumSamples"></a>

`security.protection.shortMinimumSamples` (default `30`): Minimum observed normal plus anomalous requests before the ratio trigger applies. The absolute anomaly threshold is independent of this minimum.

<a id="security.protection.shortAbsoluteAnomalies"></a>

`security.protection.shortAbsoluteAnomalies` (default `20`): Trigger when this many selected anomalies accumulate, even when normal requests dilute the ratio. Zero disables this trigger.

<a id="security.protection.shortRatioPercent"></a>

`security.protection.shortRatioPercent` (default `50`): Trigger when selected anomalies reach this percentage after the minimum sample count. Either the absolute or ratio trigger suffices. Zero disables this trigger.

<a id="security.protection.longWindowSecs"></a>

`security.protection.longWindowSecs` (default `604800`): Rolling observation window. Long windows support hours through thirty days; bucket rounding can retain at most one extra sixty-fourth of a window.

<a id="security.protection.longMinimumSamples"></a>

`security.protection.longMinimumSamples` (default `100`): Minimum observed normal plus anomalous requests before the ratio trigger applies. The absolute anomaly threshold is independent of this minimum.

<a id="security.protection.longAbsoluteAnomalies"></a>

`security.protection.longAbsoluteAnomalies` (default `100`): Trigger when this many selected anomalies accumulate, even when normal requests dilute the ratio. Zero disables this trigger.

<a id="security.protection.longRatioPercent"></a>

`security.protection.longRatioPercent` (default `25`): Trigger when selected anomalies reach this percentage after the minimum sample count. Either the absolute or ratio trigger suffices. Zero disables this trigger.

<a id="security.protection.normalCapPerBucket"></a>

`security.protection.normalCapPerBucket` (default `1000`): Bound the normal baseline learned in each rolling-window bucket. Independent absolute anomaly budgets prevent unlimited valid-request padding from hiding abuse.

<a id="security.protection.signals"></a>

`security.protection.signals` (default `["malformed","oversized","credential-rejected"]`): Select authoritative malformed, oversized, credential-rejected, permission-denied, body-deadline or opt-in admin-route-miss observations. Admin route probing excludes entity-not-found responses and static assets; generic status codes, capacity refusals and user-agent strings do not establish abuse.

<a id="security.protection.throttleSecs"></a>

`security.protection.throttleSecs` (default `3`): First threshold crossing starts a fixed throttle deadline. Requests during that deadline do not extend it.

<a id="security.protection.blockSecs"></a>

`security.protection.blockSecs` (default `300`): A later threshold crossing before recovery may escalate to this bounded block. No permanent account lock is created.

<a id="security.protection.decaySecs"></a>

`security.protection.decaySecs` (default `86400`): After this anomaly-free interval and expiry of the current action, escalation returns to its initial stage. Rolling evidence expires according to its configured windows.

<a id="security.protection.exemptClients"></a>

`security.protection.exemptClients` (default `[]`): Exact IP addresses or CIDR networks exempt only the client dimension. Verified account limits still apply. Use deliberately for shared NAT networks; forwarded addresses are trusted only through configured proxies.

<a id="security.protection.exemptAccounts"></a>

`security.protection.exemptAccounts` (default `[]`): Exact server-verified native mailbox account IDs exempt only their account dimension, never the source-client dimension. Proxy account subjects use `proxy:` followed by SHA-256 of the JSON tuple containing the verified canonical service authority and upstream account ID; independent services sharing an account ID remain separate. Proxy exemptions must use this qualified subject, never a raw upstream account ID. Legacy proxy API keys without a verified service realm cannot access protected work while account-level enforcement is enabled; observe-only mode leaves them accessible but cannot record an account dimension. Trusted-client protection still applies. Email addresses and caller-supplied account parameters are not identity evidence.

<a id="security.protection.exemptAdmins"></a>

`security.protection.exemptAdmins` (default `[]`): Exact authenticated administrator IDs exempt only their administrator dimension. Existing sign-in failure locks and strict administrator session deadlines remain unchanged.

<a id="security.protection.storageFailure"></a>

`security.protection.storageFailure` (default `"closed"`): Closed refuses protected work when an enabled enforcement budget cannot be checked. Open temporarily permits it with a bounded operational warning. Observe-only failures never impose an enforcement action.

<a id="mail.requestDeliveryReceipt"></a>

`mail.requestDeliveryReceipt` defaults to false and controls the saved delivery-notification preference for new messages. The separate `receipts.dsnAllowed` policy governs whether the request is permitted; SMTP DSN support is still required.

<a id="appearance.customCssEnabled"></a>
### Personal appearance CSS (`appearance.customCssEnabled`)

Global switch, disabled by default. Enables restricted, parsed visual CSS scoped to the owner’s mailbox or administrator workspace. Network resources, imports, attributes, and positioning are refused. Enforced mailbox appearance locks pause custom CSS without deleting the saved source.

<a id="appearance.customJsEnabled"></a>
### Personal appearance scripts (`appearance.customJsEnabled`)

Global switch, disabled by default. Scripts run in a time-limited worker inside an opaque sandbox and can request only supported appearance changes. They receive no mail or account identifiers and have no application DOM access. This is an appearance API, not general application JavaScript.

<a id="mail.modifierSelection"></a>
<a id="mail.dragToFolder"></a>

`mail.modifierSelection` and `mail.dragToFolder` default to true and support global, domain and user values. Administrators can lock either interaction. Disabling modifier selection leaves the explicit selection toolbar available. Disabling drag does not disable the ordinary Move action. Dragging carries only a temporary local token, is fenced to the current account generation, and accepts real loaded folders only; it never grants permission beyond the server-authorized move.

<a id="admin.allowLastSuperadminLoss"></a>
### Last Super admin protection (`admin.allowLastSuperadminLoss`)

Global boolean, default `false`. The final enabled, unscoped Super admin is protected from deletion, disablement, role removal, deny overrides, and removal of its final passwordless sign-in credential. Built-in role definitions remain immutable. A full Super admin may explicitly enable loss of this access, with fresh authentication and consequence confirmation. The setting does not remove any access itself. Policy writes and destructive account mutations share one database lock across instances; deleting the policy restores protection. Losing all Super admin access requires local deployment administration to recover; it is never repaired by silently granting browser users privileges. With the deployment database and server key available locally, use `mailwoman admin admins create recovery-admin` (password from standard input), then `mailwoman admin admins grant-role recovery-admin super_admin`. Do not put the password in command-line arguments.

<a id="privacy.remoteImageDownloads"></a>
### Manual remote image downloads (`privacy.remoteImageDownloads`)

Global/domain/user boolean, default false. Allows an explicit image download through the privacy proxy without granting automatic image display. Signing and redemption both verify the current policy, session and exact image reference in the owned message. Existing image address, route, type, size and tracker protections apply; SVG is refused. Accepted images are decoded and served as PNG attachments, not original remote bytes. There is no direct external-link fallback. The server inspects at most 32 MiB of cached MIME and 1 MiB of HTML across 64 parts and 4,096 references. Missing, truncated, oversized or client-only decrypted bodies are unavailable for this action; client-provided HTML and reference markers are never trusted. Linked-file downloads also require an exact accepted link in the owned message. Manual image signing requires the expected account header, and redemption requires the same currently authenticated signing session and account.

<a id="mail.resumeTabs"></a>
### Resume workspace tabs

`mail.resumeTabs` defaults to true and supports global, domain and user values with administrator locks. The browser stores bounded tab references, order, pins and active tab under both the wire account ID and verified storage authority, never message bodies or attachment URLs. Reopening resolves references under current authorization. Missing storage authority disables private persistence and automatic restoration; legacy raw-account-only records are not migrated automatically. Disabling the setting clears this workspace record without deleting the separate account-owned draft store.

<a id="mail.folderCounts"></a>
### Folder count display (`mail.folderCounts`)

Global/domain/user choice, default `"both"`. Select `none`, `unread`, `total`, or `both` for folder badges. Users can change this under Settings > Folders unless an administrator locks the inherited value. Hiding counts changes their presentation only; it does not alter read state or mailbox totals.

<a id="mail.compactFolderBadges"></a>
### Secondary folder badge size (`mail.compactFolderBadges`)

Global/domain/user boolean, default `true`. Uses smaller badges outside Inbox. Users can inherit or override this in Settings > Folders unless locked by an administrator. Folder names remain left-aligned and long names truncate without displacing the count.

<a id="metrics.defaultRangeSeconds"></a>
### Default request history range (`metrics.defaultRangeSeconds`)

Global seconds, default 3,600; bounded from 60 to 34,560,000. Applies when both history endpoints are omitted. Effective default is capped by the maximum range and retention.

<a id="metrics.maxRangeSeconds"></a>
### Maximum request history range (`metrics.maxRangeSeconds`)

Global seconds, default 2,592,000 (30 days); bounded from 3,600 to 34,560,000. The effective limit also respects `metrics.rollupDays` (1–400 days). Request history records response-ready latency for requests that reach application middleware, not full streamed transfers or transport-parser rejections. Aggregate snapshots flush every 15 seconds; a sudden process loss can lose the unflushed tail. Minute snapshots are retained for 48 hours, hourly snapshots for the configured retention. Percentiles combine histogram counts across replicas, with no request paths, addresses or identities stored. Missing periods remain unavailable, not zero. Queries return at most 1,000 points and refuse scans exceeding 20,000 stored buckets; reduce the range if the deployment has too many replica snapshots. Only unscoped Dashboard read or Observability read access can see this server-wide history.

<a id="contacts.senderIcons"></a>
### Bundled sender icons (`contacts.senderIcons`)

Enabled by default, with global/domain/user overrides and administrator locks.
Recognised sender domains use bundled SVG icons; unknown senders use deterministic
initials from a 192-colour palette with at least 4.5:1 white-text contrast. Embedded
contact photos take priority when contact-photo mode is selected; hiding avatars
also hides domain icons. Government domains use a generic civic icon, not an
identity seal. Domain matching is an appearance hint, never authentication
verification. It does not request favicons, remote logos, or external profiles.

<a id="mail.draftSaveMode"></a>
<a id="mail.draftDebounceMs"></a>
<a id="mail.draftIntervalSecs"></a>

Account draft saving supports global, domain and user defaults and administrator locks. `mail.draftSaveMode` defaults to `realtime` (after a typing pause), with `periodic` and `manual` alternatives. `mail.draftDebounceMs` defaults to 1000 and accepts 500–10000 milliseconds; `mail.draftIntervalSecs` defaults to 30 and accepts 5–3600 seconds. Manual mode disables scheduled upstream writes. These settings do not grant backend capabilities: a mapped, writable native IMAP Drafts folder and confirmed APPENDUID are required. Unsupported backends remain explicit. Interrupted or unconfirmed saves retain a durable uncertain claim and are never automatically repeated.

<a id="mail.showFrom"></a>

`mail.showFrom` is a global/domain/user boolean, default false. It controls the initial visibility of the composer From field. A per-compose ribbon action can reveal the field; visibility never grants permission to send as an address. Typed one-message addresses must pass current native or upstream sender authorization and are not saved as identities.

<a id="privacy.stripMailerHeaders"></a>
### Outgoing mailer header privacy (`privacy.stripMailerHeaders`)

Defaults to true; global, domain and account overrides are supported, with administrator locks. Native SMTP/EWS submissions remove top-level X-Mailer, User-Agent and X-MimeOLE. Bodies, attachments and ARF report payloads are unchanged; upstream JMAP proxy sends are outside this boundary. See [outgoing header privacy](../deploy/outgoing-header-privacy.md) for pre-signed message conflicts and relay limitations.
