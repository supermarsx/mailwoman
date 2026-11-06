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

<a id="accounts.storeLinkedCredentials"></a>
`accounts.storeLinkedCredentials` (default `true`; global, domain and per-user
administrative scopes) keeps the linked mailbox's password, as the link or renew
form collected it, sealed on the server (XChaCha20-Poly1305 under the server key,
bound to the owner, the link, the mailbox and its storage authority). When the
linked mailbox's private session ends (idle or absolute clock, restart, prune),
the server signs it in again from that copy without asking. If the mailbox's
server refuses it, the copy is erased and the owner is offered **Renew access**
in the mailbox picker and in Settings; the link itself is kept. Nothing is kept,
and access is renewed by signing in again, when: this setting is `false`; the
owner or the linked mailbox is in zero-access mode; the linked mailbox has a
second factor enrolled or required (a factor is never stored or replayed); or
the mailbox is EWS. Turning this or `accounts.linkingEnabled` off globally
deletes every stored copy at once; a domain or user value erases it at the
link's next use. Unlinking, an admin session revoke of either account, and
retiring either identity delete it too. Store, refusal, renewal and deletion are
recorded in the audit log without the secret.

<a id="accounts.maxLinked"></a>
`accounts.maxLinked` is the global maximum number of verified child mailboxes per
primary identity: default `5`, permitted range `1`–`16`.

This initial release supports native IMAP, POP3 and EWS accounts. Proxy JMAP,
SSO-only and header-only contexts are refused: the existing proxy second-factor
storage still needs verified-realm isolation before it can safely support links.
Adding an account requires a fresh primary proof and the child's actual password
and any enrolled second factor. A forced first-factor enrollment or password
change must be completed through a separate sign-in before linking. Existing
metadata-only legacy links are not adopted. When the child's own session ends it
is renewed from the stored credential (see `accounts.storeLinkedCredentials`), or
the owner renews access in place; unlinking is never needed for that. Switching never
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

<a id="ui.notifyNewVersion"></a>
### New version notice (`ui.notifyNewVersion`)

`true` (default) shows signed-in users a toast with **Reload now** and **Later** when a newer version of the web app has been deployed since their page loaded. The client compares the build id from the authenticated `GET /api/version` when the window regains focus, when the connection returns and about every ten minutes. **Later** snoozes the notice for an hour. `false` never shows it; users then get the new code at their next reload. Users change it under **Settings → Interface** unless the value is locked.

<a id="ui.undoHistorySize"></a>
### Undo and redo history size (`ui.undoHistorySize`)

An integer from 0 to 50, default 50. The number of recent mail actions the **Undo** and **Redo** buttons in the mailbox toolbar remember: archive, delete, move, mark as spam, mark read or unread, flag, label, pin, snooze, follow-up and the whole-folder "mark all read" and "move all" jobs. Each user gesture is one entry, however many messages it covered. The same history answers Ctrl/Cmd+Z, Ctrl/Cmd+Shift+Z and Ctrl+Y outside text fields. `0` removes the buttons and the shortcuts. Recording a new action empties the Redo list. An action that cannot be reversed any more (for example the message was moved or deleted elsewhere) is reported in a toast and dropped. The history is kept in the browser tab only, is never stored, and is cleared on sign-out and account switch. Global/domain/user policy: users choose their own size in Settings unless an administrator locks it, and a user's size is lowered to `ui.undoHistoryMax`. Enforced in the web client only.

<a id="ui.undoHistoryMax"></a>
### Undo and redo history maximum (`ui.undoHistoryMax`)

An integer from 0 to 50, default 50. The most actions any user may keep in the Undo and Redo history, whatever `ui.undoHistorySize` says for them. `0` turns the history off for everyone. Global only; enforced in the web client only.

<a id="folders.reorder.enabled"></a>
### Folder drag and drop (`folders.reorder.enabled`)

`true` (default) lets people rearrange folders by dragging them in the sidebar and in **Settings > Folders and sidebar**, with Alt+arrow keys, and with **Move...** in the folder menu. `false` removes the drag handles and the commands. Global/domain/user policy: a user can turn it off for themselves unless an administrator locks it. Enforced in the web client only.

<a id="folders.reorder.allowNesting"></a>
### Folder nesting (`folders.reorder.allowNesting`)

`true` (default) lets a folder be dropped onto another folder (nest) or back to the top level (un-nest). `false` keeps every folder under its current parent; only the order among siblings changes. Global/domain policy.

<a id="folders.reorder.allowServerMove"></a>
### Move folders on the server (`folders.reorder.allowServerMove`)

`true` (default) lets a nest or un-nest change the folder real parent: JMAP `Mailbox/set parentId`, or an IMAP RENAME to the new path with its subfolders and subscriptions. The user chooses per move, or once with "remember", between this and "only arrange in this app". `false` keeps every nest as a view-only arrangement stored in the user UI preferences. Reordering among siblings never changes the server. Global/domain policy.

<a id="folders.reorder.allowSystemFolders"></a>
### Move system folders (`folders.reorder.allowSystemFolders`)

`false` (default) keeps Inbox, Drafts, Sent, Junk, Trash and Archive fixed. `true` lets users drag them to reorder, in this app only; they are never renamed or moved on the server, and cannot be nested. Global/domain policy.

<a id="folders.reorder.maxDepth"></a>
### Folder nesting depth (`folders.reorder.maxDepth`)

An integer from 1 to 16, default 8. A drag that would put a folder, or the deepest of its subfolders, below this level is refused. Folders already deeper stay as they are. Global/domain policy.

<a id="folders.order.locked"></a>
### Fix the folder order (`folders.order.locked`)

`false` (default). `true` turns off dragging, the Move commands and Alt+arrow moves for everyone in the scope; users see a lock icon and a tooltip. Force-pinned folders and the default order still apply. Global/domain policy.

<a id="folders.pinned.admin"></a>
### Force-pinned folders (`folders.pinned.admin`)

A list, default empty, up to 50 entries. Each entry is `role:<role>` (`role:inbox`, `role:sent`, ...) or a folder path such as `Shared/Support`. Matching folders appear in a Pinned group at the top of every user sidebar and cannot be unpinned or moved by users. An entry with no matching folder is skipped. Global/domain policy.

<a id="folders.order.default"></a>
### Default folder order (`folders.order.default`)

A list, default empty, up to 100 entries, each `role:<role>` or a folder path, first to last. Users with no order of their own start from it; folders not listed follow in the server order. "Reset order to default" in the user settings returns to it. Global/domain policy.

<a id="folders.newFolderPlacement"></a>
### New folder placement (`folders.newFolderPlacement`)

`end` (default) places a folder that appears on the server after its siblings; `alphabetical` sorts it among them. Global/domain/user policy: users choose in **Settings > Folders and sidebar** unless an administrator locks it.

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

The colour of the unread-count badges on folders, inbox tabs and the mailbox picker. `default` keeps the theme's signal colour; `accent` follows the account's accent colour; `accent-lighter` and `accent-darker` use that accent with its OKLCH lightness raised or lowered by 0.2 (20% of the range, clamped), recomputed in the browser when the accent or theme changes, with black or white count text, whichever contrasts more (at least 4.5:1); the ten named colours (`red`, `orange`, `yellow`, `green`, `teal`, `blue`, `indigo`, `violet`, `magenta`, `graphite`) use the same per-theme shades as the named accents, so the count text keeps at least 4.5:1 contrast in every theme. Sets the colour for accounts without a saved choice; a lock overrides the account choice and disables the badge colour buttons in user Appearance settings.

<a id="appearance.density"></a>
### Density (`appearance.density`)

How much space lists and controls take: compact, cozy or relaxed.

<a id="appearance.font"></a>
### Interface font (`appearance.font`)

Sets the font for accounts without a saved choice. A lock overrides the account choice and disables the font buttons in user Appearance settings. Available values are `default` (theme font), `system`, `serif` (Newsreader), `mono` (JetBrains Mono), `inter`, `roboto`, `open-sans`, and `jost`. Font assets are bundled and served locally; the app does not contact Google Fonts. Account font choices synchronize through the account appearance endpoint.

<a id="appearance.leetspeak"></a>
### Leetspeak presentation (`appearance.leetspeak`)

An optional boolean, disabled by default, that changes the presentation of interface text and message reading. Stored messages, editable values, addresses, URLs, exported content and submitted data remain unchanged. Administrators can set a global, domain or per-user default and lock it; users can override an unlocked value. The admin console's own **My Preferences** stores its separate optional `leetspeak` boolean per administrator.

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

<a id="assist.profile"></a>
### AI provider profile (`assist.profile`)

The id of the provider profile that serves these users (AI assistant > Provider profiles). Empty uses the deployment endpoint. A profile id that does not exist turns the AI assistant off for these users.

<a id="upstream.default"></a>
### Default mail server (`upstream.default`)

The mail server an address signs in to when no domain names one. The domain table wins over this.

<a id="upstream.proxyTimeoutSecs"></a>
### Mail server timeout (`upstream.proxyTimeoutSecs`)

How long one request to the mail server may take, in seconds.

<a id="signin.newUsers"></a>
### New users (`signin.newUsers`)

Who may create an account at first sign-in. Changed in Sign-in policy. The rule governs NEW accounts only; an identity that already has an account is never refused, so tightening it cannot lock anyone out.

* `open` (default): any identity that authenticates against its mail server or identity provider is created.
* `preprovisioned`: an administrator must have added the address in advance as a pending user (Sign-in policy, Pre-added users, or Users, Add). The entry is used up by the first sign-in and can expire. With `signin.preprovision.domainWide` on, a capped `*@domain` entry also admits.
* `provisioned`: as `preprovisioned`, and a pending entry of any origin also admits, including imported and directory-synced users.
* `closed`: no new accounts; only existing accounts sign in.

The per-type keys below override this for one kind of sign-in. Precedence, first value that is not `inherit` wins: type for the domain, type global, then (linked mailboxes only) the provider kind before the type, then New users for the domain, New users global, then `open`. A refusal is the same 401 as a wrong password when `signin.refusalDetail` is `uniform`, otherwise 403 `signinNotProvisioned`; each is written to the security log as `signin-new-user-refused`. A stored `known` reads as `preprovisioned`.

<a id="signin.newUsers.mail"></a>
### Password sign-in: new users (`signin.newUsers.mail`)

Rule for a new mailbox user signing in with a password (IMAP, POP3, Exchange). `inherit` uses `signin.newUsers`. Changed in Sign-in policy.

<a id="signin.newUsers.sso"></a>
### Single sign-on: new users (`signin.newUsers.sso`)

Rule for a new user signing in through OIDC, OAuth, SAML or a trusted header. `inherit` uses `signin.newUsers`. Changed in Sign-in policy.

<a id="signin.newUsers.linked"></a>
### Linked mailboxes: new users (`signin.newUsers.linked`)

Rule for linking a mailbox that has no account here yet, judged by the linked mailbox's own domain. `inherit` uses `signin.newUsers`. Changed in Sign-in policy.

<a id="signin.newUsers.linked.imap"></a>
### Linked IMAP mailboxes: new users (`signin.newUsers.linked.imap`)

Rule for linked IMAP mailboxes only. `inherit` uses `signin.newUsers.linked`. Changed in Sign-in policy.

<a id="signin.newUsers.linked.pop3"></a>
### Linked POP3 mailboxes: new users (`signin.newUsers.linked.pop3`)

Rule for linked POP3 mailboxes only. `inherit` uses `signin.newUsers.linked`. Changed in Sign-in policy.

<a id="signin.newUsers.linked.ews"></a>
### Linked Exchange mailboxes: new users (`signin.newUsers.linked.ews`)

Rule for linked Exchange (EWS) mailboxes only. `inherit` uses `signin.newUsers.linked`. Changed in Sign-in policy.

<a id="signin.preprovision.ttlDays"></a>
### Pre-added user lifetime (days) (`signin.preprovision.ttlDays`)

Default lifetime of a pending pre-provisioned user, in days. Changed in Sign-in policy.

<a id="signin.preprovision.maxTtlDays"></a>
### Longest pre-added user lifetime (days) (`signin.preprovision.maxTtlDays`)

Longest lifetime an administrator may give a pending pre-provisioned user, in days. Changed in Sign-in policy.

<a id="signin.preprovision.domainWide"></a>
### Allow domain-wide pre-added entries (`signin.preprovision.domainWide`)

Allow capped `*@domain` pending entries that admit any address of a domain until their use cap is reached. Off by default. Changed in Sign-in policy.

<a id="signin.preprovision.retentionDays"></a>
### Keep expired pending users (days) (`signin.preprovision.retentionDays`)

Days an expired pending entry is kept before bounded cleanup removes it. Changed in Sign-in policy.

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

<a id="admin.session.rotateSecs"></a>
### Console session id rotation (`admin.session.rotateSecs`)

How often, in seconds, the admin console replaces the session id while it is in use (default 900, 0 turns interval rotation off; values under 60 act as 60). The id is also replaced right after a successful step-up confirmation. Sign-in already issues a new id every time. Rotation keeps the creation time, idle deadline and absolute deadline of the old session, so it never extends a session. The new cookie keeps the sign-in cookie's flags (`HttpOnly`, `SameSite=Strict`, the console path, `Secure` under the global rule) and the CSRF cookie is reissued. The old id is retired: presenting it again after `security.theft.rotationGraceSecs` (when theft detection is on) ends the whole admin session chain, writes a `session-security-alert` audit event for the administrator and adds an entry to the recent session alerts.

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

How many days read-tracking records are kept (1–400; default 90). Changes apply to token creation, history reads and image recording without a restart. Expired records and their event rows are deleted; expiry cannot be extended by loading an old pixel.

<a id="privacy.openTracking.unboundTtlHours"></a>
### Unsent tracking tokens (`privacy.openTracking.unboundTtlHours`)

How many hours a tracking token waits for its message to be sent before it is deleted (1–168; default 24). This limit applies independently of the overall history retention.

<a id="cap.compose.open_tracking"></a>
### Permit outgoing open tracking (`cap.compose.open_tracking`)

Controls whether a sender may create or bind tracking pixels and whether existing pixels may record more events. Enabled by default as an available capability; messages do not use it unless the sender opts in. Configure it globally, by domain, by admin user value, or through an assigned feature profile/group. A locked denial wins over narrower choices. The `compose.open_tracking` feature includes this capability and the default below.

<a id="mail.openTracking"></a>
### Default outgoing open tracking (`mail.openTracking`)

Off by default. Users can opt in under Compose settings, inherit the administrator default, or choose per message. Administrators can supply or lock the default globally, for domains, or for individual users and profiles. A locked false default also prevents token creation and further recording on the server. Plain text, encrypted and signed messages do not receive a tracking pixel.

The sender sees load counts and first/last timestamps, not recipient IPs or user agents. Image blocking and automatic preloading make this an estimate, not proof of reading. `MW_PUBLIC_URL` is still required. Revoking a capability or locking tracking off stops new events while existing permitted history remains readable until retention expires. Tokens created before verified sender-policy metadata was stored retain their history but stop collecting events, because their sender scope cannot safely be inferred.

<a id="privacy.trackerOverrideAllowed"></a>
### Per-message tracker consent (`privacy.trackerOverrideAllowed`)

Available by default; this shows an action in the message privacy bar, not automatic consent. Users can hide the option, and administrators can set or lock it globally, by domain, by user, or through a profile. A locked `privacy.trackers=true` always forbids the exception. Explicit consent applies only to that account's selected message and is independently revocable; it does not enable images for other messages. The proxy rechecks the grant and current policy before using cached or newly fetched images. The original HTML reader follows the same rules.

<a id="privacy.fetchQueueTimeoutMs"></a>
### Remote-image queue budget (`privacy.fetchQueueTimeoutMs`)

Global, hot, 1,000–180,000 ms; default 60,000 ms. Queue admission has a separate allowance from `privacy.fetchTotalTimeoutMs`, so waiting for other images does not consume the entire network budget. Overall work remains bounded by the queue, network and `privacy.imageDecodeTimeoutMs` budgets together (90 seconds by default). The default waiting ceiling is 2,048 requests (`privacy.maxQueuedRequests`), with independent active network, pipeline, account/message and decoder ceilings. Queue exhaustion remains a retryable, uncached 503; no limit bypass or direct-network fallback is used.

<a id="mail.reader.allowOriginalHtml"></a>
### Original HTML reader option (`mail.reader.allowOriginalHtml`)

Off by default. Enables an explicit per-message original-formatting override; it never changes the default sanitized reader. Administrators can supply or lock this preference and can deny the `reading.original_html` feature with `cap.reading.original_html`. The isolated reader still forbids scripts, forms and embedded frames, and remote content remains subject to privacy controls. This is a client presentation control: users who own a message can already download its source.

A locked `mail.reader.unsafeHtmlMode=strict` policy takes precedence and hides the original HTML option.

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

<a id="observability.clientTiming.enabled"></a>
### Browser load times (`observability.clientTiming.enabled`)

Collect page load timings from signed-in browsers as anonymous aggregate histograms. Nothing identifies a person, session or page. Turn off to stop collection. See [Client load times](observability.md#client-load-times).

<a id="observability.clientTiming.sampleRatePercent"></a>
### Browser load time sample rate (`observability.clientTiming.sampleRatePercent`)

Percentage of browser sessions that report load timings (1 to 100).

<a id="observability.clientTiming.minSamples"></a>
### Load time minimum samples (`observability.clientTiming.minSamples`)

Median and 95th percentile load times stay hidden until a period has at least this many samples.

<a id="observability.clientTiming.folderLoads"></a>
### Folder load times (`observability.clientTiming.folderLoads`)

Measure how long a folder takes to show its message list, split by whether it came from the browser cache or the network. On by default. Global, domain and user scope: set the default here or lock it; users can switch it off for themselves in Settings > Privacy unless it is locked. Needs browser load times to be on. See [Client load times](observability.md#client-load-times).

<a id="observability.clientTiming.messageLoads"></a>
### Message load times (`observability.clientTiming.messageLoads`)

Measure how long a message takes to show its headers and sanitised body, split by whether it was cached or prefetched, or came from the network. On by default; same scopes and behaviour as folder load times.

<a id="diagnostics.retentionDays"></a>
### Diagnostics history (`diagnostics.retentionDays`)

How many days of diagnostic runs are kept.

<a id="dataExport.linkHours"></a>
### Data export download window (`dataExport.linkHours`)

How many hours a data export can be downloaded before it is deleted.

<a id="dsr.enabled"></a>
### Data requests (`dsr.enabled`)

Whether people can file data requests (access, download, correction, erasure, restriction, objection) from Settings.

<a id="dsr.type.access"></a>
### Accept access requests (`dsr.type.access`)

Whether the access request type is offered to users.

<a id="dsr.type.portability"></a>
### Accept download (portable copy) requests (`dsr.type.portability`)

Whether the download (portable copy) request type is offered to users.

<a id="dsr.type.rectification"></a>
### Accept correction requests (`dsr.type.rectification`)

Whether the correction request type is offered to users.

<a id="dsr.type.erasure"></a>
### Accept erasure requests (`dsr.type.erasure`)

Whether the erasure request type is offered to users.

<a id="dsr.type.restriction"></a>
### Accept restriction requests (`dsr.type.restriction`)

Whether the restriction request type is offered to users.

<a id="dsr.type.objection"></a>
### Accept objection requests (`dsr.type.objection`)

Whether the objection request type is offered to users.

<a id="dsr.type.other"></a>
### Accept other requests (`dsr.type.other`)

Whether the generic request type is offered to users.

<a id="dsr.approval.export"></a>
### Approval of access and download requests (`dsr.approval.export`)

auto starts the export once the request is confirmed. admin waits for an administrator to approve it.

<a id="dsr.approval.erasure"></a>
### Approval of erasure requests (`dsr.approval.erasure`)

auto schedules the erasure once the request is confirmed. admin waits for an administrator to approve it.

<a id="dsr.approval.other"></a>
### Handling of other requests (`dsr.approval.other`)

auto marks the request as in progress once confirmed. admin waits for an administrator to approve it.

<a id="dsr.reauth.required"></a>
### Re-authentication for requests (`dsr.reauth.required`)

Require a fresh password or second factor to file an export or erasure request and to create a download link.

<a id="dsr.confirmation.required"></a>
### Confirmation message (`dsr.confirmation.required`)

Send a confirmation message with a code to the requester's inbox; the request does not proceed until the code is entered.

<a id="dsr.identityVerification.required"></a>
### Identity verification step (`dsr.identityVerification.required`)

An administrator must record that the requester's identity was verified before the request is approved.

<a id="dsr.erasure.graceDays"></a>
### Erasure grace period (`dsr.erasure.graceDays`)

Days between approval and the erasure. The requester can cancel during this time. 0 runs the erasure at the next worker pass.

<a id="dsr.sla.days"></a>
### Days to answer a request (`dsr.sla.days`)

The number of days from filing to the due date shown to users and administrators.

<a id="dsr.sla.maxExtensionDays"></a>
### Longest extension (`dsr.sla.maxExtensionDays`)

The most days an administrator can add to a request's due date, with a recorded reason.

<a id="dsr.sla.reminderDays"></a>
### Due date flags (`dsr.sla.reminderDays`)

Days before the due date at which a request is flagged as due soon in the queue.

<a id="dsr.export.mailAllowed"></a>
### Mail in exports (`dsr.export.mailAllowed`)

Allow a requester to include messages in an export. Only messages held in this server's own cache are included; mail that stays on the upstream mail server is not.

<a id="dsr.export.mailFormat"></a>
### Mail format in exports (`dsr.export.mailFormat`)

mbox writes one file per folder; eml writes one file per message.

<a id="dsr.export.include.settings"></a>
### Export: settings (`dsr.export.include.settings`)

Include settings in an export.

<a id="dsr.export.include.identities"></a>
### Export: identities (`dsr.export.include.identities`)

Include identities in an export.

<a id="dsr.export.include.contacts"></a>
### Export: contacts (`dsr.export.include.contacts`)

Include contacts in an export.

<a id="dsr.export.include.calendars"></a>
### Export: calendars (`dsr.export.include.calendars`)

Include calendars in an export.

<a id="dsr.export.include.notes"></a>
### Export: notes (`dsr.export.include.notes`)

Include notes in an export.

<a id="dsr.export.include.rules"></a>
### Export: filters and rules (`dsr.export.include.rules`)

Include filters and rules in an export.

<a id="dsr.export.include.sessions"></a>
### Export: sessions, devices and sign-in history (`dsr.export.include.sessions`)

Include sessions, devices and sign-in history in an export.

<a id="dsr.export.include.consent"></a>
### Export: consent records (`dsr.export.include.consent`)

Include consent records in an export.

<a id="dsr.export.include.audit"></a>
### Export: audit entries about the person (`dsr.export.include.audit`)

Include audit entries about the person in an export.

<a id="dsr.export.maxMegabytes"></a>
### Export size cap (`dsr.export.maxMegabytes`)

Largest export in megabytes. Sections that would exceed it are left out and listed in the manifest.

<a id="dsr.export.format"></a>
### Export formats (`dsr.export.format`)

json writes JSON files. jsonCsv also writes CSV files for list data.

<a id="dsr.export.encrypt"></a>
### Seal exports at rest (`dsr.export.encrypt`)

Store built exports encrypted with the server key until they are downloaded or expire.

<a id="dsr.download.linkMinutes"></a>
### Download link lifetime (`dsr.download.linkMinutes`)

Minutes a download link works after it is created. The export itself is kept for the data export download window.

<a id="dsr.download.singleUse"></a>
### Single-use download link (`dsr.download.singleUse`)

A download link stops working after it has been used once.

<a id="dsr.retention.completedDays"></a>
### Finished request retention (`dsr.retention.completedDays`)

Days a finished request and its trail are kept before they are deleted.

<a id="dsr.notify.admins"></a>
### Record new requests for admins (`dsr.notify.admins`)

Record each new request as a security event in the audit log, where admins and alert sinks see it.

<a id="dsr.notify.requester"></a>
### Progress messages to the requester (`dsr.notify.requester`)

Put a message in the requester's inbox when a request changes state.

<a id="dsr.contact.name"></a>
### Data protection contact name (`dsr.contact.name`)

Name shown to users on the data requests page.

<a id="dsr.contact.email"></a>
### Data protection contact address (`dsr.contact.email`)

E-mail address shown to users on the data requests page.

<a id="dsr.contact.url"></a>
### Data protection contact page (`dsr.contact.url`)

Link shown to users on the data requests page.

<a id="policy.publicPages"></a>
### Public policy pages (`policy.publicPages`)

Serve the generated privacy policy at /privacy and cookie policy at /cookies-policy without sign-in.

<a id="policy.autoVersion"></a>
### Automatic policy versions (`policy.autoVersion`)

Record a new policy version when a setting the policy describes changes. Off: changes wait for Publish.

<a id="policy.reconsentOnChange"></a>
### Ask for cookie consent again on a new policy version (`policy.reconsentOnChange`)

Raise the cookie policy version when a new policy version is recorded, so visitors are asked again (needs cookies.repromptOnVersion).

<a id="policy.controller.name"></a>
### Controller name (`policy.controller.name`)

Name of the organisation that decides why and how data is processed, shown in the privacy policy.

<a id="policy.controller.address"></a>
### Controller address (`policy.controller.address`)

Postal address of the controller, shown in the privacy policy.

<a id="policy.legalBases"></a>
### Legal bases (`policy.legalBases`)

The legal bases you rely on, in your own words, shown in the privacy policy.

<a id="policy.hosting.provider"></a>
### Hosting provider (`policy.hosting.provider`)

Who hosts the data and where, shown in the privacy policy.

<a id="policy.hosting.transfers"></a>
### Transfers outside the hosting country (`policy.hosting.transfers`)

Any transfers of data to other countries and the safeguards you use, shown in the privacy policy.

<a id="policy.supervisoryAuthority"></a>
### Supervisory authority (`policy.supervisoryAuthority`)

The authority users may complain to, shown in the privacy policy.

<a id="dpia.suggest"></a>
### Suggest impact assessments (`dpia.suggest`)

List a suggested impact assessment when the assistant, open tracking or analytics is enabled and none exists.

<a id="dpia.reviewMonths"></a>
### Impact assessment review interval (`dpia.reviewMonths`)

Months until the review date set when an assessment is approved.

<a id="dpia.reminderDays"></a>
### Impact assessment reminder window (`dpia.reminderDays`)

Days before the review date that an assessment is listed as due.

<a id="dsr.onBehalf.allowed"></a>
### Requests on behalf of a user (`dsr.onBehalf.allowed`)

Allow an administrator to file a request for a user, for example one received by post.

<a id="dsr.rateLimit.perDay"></a>
### Requests per person per day (`dsr.rateLimit.perDay`)

The most requests one person can file in 24 hours.

<a id="dsr.legalHold.subjects"></a>
### Legal holds (`dsr.legalHold.subjects`)

Addresses, or @domain entries, whose erasure is blocked. An erasure request for them is kept in the queue as blocked until the entry is removed.

<a id="scrub.enabled"></a>
### Person scrub (`scrub.enabled`)

Allows the Scrub person action and the scrub that can follow an erasure. Switched off, neither runs.

<a id="scrub.pseudonymAlgorithm"></a>
### Pseudonym length (`scrub.pseudonymAlgorithm`)

How the keyed pseudonym that replaces an identifier is made: HMAC-SHA256 under the current key generation, cut to 16 or 32 hex characters. The same person always gets the same pseudonym under one generation, so history can still be correlated; the address cannot be recovered without the key.

<a id="scrub.includeLogs"></a>
### Scrub log files (`scrub.includeLogs`)

A scrub also rewrites the identifier in `mailwoman.log` and its rotations, compressed ones included.

<a id="scrub.autoRunOnErasure"></a>
### Scrub after erasure (`scrub.autoRunOnErasure`)

A completed erasure also replaces the person's identifiers in audit history and logs with the pseudonym. Off by default: without it, audit entries are kept as written for the audit retention period.

<a id="scrub.keyRetentionDays"></a>
### Pseudonym key retention (`scrub.keyRetentionDays`)

Days a pseudonym key is kept after it is rotated out. 0 keeps it. Once a key is deleted, pseudonyms made with it can no longer be recomputed, and erasure tombstones made with it can no longer be replayed after a restore.

<a id="backup.propagateErasure"></a>
### Replay erasures on restore (`backup.propagateErasure`)

A restore replays the erasure tombstones that are newer than the backup, so people erased since the backup was taken are removed again.

<a id="backup.tombstoneRetentionDays"></a>
### Erasure tombstone retention (`backup.tombstoneRetentionDays`)

Days an erasure tombstone is kept. 0 keeps them forever. A backup older than this retention may contain people whose tombstone has been removed.

<a id="dsr.text.intro"></a>
### Data requests introduction (`dsr.text.intro`)

Text shown above the request form. One value per language.

<a id="dsr.text.confirmation"></a>
### Confirmation message text (`dsr.text.confirmation`)

Text added to the confirmation message. One value per language.

<a id="dsr.text.completion"></a>
### Completion message text (`dsr.text.completion`)

Text added to the message sent when a request is completed. One value per language.

<a id="breach.enabled"></a>
### Breach register (`breach.enabled`)

Switches the register and the notice clock on.

<a id="breach.noticeHours"></a>
### Hours to notify the authority (`breach.noticeHours`)

Hours from becoming aware to the authority notice. The usual window is 72.

<a id="breach.reminders.hours"></a>
### Reminder times (`breach.reminders.hours`)

Hours before the deadline that raise a reminder, for example 6, 24 and 48.

<a id="breach.reminders.overdueRepeatHours"></a>
### Overdue repeat (`breach.reminders.overdueRepeatHours`)

Hours between reminders once the deadline has passed. 0 reminds once.

<a id="breach.authority.name"></a>
### Supervisory authority (`breach.authority.name`)

Name printed on the authority notice.

<a id="breach.authority.url"></a>
### Authority notification page (`breach.authority.url`)

Address of the authority's notification page.

<a id="breach.authority.contact"></a>
### Authority contact (`breach.authority.contact`)

Contact details of the authority.

<a id="breach.dpo.name"></a>
### Data protection contact name (`breach.dpo.name`)

Name given as the contact point in notices.

<a id="breach.dpo.email"></a>
### Data protection contact email (`breach.dpo.email`)

Address given as the contact point in notices.

<a id="breach.dpo.phone"></a>
### Data protection contact phone (`breach.dpo.phone`)

Phone number given in notices.

<a id="breach.template.subject"></a>
### Message subject (`breach.template.subject`)

Subject of the message to people affected, per language.

<a id="breach.template.body"></a>
### Message text (`breach.template.body`)

Text of the message to people affected, per language.

<a id="breach.notify.audit"></a>
### Record notices in the audit log (`breach.notify.audit`)

Reminders and detections become security events.

<a id="breach.notify.webhookUrls"></a>
### Webhook addresses (`breach.notify.webhookUrls`)

Addresses told about reminders and messages. Treat them as secrets; each call is signed.

<a id="breach.notify.operatorAddresses"></a>
### Operator mailboxes (`breach.notify.operatorAddresses`)

Local mailbox addresses that receive reminders in their own inbox.

<a id="breach.subjects.inApp"></a>
### Notice in the person's account (`breach.subjects.inApp`)

Show the message inside the person's account.

<a id="breach.subjects.inbox"></a>
### Message to the person's mailbox (`breach.subjects.inbox`)

Deliver the message to the person's own mailbox.

<a id="breach.subjects.webhook"></a>
### Also send to webhooks (`breach.subjects.webhook`)

Send the message text to the webhook addresses too.

<a id="breach.retention.years"></a>
### Register retention in years (`breach.retention.years`)

Years a closed incident stays in the register.

<a id="breach.auto.enabled"></a>
### Automations (`breach.auto.enabled`)

Open draft incidents from detections.

<a id="breach.auto.startClock"></a>
### Draft clock (`breach.auto.startClock`)

Start the clock when the draft opens, or only when it is confirmed.

<a id="breach.auto.assignee"></a>
### Assign drafts to (`breach.auto.assignee`)

Who a draft is assigned to.

<a id="breach.auto.escalateAfterHours"></a>
### Escalate after hours (`breach.auto.escalateAfterHours`)

Hours a draft waits before it is escalated. 0 never escalates.

<a id="breach.auto.escalateTo"></a>
### Escalate to (`breach.auto.escalateTo`)

Who an escalated draft is assigned to.

<a id="breach.auto.windowMinutes"></a>
### Counting window in minutes (`breach.auto.windowMinutes`)

Minutes over which a rule counts events.

<a id="breach.auto.theft.threshold"></a>
### Session theft alerts (`breach.auto.theft.threshold`)

Alerts in the window that open a draft. 0 is off.

<a id="breach.auto.credentialStuffing.threshold"></a>
### Sign-in spray or spike (`breach.auto.credentialStuffing.threshold`)

Events in the window that open a draft. 0 is off.

<a id="breach.auto.massForward.threshold"></a>
### Forwarding rules in bulk (`breach.auto.massForward.threshold`)

Events in the window that open a draft. 0 is off.

<a id="breach.auto.roleEscalation.threshold"></a>
### Administrator role changes (`breach.auto.roleEscalation.threshold`)

Changes in the window that open a draft. 0 is off.

<a id="breach.auto.exportVolume.threshold"></a>
### Unusual exports (`breach.auto.exportVolume.threshold`)

Exports in the window that open a draft. 0 is off.

<a id="breach.auto.auditChain.threshold"></a>
### Audit chain failures (`breach.auto.auditChain.threshold`)

Failures that open a draft. 0 is off.

<a id="breach.auto.backupAnomaly.threshold"></a>
### Backup or restore failures (`breach.auto.backupAnomaly.threshold`)

Failures in the window that open a draft. 0 is off.

<a id="breach.auto.dsrExport.threshold"></a>
### Many data request exports (`breach.auto.dsrExport.threshold`)

Exports in the window that open a draft. 0 is off.

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
`twofa.required` defaults to false.
<a id="twofa.passkeyAttachment"></a>
`twofa.passkeyAttachment`: `any` (default), `platform` or `cross-platform`.
Which authenticators can hold a mailbox passkey. The browser reports it; it is
not signed by the authenticator.
<a id="twofa.passkeyUserVerification"></a>
`twofa.passkeyUserVerification`: `required`, `preferred` (default) or
`discouraged`. With `required` a registration or sign-in without the signed UV
flag is refused.
<a id="twofa.passkeyResidentKey"></a>
`twofa.passkeyResidentKey`: `required`, `preferred` or `discouraged` (default).
In engine mode passkeys are always discoverable and user-verified, because they
also sign in without a password.
<a id="twofa.passkeyLegacy"></a>
`twofa.passkeyLegacy`: `allow` (default) keeps passkeys added before a stricter
policy working and asks the user to replace them; `refuse` refuses them with a
message.
<a id="twofa.lastFactorRemoval"></a>
`twofa.lastFactorRemoval`: `allow` (default) or `block`, for a removal that
would leave an account with no passkey and no authenticator app. With `block`
the user cannot remove the last one and is asked to add another sign-in method
first; recovery codes do not count. Where two-factor authentication is
required, the last factor is kept whatever this says. The earlier value
`requireRecoveryCodes` is read and accepted as `block` (migration 0074
rewrites stored rows). See
[Removing the last factor](security-center.md#removing-the-last-factor).

These eleven settings support global, domain, group, profile and
administrator-set account policy, without user overrides.

## Reader privacy

<a id="privacy.remoteContent"></a>
`privacy.remoteContent`: `block`, `ask` (default), `proxy`, or `direct`.
<a id="privacy.trackers"></a>
`privacy.trackers` defaults to true.
<a id="privacy.cleanLinks"></a>
`privacy.cleanLinks` defaults to true. These three support global, domain and user settings.
<a id="links.trustedDomains"></a>
`links.trustedDomains`: up to 100 exact link domains (default empty). A message
link whose host is on the list opens straight from Review links, without the
destination step. Entries are lowercase ASCII (A-label) names of at least two
labels, with no scheme, path, port, wildcard or IP address; the web app
normalizes case and international names before it saves. Plain-HTTP links and
links to an IP address are always reviewed. An unlocked user list replaces the
inherited list ("Trust <host>" in Review links copies the inherited entries
first).
<a id="links.trustedSubdomains"></a>
`links.trustedSubdomains` defaults to false: only the exact domain is trusted.
When true, a name ending in `.<entry>` is trusted as well; matching is always on
a label boundary, so `example.com` never trusts `evilexample.com` or
`example.com.evil.test`. Both keys support global, domain and user settings;
lock them to keep users from changing the list or the switch.
<a id="privacy.maxBytes"></a>
`privacy.maxBytes`: 65536–8388608 bytes, default 8388608.
<a id="privacy.cacheBytes"></a>
`privacy.cacheBytes`: 0–1073741824 bytes, default 33554432.
<a id="privacy.imageLatencyTracking"></a>
`privacy.imageLatencyTracking`: on or off, default on. Records the image proxy's speed (median, p95, p99, maximum and count, per outcome and per phase) in fixed log-scale histogram buckets, one set per minute for 24 hours, in memory. Only bucket counts are kept: no URLs, hosts, accounts or messages. The history is per server process and resets on restart. A change applies from the next image request; turning it off stops new recording and leaves the recorded figures readable.
<a id="privacy.ratePerMin"></a>
`privacy.ratePerMin`: 10–60000 replenished fetch tokens per account per minute, default 1200 (20 per second; the fixed refill in manual rate mode). The separate burst allowance permits a large message to start without waiting for one minute of refill.

Remote image fetch limits. Every image the server fetches for the reader is
bounded in time, in redirects and in how many run at once. An image that exceeds
a bound shows a small placeholder with its alt text in its own box; the failure is
remembered for `privacy.negativeCacheTtlSecs` (10 seconds by default) so they are not retried in a loop. Queue-capacity failures are retryable and are not negatively cached. **Retry images** clears the affected failures and fetches those references again.

<a id="privacy.fetchConnectTimeoutMs"></a>
`privacy.fetchConnectTimeoutMs`: 500–30000 ms, default 5000. Opening the TCP
connection and the TLS handshake (through an egress route: reaching the proxy,
its tunnel and the origin's TLS handshake). Five seconds is two or three round
trips even on a slow link, plus one lost-packet retransmission.
<a id="privacy.fetchHeadersTimeoutMs"></a>
`privacy.fetchHeadersTimeoutMs`: 500–60000 ms, default 10000. From the start of a
request until the response headers arrive; covers an image CDN fetching from its
origin.
<a id="privacy.fetchIdleTimeoutMs"></a>
`privacy.fetchIdleTimeoutMs`: 500–60000 ms, default 10000. The longest pause
between two parts of the body, so a stalled download stops without waiting for
the total.
<a id="privacy.fetchTotalTimeoutMs"></a>
`privacy.fetchTotalTimeoutMs`: 1000–120000 ms, default 20000. The network budget covers DNS, every redirect and the response body. Queue waiting has its own `privacy.fetchQueueTimeoutMs` allowance (default 60000 ms). The overall original deadline is the queue, network and `privacy.imageDecodeTimeoutMs` budgets combined; it bounds queueing, network work, pipeline processing and decoding; a phase never receives a fresh queue allowance. Network work remains capped by its own configured total and phase timeouts.
<a id="privacy.maxRedirects"></a>
`privacy.maxRedirects`: 0–10, default 3. Each redirect is resolved and checked
against the same address rules as the first request (private, loopback,
link-local and metadata ranges, and IPv6 forms that embed them such as NAT64,
6to4, Teredo and ISATAP).
<a id="privacy.maxConcurrentPerMessage"></a>
`privacy.maxConcurrentPerMessage`: 1–128, default 30. Further images of the message wait for a slot within the queue budget.
<a id="privacy.maxConcurrentPerAccount"></a>
`privacy.maxConcurrentPerAccount`: 1–256, default 128, across all open messages and tabs of one account. Server-wide network and pipeline ceilings apply on top.

The network, queue, concurrency, rate and cache controls are global, hot settings. Timeouts are logged at debug level with the
phase that fired and the elapsed time, never the image address.
<a id="privacy.imageConcurrencyMode"></a>
`privacy.imageConcurrencyMode` is a global, hot enum: `auto` (default), `conservative`, `balanced`, `throughput`, or `manual`. Admin › Privacy offers Automatic, Low resource, Balanced, High throughput and Manual in one themed selector. Applying a profile writes this enum once with an ETag; it never rewrites the numeric settings. A stale or refused write leaves the selected choice available for review and retry.

Adaptive sizing is on by default. Automatic and the three named presets use available CPU/container-memory information, the numeric ceilings below and preset safety caps. They also account for each message's approved image count. The runtime summary reports real instance resource observations, effective limits, numeric ceilings and active network/pipeline/decode/queue counts; unknown memory observations are shown as unavailable. Its effective per-message value is an instance upper bound: an individual message uses at most that bound or its approved image count, whichever is smaller. Manual uses the numeric ceilings directly. The defaults allow ceilings of 30 per message, 128 per account and globally, 128 processing pipelines, and 10 decoders; adaptive modes can select lower effective values for the current instance.

Active or queued work is re-evaluated every five seconds without requiring an open admin page. Linux sizing honors CPU affinity, fractional cgroup quotas and ancestor memory limits/headroom; Windows uses available CPU capacity and physical memory. Decoder admission reserves the sandbox's full 512 MiB bound as well as fetched-image buffers. Retry subsets retain the message's larger approved count. Hot reductions drain existing permits, and hot increases can wake the already queued batch.

Additional parallelism and cache controls are available in Admin › Privacy › Remote image fetching, with links to the policy editors:

| Setting | Default | Range | Meaning |
| --- | ---: | --- | --- |
| <a id="privacy.imageDecodeTimeoutMs"></a>`privacy.imageDecodeTimeoutMs` | 10000 ms | 1000–60000 ms | Wall-clock budget for one sandboxed decode after fetching; independent from network timeouts. |
| <a id="privacy.maxConcurrentGlobal"></a>`privacy.maxConcurrentGlobal` | 128 | 1–256 | Active remote image network requests in this process. |
| <a id="privacy.maxConcurrentPipeline"></a>`privacy.maxConcurrentPipeline` | 128 | 1–256 | Admitted image pipelines, retained through decode and cache publication. |
| <a id="privacy.maxConcurrentDecodes"></a>`privacy.maxConcurrentDecodes` | 10 | 1–16 | CPU image decodes running simultaneously. |
| <a id="privacy.maxQueuedRequests"></a>`privacy.maxQueuedRequests` | 2048 | 16–16384 | Requests waiting for network or pipeline capacity; overflow remains retryable. |
| <a id="privacy.rateBurst"></a>`privacy.rateBurst` | 1024 | 1–12000 | Maximum per-account fetch tokens, replenished at `privacy.ratePerMin`; the fixed burst in manual rate mode. |
| <a id="privacy.imageRateMode"></a>`privacy.imageRateMode` | auto | auto, manual | Per-account image fetch rate sizing: auto scales burst and refill from replica capacity, queue load and message image count within the floors and ceilings; manual uses privacy.rateBurst and privacy.ratePerMin exactly |
| <a id="privacy.imageRateBurstFloor"></a>`privacy.imageRateBurstFloor` | 64 | 1–12000 | Lowest per-account burst allowance in automatic rate mode |
| <a id="privacy.imageRateBurstCeiling"></a>`privacy.imageRateBurstCeiling` | 2048 | 1–12000 | Highest per-account burst allowance in automatic rate mode |
| <a id="privacy.imageRateRefillFloor"></a>`privacy.imageRateRefillFloor` | 300 | 10–60000 | Lowest per-account refill, tokens per minute, in automatic rate mode |
| <a id="privacy.imageRateRefillCeiling"></a>`privacy.imageRateRefillCeiling` | 6000 | 10–60000 | Highest per-account refill, tokens per minute, in automatic rate mode |
| <a id="privacy.imageCacheMode"></a>`privacy.imageCacheMode` | auto | auto, manual | Remote image cache sizing: auto derives bytes, entries and lifetime from the memory left after decoder and pipeline reservations and the observed hit rate; manual uses privacy.cacheBytes, privacy.cacheEntries and privacy.cacheTtlSecs exactly |
| <a id="privacy.imageCacheBytesFloor"></a>`privacy.imageCacheBytesFloor` | 16777216 | 0–1073741824 | Smallest cache size in bytes in automatic cache mode |
| <a id="privacy.imageCacheBytesCeiling"></a>`privacy.imageCacheBytesCeiling` | 268435456 | 0–1073741824 | Largest cache size in bytes in automatic cache mode |
| <a id="privacy.cacheEntries"></a>`privacy.cacheEntries` | 4096 | 16–65536 | Most cached remote images; the fixed count in manual cache mode, the ceiling in automatic mode |
| <a id="privacy.cacheEntryBytes"></a>`privacy.cacheEntryBytes` | 4194304 | 65536–16777216 | Largest single re-encoded image kept in the cache, in bytes |
| <a id="privacy.imageCacheTtlFloorSecs"></a>`privacy.imageCacheTtlFloorSecs` | 300 | 0–86400 | Shortest cache lifetime in seconds in automatic cache mode, used on a low hit rate or under memory pressure |
| <a id="privacy.imageCacheTtlCeilingSecs"></a>`privacy.imageCacheTtlCeilingSecs` | 21600 | 0–86400 | Longest cache lifetime in seconds in automatic cache mode, used on a high hit rate with free memory |
| <a id="privacy.cacheTtlSecs"></a>`privacy.cacheTtlSecs` | 3600 s | 0–86400 s | Successful decoded-image reuse; zero disables reuse. |
| <a id="privacy.negativeCacheTtlSecs"></a>`privacy.negativeCacheTtlSecs` | 10 s | 0–300 s | Eligible upstream-failure reuse; zero disables it. |
| <a id="privacy.imageRamMaxMiB"></a>`privacy.imageRamMaxMiB` | 0 (no extra cap) | 0–1048576 MiB | Extra cap in MiB on the RAM tier of the image cache; zero leaves the automatic or manual cache size as the only limit. |
| <a id="privacy.imageDiskEnabled"></a>`privacy.imageDiskEnabled` | off | on, off | Keep re-encoded remote images on disk as a second cache tier behind the RAM tier; off by default. |
| <a id="privacy.imageDiskPath"></a>`privacy.imageDiskPath` | empty (a folder next to the database) | absolute path, at most 512 characters | Absolute directory for the image disk tier; empty uses an image-cache folder next to the database file. |
| <a id="privacy.imageDiskMode"></a>`privacy.imageDiskMode` | auto | auto, manual | Disk tier size: auto takes a tenth of the free space between the floor and ceiling; manual uses privacy.imageDiskMaxMiB exactly. |
| <a id="privacy.imageDiskMaxMiB"></a>`privacy.imageDiskMaxMiB` | 512 | 1–1048576 MiB | Disk tier size in MiB in manual disk mode. |
| <a id="privacy.imageDiskFloorMiB"></a>`privacy.imageDiskFloorMiB` | 64 | 0–1048576 MiB | Smallest disk tier size in MiB in automatic disk mode. |
| <a id="privacy.imageDiskCeilingMiB"></a>`privacy.imageDiskCeilingMiB` | 4096 | 0–1048576 MiB | Largest disk tier size in MiB in automatic disk mode. |
| <a id="privacy.imageDiskEntries"></a>`privacy.imageDiskEntries` | 16384 | 16–1048576 | Most images kept on disk; also bounds the in-memory index. |
| <a id="privacy.imageDiskEntryBytes"></a>`privacy.imageDiskEntryBytes` | 8388608 | 65536–67108864 | Largest single re-encoded image written to disk, in bytes. |
| <a id="privacy.imageDiskMinHits"></a>`privacy.imageDiskMinHits` | 1 | 0–100 | Hits an entry needs before an eviction from RAM writes it to disk instead of dropping it. |
| <a id="privacy.imageDiskWriteQueueMiB"></a>`privacy.imageDiskWriteQueueMiB` | 8 | 1–256 MiB | Bytes in MiB that may wait for the background disk writer; past it, evictions are dropped. |
| <a id="privacy.imageTtlStepSecs"></a>`privacy.imageTtlStepSecs` | 600 s | 0–86400 s | Base seconds added to an entry's remaining lifetime on each hit, scaled up with its decayed hit count; zero keeps every entry at the base lifetime. |
| <a id="privacy.imageTtlMaxSecs"></a>`privacy.imageTtlMaxSecs` | 86400 s | 60–2592000 s | Longest remaining lifetime in seconds an entry can accumulate through hits. |
| <a id="privacy.imageMaxAgeSecs"></a>`privacy.imageMaxAgeSecs` | 604800 s | 3600–7776000 s | Absolute age in seconds after which an entry is refetched no matter how often it is hit. |
| <a id="privacy.imageScoreHalfLifeSecs"></a>`privacy.imageScoreHalfLifeSecs` | 3600 s | 60–604800 s | Half-life in seconds of the hit count and recency used in the priority score. |
| <a id="privacy.imageScoreDemoteMilli"></a>`privacy.imageScoreDemoteMilli` | 300 (0.3) | 0–10000 | Priority score in thousandths below which a RAM entry may be demoted to disk. |
| <a id="privacy.imageScoreHysteresisPercent"></a>`privacy.imageScoreHysteresisPercent` | 50 % | 0–500 % | Percentage the promote threshold sits above the demote threshold, so entries do not flap between tiers. |
| <a id="privacy.imageContextOpenPercent"></a>`privacy.imageContextOpenPercent` | 200 % | 100–1000 % | Score weight in percent for images requested for a message that is open. |
| <a id="privacy.imageContextRecentPercent"></a>`privacy.imageContextRecentPercent` | 150 % | 100–1000 % | Score weight in percent for recently used images; background loads weigh 100. |
| <a id="privacy.imageAccountSharePercent"></a>`privacy.imageAccountSharePercent` | 50 % | 1–100 % | Largest share in percent of the RAM tier one account may fill while other accounts are present. |
| <a id="privacy.imageSweepSecs"></a>`privacy.imageSweepSecs` | 30 s | 5–3600 s | Seconds between background maintenance sweeps of the image cache. |
| <a id="privacy.imageSnapshotSecs"></a>`privacy.imageSnapshotSecs` | 300 s | 0–86400 s (0 turns snapshots off) | Seconds between atomic snapshots of the disk index; zero turns snapshots off and restarts read each file header instead. |
| <a id="privacy.imageSweepBatch"></a>`privacy.imageSweepBatch` | 256 | 16–8192 | Entries one maintenance sweep may expire, rescore or evict. |

These limits are independent: increasing the account limit does not override the global network, pipeline or decoder ceiling. Higher decoder concurrency consumes more CPU and memory. Cached successes do not consume new network tokens. Every cache lookup still requires current authorization; setting TTL to zero does not weaken grants, tracker blocking, SSRF checks or egress routing.

### Image proxy cache tiers

All `privacy.image*` and `privacy.cache*` cache keys above form the **Image proxy** section of Admin › Settings. The cache has two tiers behind one in-memory index. Authentication, the display grant and the SSRF gate run before the cache is read, so an unauthenticated or ungranted request never touches either tier.

- **RAM tier** (small): sized by `privacy.imageCacheMode` and its floors and ceilings, further capped by `privacy.imageRamMaxMiB`.
- **Disk tier** (larger, off by default): `privacy.imageDiskEnabled`, with the directory in `privacy.imageDiskPath` (empty means an `image-cache` folder next to the database file). Size is automatic (a tenth of the usable space between `privacy.imageDiskFloorMiB` and `privacy.imageDiskCeilingMiB`, never more than half of it) or `privacy.imageDiskMaxMiB` exactly.
- Lookup is RAM, then disk, then the network. A disk hit above the promote threshold moves into RAM. A RAM eviction writes to disk only when the image has at least `privacy.imageDiskMinHits` hits and a score above the demote threshold. Writes happen in a background worker, never on a request.
- **Lifetime**: a cold image keeps the base lifetime (`privacy.imageCacheTtlFloorSecs` to `privacy.imageCacheTtlCeilingSecs` in automatic mode, `privacy.cacheTtlSecs` in manual mode). Each hit adds `privacy.imageTtlStepSecs` times one plus the log of the decayed hit count to the remaining lifetime, capped by `privacy.imageTtlMaxSecs` and never past `privacy.imageMaxAgeSecs` from creation. At the maximum age the image is fetched again: the fetch path keeps no upstream `ETag` or `Last-Modified`, so conditional revalidation is not available.
- **Priority score**: value per byte, `(0.2 + ln(1 + hits)) * (0.25 + 0.75 * recency) * context / (1 + size / 32 KiB)`, with hits and recency halving every `privacy.imageScoreHalfLifeSecs`. A request that names a message weighs `privacy.imageContextOpenPercent` (`privacy.imageContextRecentPercent` after ten idle minutes); one that names none weighs 100. The server cannot tell a prefetch from a read beyond that. Placement uses `privacy.imageScoreDemoteMilli` and a promote threshold `privacy.imageScoreHysteresisPercent` above it. `privacy.imageAccountSharePercent` caps one account's share of RAM while other accounts use the cache.
- **Index**: every entry, in either tier, has one in-memory record (tier, size, expiry, last use, hits, `ETag`, file name). A lookup that misses costs a hash and a map probe and no disk access. At start the index is rebuilt by a bounded directory scan; the atomic snapshot written every `privacy.imageSnapshotSecs` supplies hit counts and extended lifetimes for files whose sizes still match. A file that disappears is dropped from the index on the next read. Index records count against the RAM tier.
- **Files**: one file per image, named by a 128-bit hash of the route and URL (the URL is never written), written under a temporary name and renamed, mode 0600 in a directory created 0700 on Unix. Only the cache's own files are ever deleted: leftover temporary files, expired, over-age, truncated or corrupt images and the snapshot. Other files in the directory are left alone.
- **Failures**: a missing, full or read-only directory turns the disk tier off for 30 seconds at a time and the cache keeps serving from RAM; no request fails because of it. The readout shows the reason.

The disk tier needs a writable path. The shipped images run as a non-root user with `/data` as the volume, and the hardened overlay makes the rest of the filesystem read-only. Nothing is written to disk unless `privacy.imageDiskEnabled` is on. To use it, either leave `privacy.imageDiskPath` empty, which uses `/data/image-cache` on the existing volume, or mount a dedicated writable volume (for example at `/cache/images`) and set the path to it. Admin › Settings › Image proxy shows the configured and effective values, per-tier size, entries, hit rate, hot and cold counts and promotions per minute.

<a id="privacy.imageRetryAttempts"></a>
<a id="privacy.imageRetryDelayMs"></a>
Automatic retry preferences support global/domain/user defaults and administrator locks. `privacy.imageRetryAttempts` is 0–8 (default 3); zero leaves manual retry available. `privacy.imageRetryDelayMs` is 500–60000 ms (default 5000). Users configure these under Settings › Privacy; the delay is shown in seconds. Retries start after the current frame settles and target only server-identified temporary failures. Permanent policy, type and size refusals are not retried automatically.

The reader follows the effective server queue-plus-fetch-plus-decode budget, with a five-second notification grace. It never cancels browser image fetches because a notification expires. Live authenticated frame progress remains available afterwards, and a late completion refreshes retry references once without restarting continuous polling. Switching accounts or messages discards stale results.

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

<a id="cap.compose.rich_text"></a>
`cap.compose.rich_text` — Whether users may write formatted (HTML) messages. Off sends plain text only. Default `true` (t98 feature profiles); enforced at JMAP Email/set create with an HTML body (feature_gate.rs); composer plain-text only.

<a id="cap.compose.attachments"></a>
`cap.compose.attachments` — Whether users may attach files to messages they write. Inline images in the body are not counted. Default `true` (t98 feature profiles); enforced at JMAP Email/set create with attachment parts (feature_gate.rs); composer attach button.

<a id="cap.compose.share_links"></a>
`cap.compose.share_links` — Whether users may send a large file as a Nextcloud share link instead of an attachment. Default `true` (t98 feature profiles); enforced at POST /api/nextcloud/share-link (feature_gate.rs).

<a id="cap.compose.scheduled_send"></a>
`cap.compose.scheduled_send` — Whether users may schedule a message to be sent later. Default `true` (t98 feature profiles); enforced at JMAP EmailSubmission/set with sendAt (feature_gate.rs).

<a id="cap.compose.read_receipts"></a>
`cap.compose.read_receipts` — Whether users may ask recipients for a read receipt. Default `true` (t98 feature profiles); enforced at JMAP Email/set create with mailwomanRequestReadReceipt (feature_gate.rs).

<a id="cap.signatures"></a>
`cap.signatures` — Whether users may create and edit their own signatures. Default `true` (t98 feature profiles); enforced at writes under /api/account/signatures (feature_gate.rs).

<a id="cap.identities"></a>
`cap.identities` — Whether users may add and edit the identities they send as. Default `true` (t98 feature profiles); enforced at writes under /api/account/identities (feature_gate.rs).

<a id="cap.keys"></a>
`cap.keys` — Whether users have the Keys module and may sign and encrypt messages. Default `true` (t98 feature profiles); enforced at JMAP CryptoKey/* (feature_gate.rs); Keys module, compose sign and encrypt.

<a id="cap.links.trusted"></a>
`cap.links.trusted` — Whether users may change their own list of trusted link domains. The administrator list still applies. Default `true` (t98 feature profiles); enforced at PUT /api/settings/links.* (feature_gate.rs).

<a id="cap.zoom"></a>
`cap.zoom` — Whether users may zoom the message reader. Default `true` (t98 feature profiles); enforced at PUT /api/settings/reader.zoom (feature_gate.rs); reader zoom controls.

<a id="cap.mail.focused_inbox"></a>
`cap.mail.focused_inbox` — Whether users may turn on the Focused and Other inbox tabs. Default `true` (t98 feature profiles); enforced at PUT /api/settings/mail.focusedInbox (feature_gate.rs); Focused and Other tabs.

<a id="cap.views.events"></a>
`cap.views.events` — Whether users have the Events view of calendar mail. Default `true` (t98 feature profiles); enforced at JMAP Email/query on mwViewEvents (feature_gate.rs); Events view.

<a id="cap.views.attachments"></a>
`cap.views.attachments` — Whether users have the Attachments view. The messages stay reachable through search. Default `true` (t98 feature profiles); enforced at Attachments view in the sidebar (web only: the same messages stay reachable by search).

<a id="cap.search.operators"></a>
`cap.search.operators` — Whether users may search with operators such as from:, subject: or has:attachment. Default `true` (t98 feature profiles); enforced at JMAP Email/query text with field operators (feature_gate.rs); search filter chips.

<a id="cap.saved_searches"></a>
`cap.saved_searches` — Whether users may save searches. Default `true` (t98 feature profiles); enforced at writes under /api/account/saved-searches (feature_gate.rs).

<a id="cap.vacation"></a>
`cap.vacation` — Whether users may set an automatic vacation reply. Default `true` (t98 feature profiles); enforced at JMAP VacationResponse/* (feature_gate.rs, besides cap.rules).

<a id="cap.labels"></a>
`cap.labels` — Whether users may put labels on messages. Default `true` (t98 feature profiles); enforced at JMAP Email/set adding a non-system keyword (feature_gate.rs).

<a id="cap.snooze"></a>
`cap.snooze` — Whether users may snooze messages. Ending a snooze stays possible. Default `true` (t98 feature profiles); enforced at JMAP Email/set snoozedUntil (feature_gate.rs).

<a id="cap.follow_up"></a>
`cap.follow_up` — Whether users may set follow-up reminders on messages. Clearing one stays possible. Default `true` (t98 feature profiles); enforced at JMAP Email/set followUpAt (feature_gate.rs).

<a id="cap.masked"></a>
`cap.masked` — Whether users may create and manage masked addresses. Default `true` (t98 feature profiles); enforced at /api/masked (feature_gate.rs).

<a id="cap.directory"></a>
`cap.directory` — Whether users may search the organisation directory. Default `true` (t98 feature profiles); enforced at /api/directory (feature_gate.rs).

<a id="cap.mcp"></a>
`cap.mcp` — Whether the user's keys and tokens may use the MCP endpoint. Default `true` (t98 feature profiles); enforced at /mcp with a key or token (scope_mw.rs).

<a id="cap.appearance.custom"></a>
`cap.appearance.custom` — Whether users may change their theme, accent, custom and badge colours, density and font. Off keeps the administrator appearance. Default `true` (t98 feature profiles); enforced at PUT /api/account/appearance (feature_gate.rs); theme, accent, custom and badge colours, density, font.

<a id="cap.appearance.custom_code"></a>
`cap.appearance.custom_code` — Whether users may save their own CSS and appearance scripts. The deployment switches in Appearance still apply above this. Default `true` (t98 feature profiles); enforced at PUT /api/account/appearance/customization (feature_gate.rs), below appearance.customCssEnabled/customJsEnabled.

<a id="cap.avatars"></a>
`cap.avatars` — Whether users may change mailbox avatars. Default `true` (t98 feature profiles); enforced at PUT /api/accounts/layout with avatars (feature_gate.rs).

<a id="cap.shortcuts"></a>
`cap.shortcuts` — Whether users may change keyboard shortcuts. The default shortcuts keep working. Default `true` (t98 feature profiles); enforced at PUT /api/settings/shortcuts.* (feature_gate.rs); shortcut editor.

<a id="cap.analytics"></a>
`cap.analytics` — Whether users are asked for analytics consent and may send usage events. Default `true` (t98 feature profiles); enforced at POST /api/analytics/events (feature_gate.rs); consent prompt.

<a id="cap.apps.native"></a>
`cap.apps.native` — Whether users may use the desktop and mobile apps. Default `true` (t98 feature profiles); enforced at native bearer sessions (feature_gate.rs).

<a id="access.protocols"></a>
`access.protocols` — Which mail server protocols these users may sign in with: imap, pop3, ews. Sign-ins through a JMAP server follow that server. Default `["imap","pop3","ews"]`; other values are refused. `/jmap/*` and `/api/v1/*` answer `403 {code:"feature_disabled", feature:"access.protocols"}` for an account whose kind is not listed.

<a id="limit.linked_mailboxes"></a>
`limit.linked_mailboxes` — How many mailboxes a user may link, below the server maximum. 0 means only the server maximum applies. Default `0`. `POST /api/accounts/link` answers `409 {code:"limit_exceeded", limit:"limit.linked_mailboxes"}` at the limit.

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

### Retention and log files

Retention limits and log rotation are global settings. They have no domain or user scope because the sweep and the log files belong to the whole server. `docs/admin/retention.md` lists every store, what it holds and who removes it. A limit of 0 keeps rows until they are deleted by hand; the console shows a warning for it. These keys are configurable controls and do not make a deployment compliant with any law.

<a id="encryption.rotationDays"></a>
### Database key rotation interval (days) (`encryption.rotationDays`)
Global, hot. Default `365`. Days after the last key change at which the database key is reported as due for rotation. 0 sets no due date. The key itself is never stored in settings.

<a id="encryption.warnings.enabled"></a>
### Report encryption findings (`encryption.warnings.enabled`)
Global, hot. Default `true`. Show plaintext stores and an overdue key rotation as a finding in the deployment assessment.

<a id="retention.sweeper.enabled"></a>
### Retention sweeper (`retention.sweeper.enabled`)
Global, hot. Default `true`. Run the daily sweep that deletes rows older than the limits on this page. Turning it off keeps every row until it is turned on again.

<a id="retention.sweeper.batchRows"></a>
### Rows per batch (`retention.sweeper.batchRows`)
Global, hot. Default `500`. Rows one delete statement removes. Smaller batches hold database locks for less time.

<a id="retention.sweeper.maxBatches"></a>
### Batches per store per run (`retention.sweeper.maxBatches`)
Global, hot. Default `20`. Most delete statements one store gets in a run. A store with more old rows continues in the next run.

<a id="retention.sweeper.jitterMinutes"></a>
### Start-time spread (minutes) (`retention.sweeper.jitterMinutes`)
Global, hot. Default `30`. The next run starts a random number of minutes up to this value after 24 hours, so several servers do not sweep together.

<a id="retention.sweeper.vacuumAfterRows"></a>
### Reclaim space after (rows) (`retention.sweeper.vacuumAfterRows`)
Global, hot. Default `100000`. When one run deletes at least this many rows the database is compacted (VACUUM). 0 never compacts after a sweep; the weekly vacuum job is separate.

<a id="retention.passwordChangeAudit.days"></a>
### Password-change records (days) (`retention.passwordChangeAudit.days`)
Global, hot. Default `365`. Days a password-change record (account, method, outcome) is kept. 0 keeps them forever.

<a id="retention.ssoLoginAudit.days"></a>
### Single sign-on login records (days) (`retention.ssoLoginAudit.days`)
Global, hot. Default `90`. Days a single sign-on login record (provider, hashed subject, outcome) is kept. 0 keeps them forever.

<a id="retention.dlpAudit.days"></a>
### Data-loss-prevention records (days) (`retention.dlpAudit.days`)
Global, hot. Default `365`. Days a record of a data-loss-prevention rule match is kept. 0 keeps them forever.

<a id="retention.assistAudit.days"></a>
### Assistant access records (days) (`retention.assistAudit.days`)
Global, hot. Default `365`. Days a record of an assistant using a capability (actor, capability, scope summary) is kept. 0 keeps them forever.

<a id="retention.alertEvents.days"></a>
### Fired alert records (days) (`retention.alertEvents.days`)
Global, hot. Default `90`. Days a record of a fired administrator alert is kept. 0 keeps them forever.

<a id="retention.jobRuns.days"></a>
### Background job history (days) (`retention.jobRuns.days`)
Global, hot. Default `90`. Days a finished background-job run is kept. A run that is still running is never removed. 0 keeps the history forever.

<a id="retention.outbox.days"></a>
### Sent and cancelled send records (days) (`retention.outbox.days`)
Global, hot. Default `30`. Days the send record of a delivered or cancelled message is kept. A message that is waiting, being retried, failed or uncertain is never removed. 0 keeps them forever.

<a id="retention.expiredCredentials.graceDays"></a>
### Expired credentials (days after expiry) (`retention.expiredCredentials.graceDays`)
Global, hot. Default `1`. Days an expired remembered device, sign-in challenge, sign-in token, OAuth token, webhook nonce, source failure counter or image link handle is kept after it expired. 0 removes it on the next run.

<a id="retention.ipAddresses.days"></a>
### Reduce stored IP addresses after (days) (`retention.ipAddresses.days`)
Global, hot. Default `90`. After this many days the IP address in authentication history and audit entries is truncated or hashed and the row is kept. 0 keeps addresses as recorded.

<a id="retention.ipAddresses.mode"></a>
### IP reduction method (`retention.ipAddresses.mode`)
Global, hot. Default `"truncate"`. Truncate keeps the IPv4 /24 or IPv6 /48 prefix. Hash replaces the address with a short keyed hash that still groups the same address.

<a id="retention.user.enabled"></a>
### Users may set retention rules (`retention.user.enabled`)
Global and domain, hot. Default `true`. Users can set their own retention rules for folders and mail. When off, existing user rules stop running; the organisation-wide rules below still run.

<a id="retention.user.minDays"></a>
### Fewest days before a user rule deletes mail (`retention.user.minDays`)
Global and domain, hot. Default `7`. A user rule that deletes mail cannot act on mail younger than this. A smaller value is refused when the rule is saved and raised when it runs.

<a id="retention.user.maxDays"></a>
### Most days a user rule may wait to delete mail (`retention.user.maxDays`)
Global and domain, hot. Default `0`. A user rule that deletes mail cannot wait longer than this. 0 sets no ceiling. A larger value is refused when the rule is saved and lowered when it runs.

<a id="retention.user.maxRules"></a>
### Most retention rules per user (`retention.user.maxRules`)
Global and domain, hot. Default `50`. The most retention rules one user may keep.

<a id="retention.user.allowPermanentDelete"></a>
### User rules may delete mail permanently (`retention.user.allowPermanentDelete`)
Global and domain, hot. Default `false`. When off, a delete rule always moves mail to Trash and the undo window applies before it is purged.

<a id="retention.user.undoDays"></a>
### Undo window for retention rules (days) (`retention.user.undoDays`)
Global and domain, hot. Default `7`. Days mail moved to Trash by a retention rule stays there before it is purged. A user may choose a shorter window. 0 purges at the next run.

<a id="retention.user.mandatory"></a>
### Organisation-wide mail rules (`retention.user.mandatory`)
Global and domain, hot. Default `[]`. Rules every account gets, as folder-role:days entries, for example trash:30 or junk:30. Mail in that folder older than the days is purged. A user cannot change, pause or exclude mail from these. Mail of an account under legal hold is skipped.

<a id="retention.user.protectedFolders"></a>
### Folders retention rules never touch (`retention.user.protectedFolders`)
Global and domain, hot. Default `["drafts"]`. Folders no user rule works on, by role (drafts, sent, inbox, archive) or by folder name. A protected folder wins over an organisation-wide rule for the same folder.

<a id="retention.user.templates"></a>
### Retention templates offered to users (`retention.user.templates`)
Global and domain, hot. Default `["newsletters30","receipts365","notifications14","trash30","junk30","read-archive180"]`. Templates users can adopt in Settings, by id: newsletters30, receipts365, notifications14, trash30, junk30, read-archive180.

<a id="retention.user.runForRestricted"></a>
### Run user rules for restricted accounts (`retention.user.runForRestricted`)
Global, hot. Default `false`. Whether user retention rules also run for accounts under a restriction of processing. Off skips those accounts.

<a id="retention.user.intervalHours"></a>
### Hours between user retention runs (`retention.user.intervalHours`)
Global, hot. Default `24`. Hours between scheduled runs of the user retention job for one account.

<a id="retention.user.batchMessages"></a>
### Messages per retention call (`retention.user.batchMessages`)
Global, hot. Default `200`. Messages one move or delete call handles. At most 200 are used.

<a id="retention.user.maxPerRun"></a>
### Most messages changed per account per run (`retention.user.maxPerRun`)
Global, hot. Default `5000`. Most messages the user retention job changes for one account in one run. The rest waits for the next run.

<a id="retention.user.pauseMs"></a>
### Pause between retention batches (ms) (`retention.user.pauseMs`)
Global, hot. Default `100`. Pause between batches, in milliseconds, so a large cleanup does not load the mail server.

<a id="retention.class.short.days"></a>
### Short class ceiling (days) (`retention.class.short.days`)
Global, hot. Default `365`. Ceiling for operational data: sessions, tokens, rate limits, job history and metrics. Applies when the class limit is on.

<a id="retention.class.medium.days"></a>
### Medium class ceiling (days) (`retention.class.medium.days`)
Global, hot. Default `1095`. Ceiling for sign-in history, request protection history and security notices. Must not be below the short class; a lower value is raised to it.

<a id="retention.class.long.days"></a>
### Long class retention (days) (`retention.class.long.days`)
Global, hot. Default `2555`. Days the long class (security audit, administrator actions, requests, consent, tombstones) is kept, archive included. 0 keeps it for ever and the console warns. Must not be below the medium class.

<a id="retention.class.enforce"></a>
### Hold each store to its class ceiling (`retention.class.enforce`)
Global, hot. Default `false`. When on, each store's own limit is clamped to its class ceiling and 0 (keep) counts as the ceiling. When off, per-store limits stand.

<a id="retention.archive.enabled"></a>
### Archive aged rows of the long class (`retention.archive.enabled`)
Global, hot. Default `false`. Move aged rows of the long class into sealed, compressed archive files instead of deleting them.

<a id="retention.archive.directory"></a>
### Archive directory (`retention.archive.directory`)
Global, hot. Default `"archive"`. Directory for archive files, absolute or relative to the server working directory. Parent-directory segments (..) are refused.

<a id="retention.archive.batchRows"></a>
### Rows per archive file (`retention.archive.batchRows`)
Global, hot. Default `1000`. Rows written into one archive file.

<a id="retention.archive.maxSegments"></a>
### Archive files per store per run (`retention.archive.maxSegments`)
Global, hot. Default `10`. Archive files one run may write for each store. The rest waits for the next run.

<a id="retention.archive.keyGeneration"></a>
### Archive sealing key generation (`retention.archive.keyGeneration`)
Global, hot. Default `1`. Generation of the key that seals new archive files. Raise it to rotate the key; earlier files still verify with their own generation.

<a id="retention.archive.verifyDays"></a>
### Days between archive checks (`retention.archive.verifyDays`)
Global, hot. Default `7`. Days between integrity checks of the archive files.

<a id="retention.archive.legalHold"></a>
### Legal hold on the archive (`retention.archive.legalHold`)
Global, hot. Default `false`. Keep every archive file, whatever the long class limit says.

<a id="logging.file.enabled"></a>
### Write log files (`logging.file.enabled`)
Global, hot. Default `false`. Also write the server log to rotating files. Off keeps logging to standard output only, which is what a container normally uses.

<a id="logging.file.directory"></a>
### Log directory (`logging.file.directory`)
Global, hot. Default `"logs"`. Directory for the log files, absolute or relative to the server's working directory. Parent-directory segments (..) are refused.

<a id="logging.file.maxSizeMb"></a>
### Rotate at size (MB) (`logging.file.maxSizeMb`)
Global, hot. Default `50`. The current log file is closed and a new one started when it reaches this size.

<a id="logging.file.rotateHours"></a>
### Rotate at age (hours) (`logging.file.rotateHours`)
Global, hot. Default `24`. The current log file is rotated once it is this old. 0 rotates by size only.

<a id="logging.file.maxAgeDays"></a>
### Keep rotated files (days) (`logging.file.maxAgeDays`)
Global, hot. Default `14`. Rotated log files older than this are deleted.

<a id="logging.file.maxFiles"></a>
### Most rotated files (`logging.file.maxFiles`)
Global, hot. Default `14`. The oldest rotated files beyond this count are deleted.

<a id="logging.file.maxTotalMb"></a>
### Most total size (MB) (`logging.file.maxTotalMb`)
Global, hot. Default `500`. The oldest rotated files are deleted until the log directory is under this size.

<a id="logging.file.compress"></a>
### Compress rotated files (`logging.file.compress`)
Global, hot. Default `true`. Rotated log files are compressed with gzip.

<a id="logging.level"></a>
### Log level (`logging.level`)
Global, hot. Default `"info"`. The level for components that have no level of their own below. The MW_LOG environment variable sets the level at start-up until this setting is read.

<a id="logging.componentLevels"></a>
### Component log levels (`logging.componentLevels`)
Global, hot. Default `[]`. One component=level entry per line, for example mw_imap=debug. Levels are error, warn, info, debug and trace.

<a id="logging.redaction.addresses"></a>
### Email addresses in logs (`logging.redaction.addresses`)
Global, hot. Default `"full"`. Full leaves addresses as logged. Truncated keeps the first letter and the domain. Hashed replaces the local part with a short hash. None removes the address.

<a id="logging.redaction.ips"></a>
### IP addresses in logs (`logging.redaction.ips`)
Global, hot. Default `"full"`. Full leaves addresses as logged. Truncated keeps the IPv4 /24 or IPv6 /48 prefix. Hashed replaces the address with a short hash. None removes it.

<a id="logging.redaction.tokens"></a>
### Secrets in logs (`logging.redaction.tokens`)
Global, hot. Default `true`. Cuts bearer tokens, API keys, passwords in URLs and similar secrets out of log lines.

<a id="retention.sessions.idleDays"></a>
### Unused sessions (days) (`retention.sessions.idleDays`)
Global, hot. Default `90`. Days a browser or native-client session record, with its address and user agent, may go unused before it is deleted. The session timeouts apply first; this only removes rows left behind. 0 keeps them.

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
| <a id="reader.zoom"></a>`reader.zoom` | 50–200 percent (the reader snaps to 50, 67, 75, 90, 100, 110, 125, 150, 175, 200) | 100 |
| <a id="reader.attachmentsCollapsed"></a>`reader.attachmentsCollapsed` | true, false (the reader's attachments bar starts collapsed; the user's last toggle is saved here) | false |

Automatic colors adapt in dark themes and preserve original colors in light themes. Inversion reverses message colors while counter-inverting images. Adaptive mode removes sender foreground/background overrides while preserving layout and media sources. Loader delay affects only feedback, never delivery.

<a id="login.showDomainHint"></a>
### `login.showDomainHint`

Global boolean, default `false`. Shows the preferred-domain hint below Email or username on the sign-in page when enabled. Hiding this hint does not change username routing or authentication.

<a id="login.rememberSignins"></a>
### `login.rememberSignins`

Global boolean, default `true`. Allows this browser to remember successful sign-in addresses for later selection. Only addresses are remembered, never passwords, tokens or other credentials. The public sign-in configuration exposes `rememberSignins`; when it is false, the client clears remembered addresses and stops recording them. This does not change session lifetime or authentication requirements.

<a id="login.showAccountHelpNotice"></a>
### `login.showAccountHelpNotice`

Global/domain boolean, default `false`. Shows the account-help notice after a refused mailbox sign-in only when enabled by an administrator. The public configuration exposes only `showAccountHelpNotice`; domain policy uses the trusted sign-in host mapping, never an account lookup. A wrong password and a disabled account retain the same generic refusal.

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
Global/domain/user composer safeguard: `off`, `warn` (default), `confirm`, `warn-remember` or `deny` (see `mail.compose.checkInvalidRecipients` for what warn, warn-remember and deny do). Checks To, Cc and Bcc. Confirmation is invalidated when the external recipient set or sending identity changes. This is not a server-side delivery block and does not restrict other mail clients or direct API submissions.

<a id="mail.trustedRecipientDomains"></a>
### `mail.trustedRecipientDomains`
Global/domain/user list of up to 100 exact domains, each at most 253 characters. DNS case and IDNA are normalized; invalid entries grant no trust. Subdomains are not implicitly trusted. The selected sending identity's domain is trusted automatically. Administrators can lock both settings to prevent user overrides; an unlocked user list replaces the inherited list.

<a id="mail.compose.checkInvalidRecipients"></a>
<a id="mail.compose.checkMissingAttachment"></a>
<a id="mail.compose.checkEmptySubject"></a>
### `mail.compose.checkInvalidRecipients`, `mail.compose.checkMissingAttachment`, `mail.compose.checkEmptySubject`

Global/domain/user pre-send checks in the composer. Each takes one of four values; the default is `warn`.

- `allow`: no check and no warning.
- `warn`: Send shows one combined dialog listing every problem, with **Send anyway** and **Review**. The message is never blocked.
- `warn-remember`: like `warn`, with a "Don't ask again for these" option. Choosing it together with Send anyway stores `allow` for that check in the user's own settings. The option is not offered when an administrator has locked the value.
| <a id="attachments.bar.clickToToggle"></a>`attachments.bar.clickToToggle` | true, false (clicking anywhere on the reader's attachments bar header expands or collapses it; when false only the arrow button does) | true |
| <a id="attachments.bar.showDownloadButtons"></a>`attachments.bar.showDownloadButtons` | always, hover, never (when each attachment in the bar shows its download button; touch screens always show it in hover mode) | hover |
| <a id="attachments.bar.downloadAll"></a>`attachments.bar.downloadAll` | true, false (the bar's Download all button, which saves the allowed attachments as one ZIP file, or a single attachment directly) | true |
| <a id="attachments.bar.zipInlineImages"></a>`attachments.bar.zipInlineImages` | true, false (include images shown inside the message text in the Download all ZIP) | false |
- `deny`: Send is refused with the reason shown in the composer. This is the only value that blocks a message.

The checks are: a recipient address in To, Cc or Bcc that is not a valid mailbox; text that mentions an attachment ("attached", "enclosed", "anexo", "adjunto", "pièce jointe", "angehängt", "allegato", "bijlage" and similar) while nothing is attached (quoted lines, the quoted original and the signature are ignored); and an empty subject. The external-recipient check is `mail.externalRecipientWarning`, which also accepts `warn-remember` and `deny`; its `off` value means no check, and `confirm` keeps its separate "I have checked these recipients" box. Administrators can lock any of these to prevent user overrides. While a draft is open, the composer also shows invalid addresses and a missing subject in a compact strip above the fields; recipients outside the trusted domains appear in a smaller, dismissible line.

<a id="mail.compose.autoChipRecipients"></a>
<a id="mail.compose.recipientSeparator"></a>
### `mail.compose.autoChipRecipients`, `mail.compose.recipientSeparator`

Global/domain/user. When `autoChipRecipients` is `true` (default), a valid address followed by a space, comma or semicolon (or by Enter or leaving the field) becomes a finished recipient, underlined with the contact's name when one is known, and the separator is added for you. Invalid text stays as typed with a wavy underline. Pasting several addresses splits them. `recipientSeparator` is `comma` (default) or `semicolon` and sets the separator inserted after a finished recipient.

<a id="mail.compose.headerCollapse"></a>
### `mail.compose.headerCollapse`

Global/domain/user. How the composer header (From, To, Cc, Bcc, Subject) behaves: `expanded` (default; always shown until the user folds it), `collapsed` (folded to a one-line summary such as `To: Ana +2 · Cc: 1 · Subject: Re: Invoice`, opened by clicking it) or `auto` (folded while the user writes the body, open while a header field has focus). A header with no recipient and no subject is always open, and a click on a problem in the composer's status bar opens it. A user's own fold or unfold is remembered in that browser. An administrator can set the default and lock it. The composer ribbon's pinned or auto-hide behaviour is the existing `ui.bar.composerRibbon` key.

<a id="mail.replyImages"></a>
### `mail.replyImages`
Global/domain/user policy for including the original message's images (inline parts and proxied remote images) in a reply or forward: `ask` (default; the quote shows a prompt with Include images, Not now and Always trust this domain), `allow` (always include), `trusted` (include when the sender's domain is in `mail.replyImageTrustedDomains`, otherwise ask) or `deny` (the quote shows no images and none are fetched). Included images are copied into the draft as inline attachments, so they are fetched through the same authorised, tracker-filtering routes the reader uses. Administrators can lock the value to prevent user overrides. Only newly opened compositions change.

<a id="mail.replyImageTrustedDomains"></a>
### `mail.replyImageTrustedDomains`
Global/domain/user list of up to 200 exact sender domains, each at most 253 characters, used by the `trusted` policy of `mail.replyImages`. Subdomains are not implicitly trusted. Choosing Always trust this domain in the quote prompt appends the sender's domain to the user's own list and, when the policy is still `ask`, switches the user's policy to `trusted`. When an administrator locks either setting the prompt offers only Include images and Not now.

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
- <a id="shortcuts.zoomIn"></a>`shortcuts.zoomIn` — Zoom the message body in (preset Ctrl+= and Ctrl++), only while focus is in the reader; default `inherit`.
- <a id="shortcuts.zoomOut"></a>`shortcuts.zoomOut` — Zoom the message body out (preset Ctrl+-); default `inherit`.
- <a id="shortcuts.zoomReset"></a>`shortcuts.zoomReset` — Reset the message body to 100% (preset Ctrl+0); default `inherit`.
- <a id="shortcuts.openInTab"></a>`shortcuts.openInTab` — Open the focused message-list row in its own workspace tab, whatever the reading pane setting (preset Shift+Enter); default `inherit`.

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
allows or denies unmapped addresses (default deny). An active policy with a missing or unreadable
country database refuses admission, including when unknown addresses are allowed. The local database
is managed under Security → Country database or supplied with `MW_GEOIP_DB`
([Country database](country-database.md)); lookups never leave the server. Inspect the current-source
diagnostic under Observability before enabling. Existing authenticated requests and sessions are
not governed by this initial sign-in policy; broader session geofencing is not implemented here.

<a id="trust-tiers"></a>
### Trust tiers, bot classes and the unknown-country admin block

Every request gets one tier: authenticated (a valid session; never scaled), known (anonymous from an IPv4 address or IPv6 /64 with a successful sign-in inside `security.trust.knownTtlDays`) or unknown. The tier scales only anonymous request allowances: the per-address request budget, `/api/discover`, the sign-in cooldown and the challenge difficulty. It never bypasses a password, second factor, sign-in lockout, SSRF gate, active ban or abuse signal. Failed sign-ins and unauthenticated requests never make a source known. The table is bounded and holds a keyed hash of the source, its country, a last-seen time and a small account count. It is kept in memory and saved to the settings store (see Saved known sources below); at startup the saved copy is merged with a rebuild from the successful rows the authentication log still holds (`logs.auth.retentionDays`). A forwarded address counts only through the trusted-proxy rules; `/healthz` and `/readyz` are untouched. Changes apply within 30 seconds.

**Saved known sources.** With `security.trust.persist` on (the default) the table is saved every `security.trust.persistIntervalSecs` (default 300 s, 30-3600), only when it changed, and once at graceful shutdown; a save never delays or fails a sign-in, and a failed save is logged once per interval and retried. Stored, in one row of the settings table named `trust-tier:v1` (no extra table or file): for each source a hash keyed from the server key (the address, or the IPv6 /64, is never stored), its two-letter country code, the last sign-in time, and up to 16 keyed hashes of account identifiers (used only as a count), plus the latest sign-in time per country. At most `security.trust.knownCapacity` sources are kept, the oldest dropped first, and entries older than `security.trust.knownTtlDays` are dropped both when saving and when loading, so nothing outlives the window. The row is authenticated with a key derived from the server key: a row that was edited, copied from another server or written by a newer version is ignored with one log line and the table starts from the authentication-log rebuild. Changing the server key therefore makes the saved table unusable. Replicas that share one store share the row: each save merges what is there (latest sign-in time wins, account hashes are united), and a save can briefly omit a peer's newest entries until the next interval. A snapshot restored after a rollback of the database can bring back sources whose sign-ins were within the window at the time of the snapshot; they expire by their own age. To forget every known source, call `DELETE {console}/security/trust-table` (needs `bans.write` and the ban step-up; Security, Admin probe shows the entry count, last save and entries restored at startup): the table and the saved row are emptied and earlier authentication-log rows are no longer used to rebuild it. Turning the setting off stops writing and loading; use the same action to remove a copy already saved.

Bot classes (`security.trust.ua*`) act only on anonymous requests, by claimed user-agent (a header, not verified identity): `allow` skips the user-agent budget, `limit` keeps it, `block` answers 403 with `X-Robots-Tag: noindex, nofollow, noai`. A signed-in session is never blocked by class. `security.protection.userAgentExceptions` keeps a needed client allowed. The app serves no `robots.txt` or `ai.txt`.

The admin-console country block adds a rule to admin-probe protection (it needs `security.adminProbe.enabled`): a probe of the console (probe list, console path guesses, unauthenticated console API calls, refused console sign-ins) from a source that is not known, not an authenticated administrator, and whose country has no successful sign-in inside the window and is not in `security.trust.adminGeoCountries`, counts toward `security.trust.adminGeoThreshold` and then blocks the source through the same tracker as other admin-probe blocks, with escalating lengths. The block shows in Security, Admin probe with the reason "admin discovery from unknown country" and is lifted with the existing one-click unblock. Without a readable country database nothing is blocked by this rule. To recover a locked-out administrator: sign in from another network, add the network to `security.adminProbe.allowlist`, add the country to `security.trust.adminGeoCountries`, or set `MW_ADMIN_GEO_ENABLED=false` (or `MW_ADMIN_PROBE_ENABLED=false`) and restart.

<a id="account-signin-geography"></a>
### Per-account sign-in country policy

Off by default. `security.signin.accountGeoMode` is a global/domain/user setting: the administrator sets the instance default and can lock or force it; a user changes it under Settings, Security when not locked. `block-unusual-countries` refuses a sign-in from a country the account has not signed in from (the first sign-in is always allowed; the history is the account's own durable country list, see below); `block-foreign-countries` refuses countries outside the allowed list. Allowlisted IPs and countries (an instance-wide pair set by the administrator plus the account's own pair; the administrator can also write the account's own pair) always pass this policy and nothing else. It applies after credentials are proven, to webmail password, passkey, SSO and second-factor sign-ins, never to existing sessions, IMAP/SMTP/app-password clients or the admin console. A blocked sign-in is recorded in the authentication log (reason `account-geo-*`, with the country) and shows a generic message. Saving a policy that would block the caller's current address and country is refused (409 `signinGeographySelfLockout`). Recovery: another allowed network, an administrator editing the user's allowlist or turning the mode off for that user, or the operator setting the instance default and lock.

**Accuracy:** country detection uses an IP geolocation database that can be wrong or out of date. VPNs, mobile carriers, proxies and travel can block a legitimate sign-in or let a foreign one through. Add trusted IPs or countries to the allowlist.

<a id="account-geo-history"></a>
#### Country history for "unusual countries"

`block-unusual-countries` used to judge "usual" from the authentication log's retained rows, so it forgot as the log aged out (`logs.auth.retentionDays`, 7 days by default). It now keeps a separate, durable list per account, for `security.signin.accountGeoHistoryDays` days (default 365, 30 to 3650); a longer or shorter log retention no longer changes it.

What is stored, exactly: one row in the settings table per account, key `acct-geo:v1:<account id>`, holding a list of at most 64 entries. Each entry is an ISO 3166 country code, the time of the first and the last successful sign-in from that country, and how many such sign-ins there were. Nothing else: no IP addresses, devices, user agents or sign-in methods. When the list is full the entry with the oldest last-seen time is dropped. Entries whose last sign-in is older than the retention are ignored immediately and removed by an hourly sweep.

When it is written: only after a sign-in has completed (the geography decision allowed it and the session was kept), and only while the account's effective mode is `block-unusual-countries`. The first time the policy needs an account that has no list, the list is seeded once from the successful rows the authentication log still holds. The first sign-in of an account with an empty list is always allowed.

The user sees the list under Settings, Security, Sign-in geography ("Countries seen") and can forget it. Forgetting empties the list, keeps a marker so the log is not read again for what happened before, and the next sign-in is allowed from anywhere and starts the list again. The row is removed when the account's identity is retired. Local-network addresses have no country and are never recorded.

| Key | Default | Range | Meaning |
|---|---|---|---|
| <a id="security.trust.enabled"></a>`security.trust.enabled` | `true` | bool | Derive a trust tier per request. Off treats every anonymous request as unknown with the baseline limits. |
| <a id="security.trust.knownTtlDays"></a>`security.trust.knownTtlDays` | `30` | 1-365 | How long a successful sign-in keeps its source known. |
| <a id="security.trust.knownCapacity"></a>`security.trust.knownCapacity` | `10000` | 100-100000 | Most known sources kept in memory; the oldest is dropped first. |
| <a id="security.trust.persist"></a>`security.trust.persist` | `true` | bool | Save the known-source table to the settings store and load it at startup (see Trust tiers). Off writes and loads nothing; a copy already saved stays until it is forgotten. |
| <a id="security.trust.persistIntervalSecs"></a>`security.trust.persistIntervalSecs` | `300` | 30-3600 | Seconds between saves. A save happens only when the table changed since the last one, and once more at graceful shutdown. |
| <a id="security.trust.unknownPercent"></a>`security.trust.unknownPercent` | `100` | 10-100 | Allowance of an unknown source as a percentage of the configured limits; lower is stricter. |
| <a id="security.trust.knownPercent"></a>`security.trust.knownPercent` | `200` | 100-1000 | Allowance of a known source; 200 doubles request budgets and halves the sign-in cooldown and challenge difficulty. |
| <a id="security.trust.uaMissing"></a>`security.trust.uaMissing` | `limit` | allow, limit, block | Anonymous request with an empty user-agent. |
| <a id="security.trust.uaUncommon"></a>`security.trust.uaUncommon` | `limit` | allow, limit, block | Anonymous request with an unrecognised user-agent. |
| <a id="security.trust.uaAutomation"></a>`security.trust.uaAutomation` | `limit` | allow, limit, block | Anonymous request claiming a script, HTTP library, headless browser or generic bot. |
| <a id="security.trust.uaSearch"></a>`security.trust.uaSearch` | `limit` | allow, limit, block | Anonymous request claiming a search-engine crawler. |
| <a id="security.trust.uaAiTraining"></a>`security.trust.uaAiTraining` | `block` | allow, limit, block | Claimed AI training crawler (GPTBot, ClaudeBot, CCBot, Amazonbot, Bytespider, Google-Extended, Applebot-Extended). |
| <a id="security.trust.uaAiSearch"></a>`security.trust.uaAiSearch` | `block` | allow, limit, block | Claimed AI search crawler (OAI-SearchBot, PerplexityBot, Claude-SearchBot). |
| <a id="security.trust.uaAiUser"></a>`security.trust.uaAiUser` | `block` | allow, limit, block | Claimed user-triggered AI fetcher (ChatGPT-User, Claude-User). |
| <a id="security.trust.uaCustom"></a>`security.trust.uaCustom` | `limit` | allow, limit, block | Anonymous request matching `security.protection.userAgentPatterns`. |
| <a id="security.trust.adminGeoEnabled"></a>`security.trust.adminGeoEnabled` | `true` | bool | Unknown-country admin-discovery block. `MW_ADMIN_GEO_ENABLED=false` pins it off for recovery. |
| <a id="security.trust.adminGeoCountries"></a>`security.trust.adminGeoCountries` | `[]` | uppercase codes | Countries always treated as known, besides those of recent successful sign-ins. |
| <a id="security.trust.adminGeoLookupFailure"></a>`security.trust.adminGeoLookupFailure` | `allow` | allow, unknown | Source without a country: allow (not blocked, logged) or treat as an unknown country. |
| <a id="security.trust.adminGeoThreshold"></a>`security.trust.adminGeoThreshold` | `3` | 1-100 | Probes inside the window that start a block. |
| <a id="security.trust.adminGeoWindowSecs"></a>`security.trust.adminGeoWindowSecs` | `600` | 60-86400 | Window for the threshold. |
| <a id="security.trust.adminGeoBlockOnFirst"></a>`security.trust.adminGeoBlockOnFirst` | `false` | bool | Block on the first probe instead of the threshold. |
| <a id="security.trust.adminGeoBlockSecs"></a>`security.trust.adminGeoBlockSecs` | `3600` | 60-2592000 | First block length. |
| <a id="security.trust.adminGeoSecondBlockSecs"></a>`security.trust.adminGeoSecondBlockSecs` | `86400` | 60-2592000 | Second block length inside the repeat period. |
| <a id="security.trust.adminGeoMaxBlockSecs"></a>`security.trust.adminGeoMaxBlockSecs` | `604800` | 60-2592000 | Third and later block length. |
| <a id="security.trust.adminGeoRepeatDays"></a>`security.trust.adminGeoRepeatDays` | `30` | 1-365 | A block within this many days of the previous one is a repeat. |
| <a id="security.signin.accountGeoMode"></a>`security.signin.accountGeoMode` | `off` | off, block-unusual-countries, block-foreign-countries | Per-account sign-in country policy (global/domain/user; admin lock supported). |
| <a id="security.signin.accountGeoAllowCountries"></a>`security.signin.accountGeoAllowCountries` | `[]` | up to 50 ISO 3166 codes | The account's allowed countries. |
| <a id="security.signin.accountGeoAllowIps"></a>`security.signin.accountGeoAllowIps` | `[]` | up to 50 IPs or CIDRs | The account's allowed IP addresses and networks; a single IPv6 means its /64. |
| <a id="security.signin.accountGeoGlobalAllowCountries"></a>`security.signin.accountGeoGlobalAllowCountries` | `[]` | up to 100 ISO 3166 codes | Countries allowed for every account. |
| <a id="security.signin.accountGeoGlobalAllowIps"></a>`security.signin.accountGeoGlobalAllowIps` | `[]` | up to 100 IPs or CIDRs | Addresses and networks allowed for every account. |
| <a id="security.signin.accountGeoLookupFailure"></a>`security.signin.accountGeoLookupFailure` | `allow` | allow, deny | Source without a country while the per-account policy is on. |
| <a id="security.signin.accountGeoHistoryDays"></a>`security.signin.accountGeoHistoryDays` | `365` | 30-3650 | Days a country stays in an account's own sign-in history (`block-unusual-countries`). Older entries are ignored and removed. |
| <a id="security.geo.noDatabase"></a>`security.geo.noDatabase` | `allow` | allow, treat-as-unknown, block | Shared default when no country database is loaded. |
| <a id="security.geo.notFound"></a>`security.geo.notFound` | `allow` | allow, treat-as-unknown, block | Shared default when the database has no entry for the address. |
| <a id="security.signin.geoNoDatabase"></a>`security.signin.geoNoDatabase` | `block` | allow, block | Instance sign-in country policy (`geoScope`) when no database is loaded. |
| <a id="security.geo.sessionLookupFailure"></a>`security.geo.sessionLookupFailure` | `ignore` | ignore, alert | Session alerts when a request's country cannot be told. |

<a id="geo-lookup-failure"></a>
### When a country cannot be determined

Every country feature needs the local country database (Security, Country database). "No country" has two different causes, which the logs and the console keep apart:

* **no database** (`no-database`): none is loaded or readable. The features below are effectively inert; the Security centre shows a warning card with a link to install one whenever a country feature is switched on while no database is loaded, and the Country database page shows its build date and age.
* **not found** (`not-found`): a database is loaded and has no entry for the address (unallocated or reserved ranges).

A third case is never a failure: **local network** addresses (loopback, private, link-local, 100.64.0.0/10, unique-local IPv6) have no country by nature. They are allowed by every country feature and never blocked, counted or alerted on.

Values: `allow` (carry on, logged), `treat-as-unknown` (handle the source as a country on no list) and `block` (refuse a sign-in). `block` exists only where refusing is safe, which is a new sign-in; the admin-console rule treats it as `treat-as-unknown`, and no setting blocks an already-authenticated session for this reason except the older, explicit `security.signin.geoEnforceSessions`. Defaults keep today's behaviour.

| Feature | No database | Not found |
|---|---|---|
| Per-account sign-in policy | stricter of `security.geo.noDatabase` and `security.signin.accountGeoLookupFailure` (`deny` = `block`) | stricter of `security.geo.notFound` and `security.signin.accountGeoLookupFailure` |
| Instance sign-in policy (`security.signin.geoScope`) | `security.signin.geoNoDatabase` (`block` = fail closed, the default) | `security.signin.geoUnknown` |
| Admin-console unknown-country block | stricter of `security.geo.noDatabase` (capped at `treat-as-unknown`) and `security.trust.adminGeoLookupFailure` (`unknown`) | the same with `security.geo.notFound` |
| Session country-change alerts and travel checks | `security.geo.sessionLookupFailure` (`ignore` or `alert`, one alert per session per day) | same |
| Trust tier known countries, adaptive protection country rules | no country is recorded; nothing is blocked | same |

Precedence: a feature's own key and the shared `security.geo.*` default both apply and the stricter value wins (`allow` < `treat-as-unknown` < `block`). Treat-as-unknown in the per-account policy means: `block-foreign-countries` refuses the sign-in, and `block-unusual-countries` refuses it unless the account has no history (the first sign-in is still allowed).

**Warning:** `block` for the sign-in policies refuses every non-local sign-in whose country cannot be told. With no database loaded that is every sign-in. Install and check the database before choosing it, and keep an allowlisted IP for the administrator.

### Password spray: too many usernames from one source

A person who mistypes a password fails with one or two usernames. A password-spraying client tries one common password against many usernames, so the number of different usernames one source has failed with is a better signal than the number of failures. The tracker counts distinct usernames per source (IPv4 /32, IPv6 /64) inside `security.spray.windowSecs`. The source is the client address resolved through the trusted-proxy rules only: a spoofed `X-Forwarded-For` from an untrusted peer is ignored. Only failures with the reasons auth-rejected and credential-rejected count; authenticated requests, successes and second-factor refusals are exempt. A successful sign-in never counts and removes that username from the source's set, so users behind an office or campus NAT who succeed do not add up.

Defaults: 10 distinct usernames force the proof-of-work challenge for that source, 15 add extra pacing (`security.spray.paceSecs`, 10 seconds between attempts), 25 lock the source for `security.spray.lockSecs` (900) through the existing sign-in lock. A known source (a recent successful sign-in from the same address) has all three thresholds multiplied by `security.spray.knownMultiplier` (3). When the server starts requiring the challenge for a source that was not asked before, the public config turns `antiBot` on for that source and the refused request is answered 403 with `signinChallengeRequired`; the sign-in page then loads the challenge.

Privacy: only a keyed HMAC-SHA256 hash (16 bytes) of the lower-cased username is kept, in memory only, at most 2048 names per source and `security.spray.capacity` sources (the oldest are dropped). Usernames and passwords are never stored. An administrator unlocking a source resets its count.

| Key | Default | Range | Meaning |
| --- | --- | --- | --- |
| <a id="security.spray.enabled"></a>`security.spray.enabled` | `true` | bool | Count the distinct usernames one source fails with. |
| <a id="security.spray.windowSecs"></a>`security.spray.windowSecs` | `900` | 60-86400 | Window in which a source's failures are counted. |
| <a id="security.spray.challengeAt"></a>`security.spray.challengeAt` | `10` | 2-1000 | Distinct failing usernames that force the proof-of-work challenge for the source. |
| <a id="security.spray.paceAt"></a>`security.spray.paceAt` | `15` | 2-1000 | Distinct failing usernames that add extra pacing. |
| <a id="security.spray.lockAt"></a>`security.spray.lockAt` | `25` | 2-1000 | Distinct failing usernames that lock the source through the existing sign-in lock. |
| <a id="security.spray.paceSecs"></a>`security.spray.paceSecs` | `10` | 1-300 | Seconds between attempts at or above the pacing step. |
| <a id="security.spray.lockSecs"></a>`security.spray.lockSecs` | `900` | 30-86400 | Length of the temporary lock. |
| <a id="security.spray.knownMultiplier"></a>`security.spray.knownMultiplier` | `3` | 1-20 | Multiplier on the three thresholds for a known source (recent successful sign-in). |
| <a id="security.spray.capacity"></a>`security.spray.capacity` | `20000` | 100-200000 | Most sources tracked in memory; the oldest are dropped first. |

### Instance-wide spike detection (heightened mode)

Spike detection watches the whole instance rather than one source. Failed sign-ins are counted in five-minute buckets with five metrics: failed sign-ins, distinct failing sources, distinct failing usernames, failure ratio, and a distributed-spray signature (sources that each failed with at least 3 usernames in the bucket). The baseline is a slow exponentially weighted average, kept globally and per hour-of-week slot (168 slots), with time constant `security.spike.adaptHours` (168). The effective baseline is the larger of the slot and the global value until a slot has 36 visits, then the slot (not below 25% of the global value).

A signal needs the observed value to reach `security.spike.multiplierPercent` (400 = 4x) of the baseline and an absolute floor: 20 failures per minute (100 per bucket), 10 sources, 30 usernames per bucket; the ratio signal needs the failure floor and at least 60%. `security.spike.signalsRequired` (2) signals must trigger together; the distributed-spray signature counts as two. During `security.spike.warmupHours` (48) the baseline only learns and nothing is enforced.

A spike bucket and every bucket during heightened mode never update the baseline; each baseline update is clamped to 1.5x the baseline plus half the floor, and a gap over one hour (a restart) is skipped, not filled with zeros. Heightened mode lasts `security.spike.durationSecs` (1800), extended while signals stay above half their trigger level, then expires.

Heightened mode never locks the instance. Unknown-tier anonymous sign-ins need the proof-of-work challenge (`security.spike.forceChallenge`) and wait `security.spike.paceFactorPercent` (300) percent of the usual cooldown. Authenticated users and known sources are unaffected. The audit event `signin-spike-heightened` is the administrator alert, and Security, Adaptive protection shows the state read-only (`GET /security/session-watch`, capability `security.read`).

The baseline is stored as one fixed-size JSON document (about 30 KB) under the settings key `spike:baseline:v1`, written at most once per bucket. The current-bucket sets are in memory, use 64-bit keyed hashes and are bounded.

| Key | Default | Range | Meaning |
| --- | --- | --- | --- |
| <a id="security.spike.enabled"></a>`security.spike.enabled` | `true` | bool | Learn a failed-sign-in baseline and enter heightened mode on an instance-wide spike. Never locks the instance. |
| <a id="security.spike.multiplierPercent"></a>`security.spike.multiplierPercent` | `400` | 150-2000 | A signal counts at this percentage of its baseline (400 = four times). |
| <a id="security.spike.minFailuresPerMin"></a>`security.spike.minFailuresPerMin` | `20` | 1-10000 | Floor for the failure-rate signal (failed sign-ins per minute). |
| <a id="security.spike.minSources"></a>`security.spike.minSources` | `10` | 2-10000 | Floor for distinct failing sources per five-minute bucket. |
| <a id="security.spike.minUsernames"></a>`security.spike.minUsernames` | `30` | 2-10000 | Floor for distinct failing usernames per five-minute bucket. |
| <a id="security.spike.signalsRequired"></a>`security.spike.signalsRequired` | `2` | 1-4 | Signals that must trigger together; the distributed-spray signature counts as two. |
| <a id="security.spike.warmupHours"></a>`security.spike.warmupHours` | `48` | 1-720 | Hours of traffic observed before any enforcement. |
| <a id="security.spike.adaptHours"></a>`security.spike.adaptHours` | `168` | 6-2160 | Time constant of the baseline. |
| <a id="security.spike.durationSecs"></a>`security.spike.durationSecs` | `1800` | 60-86400 | Length of heightened mode, extended while signals stay above half their trigger. |
| <a id="security.spike.forceChallenge"></a>`security.spike.forceChallenge` | `true` | bool | Require the proof-of-work challenge for anonymous sign-ins from unknown sources in heightened mode. |
| <a id="security.spike.paceFactorPercent"></a>`security.spike.paceFactorPercent` | `300` | 100-1000 | Sign-in wait for unknown sources in heightened mode, as a percentage of the usual wait. |

### Session binding

Each web cookie session is bound to keyed hashes of the client that created it: browser family and browser major version number (the number is stored plain so that automatic updates are tolerated), OS family, the `Sec-CH-UA-Platform` hint, optionally the primary `Accept-Language` tag, the country (on by default) and the network (IPv4 /16, IPv6 /48; off by default because mobile networks move). There is no ASN binding; the app has no ASN database. Only web cookie sessions are bound: webmail `/api` and `/jmap`, and the admin console. Requests with an `Authorization` header (native shells), `x-api-key`, IMAP, SMTP, DAV, MCP and OAuth protocol clients, and the sign-in routes are exempt. The record is created at the first cookie request of a session (the first call after the login response), also for sessions that predate the feature.

Tolerant (default): any browser update is accepted; a version going back more than 2 majors, or a different browser family or OS, is a mismatch; the soft attributes (client-hints platform, language, country, network) mismatch only when two of them differ. Strict: any soft difference is a mismatch and a version more than 3 ahead is a mismatch. Missing headers never count. A blank User-Agent is recorded only and never acted on.

The action is set per surface: `off`, `detect` (record only), `step-up` or `revoke`. Defaults are `step-up` for webmail and `revoke` for the admin console. Step-up flags the session: every request except sign-out and the step-up route gets 403 `step_up_required` until the user confirms the password and second factor in the existing step-up dialog.

| Key | Default | Range | Meaning |
| --- | --- | --- | --- |
| <a id="security.sessionBinding.enabled"></a>`security.sessionBinding.enabled` | `true` | bool | Bind each browser session to keyed hashes of its client attributes. |
| <a id="security.sessionBinding.attributes"></a>`security.sessionBinding.attributes` | `["browser","os","hints","country"]` | list | Attributes bound: browser, os, hints, language, country, network (IPv4 /16, IPv6 /48). Unknown names are ignored. |
| <a id="security.sessionBinding.strictness"></a>`security.sessionBinding.strictness` | `tolerant` | tolerant, strict | Tolerant accepts browser updates and one soft change; strict flags any soft difference. |
| <a id="security.sessionBinding.webmailAction"></a>`security.sessionBinding.webmailAction` | `step-up` | off, detect, step-up, revoke | Action on a webmail session mismatch. |
| <a id="security.sessionBinding.adminAction"></a>`security.sessionBinding.adminAction` | `revoke` | off, detect, step-up, revoke | Action on an admin console session mismatch. |

### Session theft detection and country-change alerts

Three findings are checked per cookie session. Rotated-id reuse: after `/api/session/rotate` the old id leaves a hash marker; presenting it again after `security.theft.rotationGraceSecs` (30) revokes the whole session family and alerts, regardless of the configured action. Impossible travel: a country change within `security.theft.minTravelSecs` (3600). Concurrent use: A, B, A alternation within 5 minutes, by country (default) or optionally by network. The response to the last two is set per surface: `alert`, `step-up` or `revoke`; defaults `step-up` for webmail and `revoke` for the admin console.

Retention: `security.theft.retentionDays` (15). Per-session records and rotation markers (hashes, country codes, times) are swept hourly. They are stored in the settings table under `sess-sec:v1:<sha256>`, `sess-rot:v1:<sha256>`, notices under `sec-notice:v1:...` and a ring of the 50 most recent alerts under `sec-alerts:v1:recent`.

Country-change alert: on by default for every session. It alerts once per new country per session (the start country counts as seen), with `security.alerts.countryChangeCooldownSecs` (300) between alerts. An administrator can change the default (`security.alerts.countryChange` is global/domain/user), lock it, or switch it off globally with `security.alerts.countryChangeEnabled`. A user's own "off" is honoured only if `security.alerts.countryChangeUserMute` is true; otherwise the server keeps alerting. Country detection may be inaccurate; VPNs and mobile networks can cause false changes.

Channels (`security.alerts.channels`, a user choice that can be locked): `app`, `email` or `both`. Email means a message delivered into the user's own mailbox through the same path as other security alerts (engine APPEND or the guarded JMAP import). No outbound SMTP is used. If the account has no delivery session the alert stays in the app and the audit log. `security.alerts.emailEnabled` is the administrator switch. Sensitivity (`security.alerts.sensitivity`, a user choice): `low` reports only location findings and never steps up or revokes, halves the travel interval and turns concurrent use off; `normal` follows the configured actions; `high` doubles the travel interval and also checks concurrent networks.

In-app notices: at most 20 per account, kept `security.alerts.noticeRetentionDays` (30). The audit event `session-security-alert` carries the reason, the country pair and the short session id, never the cookie. The user sees notices under Settings, Security, Sessions and can dismiss them or sign out everywhere.

| Key | Default | Range | Meaning |
| --- | --- | --- | --- |
| <a id="security.theft.enabled"></a>`security.theft.enabled` | `true` | bool | Session theft detection: rotated-id reuse, impossible travel, concurrent use. |
| <a id="security.theft.rotationGraceSecs"></a>`security.theft.rotationGraceSecs` | `30` | 0-600 | How long a rotated id may still arrive (an in-flight request) before it counts as reuse. |
| <a id="security.theft.minTravelSecs"></a>`security.theft.minTravelSecs` | `3600` | 60-172800 | A country change sooner than this is impossible travel. |
| <a id="security.theft.concurrentUse"></a>`security.theft.concurrentUse` | `country` | off, country, network | Flag alternating use (A, B, A) from different countries or networks within five minutes. |
| <a id="security.theft.webmailAction"></a>`security.theft.webmailAction` | `step-up` | alert, step-up, revoke | Response on a webmail session. Rotated-id reuse always revokes. |
| <a id="security.theft.adminAction"></a>`security.theft.adminAction` | `revoke` | alert, step-up, revoke | Response on an admin console session. |
| <a id="security.theft.retentionDays"></a>`security.theft.retentionDays` | `15` | 1-60 | How long per-session records and rotation markers are kept; at least the longest session lifetime. |
| Key | Default | Range | Meaning |
| --- | --- | --- | --- |
| <a id="security.alerts.countryChange"></a>`security.alerts.countryChange` | `true` | bool | Country-change alert (global/domain/user; admin lock supported). Country detection may be inaccurate; VPNs and mobile networks can cause false changes. |
| <a id="security.alerts.countryChangeEnabled"></a>`security.alerts.countryChangeEnabled` | `true` | bool | Global switch; off disables the alert for everyone. |
| <a id="security.alerts.countryChangeCooldownSecs"></a>`security.alerts.countryChangeCooldownSecs` | `300` | 0-86400 | Minimum time between country-change alerts of one session. |
| <a id="security.alerts.countryChangeUserMute"></a>`security.alerts.countryChangeUserMute` | `false` | bool | Whether a user's own "off" is honoured. |
| <a id="security.alerts.channels"></a>`security.alerts.channels` | `both` | app, email, both | Delivery of session security warnings (global/domain/user; lockable). |
| <a id="security.alerts.emailEnabled"></a>`security.alerts.emailEnabled` | `true` | bool | Allow delivery into the user's own mailbox. |
| <a id="security.alerts.sensitivity"></a>`security.alerts.sensitivity` | `normal` | low, normal, high | Per-user sensitivity (global/domain/user; lockable). |
| <a id="security.alerts.noticeRetentionDays"></a>`security.alerts.noticeRetentionDays` | `30` | 1-365 | How long in-app notices are kept (at most 20 per account). |

Restart behaviour: mailbox idle and absolute timeouts and rotation state were already persisted (`sessions.created_at` and `activity_at`; `rotate_session_security` preserves them; admin sessions use `admin_sessions.expires_at` and `absolute_expires_at`). The in-process `SessionGuard` in `hardening.rs` is a secondary clock that re-seeds leniently after a restart and never overrides the persisted clock. Step-up proofs are in memory and are dropped on restart; the user confirms again.


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

## Diagnostics budget (`rateLimit.*`)

Diagnostic runs (`POST {console}/diag/run`) and backend checks (`POST {console}/diag/jobs`) are charged to the signed-in admin session that starts them, in a bucket keyed by the session and not by its address, so admins behind one office address do not share it. Starting a backend check while the same session's check is still running returns the running check and is not charged. A refusal is `429` with `Retry-After` and `retryAfterSecs`. Sign-in, step-up, password changes and linked-mailbox renewals keep their own strict limiters and are not affected.

These keys are written only through `PUT {console}/rate-limits` (security policy permission, step-up confirmation, audited as `rate-limits-changed`); the generic settings routes refuse them. Values are per replica and reset on restart.

<a id="rateLimit.diagnosticsBurst"></a>

`rateLimit.diagnosticsBurst` (default `10`): Diagnostic runs and backend checks an admin session may start in a row. Starting a check that is already running returns that check instead of counting again.

<a id="rateLimit.diagnosticsPerMinute"></a>

`rateLimit.diagnosticsPerMinute` (default `10`): Diagnostic runs and backend checks per minute an admin session regains.

## Leeway for signed-in clients (`ratelimit.*`)

Concurrency gates are per signed-in account, with a process ceiling behind them, and they queue for `ratelimit.queueMs` before refusing. A refusal is `429` (or `503` for a saturated image proxy) with `Retry-After` and `RateLimit-Limit`, `RateLimit-Remaining` and `RateLimit-Reset`. Every key is global; the ones marked domain also accept a per-domain value, resolved from the domain of the signed-in address. The values apply within about fifteen seconds. See [Request protection limiters](request-protection.md) for every limiter and what it is keyed by.

<a id="ratelimit.queueMs"></a>

`ratelimit.queueMs` (default `3000`): Longest a request waits for a free slot (JMAP concurrency, image decoders, preview workers, delivery-status parsing) before it is refused with `429`. Zero refuses at once. Global and per domain.

<a id="ratelimit.jmap.perAccountConcurrent"></a>

`ratelimit.jmap.perAccountConcurrent` (default `8`): JMAP requests one signed-in account may run at once, across its tabs and devices; never shared with another account or with other users behind the same address. Not below 4, the value the session resource advertises as `maxConcurrentRequests`. Global and per domain.

<a id="ratelimit.jmap.globalConcurrent"></a>

`ratelimit.jmap.globalConcurrent` (default `256`): JMAP requests the process runs at once, all accounts together. Global only.

<a id="ratelimit.session.burst"></a>

`ratelimit.session.burst` (default `60`): Client timing reports one session may send in a row. Keyed by session, never by address. Global and per domain.

<a id="ratelimit.session.perMinute"></a>

`ratelimit.session.perMinute` (default `30`): Client timing reports per minute a session regains. Global and per domain.

<a id="ratelimit.anon.burst"></a>

`ratelimit.anon.burst` (default `60`): Requests one address may make to the anonymous cookie-consent routes in a row. The sign-in routes and the per-address request-protection budget are not affected. Global only.

<a id="ratelimit.anon.perMinute"></a>

`ratelimit.anon.perMinute` (default `30`): Requests per minute an address regains on those routes. Global only.

<a id="ratelimit.trustedMultiplier"></a>

`ratelimit.trustedMultiplier` (default `150`): Percentage of `security.protection.accountRequests` a signed-in mailbox gets when the request comes from a source with a successful sign-in on record (`security.trust.knownTtlDays`). 100 keeps the base allowance. Applies only while request protection enforces the account budget; it never relaxes the anonymous, administrator or user-agent budgets or an active block. Global only.

## Aggregate and adaptive request protection

State is bounded to 20,000 subjects and 10,000 decision events. Expired state is excluded from enforcement and views immediately. A lifecycle-bound maintenance task removes at most 128 expired subjects and 128 expired events per minute, with additional cleanup during administrative reads and new-subject admission; physical removal can therefore lag logical expiry during a backlog. Empty OFF deployments perform no counter or maintenance writes. Decision events expire logically after 30 days.

<a id="security.protection.rateMode"></a>

`security.protection.rateMode` (default `"off"`): Off by default. Observe records budget exhaustion without refusal; enforce returns a temporary 429. Existing protocol concurrency limits remain independent.

<a id="security.protection.adaptiveMode"></a>

`security.protection.adaptiveMode` (default `"off"`): Off by default. Observe records explainable typed anomalies and proposed actions; enforce applies bounded temporary throttles or blocks. This is a deterministic counter policy, not an AI classifier.

<a id="security.protection.clientRequests"></a>

`security.protection.clientRequests` (default `600`): Requests per window from one source address that is not signed in, and every sign-in or identity-confirmation request, across replicas. A request with a valid mailbox or admin session uses its own allowance instead. Zero disables this dimension; the source address comes only from the trusted proxy resolver.

<a id="security.protection.accountRequests"></a>

`security.protection.accountRequests` (default `1800`): Requests per window for one signed-in mailbox, across its sessions, devices and addresses and across replicas. Kept above the anonymous allowance so signed-in use has more room than an unknown address. Zero disables this dimension.

<a id="security.protection.adminRequests"></a>

`security.protection.adminRequests` (default `1200`): Requests per window for one signed-in administrator, across replicas. Kept above the anonymous allowance. Zero disables this dimension.

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

<a id="security.adminProbe"></a>
### Admin-probe protection (`security.adminProbe.*`)

Counts requests that look for the admin console, per client network (an IPv4
address or an IPv6 /64, resolved through the trusted-proxy rules), and slows
then blocks a source that keeps going. These keys are changed through
`PUT {console}/security/admin-probe/policy` (step-up `banManage`), not the
generic settings route. See [Admin-probe protection](../security/admin-panel.md#admin-probe-protection).

| Key | Default | Range | Meaning |
|---|---|---|---|
| <a id="security.adminProbe.enabled"></a>`security.adminProbe.enabled` | `true` | bool | On or off. `MW_ADMIN_PROBE_ENABLED` pins it. |
| <a id="security.adminProbe.graceAllowance"></a>`security.adminProbe.graceAllowance` | `10` | 0–1000 | Misses per window that are ignored; the score drains at this many points per window. |
| <a id="security.adminProbe.windowSecs"></a>`security.adminProbe.windowSecs` | `600` | 60–86400 | The grace window. |
| <a id="security.adminProbe.blockAfter"></a>`security.adminProbe.blockAfter` | `10` | 1–1000 | Misses above the grace allowance that start a block. |
| <a id="security.adminProbe.tarpitDelayMs"></a>`security.adminProbe.tarpitDelayMs` | `1500` | 0–10000 | Delay on each counted miss above the grace allowance (0 = off). |
| <a id="security.adminProbe.blockSecs"></a>`security.adminProbe.blockSecs` | `900` | 60–604800 | First block. |
| <a id="security.adminProbe.maxBlockSecs"></a>`security.adminProbe.maxBlockSecs` | `86400` | 60–2592000 | Cap for repeat blocks, which double each time. Must be at least `blockSecs`. |
| <a id="security.adminProbe.repeatWindowSecs"></a>`security.adminProbe.repeatWindowSecs` | `86400` | 0–2592000 | A block starting this soon after the previous one ended is a repeat. |
| <a id="security.adminProbe.blockStatus"></a>`security.adminProbe.blockStatus` | `"429"` | `429`, `403` | Status of the blocked answer (with `Retry-After`). |
| <a id="security.adminProbe.signals"></a>`security.adminProbe.signals` | all four | `probe-path`, `slug-guess`, `admin-signin-failed`, `admin-unauthenticated` | Which misses count. |
| <a id="security.adminProbe.probePaths"></a>`security.adminProbe.probePaths` | 20 paths (`/admin`, `/wp-admin`, `/.env`, `/manager/html`, …) | up to 100, each starting with `/` | Paths scanners ask for; matched as whole segments, any case. |
| <a id="security.adminProbe.allowlist"></a>`security.adminProbe.allowlist` | `[]` | up to 100 IPs/CIDRs | Never counted or blocked. |
| <a id="security.adminProbe.auditIp"></a>`security.adminProbe.auditIp` | `"masked"` | `masked`, `full` | Source in the audit log and the API: IPv4 /24 and IPv6 /48, or the address. |
| <a id="security.adminProbe.fail2banLog"></a>`security.adminProbe.fail2banLog` | `true` | bool | Write the `mailwoman[admin-probe]` line on each block. |

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

<a id="privacy.remoteImagesToast"></a>
### Loading-images notification (`privacy.remoteImagesToast`)

Global/domain/user boolean, default true, with administrator locks. When a message's remote images load through the privacy proxy (after Show images, an existing sender or domain grant, or the `proxy` mode), the reader shows one notification for that message: "Loading images…", replaced in place by "Images loaded" or by the number of images that could not load with a Retry action. A load that is displayed within about 300 ms shows nothing. Switching message closes it. It is presentation only: which images load, and through what path, is still decided by `privacy.remoteContent` and the image grants. Users can turn it off in Settings › Privacy unless an administrator locks it.

<a id="mail.resumeTabs"></a>
### Resume workspace tabs

`mail.resumeTabs` defaults to true and supports global, domain and user values with administrator locks. The browser stores bounded tab references, order, pins and active tab under both the wire account ID and verified storage authority, never message bodies or attachment URLs. Reopening resolves references under current authorization. Missing storage authority disables private persistence and automatic restoration; legacy raw-account-only records are not migrated automatically. Disabling the setting clears this workspace record without deleting the separate account-owned draft store.

<a id="mail.folderCounts"></a>
### Folder count display (`mail.folderCounts`)

Global/domain/user choice, default `"both"`. Select `none`, `unread`, `unreadIfAny`, `total`, or `both` for folder badges. `unread` shows the unread count, including 0; `unreadIfAny` shows it only while the folder has unread mail, and no bubble at 0. The Focused/Other inbox tabs follow the same choice. Users can change this under Settings > Folders unless an administrator locks the inherited value. Hiding counts changes their presentation only; it does not alter read state or mailbox totals. The value is resolved for the sign-in account, so every linked mailbox shows the same counts.

<a id="mail.folderCountScope"></a>
### Folders with counts (`mail.folderCountScope`)

Global/domain/user choice, default `"all"`. `all` keeps a count on every folder; `main` limits counts to folders with a role: Inbox, Drafts, Sent, Archive, Junk, Trash, Outbox and any other special-use folder the server reports. Custom folders and their subfolders then show no count. Users can change this under Settings > Folders unless locked. Resolved for the sign-in account, like `mail.folderCounts`.

<a id="mail.compactFolderBadges"></a>
### Secondary folder badge size (`mail.compactFolderBadges`)

Global/domain/user boolean, default `true`. Uses smaller badges outside Inbox. Users can inherit or override this in Settings > Folders unless locked by an administrator. Folder names remain left-aligned and long names truncate without displacing the count.

<a id="mail.focusedInbox"></a>
### Focused inbox (`mail.focusedInbox`)

Global/domain/user boolean, default `false`. When on, Inbox shows two tabs above the message list, Focused and Other; messages from newsletters and automated senders (no-reply, notification, marketing and similar addresses) go to Other. Other folders and search results always list every message. The Inbox header has no switch of its own: users turn it on or off in Settings > Folders and sidebar, and an administrator can set the deployment or domain default and lock it, which disables that control. Resolved for the sign-in account, like `mail.folderCounts`, so every linked mailbox shows the same tabs.

<a id="mail.accountUnreadBadge"></a>
### Unread count beside the mailbox name (`mail.accountUnreadBadge`)

Global/domain/user boolean, default `false`. When on, the sidebar head shows the Inbox unread count next to the signed-in address or nickname (the plain address, and the mailbox picker's button). The per-mailbox counts inside the mailbox menu are shown either way. Users can change this under Settings > Folders unless locked. Resolved for the sign-in account, like `mail.folderCounts`.

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

<a id="mail.brandAvatars"></a>
### Brand logos beside verified senders (`mail.brandAvatars`)

Enabled by default, with global/domain/user overrides and administrator locks.
The reader shows a bundled brand logo beside the sender when the sender's domain belongs to a known brand and the message passed DKIM and an aligned DMARC check
for a domain of that same brand. Messages that fail, lack those checks or come from
an unknown domain keep the normal avatar; the message list, which carries no check result, never shows one.
Domain matching uses whole registrable-domain suffixes, so `paypal.com.evil.io` or `evilpaypal.com` never match. Logos are bundled with the app and nothing is requested
from the sender or a logo service. See `docs/brand-logos.md`.

<a id="contacts.favoritesFirst"></a>
### Favorites first in suggestions (`contacts.favoritesFirst`)

Enabled by default, with global/domain/user overrides and administrator locks.
When on, recipient suggestions in Compose list contacts marked as favorites ahead
of other matches; within each group the usual ranking applies. It only changes
the order: no suggestion is hidden, and favorites are a flag on the contact
(`CATEGORIES:favorite` in its vCard), stored per address book.

<a id="mail.draftSaveMode"></a>
<a id="mail.draftDebounceMs"></a>
<a id="mail.draftIntervalSecs"></a>

Account draft saving supports global, domain and user defaults and administrator locks. `mail.draftSaveMode` defaults to `realtime` (after a typing pause), with `periodic` and `manual` alternatives. `mail.draftDebounceMs` defaults to 1000 and accepts 500–10000 milliseconds; `mail.draftIntervalSecs` defaults to 30 and accepts 5–3600 seconds. Manual mode disables scheduled upstream writes. These settings do not grant backend capabilities: a mapped, writable native IMAP Drafts folder and confirmed APPENDUID are required. Unsupported backends remain explicit. Interrupted or unconfirmed saves retain a durable uncertain claim and are never automatically repeated.

<a id="mail.showFrom"></a>

`mail.showFrom` is a global/domain/user boolean, default false. It controls the initial visibility of the composer From field. A per-compose ribbon action can reveal the field; visibility never grants permission to send as an address. Typed one-message addresses must pass current native or upstream sender authorization and are not saved as identities.

<a id="mail.replyFromMode"></a>

`mail.replyFromMode` is a global/domain/user enum, default `match`. When replying, replying to all or forwarding, `match` sends from the identity whose address the original was addressed to (To, then Cc; case-insensitive, and `user+tag@host` matches `user@host`; an exact address beats a plus-tag match) and falls back to the default identity. `default` always uses the default identity, `ask` matches the same way but lists the other candidates for a one-click change, and `keep` leaves the composer identity unchanged. The choice never grants permission to send as an address.

<a id="mail.replyFrom.headerPriority"></a>

`mail.replyFrom.headerPriority` is a global/domain/user list of header names, default `Delivered-To`, `X-Original-To`, `To`, `Cc`. It is read when `mail.replyFromMode` is `match` or `ask`: the headers are tried in this order, and the first one that names an address of one of the user's identities decides which identity the reply is sent from. Accepted names are `Delivered-To`, `X-Original-To`, `Envelope-To`, `X-Forwarded-To`, `Resent-To`, `To` and `Cc` (case-insensitive; unknown names and repeats are ignored, an empty or invalid list means the default). A header that is not listed is not read. Within one header the match order is: the exact address; plus-addressing (`user+tag@host` reaches `user@host`, and the reverse); an identity address written with `*` (`*@host` catches every address at that host, `user+*@host` any tag, `user@*.host` any subdomain); and, for the delivery headers only (never `To` or `Cc`, which also list other people), the same domain or a parent domain of the received address, which is how mail delivered to a catch-all alias is answered from the identity at that domain. The composer's status line names the header that matched ("From me@example.org (matched Delivered-To)"). The delivery headers are read from the stored message when a reply is opened, so they work for mail of IMAP, POP3 and JMAP accounts alike. The choice never grants permission to send as an address; a header cannot select an identity the account does not have.

<a id="privacy.stripMailerHeaders"></a>
### Outgoing mailer header privacy (`privacy.stripMailerHeaders`)

Defaults to true; global, domain and account overrides are supported, with administrator locks. Native SMTP/EWS submissions remove top-level X-Mailer, User-Agent and X-MimeOLE. Bodies, attachments and ARF report payloads are unchanged; upstream JMAP proxy sends are outside this boundary. See [outgoing header privacy](../deploy/outgoing-header-privacy.md) for pre-signed message conflicts and relay limitations.

<a id="security.protocol.framingMode"></a>
<a id="security.protocol.hostMode"></a>
<a id="security.protocol.hostPinningMode"></a>
<a id="security.protocol.pathControlMode"></a>
<a id="security.protocol.targetMode"></a>
<a id="security.protocol.pathLengthMode"></a>
<a id="security.protocol.methodMode"></a>
<a id="security.protocol.duplicateHeadersMode"></a>
<a id="security.protocol.forwardedMode"></a>
<a id="security.protocol.headerHygieneMode"></a>
<a id="security.protocol.bodyMode"></a>
<a id="security.protocol.maxPathBytes"></a>
<a id="security.protocol.maxQueryBytes"></a>
<a id="security.protocol.maxHeaderCount"></a>
<a id="security.protocol.maxHeaderValueBytes"></a>
<a id="security.protocol.maxCookieBytes"></a>
<a id="security.protocol.maxBodyBytes"></a>
<a id="security.protocol.allowedMethods"></a>
<a id="security.protocol.allowedHosts"></a>
<a id="security.protocol.exemptPrefixes"></a>

An early middleware checks the method, version, request target and headers of every request before routing, sign-in or body parsing, and classes what it finds. Each class `*Mode` is `off`, `detect` (log, count, feed adaptive protection, allow) or `enforce` (reject with a minimal 400, 405, 413, 414, 417 or 431 that repeats nothing the client sent; `Connection: close` after framing and Host rejects). Defaults: `framingMode`, `hostMode` and `pathControlMode` enforce; every heuristic class detects. `/healthz` and `/readyz` are never refused. Counters per class and mode are shown in the Adaptive protection page and `GET /bans/protection`. `exemptPrefixes` switches the heuristic classes off for whole path segments under a prefix; framing and path-control findings are never exempted. The guard sees only what the HTTP server hands the application: see [protocol guard](protocol-guard.md) for what the framework already refuses, what a front proxy must do, and the deferred custom-rule and virtual-patching plan.

<a id="security.health.mode"></a>
<a id="security.health.networks"></a>

`/healthz` and `/readyz` answer only to internal clients by default; anyone else gets the same 404 as an unknown path. `security.health.mode` is `internal-only` (loopback, private, link-local and unique-local client addresses; the default), `allowlist` (those plus `security.health.networks`, for example an external uptime monitor) or `open`. A request carrying the token in `MW_HEALTH_TOKEN` (header `X-Health-Token` or query parameter `token`) is admitted from any address in every mode; the token is an environment variable and is never stored or shown. A request that is not admitted is also not exempt from source bans, the protocol guard, admin-probe or request protection: the exemption those layers give the health paths applies only to admitted requests. The client address is the one resolved through `MW_FORWARDED_MODE` and `MW_TRUSTED_PROXIES`. **Behind a reverse proxy that is not configured that way, every request appears to come from the proxy, which is normally a private address, so the default rule admits the whole internet.** Configure the trusted proxy (see [reverse proxy](../deploy/reverse-proxy.md)) or set a token and the allowlist mode. Kubernetes kubelet probes come from the node address and Docker healthchecks from loopback, both internal.

<a id="security.protection.userAgentMode"></a>
<a id="security.protection.userAgentBlockBots"></a>
<a id="security.protection.userAgentCategories"></a>
<a id="security.protection.userAgentWeakRequests"></a>
<a id="security.protection.userAgentCrawlerRequests"></a>
<a id="security.protection.userAgentWindowSecs"></a>
<a id="security.protection.userAgentMaxAction"></a>
<a id="security.protection.userAgentScope"></a>
<a id="security.protection.userAgentAdaptive"></a>
<a id="security.protection.userAgentPatterns"></a>
<a id="security.protection.userAgentExceptions"></a>

User-agent protection is independently enabled (`userAgentMode: enforce`) even when the older rate/adaptive modes are off. It classifies self-claimed headers, not verified bot identities; see [crawler identification](crawler-identification.md). `userAgentBlockBots` defaults to `true`: selected recognized search, AI, automation and custom bot classes receive a temporary HTTP 429 block on their first request for `security.protection.blockSecs`. Missing or unfamiliar headers retain request budgets, and a single such header does not cause immediate refusal. Known browser and mail-client families have no extra budget. Authenticated sessions are excluded unless `userAgentScope` is `all`; sign-in requests remain anonymous. Public documents are covered; static resources and narrow recovery controls remain available.

`userAgentCategories` selects missing, uncommon, automation, search, ai-training, ai-search and ai-user. Setting `userAgentBlockBots: false` restores the previous request-budget behavior for bot classes: 600 requests per 60 seconds for missing/uncommon/automation/custom claims and 300 for search/AI claims by default. `userAgentWeakRequests`, `userAgentCrawlerRequests` and `userAgentWindowSecs` tune those allowances. `userAgentAdaptive` combines weak/repeated-request evidence with existing authoritative anomaly signals and rolling windows; missing or unfamiliar headers alone are not anomalies. `userAgentMaxAction` defaults to throttle and caps only weak/repeated escalation, not immediate bot blocks. Rate exhaustion and bot blocks use HTTP 429 with a temporary retry duration. Observe records decisions without refusing; off disables this extra dimension.

`userAgentPatterns` and `userAgentExceptions` accept at most 32 case-insensitive Rust regular expressions, each at most 256 bytes, with bounded compiled size and no backreferences/lookaround. Additional matches use the custom automation class and are covered by first-request bot blocking when selected. Exceptions affect only this dimension, never other sign-in, rate or adaptive protections. Source addresses use the existing trusted-proxy rules and keyed identifiers; raw headers are not retained. The editor simulator previews the same classifier and policy without updating counters.

The "what would happen" simulator (Security, Adaptive protection) can also include the trust tier (unknown, known, signed in), the claimed user-agent class, the country lookup result (a country code, not found, no database, or a local network address) and whether the request is an admin-console discovery probe. It then explains, in plain language: the per-address budget after `security.trust.unknownPercent` or `security.trust.knownPercent`, the sign-in cooldown and challenge difficulty after the same scaling, the action for the user-agent class (a signed-in session is never blocked by class), whether the unknown-country admin block would count the probe, with its threshold, window and first block length, and what sign-in heightened mode adds while it is on. These numbers come from the same functions the live request path calls (the simulator does not repeat their arithmetic), and the simulator never writes, counts or blocks anything. Saved trust and geography settings are used; unsaved drafts apply only to the protection rules.

The optional [client interface access gate](client-access.md) has a dedicated redacted, conditional API rather than plaintext settings-registry keys. Access-key and entry-slug restrictions both default to disabled.

## Session mail cache and speculative reads

The `mail.cache.*` policies apply to the current authenticated mailbox session.
Global/domain defaults and locks govern user overrides. This cache stores only
session memory; it does not enable persistent offline storage. Opening a cached
folder or message can paint immediately while the authenticated transport refreshes
it. Changes, sign-out and account-authority transitions invalidate cached data.
Serialized UTF-8 payload bytes count against the configured budget; browser object
and allocator overhead is not included in that number.

<a id="mail.cache.enabled"></a>
`mail.cache.enabled` — Enable session-memory caching. Default: `true`.

<a id="mail.cache.mailboxes"></a>
`mail.cache.mailboxes` — Cache folder first pages. Default: `true`.

<a id="mail.cache.messages"></a>
`mail.cache.messages` — Cache opened message payloads. Default: `true`.

<a id="mail.cache.refreshOnOpen"></a>
`mail.cache.refreshOnOpen` — Refresh in the background on every explicit open; false permits fresh-cache reuse. Default: `true`.

<a id="mail.cache.freshSeconds"></a>
`mail.cache.freshSeconds` — Fresh reuse interval,0–3600seconds, clamped to maximum age. Default: `30`.

<a id="mail.cache.maxAgeSeconds"></a>
`mail.cache.maxAgeSeconds` — Maximum age,1–86400seconds. Default: `300`.

<a id="mail.cache.maxEntries"></a>
`mail.cache.maxEntries` — LRU entry limit,1–512. Default: `32`.

<a id="mail.cache.maxBytes"></a>
`mail.cache.maxBytes` — Serialized payload budget,65536–268435456bytes. Default: `16777216`.

<a id="mail.cache.prefetchEnabled"></a>
`mail.cache.prefetchEnabled` — Master opt-in for speculative reads. Default: `false`.

<a id="mail.cache.hoverEnabled"></a>
`mail.cache.hoverEnabled` — Pointer dwell triggers, only when master enabled. Default: `true`.

<a id="mail.cache.focusEnabled"></a>
`mail.cache.focusEnabled` — Keyboard focus triggers, only when master enabled. Default: `true`.

<a id="mail.cache.dwellMs"></a>
`mail.cache.dwellMs` — Pointer/focus dwell,0–10000milliseconds. Default: `200`.

<a id="mail.cache.idleEnabled"></a>
`mail.cache.idleEnabled` — Idle folder warming, only when master enabled. Default: `false`.

<a id="mail.cache.idleDelayMs"></a>
`mail.cache.idleDelayMs` — Idle delay,100–60000milliseconds. Default: `1500`.

<a id="mail.cache.maxConcurrency"></a>
`mail.cache.maxConcurrency` — Background read concurrency,1–4. Default: `1`.

<a id="mail.cache.maxQueued"></a>
`mail.cache.maxQueued` — Waiting speculative requests,1–64. Default: `8`.

<a id="mail.cache.preloadMessages"></a>
`mail.cache.preloadMessages` — Allow speculative message payload reads. Default: `false`.

<a id="mail.cache.excludedFolders"></a>
`mail.cache.excludedFolders` — Exact role:<role> or id:<folder ID> exclusions, up to100rules. Default: `["role:trash","role:junk"]`.

Speculative reads use the existing configured transport and never activate a
reader, mark messages read, issue read receipts, fetch attachments, or fetch/sign
remote images. Foreground reads take priority; disablement and authority changes
cancel pending work. Save-Data, hidden documents and offline state suppress new
speculative work. Existing sanitizer/quarantine rules still apply to displayed
message bodies.

## Plain-text response templates

<a id="mail.templates.enabled"></a>
`mail.templates.enabled` defaults to true and supports global/domain/user defaults
and locks. It controls template use; it never sends a message automatically.

<a id="mail.templates.allowPersonal"></a>
`mail.templates.allowPersonal` defaults to true and is administrative only
(global/domain/admin-user). False refuses new personal-template writes; removal
remains available.

<a id="mail.templates.shared"></a>
`mail.templates.shared` defaults to an empty list. Administrators control writes; authenticated users can read the shared library and the effective `mail.templates.allowPersonal` flag for their own account. Neither setting accepts ordinary user writes.

<a id="mail.templates.personal"></a>
`mail.templates.personal` defaults to an empty list and supports user overrides.
Writes require effective enabled and allowPersonal policies. Each list permits
20 unique entries with exact fields id/title/body: ASCII identifier1–64,
nonblank title1–80 Unicode characters without controls, and nonblank literal body
up to8000 Unicode characters permitting line breaks and tabs. The serialized list
is limited to65536bytes. Unknown fields, duplicate IDs and forbidden controls are
rejected. Text and whitespace are preserved; templates are never evaluated as code
or HTML.

<a id="mail.composeWindow"></a>
`mail.composeWindow` chooses tab (default) or window for composing; global/domain/user policy.
<a id="mail.replyCopyToOriginal"></a>
`mail.replyCopyToOriginal` defaults false; prefer the original folder for reply copies when allowed.
<a id="mail.forwardMode"></a>
`mail.forwardMode` chooses inline (default) or attachment forwarding; global/domain/user policy.
<a id="mail.allowSend"></a>
`mail.allowSend` defaults true. When false for a user, the server refuses `EmailSubmission/set` creates with `403 setting_disabled` and the composer disables Send and its shortcut; drafts can still be saved. Global/domain/user policy; users can only change it when it is not locked.
<a id="mail.allowReply"></a>
`mail.allowReply` defaults true. When false, the reader and its shortcuts offer no Reply or Reply all. Enforced in the web client only.
<a id="mail.allowForward"></a>
`mail.allowForward` defaults true. When false, the reader and its shortcuts offer no inline or attachment Forward, and the selection toolbar's forward-as-attachment is disabled. Enforced in the web client only.
<a id="mail.allowDownload"></a>
`mail.allowDownload` defaults true. When false, the web client offers no message download, bulk download or attachment download, and `GET /api/export/{stableId}` (export and bulk mbox download) answers `403 setting_disabled`. `GET /jmap/download/...?download=true` answers the same `403`; the web client adds `download=true` to the single-message `.eml` download and to an attachment download link served by the server. Requests without it (message headers, spam score, inline parts and previews, which the reader fetches through the same route) are not refused, and an attachment the reader already fetched for preview is saved from the browser without another request, so a user who calls `/jmap/download` directly without `download=true` can still fetch a blob.
<a id="mail.allowPrint"></a>
`mail.allowPrint` defaults true. When false, the reader offers no Print action. Enforced in the web client only.
<a id="mail.showSpam"></a>
`mail.showSpam` defaults true. When false, the Junk folder is left out of the folder list and navigation, and messages marked `$junk` are left out of search results. Nothing is deleted. Enforced in the web client only.
<a id="mail.showAuthStatus"></a>
`mail.showAuthStatus` defaults true. When false, the reader does not show the SPF, DKIM and DMARC results of the Authentication-Results header. Unrelated warnings (spoofed sender, unsafe content) are not tied to this setting.
<a id="mail.closeOnDelete"></a>
`mail.closeOnDelete` defaults true. When true, deleting or moving to Trash the message open in the reader, a tab or a detached pane closes that view, whether the delete came from the reader, the list, a shortcut, a bulk action or another client. When false the reader keeps its previous behaviour (`mail.afterDelete` decides what is shown next).
<a id="mail.reader.unsafeHtmlMode"></a>
`mail.reader.unsafeHtmlMode` defaults `sanitized`. For messages whose HTML is detected as unsafe: `sanitized` shows the cleaned HTML automatically, `ask` offers a per-message button to show it, `strict` shows only safe text. Active content and automatic remote resources stay blocked in every mode. Global/domain/user policy; a lock enforces the mode.
<a id="reader.attachmentsDensity"></a>
`reader.attachmentsDensity` is `compact` (default) or `comfortable` and sets the spacing of the reader's attachment list. Global/domain/user policy.
<a id="cap.reading.original_html"></a>
`cap.reading.original_html` defaults true. When false, the isolated original HTML reader option is not offered. It does not authorize scripts or unrestricted remote resources. See `mail.reader.allowOriginalHtml`.
<a id="images.trustedHosts"></a>
`images.trustedHosts` is a list of image hosts (up to 100 canonical domains, 253 characters each; default empty). Remote images from a listed host, or a subdomain of it, load through the image proxy without a per-message grant while `privacy.remoteContent` is `ask`. It never lifts `privacy.remoteContent=block`, and a host also in `images.blockedHosts` stays blocked. The reader sets one state per host (Always show, Ask, Never show), so a host is in at most one of the two lists. Global/domain/user policy; a locked value hides the control.
<a id="images.blockedHosts"></a>
`images.blockedHosts` is a list of image hosts (up to 100 canonical domains; default empty) whose images never load, in any `privacy.remoteContent` mode, whatever grants exist. The server refuses to sign or fetch them. Blocked wins over `images.trustedHosts`. Global/domain/user policy.
<a id="links.trustedSenders"></a>
`links.trustedSenders` is a list of exact sender mailboxes (up to 100 entries, 254 characters each). Links in messages from a listed sender may open from Review links without per-message consent. Default empty. Global/domain/user policy.
<a id="appearance.fontSize"></a>
`appearance.fontSize` is the interface font size in pixels, 12 to 24, default 14. Global/domain/user policy.
<a id="appearance.fontWeight"></a>
`appearance.fontWeight` is the base interface font weight: `300`, `400` (default), `500` or `600`. Emphasized text keeps its hierarchy relative to the base. Global/domain/user policy.
<a id="appearance.fontColor"></a>
`appearance.fontColor` is the interface text colour as `#RRGGBB`, up to 7 characters. Empty (default) follows the theme. Global/domain/user policy.
<a id="appearance.lowContrast"></a>
`appearance.lowContrast` defaults false. When true, interface text contrast is softened. Global/domain/user policy.
<a id="mail.messageListStyle"></a>
`mail.messageListStyle` is `cards` (default) or `table`; `table` shows Gmail-style rows with column headings. Global/domain/user policy.
<a id="mail.messageListOrder"></a>
`mail.messageListOrder` orders the message list: `newest` (default), `oldest`, `sender` or `subject`. Global/domain/user policy.
<a id="mail.messageListPreview"></a>
`mail.messageListPreview` defaults true. When false, the message list shows no body snippets. Global/domain/user policy.
<a id="mail.badge.auth"></a>
### Reader badges (`mail.badge.auth`, `mail.badge.spam`, `mail.badge.sanitized`, `mail.badge.folder`)

Global/domain/user settings, one per badge in the reader's header row: the DMARC authentication badge (default `"always"`), the spam score badge (`"always"`), the sanitized HTML badge (`"notable"`) and the folder badge (`"always"`). Values: `always`, `notable` (only when there is something to look at: an authentication problem, a spam score above the ham band, HTML that was blocked, a message in Junk or Trash) and `never`. Users choose under Settings > Message display > Message badges, or right-click a badge, unless locked; a lock disables the control and the right-click items. The spam header is still read when its badge is hidden, because the notices below the header depend on it. `mail.showAuthStatus` keeps its own meaning: it leaves the SPF, DKIM and DMARC results out of the security details. Priority marks keep their own setting, `mail.priorityMarks`.

<a id="mail.badge.density"></a>
### Reader badge density (`mail.badge.density`)

Global/domain/user setting, default `"compact"`. `compact` or `comfortable` height and padding for every badge in the reader's header row.

<a id="mail.badge.labels"></a>
### Reader badge labels (`mail.badge.labels`)

Global/domain/user setting, default `"text"`. `text` shows an icon and a short label; `icon` shows the icon only (the accessible name and the tooltip keep the full text; the spam score keeps its number).

<a id="mail.rowActionsMode"></a>
`mail.rowActionsMode` defaults `open`. Controls the quick-action buttons on each message list row: `open` shows the full set as before (revealed on row hover or keyboard focus), `hover` shows three controls (flag, archive, delete) and a More button that reveals pin, snooze and label, `click` shows one button per row that opens the actions, and `off` removes them from the rows (the context menu and toolbar still carry every action). On touch screens `hover` behaves like `click`. Global/domain/user policy: an administrator sets the default or locks the value for everyone, otherwise each user chooses in Settings; enforced in the web client only.
<a id="mail.send.recallDelaySeconds"></a>
`mail.send.recallDelaySeconds` defaults 10 and accepts 0 to 1800 seconds. After Send, the message waits in the Outbox for this long and can be recalled; 0 sends at once. Global/domain/user policy: each user chooses in Settings (Off, 5 s, 10 s, 20 s, 30 s, 1, 2, 5, 10 or 30 minutes, or a custom value) up to the limit below, unless an administrator locks the value. See [Outbox, recall and scheduled sending](outbox-recall.md).
<a id="mail.send.recallDelayMaxSeconds"></a>
`mail.send.recallDelayMaxSeconds` defaults and is capped at 1800. The longest window a user may choose. The engine shortens a longer request to this value when the submission is created. Global/domain policy.
<a id="mail.send.allowRecall"></a>
`mail.send.allowRecall` defaults true. When false, new messages are sent immediately (no hold) and cannot be recalled; scheduled sends remain cancellable until their time. Global/domain policy.
<a id="mail.send.maxDelayedSeconds"></a>
`mail.send.maxDelayedSeconds` defaults 7776000 (90 days) and accepts 0 to 31536000 (1 year). How far ahead a message may be scheduled; a later `sendAt` is refused. 0 turns scheduled sending off. Global/domain policy.
<a id="mail.send.maxQueued"></a>
`mail.send.maxQueued` defaults 200 and accepts 1 to 5000. The number of messages one mailbox may have waiting (undo window or scheduled); a further send is refused with `overQuota` and is never dropped. Global/domain policy.
<a id="mail.showGoToTop"></a>
`mail.showGoToTop` defaults false. When true, the message list shows a Go to top button that scrolls the list back to its first message without reloading it; scrolling is instant when the user prefers reduced motion. When false the button is not shown. Global/domain/user policy; enforced in the web client only.
<a id="mail.showCreateFolderButton"></a>
`mail.showCreateFolderButton` defaults false. When true, the sidebar shows an icon-only Create folder button next to Compose; it opens the same dialog as the Create folder menu item, with the parent set to the top level. When false the button is not shown (the folder context menu still offers New subfolder). Global/domain/user policy; enforced in the web client only.
<a id="mail.afterDelete"></a>
`mail.afterDelete` chooses what the reader shows after the open message is deleted: `next` (default), `previous` or `close`. Global/domain/user policy. See `mail.closeOnDelete`.
<a id="reader.attachmentsAtEnd"></a>
`reader.attachmentsAtEnd` defaults false. When true, the reader shows attachments after the message body instead of before it. Global/domain/user policy.
<a id="notifications.askPermission"></a>
`notifications.askPermission` defaults true. When true, the web client offers the browser notification permission prompt on devices that have neither allowed nor blocked notifications. Global/domain/user policy.

### Sending identity defaults

An account's saved sending identities retain an optional organization (up to 255 characters), automatic Bcc recipient list (up to 20 addresses and 4096 characters), and associated OpenPGP signing-key ID. These are defaults for the composer, not additional permission to use a sender address. Reply-To and the selected signature remain separate fields. Header control characters and invalid recipient lists are refused. Signature references must belong to the current personal identity; signing-key associations must refer to a valid, own, signing-capable OpenPGP key in the current native account. Existing identities are preserved when upgrading. SQLite and PostgreSQL persist the fields, including store transfers.

<a id="cookies.enabled"></a>
### `cookies.enabled`

Global/domain boolean, default `false`. Turns the consent control on for the sign-in page and the mail app. The public configuration at `GET /api/cookies/config` reports `enabled` and the resolved mode; a client with the control off applies no consent rules and blocks nothing.

<a id="cookies.mode"></a>
### `cookies.mode`

Global/domain enum `optIn` | `optOut` | `noticeOnly`, default `optIn`. Applies outside the regions listed in `cookies.strictRegions`. In `optIn` every optional category starts off and storage of a category that was not accepted is blocked and purged. In `optOut` the `cookies.<category>.default` values apply until the visitor rejects. In `noticeOnly` the visitor sees the information and the defaults apply; there are no toggles.

<a id="cookies.regionMode"></a>
### `cookies.regionMode`

Global/domain enum `optIn` | `optOut` | `noticeOnly` | `same`, default `optIn`. The model for visitors whose country is in `cookies.strictRegions`.

<a id="cookies.strictRegions"></a>
### `cookies.strictRegions`

Global/domain list of up to 80 two-letter country codes. Default: the EU and EEA states, GB and CH. The visitor country comes from the header named by `cookies.regionHeader`, else from the region subtag of the first `Accept-Language` entry. The decision is an input to the control only and is stored as a region code, never an address.

<a id="cookies.regionHeader"></a>
### `cookies.regionHeader`

Global/domain text, default `X-Country-Code`. Only a trusted proxy should set this header; a client that can reach the server directly can send any value, so enable it only behind a proxy that overwrites it. When absent, the region subtag of `Accept-Language` is used.

<a id="cookies.unknownRegion"></a>
### `cookies.unknownRegion`

Global/domain enum `strict` | `normal`, default `strict`.

<a id="cookies.functional.enabled"></a>
### `cookies.functional.enabled`

Global/domain boolean, default `true`. When off the category is not offered, and its storage is treated as not consented.

<a id="cookies.functional.default"></a>
### `cookies.functional.default`

Global/domain boolean, default `false`. Used as the initial toggle in `optIn` and as the applied state in `optOut` and `noticeOnly`.

<a id="cookies.analytics.enabled"></a>
### `cookies.analytics.enabled`

Global/domain boolean, default `false`. The application sets no analytics storage itself; turn this on when the analytics tag (Analytics section) or a plugin does.

<a id="cookies.analytics.default"></a>
### `cookies.analytics.default`

Global/domain boolean, default `false`.

<a id="cookies.marketing.enabled"></a>
### `cookies.marketing.enabled`

Global/domain boolean, default `false`. The application sets no marketing storage itself.

<a id="cookies.marketing.default"></a>
### `cookies.marketing.default`

Global/domain boolean, default `false`.

<a id="cookies.layout"></a>
### `cookies.layout`

Global/domain enum `banner` | `card` | `modal`, default `banner`. `modal` traps focus; the others leave the page usable.

<a id="cookies.position"></a>
### `cookies.position`

Global/domain enum `bottom` | `top` | `start` | `end` | `center`, default `bottom`. `start` and `end` follow the reading direction.

<a id="cookies.rejectProminent"></a>
### `cookies.rejectProminent`

Global/domain boolean, default `true`. When off, Reject all is a text link.

<a id="cookies.policyUrl"></a>
### `cookies.policyUrl`

Global/domain `https://` URL or null. Shown as a link in the control and in the Cookie preferences dialog.

<a id="cookies.privacyUrl"></a>
### `cookies.privacyUrl`

Global/domain `https://` URL or null.

<a id="cookies.policyVersion"></a>
### `cookies.policyVersion`

Global/domain integer 1 to 1,000,000, default 1. Stored with each decision. A stored decision with a different version is stale when `cookies.repromptOnVersion` is on.

<a id="cookies.lifetimeDays"></a>
### `cookies.lifetimeDays`

Global/domain integer 1 to 730, default 180.

<a id="cookies.repromptOnVersion"></a>
### `cookies.repromptOnVersion`

Global/domain boolean, default `true`.

<a id="cookies.honourGpc"></a>
### `cookies.honourGpc`

Global/domain boolean, default `true`. The client reads `navigator.globalPrivacyControl`. A signal sets the optional categories to off, and the control records the signal with the decision.

<a id="cookies.honourDnt"></a>
### `cookies.honourDnt`

Global/domain boolean, default `false`. Do Not Track is deprecated in browsers; this is available when your policy asks for it.

<a id="cookies.logConsent"></a>
### `cookies.logConsent`

Global/domain boolean, default `true`. Each row holds time, domain, policy version, decision, category choices, region code, the privacy signal and, for a signed-in visitor, the account id. No address, IP address or user agent is stored. When off, decisions stay in the browser and statistics stay empty.

<a id="cookies.logRetentionDays"></a>
### `cookies.logRetentionDays`

Global/domain integer 1 to 3650, default 365.

<a id="cookies.scanIntervalHours"></a>
### `cookies.scanIntervalHours`

Global/domain integer 0 to 720, default 24. The scan reads names only. A scan runs at load when the last one in this browser is older than the interval; a visitor or administrator can also run it on demand. Names no inventory entry or rule classifies are reported to the Unclassified list of the admin Cookies tab.

<a id="cookies.unclassified"></a>
### `cookies.unclassified`

Global/domain enum `report` | `functional` | `analytics` | `marketing`, default `report`. With a category, storage that is not in the inventory or a rule is purged when that category is not consented.

<a id="cookies.text.title"></a>
### `cookies.text.title`

Global/domain map of locale to text, up to 1,000 characters each. Empty uses the built-in text of the visitor's language.

<a id="cookies.text.body"></a>
### `cookies.text.body`

Global/domain map of locale to text, up to 1,000 characters each. Plain text; no markup is rendered.

<a id="cookies.label.necessary"></a>
### `cookies.label.necessary`

Global/domain map of locale to text, up to 120 characters each.

<a id="cookies.description.necessary"></a>
### `cookies.description.necessary`

Global/domain map of locale to text, up to 600 characters each.

<a id="cookies.label.functional"></a>
### `cookies.label.functional`

Global/domain map of locale to text, up to 120 characters each.

<a id="cookies.description.functional"></a>
### `cookies.description.functional`

Global/domain map of locale to text, up to 600 characters each.

<a id="cookies.label.analytics"></a>
### `cookies.label.analytics`

Global/domain map of locale to text, up to 120 characters each.

<a id="cookies.description.analytics"></a>
### `cookies.description.analytics`

Global/domain map of locale to text, up to 600 characters each.

<a id="cookies.label.marketing"></a>
### `cookies.label.marketing`

Global/domain map of locale to text, up to 120 characters each.

<a id="cookies.description.marketing"></a>
### `cookies.description.marketing`

Global/domain map of locale to text, up to 600 characters each.

<a id="restriction.enabled"></a>
### Restrictions of processing allowed (`restriction.enabled`)
Global and domain, hot. Default `true`. When off, restrictions can no longer be applied; existing restrictions stay in force until lifted.

<a id="restriction.autoApplyOnApproval"></a>
### Apply a restriction when its request is approved (`restriction.autoApplyOnApproval`)
Global and domain, hot. Default `true`. Approving an Art. 18 restriction request applies the restriction at once instead of waiting for an administrator.

<a id="restriction.mailDelivery"></a>
### New mail while restricted (`restriction.mailDelivery`)
Global and domain, hot. Default `keep`. `keep` stores new mail normally, `queue` holds it, `reject` refuses it. Stored mail is never deleted by a restriction.

<a id="restriction.defaultExpiryDays"></a>
### Days until a restriction ends (`restriction.defaultExpiryDays`)
Global and domain, hot. Default `0`. 0 keeps a restriction until an administrator lifts it.

<a id="restriction.notifyUser"></a>
### Tell the person when a restriction applies or ends (`restriction.notifyUser`)
Global and domain, hot. Default `true`.

<a id="restriction.scope.indexing"></a>
### Restriction pauses search indexing by default (`restriction.scope.indexing`)
Global and domain, hot. Default `true`. A new restriction includes this scope unless the administrator clears it.

<a id="restriction.scope.assistant"></a>
### Restriction pauses the assistant by default (`restriction.scope.assistant`)
Global and domain, hot. Default `true`. A new restriction includes this scope unless the administrator clears it.

<a id="restriction.scope.analytics"></a>
### Restriction pauses analytics by default (`restriction.scope.analytics`)
Global and domain, hot. Default `true`. A new restriction includes this scope unless the administrator clears it.

<a id="restriction.scope.classification"></a>
### Restriction pauses local rules, spam verdicts and auto-filing by default (`restriction.scope.classification`)
Global and domain, hot. Default `true`. A new restriction includes this scope unless the administrator clears it.

<a id="restriction.scope.push"></a>
### Restriction pauses push notifications by default (`restriction.scope.push`)
Global and domain, hot. Default `true`. A new restriction includes this scope unless the administrator clears it.

<a id="restriction.scope.thirdParty"></a>
### Restriction pauses sharing with third parties by default (`restriction.scope.thirdParty`)
Global and domain, hot. Default `true`. A new restriction includes this scope unless the administrator clears it.

<a id="consent.enabled"></a>
### Purpose consent ledger and Your data section (`consent.enabled`)
Global and domain, hot. Default `true`. Records each person's decisions per purpose and shows them under Settings > Privacy > Your data.

<a id="consent.receipts.retentionDays"></a>
### Days consent receipts are kept (`consent.receipts.retentionDays`)
Global, hot. Default `1825`. Older ledger rows are purged. 0 keeps them.

<a id="consent.art22.notice"></a>
### Automated processing notice (`consent.art22.notice`)
Global and domain, hot. Default `true`. Shows people a notice about automated processing.

<a id="consent.analytics.basis"></a>
### Legal basis shown for analytics (`consent.analytics.basis`)
Global and domain, hot. Default `consent`. One of `consent`, `legitimateInterest`, `contract`, `legalObligation`. Only `consent` makes the purpose wait for a yes; for the others it runs until the person withdraws.

<a id="consent.analytics.version"></a>
### Consent text version of analytics (`consent.analytics.version`)
Global and domain, hot. Default `1`. Raising it asks everyone again; a yes for an older version no longer counts.

<a id="consent.analytics.text"></a>
### Consent text of analytics (`consent.analytics.text`)
Global and domain, hot. Default `{}`. Map of locale to text, up to 2000 characters each. Empty uses the built-in text.

<a id="consent.analyticsTag.basis"></a>
### Legal basis shown for the analytics tag (`consent.analyticsTag.basis`)
Global and domain, hot. Default `consent`. One of `consent`, `legitimateInterest`, `contract`, `legalObligation`. Only `consent` makes the purpose wait for a yes; for the others it runs until the person withdraws.

<a id="consent.analyticsTag.version"></a>
### Consent text version of the analytics tag (`consent.analyticsTag.version`)
Global and domain, hot. Default `1`. Raising it asks everyone again; a yes for an older version no longer counts.

<a id="consent.analyticsTag.text"></a>
### Consent text of the analytics tag (`consent.analyticsTag.text`)
Global and domain, hot. Default `{}`. Map of locale to text, up to 2000 characters each. Empty uses the built-in text.

<a id="consent.assistant.basis"></a>
### Legal basis shown for the assistant (`consent.assistant.basis`)
Global and domain, hot. Default `contract`. One of `consent`, `legitimateInterest`, `contract`, `legalObligation`. Only `consent` makes the purpose wait for a yes; for the others it runs until the person withdraws.

<a id="consent.assistant.version"></a>
### Consent text version of the assistant (`consent.assistant.version`)
Global and domain, hot. Default `1`. Raising it asks everyone again; a yes for an older version no longer counts.

<a id="consent.assistant.text"></a>
### Consent text of the assistant (`consent.assistant.text`)
Global and domain, hot. Default `{}`. Map of locale to text, up to 2000 characters each. Empty uses the built-in text.

<a id="consent.push.basis"></a>
### Legal basis shown for push notifications (`consent.push.basis`)
Global and domain, hot. Default `contract`. One of `consent`, `legitimateInterest`, `contract`, `legalObligation`. Only `consent` makes the purpose wait for a yes; for the others it runs until the person withdraws.

<a id="consent.push.version"></a>
### Consent text version of push notifications (`consent.push.version`)
Global and domain, hot. Default `1`. Raising it asks everyone again; a yes for an older version no longer counts.

<a id="consent.push.text"></a>
### Consent text of push notifications (`consent.push.text`)
Global and domain, hot. Default `{}`. Map of locale to text, up to 2000 characters each. Empty uses the built-in text.

<a id="consent.openTracking.basis"></a>
### Legal basis shown for open tracking (`consent.openTracking.basis`)
Global and domain, hot. Default `contract`. One of `consent`, `legitimateInterest`, `contract`, `legalObligation`. Only `consent` makes the purpose wait for a yes; for the others it runs until the person withdraws.

<a id="consent.openTracking.version"></a>
### Consent text version of open tracking (`consent.openTracking.version`)
Global and domain, hot. Default `1`. Raising it asks everyone again; a yes for an older version no longer counts.

<a id="consent.openTracking.text"></a>
### Consent text of open tracking (`consent.openTracking.text`)
Global and domain, hot. Default `{}`. Map of locale to text, up to 2000 characters each. Empty uses the built-in text.

<a id="consent.remoteContent.basis"></a>
### Legal basis shown for remote content (`consent.remoteContent.basis`)
Global and domain, hot. Default `consent`. One of `consent`, `legitimateInterest`, `contract`, `legalObligation`. Only `consent` makes the purpose wait for a yes; for the others it runs until the person withdraws.

<a id="consent.remoteContent.version"></a>
### Consent text version of remote content (`consent.remoteContent.version`)
Global and domain, hot. Default `1`. Raising it asks everyone again; a yes for an older version no longer counts.

<a id="consent.remoteContent.text"></a>
### Consent text of remote content (`consent.remoteContent.text`)
Global and domain, hot. Default `{}`. Map of locale to text, up to 2000 characters each. Empty uses the built-in text.

<a id="consent.integrations.basis"></a>
### Legal basis shown for integrations (`consent.integrations.basis`)
Global and domain, hot. Default `contract`. One of `consent`, `legitimateInterest`, `contract`, `legalObligation`. Only `consent` makes the purpose wait for a yes; for the others it runs until the person withdraws.

<a id="consent.integrations.version"></a>
### Consent text version of integrations (`consent.integrations.version`)
Global and domain, hot. Default `1`. Raising it asks everyone again; a yes for an older version no longer counts.

<a id="consent.integrations.text"></a>
### Consent text of integrations (`consent.integrations.text`)
Global and domain, hot. Default `{}`. Map of locale to text, up to 2000 characters each. Empty uses the built-in text.

<a id="consent.telemetry.basis"></a>
### Legal basis shown for telemetry (`consent.telemetry.basis`)
Global and domain, hot. Default `legitimateInterest`. One of `consent`, `legitimateInterest`, `contract`, `legalObligation`. Only `consent` makes the purpose wait for a yes; for the others it runs until the person withdraws.

<a id="consent.telemetry.version"></a>
### Consent text version of telemetry (`consent.telemetry.version`)
Global and domain, hot. Default `1`. Raising it asks everyone again; a yes for an older version no longer counts.

<a id="consent.telemetry.text"></a>
### Consent text of telemetry (`consent.telemetry.text`)
Global and domain, hot. Default `{}`. Map of locale to text, up to 2000 characters each. Empty uses the built-in text.
