# Password change

Users can change their mail password from Mailwoman under **Settings** > **Security centre** > **Sign-in & passkeys** > **Change password**. Mailwoman does not change the password itself; it passes the change to a backend that the operator chooses. This page describes the backends, what the server sees and stores, and how a forced change works.

![The Change password form: Current password, New password and Confirm new password, the length rule, and the Change password button.](/screenshots/security/password-change-form.webp){.mw-crop}

## Choosing a backend

`MW_PASSWD_BACKEND` selects the backend. A missing required variable or an unknown value stops the server at start with an error naming the variable; nothing falls back silently.

| Value | What it does | Variables |
|---|---|---|
| unset or `local` | Replaces an Argon2id hash kept in Mailwoman's own settings table (`passwd_local:<account>`) | none |
| `ldap3062` | Sends an LDAP Password Modify extended operation (RFC 3062) | `MW_PASSWD_LDAP_URL` (required), `MW_PASSWD_LDAP_STARTTLS=1`, `MW_PASSWD_LDAP_BIND_DN`, `MW_PASSWD_LDAP_BIND_PW` |
| `dovecot` | Calls the Dovecot doveadm HTTP API | `MW_PASSWD_DOVECOT_URL`, `MW_PASSWD_DOVECOT_API_KEY` (both required), `MW_PASSWD_DOVECOT_COMMAND` (default `pw`) |
| `poppassd` | Speaks the poppassd line protocol over TCP | `MW_PASSWD_POPPASSD_HOST` (required), `MW_PASSWD_POPPASSD_PORT` (default 106) |
| `webhook` | POSTs a signed JSON request to your endpoint | `MW_PASSWD_WEBHOOK_URL`, `MW_PASSWD_WEBHOOK_SECRET` (both required; the secret must be at least 16 bytes) |

::: warning The default backend does not change a sign-in password
The `local` backend changes a hash that neither Mailwoman's sign-in nor your mail server reads, and nothing creates that hash in the first place, so a change through it fails with a wrong-current-password error. To let users change their mailbox password, configure one of the other backends.
:::

### Transport security of each backend

The new password is sent to the backend, so the connection matters:

- **ldap3062**: use an `ldaps://` URL or set `MW_PASSWD_LDAP_STARTTLS=1`. A plain `ldap://` URL without StartTLS sends the password unencrypted.
- **dovecot** and **webhook**: `http://` URLs are accepted, with a warning in the log that new passwords travel unencrypted. Use `https://`.
- **poppassd**: the protocol has no encryption; the password crosses the network in clear text. Use it only on a trusted local link.

### The webhook request

The webhook backend sends a JSON body with the fields `event`, `account_id`, `username` and `new_password`, and an `X-Signature: sha256=<hex>` header: HMAC-SHA256 of the exact body bytes under `MW_PASSWD_WEBHOOK_SECRET`. Verify the signature with a constant-time comparison before acting. The signature proves the request came from Mailwoman; it does not encrypt the password, so the endpoint must use HTTPS.

## What happens on a change

1. The browser sends the current and the new password to `POST /api/password` over the session's connection.
2. The server checks the new password against the policy (by default at least 12 characters) and refuses a new password equal to the current one.
3. The current password is checked: by the backend itself (`local`, `ldap3062`, `poppassd`), or, for `dovecot` and `webhook`, which do not check it, against the password stored with the user's signed-in sessions.
4. The backend makes the change.
5. A row is written to `password_change_audit` with the account, the backend and the outcome (success, or failure with its reason). No password is written to the audit row or to the logs.
6. If the user's sessions hold the upstream mail password (so the server can sign in to IMAP and SMTP for them), the server replaces it with the new password, sealed with the server key. If that step fails, the request returns an error even though the password was already changed.

The server handles both passwords in clear text in memory during the request, and it keeps the new mail password sealed with the server key, as it did the old one. An operator who holds the server key can therefore read stored mail passwords; see [Encryption at rest](./encryption-at-rest.md).

## Forced change

An administrator can require a user to change their password. Until the user does, the server refuses that account's requests except the password policy and the change itself, and the user sees "Your administrator requires you to change your password before continuing." A successful change clears the requirement.

## Command line

`mailwoman password --account-id <id> --old-password <current> --new-password <new>` (or the `MW_ACCOUNT_ID`, `MW_OLD_PASSWORD`, `MW_NEW_PASSWORD` variables) runs the same change through the configured backend on the host. Passing passwords as command-line arguments exposes them to other local users through the process list; prefer the environment variables.

## Zero-access accounts

Zero-access keys are derived from their own passphrase or passkey, not from the account password, so a password change does not affect them. The form in **Settings** is shown without its zero-access step: it does not display a recovery phrase or re-wrap any key, and the server stores nothing new for the zero-access keys during a password change. The confirmation mentions a re-wrap only when one ran, so it does not appear. See [Zero-access storage](./zero-access.md).

## Related

- [Encryption at rest](./encryption-at-rest.md)
- [Zero-access storage](./zero-access.md)
- [LDAP directory](/deploy/ldap)
- [Security centre (user settings)](/user/settings/account)
