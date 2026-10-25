# Administrator authenticator setup

Administrator MFA belongs to the admin account, independently of any mailbox account with a similar username. Administrators manage their own authenticator from personal security settings. A local administrator account is required; an externally managed identity without a local account cannot enroll through these endpoints.

Starting setup requires the central sensitive-action authentication proof. The server returns an authenticator provisioning URI for a locally generated QR code and a manual setup key, with `Cache-Control: no-store`. Keep the setup key private. The factor becomes active only after a valid code is confirmed in the same administrator session that started setup. Setup expires after ten minutes and is discarded after five incorrect confirmation attempts. A new setup replaces that account's pending setup.

Confirmation atomically stores the sealed TOTP secret, consumes the confirmation code, and creates a recovery-code set if none was previously issued. Recovery codes are displayed once; the database stores only their hashes. Existing recovery codes are preserved when adding another factor. Explicit recovery-code rotation replaces the previous set and is available to administrators with either TOTP or a passkey.

Adding or changing factors invalidates older sensitive-action proofs. Password sign-in then requires an enrolled factor. A recovery code can complete password sign-in once, and another fresh recovery code can confirm a sensitive action when the authenticator is unavailable. Removing the authenticator uses that central proof, so the removal endpoint does not request the consumed TOTP code again. Passkeys remain independent and are not removed by authenticator removal.

Pending setup and short-lived authentication proofs are process-local, so deployments with multiple server processes need session affinity for these ceremonies. Confirmed factors and recovery codes are durable; enrollment checks the current enabled account and live session under database locks. It cannot overwrite an already enrolled authenticator.

API endpoints, relative to the configured admin prefix:

- `POST /me/totp`: begin setup; returns `{secret, otpauthUri}`.
- `POST /me/totp/confirm` with `{code}`: returns `{ok: true, recoveryCodes: [...]}`; the array is empty when preserving an existing recovery set.
- `POST /me/recovery`: rotate recovery codes after a fresh proof; returns `{recoveryCodes: [...]}`.
- `DELETE /me/totp`: remove the factor after a fresh proof; returns 204.

These self-service endpoints do not read back a saved TOTP secret. The setup secret is returned only during the pending enrollment ceremony.
