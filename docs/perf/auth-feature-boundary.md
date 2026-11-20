# Cookie JMAP authentication boundary

For ordinary cookie `POST /jmap/api` requests, feature middleware passes only
request requirements and a Store-bound settings revision into the handler.
The handler resolves feature policy using an untrusted primary-session hint,
then runs the complete ordinary authentication chain. A final single-statement
source witness checks the session, credential ciphertext, native account
kind/presence and settings revision after all authentication awaits. Equality
with the original hint and the final Session is checked without another async
feature lookup. No authenticated Session or allow decision is cached globally
or reused across requests.

The initial hint decrypts credentials as required by existing subject resolution.
The final witness reads/hashes the same source without decrypting it again.
Normal authentication still performs its original credential decode. Session
activity timestamps are excluded from equality so ordinary throttled touches do
not cause spurious policy changes.

The existing clock, linked-login, domain routing, account flags, linked context
and request-protection checks remain intact. All feature decisions, including
refusals and read errors, cross final authentication before being exposed.
Feature errors remain separate from authentication errors and never enter the
handler's admin-metadata authentication-failure fallback.

Authorization headers, linked selectors, scoped REST principals, missing policy
revisions, other routes and declared oversized bodies keep the legacy path.
Changes to policy/subject/session while a request is being resolved return a
retryable HTTP503 instead of dispatching with stale feature authority. There
is no unbounded retry loop or SQL transaction held through request dispatch.
The witness is a fresh observation, not a lease against a later independent
write; the existing authorization boundaries are preserved.

Migration0078 already advances settings_revision for policy/value rows,
groups/membership, profiles/assignments, domain aliases, identity principals,
identity bindings and session principals. Exact primary-session source fields
and native account kind/presence cover the other feature-subject dependencies.

Focused verification sources are `primary_policy_snapshot::tests` (four cases,
including an owned UUID PostgreSQL schema via MW_TEST_PG) and
`feature_gate::auth_boundary::tests` (thirteen production-router cases). The
successful count regression prints actual query and complete-auth counts for
the legacy and candidate paths. Source preparation alone proves neither test
success nor an application throughput improvement.


The authenticated source-fence refusal is HTTP503 `application/problem+json`,
with type `urn:mailwoman:problem:authorization-not-confirmed`, machine code
`authorizationNotConfirmedBeforeDispatch`, `notDispatched:true`, `Retry-After:1`
and `Cache-Control:no-store`. Only the new pre-dispatch source fence emits this
contract. Actual authentication/feature refusals and downstream/Engine/SMTP
outcomes retain their own semantics. Existing read-only transport pacing can
honor Retry-After; this change adds no client mutation replay. A generic503,
transport exception or `outcomeUnknown` is never relabeled as this refusal.
