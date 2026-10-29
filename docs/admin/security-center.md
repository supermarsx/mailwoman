# Security centre

The account Security centre contains authentication factors, active sessions,
sign-in history, remembered devices and account activity. Session listings show
client metadata and opaque handles, never authentication tokens or passwords.
Country labels use only the operator's local `MW_GEOIP_DB`; without it no country
is displayed. Exporting sign-in history requires recent verification.

## Administrator navigation

The administrator **Security centre** (`#/security`) groups overview and factor
policy, IP bans, temporary sign-in locks, sign-in country policy, aggregate and
adaptive protection, security policy, inbound access, deployment checks and sessions.
Only authorized pages mount or fetch data. Shared source-IP controls require
unscoped grants; mailbox-session listings retain their domain-scoped permissions.
Each write still uses its existing capability, fresh verification and confirmation.

Old `#/two-factor`, `#/sessions`, `#/security-policy`, `#/inbound-allowlist` and
`#/observability/bans`, `/locks`, `/geography`, `/protection` bookmarks resolve to
the corresponding Security centre page, including the ban editor and field query.
Observability contains audit, request statistics and telemetry. Audit and ban lists
load independently, so lacking one capability does not break the other page.

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

## Passkey policy

Five settings narrow which mailbox passkeys count. Each one can be set
globally or for a domain, and locked like any other registry setting
(Admin › Settings, category `signin`). The defaults keep the behaviour from
before these settings existed.

| Setting | Values (default first) | Effect |
| --- | --- | --- |
| `twofa.passkeyAttachment` | `any`, `platform`, `cross-platform` | `platform`: passkeys stored on the device (Windows Hello, Touch ID, Android screen lock). `cross-platform`: security keys and phones. |
| `twofa.passkeyUserVerification` | `preferred`, `required`, `discouraged` | `required`: the passkey must check a PIN or biometric at registration and at every sign-in. |
| `twofa.passkeyResidentKey` | `discouraged`, `preferred`, `required` | `required`: the credential must be stored on the authenticator (discoverable). |
| `twofa.passkeyLegacy` | `allow`, `refuse` | What happens to a passkey added before a stricter policy. See below. |
| `twofa.lastFactorRemoval` | `allow`, `requireRecoveryCodes`, `block` | See [Removing the last factor](#removing-the-last-factor). |

The values are sent to the browser in the registration and sign-in options,
and the server checks the result:

- **User verification** is a flag the authenticator signs. With `required`, a
  registration or sign-in without it is refused after the signature has been
  checked.
- **Attachment** is what the browser reports (`authenticatorAttachment`) and
  the transports it lists. Mailwoman requests attestation `none`, so the
  authenticator does not sign this. The server refuses a browser that reports
  the wrong kind, or reports nothing while the setting is not `any`. It cannot
  prove which hardware was used.
- **Resident key**: the browser is asked for one. A registration whose
  `credProps.rk` is reported `false` while the setting is `required` is
  refused. A browser that does not report `credProps` is trusted to have
  followed `residentKey: "required"`.

In engine mode a passkey is also a passwordless sign-in credential, so
registration always asks for a discoverable, user-verified credential. There
the settings can only make user verification and resident key stricter.

The same rules apply to passkey step-up confirmation. Every refusal gives the
user a message that names the rule. It writes a `passkey-policy-refused` audit
entry (actor: the user; detail: flow and reason) and a failed sign-in event in
the account's activity. A policy refusal is only shown once the signature has
been checked. A forged or tampered assertion still gets the uniform
"did not verify" answer.

### Passkeys added before a stricter policy

The default is **grandfathering** (`twofa.passkeyLegacy = allow`), so that a
policy change does not lock anyone out. An older passkey that does not meet
the current policy keeps working under the rule it was registered with. In
Settings › Two-factor authentication it is marked "Added before the current
policy and does not meet it", with a request to add a new passkey and remove
the old one. A passkey that has completed user verification at least once is
then always asked for it when user verification is `required`, so it cannot
fall back to signing in without verification.

With `refuse`, a passkey whose stored attachment or resident-key property does
not meet the policy is refused with "This passkey was added before your
organization's current passkey policy and no longer meets it". User
verification is checked at every sign-in rather than stored, so with `refuse`
a passkey that has not verified the user before is asked to do so, and is
refused only if it does not. The user can still sign in with an authenticator
app code or a recovery code and add a new passkey. Before switching to
`refuse`, check that affected users have another factor.

An older passkey with no recorded transports has an unknown attachment. Under
an attachment policy it counts as not meeting it.

### Removing the last factor

When removing an authenticator app or passkey would leave the account with no
second factor:

- if `twofa.required` (or a two-factor policy row) applies to the account,
  the removal is refused, as before;
- otherwise `twofa.lastFactorRemoval` decides: `allow` removes it (two-step
  sign-in turns off); `requireRecoveryCodes` refuses until the account has
  unused recovery codes; `block` refuses.

A refusal under `requireRecoveryCodes` or `block` writes the same
`passkey-policy-refused` audit entry with flow `last-factor-removal`.

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
