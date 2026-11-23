# Passkeys

Mailwoman accepts passkeys (WebAuthn credentials) for mailbox accounts and,
separately, for administrator accounts. This page describes what the server checks
and what it does not.

## Mailbox passkeys

Users add and remove passkeys in **Settings** → **Security centre** → **Sign-in &
passkeys** → **Passkeys** (**Add a passkey**).

![The Passkeys section of the Security centre: the authenticator requirements, a passkey name field and Add a passkey.](/screenshots/security/passkeys-user-list.webp){.mw-crop}

What a passkey does depends on how the server reaches mail:

- **Engine mode:** a passkey can sign the user in without a password (**Sign in
  with a passkey** on the sign-in page). This needs a discoverable credential and
  user verification (PIN, fingerprint or face).
- **Proxy mode:** the server needs the user's mail password to reach the
  upstream server, so a passkey is only a second factor after the password.

The administrator controls mailbox passkeys with these settings (global, per
domain, or per account):

| Setting | Default | Effect |
|---|---|---|
| [`twofa.passkeysEnabled`](/admin/settings#twofa.passkeysenabled) | `true` | Allow passkey sign-in and enrolment. |
| [`twofa.passkeyUserVerification`](/admin/settings#twofa.passkeyuserverification) | `preferred` | Whether a second-factor passkey must verify the user. Passwordless sign-in always requires it. |
| [`twofa.passkeyResidentKey`](/admin/settings#twofa.passkeyresidentkey) | `discouraged` | Whether a passkey must be discoverable. |
| [`twofa.passkeyAttachment`](/admin/settings#twofa.passkeyattachment) | `any` | `platform` (stored on the device) or `cross-platform` (security key or phone). |
| [`twofa.passkeyLegacy`](/admin/settings#twofa.passkeylegacy) | `allow` | Whether passkeys registered before a stricter policy keep working (`refuse` stops them). |

::: warning Attachment is reported by the browser
Mailwoman requests `none` attestation, so the authenticator type that
`twofa.passkeyAttachment` checks is what the browser reports. It keeps honest users
on the intended kind of authenticator; it does not prove the credential is stored
in hardware.
:::

## Administrator passkeys

Administrators enrol and remove passkeys in the console under **My preferences**
(**Administrator passkeys**, **Add administrator passkey**). Enrolment requires a
fresh confirmation of the administrator's identity.

![The Administrator passkeys section of My preferences, with no passkey registered.](/screenshots/security/passkeys-admin-preferences.webp){.mw-crop}

- Enrolment and sign-in always require user verification and a discoverable
  credential.
- Once an administrator has a passkey, a password alone does not sign them in
  or confirms a sensitive action. A user-verified passkey can do both on its own;
  an authenticator app code remains an alternative second factor.
- Recovery codes (shared with the authenticator app, see
  [Administrator authenticator app](./admin-totp.md)) work once each, together
  with the administrator password.
- The last usable sign-in credential of the last Super admin cannot be removed.

Administrator credentials live in their own table with their own user-handle
namespace. A mailbox passkey cannot authorise an administrator session, and an
administrator passkey cannot sign in to a mailbox.

An administrator who may manage another administrator's account can remove all of
that account's passkeys with **Admin accounts** → **Password and authenticator** →
**Revoke all passkeys**. This needs a fresh confirmation and, in one database
transaction, deletes the account's passkeys, its recovery codes and its active
console sessions.

![The Password and authenticator group of an administrator's account details: the passkey count, Revoke all passkeys, a new-password field with Reset password and end sessions, Issue a new authenticator secret and Remove authenticator.](/screenshots/security/passkeys-admin-revoke.webp){.mw-crop}

## What the server verifies

For every registration and sign-in:

- **Algorithms:** ES256 (ECDSA P-256), Ed25519, and RS256. RSA keys must have a
  2048 to 8192-bit modulus.
- **Attestation:** only `none` is accepted. Manufacturer or enterprise attestation
  chains are not requested or validated.
- **Binding:** the challenge (32 random bytes, single use, valid five minutes),
  the origin and the relying-party ID must match. Cross-origin (iframe)
  ceremonies are refused.
- **Origin and RP ID:** taken from `MW_WEBAUTHN_ORIGIN` and `MW_WEBAUTHN_RP_ID`
  when set, otherwise from the deployment's public origin (`MW_PUBLIC_URL`, or the
  request's host as resolved through trusted proxies). Set `MW_PUBLIC_URL` so the
  origin does not depend on request headers.
- **User handle:** new mailbox credentials use a 32-byte SHA-256 value derived from
  the account id (not the id itself). A passwordless assertion must return the
  account's user handle; an assertion made after the account is already known may
  omit it, but a supplied handle must match. Credentials registered with the
  account id as their handle keep working.

Stored state:

- A credential ID can be registered once. Registering an existing ID fails; it
  cannot replace another account's key or reset its counter.
- Each successful assertion updates the counter with a conditional write that
  compares account, public key, previous counter and a state revision. A stale
  concurrent assertion, or one racing the credential's deletion, does not start
  a session.
- Backup eligibility is fixed once recorded; backup state may change but cannot
  be set without eligibility. A credential stored before this was recorded learns
  its eligibility from its first verified assertion.
- **Counters:** a device-bound credential whose counter does not increase is
  refused. A synced (backup-eligible) credential with a counter anomaly is
  accepted, the larger counter is kept, and a diagnostic log line records only a
  hash of the credential handle.

## Limits

- Browser tests use Chromium's virtual CTAP2 authenticator. They do not establish
  that every physical security key, phone (cross-device) flow or vendor works.
- Pending ceremonies and confirmations are held in the memory of the server
  process. With several processes, use session affinity. Credential ownership,
  revocation and counters are in the shared database.
- Disabling or deleting an administrator stops their passkeys from starting new
  sessions.
- Sign-in endpoints used from a fresh browser do not require a CSRF cookie, but
  the Origin check still applies. Enrolment, removal and confirmation are
  signed-in requests and go through the CSRF checks of the session (see
  [Sessions and CSRF](./admin-panel.md#sessions-cookies-and-csrf)).

These rules follow [WebAuthn Level 3](https://www.w3.org/TR/webauthn-3/)
(authenticator data, registration and assertion verification).

## Related

- [Administrator authenticator app](./admin-totp.md)
- [Admin console security](./admin-panel.md)
- [Require two-factor](/admin/authenticator)
- [Security centre (user)](/user/settings/account#security-centre)
