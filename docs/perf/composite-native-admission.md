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

## Connector credential source follow-up (source only)

The snapshot now includes a separate deferred account sealed-credentials section
joined through the SAME current primary session/account. It shares the original
Store credential/revision decoder with `account_credentials_with_revision`.
`NativeAccountCredentials` Debug is redacted; its revision is the existing opaque
randomized sealed bytes, never a newly invented hash or a credential memo.
Engine runtime rejection and the original conditional rejection update must use
those exact bytes. Invalid ciphertext fails at the connector's original boundary;
oversized bytes are masked by the SQL aggregate guard before decoder/transfer.

Two additional source-authored cases cover SQLite and real UUID-schema
PostgreSQL replacement during preparation, stale rejection CAS false versus
current true, current credential_rejected policy visibility, malformed/deferred
ciphertext and over-budget pre-transfer masking. The snapshot filter now has
EIGHT cases (two actual-PG cases when MW_TEST_PG is configured); the warm-policy
filter remains TWO. These follow-up changes have not executed. The earlier
24-case root gate verifies the preceding source only and is not evidence for
this follow-up or handler integration.

No handler shortcut has landed. In particular, a registered runtime still runs
the full existing `engine_mode::ensure_account`: account gate, complete account,
credentials/rejection revision, configured upstream/account match and domain
binding checks under connect_lock. A later candidate must replace those checks
from this coherent source, not omit them because `is_registered` is true.


## Isolated first handler integration (source only)

The follow-up candidate attempts the actual `/jmap/api` handler only for an
ordinary cookie with deferred feature requirements and one `Email/get` call:
explicit 1..50 IDs, at most64 properties, and the production Engine metadata
predicate. That predicate includes the real default36 properties, authentication,
Focus, Mentions, priority and safe preloaded optional/delivery headers. Reader
body properties, result references, bearer/API keys, linked selectors and SSO
realms retain the original path. This is not whole-HTTP two-read delivery.

The immediate warm common admission path performs a fresh primary hint and TWO
composite statements: a preparation observation, then a final observation after
original JMAP permit acquisition and connector mutex acquisition. A bounded
extra shaping observation supports newly observed account/sibling names. The
original permit-before-connector order is retained; no connector guard spans a
permit queue. The proposed immediate two-read callback optimization is NOT
implemented. Earlier middleware and Engine data/posture reads remain additional
physical statements and must be measured separately.

Up to32 same-account sessions are represented, with a33rd-row sentinel selecting
the complete old path. This reuses the original optional login-name decoder:
damaged ciphertext retains its clear username but contributes no invented
credential name. The only transferred sibling fields are username and sealed
credentials, within the same aggregate8MiB SQL guard; unrelated accounts cannot
join or consume that budget. `idx_sessions_account` covers the account predicate.
Window ordering/sorting and hidden subquery/pager cost are not constant-time
claims; an actual full composite EXPLAIN regression is authored but unexecuted.

Only fully warm, exact Store/revision inputs enter shared pure registry
resolution. Global/domain clocks, both session and connector account gates,
canonical domains, configured transport, active primary namespaces, account
credentials and rejection revision, binding, registered runtime and existing
process guard remain checked. Cold/unsupported/error/deny before permit
acquisition returns to complete original authentication/connector handling.
After permits are acquired, changed/unclear inputs cannot dispatch: ordinary
fresh authentication preserves revocation/expiry handling, then the existing
typed pre-dispatch refusal is returned if that chain succeeds. No authority,
credential, Session or allow TTL cache is introduced.

The existing request-protection admission is reused only if its own per-request
context proves no account policy applies or the same native account has already
been charged. An uncharged or contended context uses the complete old path.
Actual Engine ownership-before-decrypt and fresh posture checks remain in the
unchanged data handler; the composite is not a SQL mutation lease.

Authored execution gates (not run for this follow-up):

- Store `native_admission_snapshot::tests`:11 tests, including3 PostgreSQL
  cases when `MW_TEST_PG` is privately configured; one actual SQLite full-query
  EXPLAIN, sibling-name parity/cap/owner exclusion, and existing credential,
  revision, corrupt-data and held-directory cases.
- Settings `feature_inputs_resolution_tests`:1 pure-loop parity case with extras,
  unknown keys, family discovery/order and locked override behavior.
- Server `feature_gate::native_metadata_admission::tests`:8 cases, including a
  real50-message/default36-property router oracle which must prove candidate
  dispatch, equal answers and lower whole-request physical reads. It compares
  to the current ordinary deferred route, not an artificially weaker guard.
  Additional actual-router cases cover a really queued permit, connector waits,
  independent writer disable/domain/kind/rejection changes, restrictive policy
  changes and a sibling-only credential-name deny. Eligibility covers the full
  fixture property source and rejects reader/result-reference requests.
- Existing account-gate, domain parsing/resolver, connector binding and clock
  policy suites must run because their original readers now invoke the same
  extracted pure evaluators. Source parsing alone is not runtime verification.
