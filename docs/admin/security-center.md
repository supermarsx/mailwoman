# Security centre

The account Security centre contains authentication factors, active sessions,
sign-in history, remembered devices and account activity. Session listings show
client metadata and opaque handles, never authentication tokens or passwords.
Country labels use only the operator's local `MW_GEOIP_DB`; without it no country
is displayed. Exporting sign-in history requires recent verification.

## Passwordless sign-in

Browser passkeys support engine-mode sign-in using required user verification
and a single-use challenge. The server reconnects the existing account using
its sealed upstream credentials. Proxy mode continues to require the upstream
password; its passkeys remain second factors. Native shell passwordless sign-in
is not exposed until platform relying-party integration is available.

## Recent verification

Sensitive actions challenge the user through the shared confirmation dialog.
A proof lasts five minutes and belongs to one session. Password verification
also needs the account's TOTP when enrolled; a user-verified passkey can confirm
on its own. TOTP codes cannot be replayed. Changing authentication factors
invalidates outstanding proofs. Header-auth service passwords never count as
end-user proof: sign-in provenance is persisted with the session.

## Session policy and remembered devices

Settings `session.maxConcurrent`, `session.idleSecs`, `session.absoluteSecs`,
`twofa.required`, `twofa.passkeysEnabled` and `twofa.rememberDays` use scoped
policy resolution. Deployment timeout limits remain caps. Rotation preserves
the original timestamps, device and authentication method; concurrent logout
cannot be undone by rotating a stale cookie.

Remembered devices are disabled by default. A remembered-device cookie can
skip a login factor after the password verifies; it does not satisfy fresh
verification. Tokens are hashed with the current password/factors and expire
according to policy. Factor or password changes invalidate their use.

## Administrator sessions

The Sessions page requires `sessions.read`; revocation requires
`sessions.revoke`. Domain grants apply to both. Preview is the default and does
not revoke anything. Confirmation submits only the opaque handles reviewed;
newly opened sessions are not implicitly added to the action. Results are
bounded at 1,000 and explicitly report truncation.

## Security notices

Security events are recorded durably in account activity. Inbox delivery uses
backend APPEND in engine mode and guarded JMAP upload/import in proxy mode.
Delivery runs asynchronously with a 30-second bound; transport refusal leaves
the activity entry and writes an operator warning. Read-only POP3 backends
cannot accept APPEND and therefore retain the activity notice only.

Subjects and bodies follow the resolved `ui.language` policy. Bundled server
notices cover ar, de, en, es, fr, it, ja, nl, pl, pt-BR, ru, uk and zh. Regional
variants use the matching base language; unknown languages fall back to English.
Messages contain fixed security guidance, not raw request headers, credentials,
or user-supplied message content.

Upgrading to the provenance schema (migration 0036) signs existing mailbox
sessions out once because their authentication origin cannot be recovered
safely. Native mailbox markers are removed with them. Accounts, mailbox data
and administrator sessions remain intact. Newly created sessions without
recorded provenance cannot mint sensitive-action proofs or enroll factors;
a fresh sign-in records the method.

## Mailbox activity and the fifteen-day limit

Mailbox sessions default to a fifteen-day absolute lifetime from the original
sign-in. Lower deployment or account policy limits still apply. Foreground,
trusted pointer, keyboard, wheel and touch interactions renew the idle clock,
at most once every thirty seconds; background requests, focus/visibility events
and scripted scrolling do not. Existing sessions keep their previous idle
baseline during the upgrade, rather than receiving a new lifetime.

The idle clock is persisted separately from request telemetry. Token rotation
preserves both original sign-in time and user activity. Browser cookies remain
session cookies; this does not introduce a persistent remember-me cookie.
Native bearer sessions use the same persisted clocks and absolute cap.
Administrator idle/absolute limits and fresh verification remain unchanged.
