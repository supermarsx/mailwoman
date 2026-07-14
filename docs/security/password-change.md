# In-app password change (V7)

V7 (release 26.8.0) adds in-app password change (`mw-passwd`): a user can change their
account password from Mailwoman, against a pluggable backend. It also handles the
consequences of a password change for sealed upstream credentials and for zero-access
accounts.

Password change is **opt-in and backend-selected** per deployment. If no backend is
configured, the password-change UI is not shown.

## Backends

`mw-passwd` implements a `PasswordChangeBackend` trait with these backends:

| Backend | How it changes the password |
|---|---|
| `Local` | updates the local Mailwoman credential store |
| `Ldap3062` | LDAP Password Modify Extended Operation (RFC 3062) |
| `DovecotHttp` | Dovecot HTTP admin API |
| `Poppassd` | the poppassd protocol |
| `WebhookHmac` | POST to a custom webhook, payload signed with HMAC-SHA256 |

Each backend reports a `PasswordPolicy` (min length, character classes, etc.) that the
UI displays before the user types a new password. A backend returns a
`PasswordChangeOutcome` with two flags that drive follow-up work:

- `reencrypt_credentials` — the server re-seals stored upstream credentials under the
  new secret (see below).
- `zeroaccess_rewrap_required` — the client runs a zero-access key-hierarchy re-wrap
  (see below).

Every change writes a `password_change_audit` row (timestamp, account, backend,
outcome). The audit is **content-free** — it never records the old or new password.

### LDAP-bind login and RFC 3062

The `Ldap3062` backend encodes the RFC 3062 Password Modify Extended Operation. The
LDAP connection/transport itself is injected at mount time (it reuses the
`mw-directory` connection layer) so `mw-passwd` owns the exop encoding without
depending on the directory crate's internals.

### Webhook (HMAC-signed)

The `WebhookHmac` backend POSTs a JSON payload to your endpoint, signed with
**HMAC-SHA256** using a shared secret. Verify the signature server-side before acting.
This lets you drive any password store Mailwoman does not natively support.

## Sealed upstream credentials

Mailwoman stores upstream account credentials (IMAP/SMTP passwords, OAuth refresh
tokens) **sealed**. When those are sealed under a key derived from the account
password and the password changes, the server **re-encrypts (re-seals)** them under
the new secret on a successful change (`reencrypt_credentials`). This is a
server-relayed operation on already-sealed material; the server does not learn the
plaintext credentials in the process.

## Zero-access accounts: key-hierarchy re-wrap

For zero-access accounts (V6, `mw-crypto`), the account password wraps a client-side
key hierarchy. Changing the password requires **re-wrapping** that hierarchy under the
new password. This ceremony runs **client-side** in the crypto worker; the server only
relays ciphertext and never sees the keys or the password.

Because a lost password would otherwise make a zero-access account unrecoverable, the
UI **offers the recovery-key path before the change proceeds** — the user is prompted
to have their recovery key available first. This ordering is enforced.

## Forced change on next login

An admin can set a forced-change flag (persisted in migration 0008). On next login the
user must change their password before continuing. The policy display and the same
backend path apply.

## Configuration

The password-change backend and its parameters are stored per deployment (migration
0008 `passwd_config`). The server exposes `POST /api/password`; on success it performs
the re-seal, and the web zero-access module runs the re-wrap when required.

There is also a `mailwoman password` CLI subcommand for administrative/local password
changes.

## Scope boundary (honest)

- Password **change** and **LDAP-bind login** are in scope. Enterprise **OIDC/SAML
  SSO is not implemented** and is a tracked 1.0 gap (`docs/ROADMAP-1.0.md`).
