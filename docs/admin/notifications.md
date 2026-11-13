# Notification delivery and browser permissions

## Notification centre and security evidence

The Notifications workspace appears above Calendar in the sidebar. It keeps real sign-ins, new-device/new-country alerts, session-security warnings, two-factor changes, delegation/forwarding/API-key alerts and detected web-app updates. Background linked-credential renewals do not create sign-in notifications. New events are recorded from this version onward; old raw-account activity is not automatically attributed to a verified personal owner.

Active, Archived and Dismissed views share search, type filtering, Open related action, Dismiss, Archive and Restore. Opening an item marks it read and routes to its related security panel, rules or access settings. Updates offer the existing draft-aware Reload action. These presentation actions do not acknowledge a session warning or repair a security problem. The sidebar count reflects unread active items. The centre refreshes on focus, reconnect, push wake and a one-minute visible-window poll.

`GET/POST /api/account/notification-center` stores the latest 500 records under the verified personal subject, in the existing personal-record table. Dismiss/archive state survives reloads and devices. Reads and writes carry the account and storage-authority fence; account transitions clear the visible list and discard stale replies. The existing personal-record export/erasure mapping covers these rows. The centre is separate from the incoming-mail delivery preference and browser permission.

New session warnings and new-device/new-country security emails carry an event-time snapshot: trusted-proxy-derived IP, network classification, device class, browser family/major version/engine, OS, client type/version, available authentication method, browser recognition and a short device reference. Raw browser cookies, binding hashes and session credentials are never shown. The reference is keyed and scoped to the verified personal owner; it is not a hardware fingerprint. Browser/device attributes are client-reported, and recognition is informational, not a trust grant. Missing fields and older records are explicitly unavailable. Notices use the same personal ownership boundary as the centre.

Private IPv4, loopback, link-local, CGNAT, IPv6 unique-local and mapped local IPv4 addresses display **Internal network** in security notices, sign-in geography and session listings. Public addresses without a country match remain unknown. Country codes and IP-based travel findings retain their accuracy caveat.

## Incoming mail

`notifications.enabled` follows the existing global/domain/user setting policy. The account notification editor saves ordered sender, mailbox and subject substring rules and an explicit IANA quiet-hours time zone. The first matching rule wins. Old quiet-hours records without a zone use UTC. Failed or malformed policy reads do not grant delivery.

Incoming-mail alerts default to off. Enable them in Settings → Notifications; administrator policy can override availability. While the app is open, unread newly created messages in the active account produce a foreground toast with an Open action. Hidden or unfocused windows can show a system notification when browser permission is granted. Toasts do not require browser notification permission. Both paths obey the same enabled setting, rules and quiet hours, and discard results after account changes. Inactive linked accounts do not currently produce these detailed alerts.

Detection follows JMAP `Email/changes`, independently of the displayed folder, pagination or search. Initial state is seeded silently; drafts, read messages, spam/trash/sent-only messages and historical backfill older than five minutes before monitoring began do not trigger alerts. A batch is grouped into one alert. Clicking a live alert opens its message in the mail workspace. System notifications opened after the app has closed or reloaded return to the mailbox. Mobile browsers that reject the page Notification constructor use the existing service worker instead.

Delivery while the app is closed additionally requires explicit browser push registration and configured server push delivery. A generic background push and its resulting live refresh are deduplicated for ten seconds in an open app; a failed push permission check still wakes open tabs for refresh.

`notifications.pushAllowed` is an administrator-only global/domain ceiling (default true). It gates subscription registration and every background dispatch. `push.quietHours` adds a deployment-wide UTC quiet window; the old `push.quiet_hours` value is used only when the canonical setting is unconfigured. A configured null disables that legacy fallback.

Browser notification permission is requested only by the explicit device button. The application no longer prompts from passive delivery. Foreground-generated system notifications re-read account policy before each batch and discard delayed results after an account transition.

Background push payloads contain only the encrypted opaque wake marker, never message content. Content-rule accounts currently suppress background system notifications: the opaque event has insufficient metadata to safely evaluate sender/subject rules. Full authoritative rule-aware background events remain a separate follow-up. The service worker rechecks authenticated ownership of its endpoint hash and current policy before displaying a delayed wake, including after logout/account switches; it fails closed when this check is unavailable.

Push subscription changes are account-bound. Another account cannot delete or take over an existing endpoint. Web Push and UnifiedPush delivery use the `push` egress traffic class: HTTPS, verified public destination addresses, pinned resolution, no redirects, no direct fallback from a failed selected route, bounded response and time. Remote-DNS routes are refused for this class because the destination address cannot be verified locally. The APNs implementation remains a mock; this slice does not establish production Apple push delivery.

`offline.persistentStorageAllowed` is an administrator-only global/domain ceiling (default false) for an explicit browser persistence request. The browser makes the actual grant decision. The page reports real origin storage usage and persistence status; this control does not implement cache-budget eviction or server retention. Browser grants are device-local. Account notification preferences remain server-backed.

The offline mutation queue separately binds every captured JMAP call to one account. Unknown or mixed-account legacy rows are preserved but quarantined from replay. Logout/account changes clear visible queue/header state and fence later dispatches. A mutation already acknowledged by the original server response is removed from that original account's queue to avoid duplicate replay.

## Verified ownership and legacy records

Migration 0066 stores notification rules and push subscriptions under the durable verified personal subject, rather than a raw upstream account ID. Two services that both call an account `1` have separate records. Mutations require the current browser storage authority; delayed responses cannot restore a previous account's editor state. The browser registration display checks the current account's server subscription, not just the browser's local subscription object.

Legacy raw-account notification and push rows remain intact but are not attributed automatically. Users must review and save notification preferences and register this browser again. A local subscription belonging to another account can be explicitly removed before registration; removal does not delete the other account's server record. Expired-delivery cleanup and wake timestamps are conditional on the original subscription owner and ID.

Background native-engine events use an actual native account and its verified personal subject. A raw proxy account ID alone is insufficient to select an owner's subscriptions; such background delivery is suppressed until a verified source can be established. Existing administrator setting ceilings still apply. This change does not establish realm isolation for every generic setting-policy subject or background integration.

Editable sender profiles use the separate `personal_identity` table with the same verified ownership boundary. Old raw-account identity rows are retained without automatic attribution. A sender profile never grants permission to send: native submissions, saved drafts and receipts still validate the actual sender against the current upstream/operator authority. Supported database copy retains these typed rows; copying sealed credentials requires the matching server key.
