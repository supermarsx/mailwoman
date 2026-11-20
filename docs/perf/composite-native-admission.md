# Composite native admission prerequisites

This additive candidate does not change request routing, authentication,
configuration defaults, admission policy or any existing guard. It is source
work only until the listed gates execute. No throughput or whole-HTTP two-read
claim follows from this change.

`Store::native_cookie_admission_snapshot` executes one fresh SELECT returning
independently deferred session/credential, persisted clock, full native account,
verified principal/binding, linked-target classification, known names, full
one-hop domain configuration, ordered held-directory, exact flags, ZeroAccess
and restriction sections. `NativeAdmissionRequest` only shapes bounded domain
and exact-key queries; it never establishes identity or proves name coverage.
No caller is admitted by this API. Snapshots contain no cached allow, no TTL and
no held SQL transaction. Debug output excludes private values.

The SQL uses a metadata-only size union and an aggregate payload gate before
returning source fields. Inputs allow16 domains/256 keys, at most1024 bytes per
parameter and128KiB total JSON. Held lookup retains the existing ordered5001
sentinel. Restriction lookup retains65 rows;65 explicitly requires fallback,
not a truncated no-restriction result. The payload budget is8MiB including a
conservative6x text allowance and fixed row overhead; source byte metadata is
BIGINT before multiplication. Row count is bounded independently. This is not
a hard RSS, query-work, disk-I/O or SQLite-overflow-page ceiling. All returned
payloads are masked on aggregate overflow. Invalid storage is deferred to its
section; full account/domain/privacy decoders are reused. Clock decoding is
separate from credential decoding to retain the original error boundaries.
`last_seen` is non-authoritative telemetry: text over1024 bytes or unreadable
storage returns unknown, conservatively preserving the original touch path.
For that unsupported telemetry value this is intentionally not raw-field parity;
it never changes session activity/creation clocks or authenticates a request.

`Store::try_settings_resolution_inputs_at` never performs I/O. It copies only
existing bounded canonical-C1/C2, all-policy and scoped-value memos at the EXACT
present Store-bound revision (zero is valid). Both canonical passes must be
idempotent. Missing, dirty, cold, oversized or foreign inputs cannot fall back
to a prior policy decision. Existing pure registry resolution still owns
defaults, environment precedence, extra/family keys and locks.

A future opt-in route must validate every fresh source relation and that all
prepared names/flag keys cover the observed credentials/account-names source;
perform its complete pure evaluation after body and queued permits; and use the
UNCHANGED complete fresh chain for unsupported/cold/error cases. No policy or
authority awaits may follow its final successful composite boundary. Existing
process SessionGuard, step-up, source/geo/theft, protection and concurrency
controls remain enforced by current code and are not represented by a Store
settings revision. This stage deliberately does not replace them.

Two total HTTP statements additionally require early guard ownership: current
protocol/protection and other early readers cannot simply reuse an earlier
revision after body/permit waits. A later route-owned design can combine early
guard preparation in one statement and fresh final authority plus bounded
owner-scoped data in another. That is not implemented here.

Authored focused gates, NOT executed:

- `cargo test -p mw-store --lib native_admission_snapshot::tests --locked -- --test-threads=1`:
  six cases, including an environment-gated real PostgreSQL UUID-schema contract
  (a missing fixture is explicitly not actual PostgreSQL evidence).
- `cargo test -p mw-store --lib native_admission_warm_policy::tests --locked -- --test-threads=1`:
  two cases for no-I/O warm parity, cold/dirty/foreign scopes and zero/missing
  revision plus non-idempotent canonical fallback.

Before integration into routing, additional real-router independent-writer,
body/permit-wait revocation, process/security guard parity and physical SQL trace
verification are required. Static statement construction is finite (two pinned
backend shapes); fewer roundtrips does not remove the joins, scans or sorts.
