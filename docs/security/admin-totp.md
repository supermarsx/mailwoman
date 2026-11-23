# Administrator authenticator app

An administrator can protect their console sign-in with an authenticator app
(TOTP). The factor belongs to the administrator account, not to any mailbox with a
similar name, and is managed by the administrator themselves under **My
preferences** → **Personal security** → **Authenticator app**.

![The Authenticator app section of Personal security in My preferences, with the Manage authenticator and recovery codes button.](/screenshots/security/admin-totp-preferences.webp){.mw-crop}

## What the factor is

- **Algorithm:** RFC 6238 TOTP with HMAC-SHA1, 6 digits, a 30-second step, and
  one step of clock skew accepted on each side. These are the parameters most
  authenticator apps use by default.
- **Secret:** 20 random bytes generated on the server. The confirmed secret is
  stored in the `admin_totp` table sealed with XChaCha20-Poly1305 under the
  server key (`MW_SERVER_KEY`). Anyone who has both the database and the server
  key can read it.
- **One use per code:** the server records the last accepted time step and refuses
  a code from the same or an earlier step, so a code cannot be replayed.
- **Recovery codes:** 10 codes of 10 characters from a 30-character alphabet,
  stored only as Argon2id hashes. They are shared by all of the administrator's
  second factors (authenticator app and passkeys).

## Set up the authenticator

Setup requires a local administrator account. An identity whose sign-in
credentials come from the server configuration and has no local account sees
"Your sign-in credentials are managed by the server configuration" and cannot
enrol here.

1. Open **My preferences** → **Personal security**, select **Manage authenticator
   and recovery codes**, then **Set up authenticator**. The console asks you to
   confirm it is you first (**Confirm it is you**: password plus any enrolled
   factor, or a user-verified passkey).
2. Scan the QR code or type the **Manual setup key** into your authenticator app.
   The QR code is drawn in your browser from the setup URI; the setup key is not
   sent to any other service. The server answers with `Cache-Control: no-store`.
3. Enter the six-digit code in **Authenticator code** and select **Confirm
   authenticator**.
4. If this is your first second factor, save the recovery codes that appear and
   select **I have saved my recovery codes**. They are shown only once.

![The authenticator setup step: instructions, the QR code and manual setup key (greyed out in this image), and the field for the six-digit authenticator code.](/screenshots/security/admin-totp-setup.webp){.mw-crop}

Limits of the pending setup:

- It expires after ten minutes.
- It is bound to the console session that started it; confirming from another
  session fails.
- Five wrong codes discard it; start setup again.
- Starting a new setup replaces the account's pending one.
- An account that already has an authenticator cannot enrol a second one; remove
  the existing one first.

Confirmation runs in one database transaction that checks the account is still
enabled, the session is still live, and no authenticator exists yet; it then
stores the sealed secret and, only if the account has never had recovery codes,
creates a recovery set. An existing set (including used codes) is kept.

## How the factor is used

- **Sign-in:** once an account has an authenticator or a passkey, a correct
  password alone does not start a session. The server returns a one-time pending
  token (valid five minutes, five wrong codes) and the session starts only after a
  valid code, a passkey, or a recovery code. The account is checked again (enabled,
  password unchanged) before the session is created.
- **Recovery code:** a recovery code together with the password can complete one
  sign-in. A recovery code can also confirm a sensitive action when the
  authenticator is lost. Each code works once.
- **Sensitive actions:** see [Confirmation of sensitive actions](./admin-panel.md#confirmation-of-sensitive-actions).
  Adding, removing or changing a factor invalidates earlier confirmations.

## Remove the authenticator or replace recovery codes

- **Remove authenticator** asks for a fresh confirmation, which can use the
  authenticator, a passkey or a recovery code. Passkeys are not affected.
- **Replace recovery codes** creates a new set after a fresh confirmation and
  invalidates every unused code of the previous set.

Another administrator who holds `admins.manage` and could grant every capability
your account holds can remove your authenticator, or set a new one and
receive its setup key, from **Admin accounts** → **Password and authenticator**.
Both actions require their own fresh confirmation and are written to the audit
log. Together with a password reset, this lets such an administrator take over
your account, so grant `admins.manage` only to people who are trusted with every
account it covers.

## Multiple server processes

Pending setups, pending sign-ins and confirmations are held in the memory of the
server process that created them. Behind a load balancer with several processes,
use session affinity, or a setup or sign-in may fail on the second request.
Confirmed factors and recovery codes are in the database and work on every
process.

## API

Relative to the console path (see [Console path](./admin-panel.md#console-path)):

| Request | Result |
|---|---|
| `POST /me/totp` | Begins setup; returns `{secret, otpauthUri}`. Requires a fresh confirmation. |
| `POST /me/totp/confirm` with `{code}` | Returns `{ok: true, recoveryCodes: [...]}`; the array is empty when an existing recovery set is kept. |
| `POST /me/recovery` | Replaces recovery codes after a fresh confirmation; returns `{recoveryCodes: [...]}`. |
| `DELETE /me/totp` | Removes the factor after a fresh confirmation; returns 204. |

No endpoint reads back a saved secret. The secret is returned only during a pending
setup.

## Related

- [Admin console security](./admin-panel.md)
- [Passkeys](./passkeys.md)
- [Require two-factor](/admin/authenticator)
