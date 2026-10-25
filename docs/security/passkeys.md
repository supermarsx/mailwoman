# Passkey verification

Mailwoman requests WebAuthn `none` attestation. It verifies ES256, Ed25519, and
RS256 assertions; RSA public keys must have a 2048–8192 bit modulus. It does not
validate enterprise or manufacturer attestation chains.

Registration and assertions bind the challenge, origin and RP ID. Cross-origin
ceremonies are refused. Discoverable mailbox registration and passwordless login
require user verification. A passwordless assertion must return the account's
user handle; an assertion made after identifying the account may omit it, but a
supplied handle must match. New handles are opaque 32-byte values. Existing
credentials using the account ID as their handle remain compatible.

Credential IDs are insert-only: registering an existing ID cannot replace another
account's verification key or reset a counter. Assertion updates compare the
account, key, previous counter and state revision atomically. Concurrent stale
assertions and assertions racing deletion do not establish a session.

Backup eligibility is immutable once recorded. Backup state may change but cannot
be set without backup eligibility. Credentials enrolled before this metadata was
stored learn their eligibility from their first verified assertion. Device-bound
credentials reject counter regression. Multi-device credentials tolerate counter
anomalies, retain the largest stored counter, and emit a diagnostic log containing
only a hashed credential handle. This accommodates synced authenticators without
silently rewriting the stored counter backwards.

These rules follow [WebAuthn Level 3](https://www.w3.org/TR/webauthn-3/), particularly
authenticator data and registration/assertion verification. Browser tests use
Chromium's virtual CTAP2 authenticator. They do not establish interoperability with
physical hardware, mobile cross-device ceremonies or every authenticator vendor.

Administrator credentials use a separate table and user-handle namespace. An
administrator enrolls and removes passkeys from **My preferences** after fresh
step-up verification. Enrollment and sign-in require user verification and a
discoverable credential. Enrolling a passkey prevents password-only sign-in and
password-only step-up. An existing authenticator code remains an alternative
second factor; a user-verified passkey can satisfy step-up on its own.

Administrator recovery codes work once, together with the administrator password.
They can complete sign-in or step-up when the authenticator is unavailable.
Replacing recovery codes atomically invalidates the previous set. Codes are shown
only when generated; the database stores Argon2 hashes. Removing or disabling an
administrator prevents their passkeys from authorizing new assertions.

Enrollment challenges are bound to the administrator session. Credential mutation
invalidates previous step-up proofs, and the registry records enrollment, removal
and recovery rotation without recording codes or key material. A mailbox passkey
cannot authorize an administrator session, or vice versa.

Pending ceremonies and short-lived step-up proofs are process-local. Deployments
with multiple application instances need session affinity during a ceremony;
credential ownership, revocation and counter updates are enforced in the shared
database across processes.

Administrators with authority over the target can revoke all of that target's
passkeys from **Admins → Password and authenticator**. This requires fresh step-up
and typing the account name. The reset removes recovery codes and active sessions
in the same database transaction. Passkey and recovery sign-in recheck the verified
factor under the same account lock when inserting the session, so a reset cannot
be bypassed by an assertion that finished verification just before the reset.
Fresh-browser sign-in endpoints do not require a pre-existing CSRF cookie; Origin
checks still apply. Signed-in enrollment, removal and step-up retain CSRF checks.
