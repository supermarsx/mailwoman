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

Administrator passkey login, enrollment and recovery are a separate integration
scope; mailbox verification alone does not establish support for those flows.
