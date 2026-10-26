# Notification delivery and browser permissions

`notifications.enabled` follows the existing global/domain/user setting policy. The account notification editor saves ordered sender, mailbox and subject substring rules and an explicit IANA quiet-hours time zone. The first matching rule wins. Old quiet-hours records without a zone use UTC. Failed or malformed policy reads do not grant delivery.

`notifications.pushAllowed` is an administrator-only global/domain ceiling (default true). It gates subscription registration and every background dispatch. `push.quietHours` adds a deployment-wide UTC quiet window; the old `push.quiet_hours` value is used only when the canonical setting is unconfigured. A configured null disables that legacy fallback.

Browser notification permission is requested only by the explicit device button. The application no longer prompts from passive delivery. Foreground-generated system notifications re-read account policy before each batch and discard delayed results after an account transition.

Background push payloads contain only the encrypted opaque wake marker, never message content. Content-rule accounts currently suppress background system notifications: the opaque event has insufficient metadata to safely evaluate sender/subject rules. Full authoritative rule-aware background events remain a separate follow-up. The service worker rechecks authenticated ownership of its endpoint hash and current policy before displaying a delayed wake, including after logout/account switches; it fails closed when this check is unavailable.

Push subscription changes are account-bound. Another account cannot delete or take over an existing endpoint. Web Push and UnifiedPush delivery use the `push` egress traffic class: HTTPS, verified public destination addresses, pinned resolution, no redirects, no direct fallback from a failed selected route, bounded response and time. Remote-DNS routes are refused for this class because the destination address cannot be verified locally. The APNs implementation remains a mock; this slice does not establish production Apple push delivery.

`offline.persistentStorageAllowed` is an administrator-only global/domain ceiling (default false) for an explicit browser persistence request. The browser makes the actual grant decision. The page reports real origin storage usage and persistence status; this control does not implement cache-budget eviction or server retention. Browser grants are device-local. Account notification preferences remain server-backed.

The offline mutation queue separately binds every captured JMAP call to one account. Unknown or mixed-account legacy rows are preserved but quarantined from replay. Logout/account changes clear visible queue/header state and fence later dispatches. A mutation already acknowledged by the original server response is removed from that original account's queue to avoid duplicate replay.
