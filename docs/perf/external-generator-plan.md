# Separate generator and application resource budgets

The test-only external driver and completion-file protocol are implemented.
Default fixture behavior still starts Node as a child; explicit external mode
instead waits for a separately budgeted sidecar. Node control/driver tests cover
private files, nonce/expiry validation, errors and report-byte binding. Rust/Linux
container integration remains a separate verification gate. No capacity acceptance
is implied by the harness being available.

## Private network, independent cgroups

Run the fixture application in a named container, binding only an ephemeral
`127.0.0.1` port and publishing no host port. Start a separate Node sidecar using
Docker `--network container:<fixture-container>`. This shares the application's
network namespace, so its existing literal-loopback origin remains private,
while the sidecar keeps independent CPU/memory limits. Avoid `docker exec` for
the measured generator: it would remain in the application's resource group.

Mount one newly created per-run scratch volume into those two containers only.
Use the same unprivileged UID, a 0700 directory and 0600 secret files. No source
directory, production database, Docker socket or unrelated host path is mounted
into the generator. Record container IDs and volume name for exact-owned cleanup;
do not use broad prune/delete operations.

## Handoff and completion contract

1. The application starts its disposable SQLite store or UUID-isolated local
   PostgreSQL schema, creates real fixture sessions, and runs endpoint preflights.
   It atomically publishes a versioned private handoff file in the shared volume.
2. The handoff identifies a fresh run nonce, expiry, literal-loopback origin,
   selected scenarios, resource labels and private session/diagnostic credentials.
   A larger multi-user credential manifest uses a bounded separate private file;
   only an owned relative basename may be referenced. No DSN is needed by Node.
3. The application waits for completion instead of spawning a Node child. The
   external generator reads the handoff, verifies version/nonce/expiry/origin,
   warms the specified workload, then runs the same campaign implementation.
4. The generator publishes an aggregate progress heartbeat every five seconds
   and writes redacted JSON/Markdown reports. It atomically publishes a small
   completion record containing the run nonce, result classification and report
   hash, never cookies, session IDs, account IDs, request bodies or headers.
5. The application accepts only that run's regular completion file, after bounded
   size/path checks. Expiry, missing heartbeat, container failure or a mismatched
   nonce ends the run as incomplete. A hard campaign deadline remains authoritative.
   It then closes the listener/pools and removes only its generated PG schema.

Use temporary files plus flush/atomic rename for control records. Refuse links,
path traversal and stale completion records. Keep credential files out of logs,
artifact uploads and public reports. Delete the per-run scratch volume only
after both owned containers have stopped and reports have been copied out.

## Resource interpretation

Record separate application, PostgreSQL, cache and generator quotas, actual
cgroup usage, generator CPU/RSS, pool waits and host pressure. The initial
application profiles are four vCPUs / 4 GiB and eight vCPUs / 8 GiB. An eight-vCPU
quota on a four-to-eight-shared-core host is not eight dedicated cores: colocated
database and generator activity still competes with it. Do not treat the sum of
container CPU quotas as available physical CPU capacity. Keep total memory
commitment within the host's 16 GiB, including OS and page cache headroom.

If the generator saturates before the application, use bounded worker processes
or another load-generator machine. Merge histogram buckets and counts, never
average percentiles or sum overlapping phase rates. Report generator scheduling
loss and server refusals separately. A server capacity claim requires sustained
validated successful operations, the stated latency/error budgets and a load
generator that actually delivered the intended workload.

## Distinct user and connection experiments

First establish per-endpoint and mixed-request saturation with 30-second or
longer warmed phases. Keep default-policy runs separate from explicitly tuned
but supported admission budgets. Profiling runs diagnose costs; profiling must
be disabled for final throughput acceptance.

The native-engine fixture seeds distinct account principals and representative
mail using public Store/Engine APIs. Its sessions
have normal identity/account bindings and pass production authentication;
no production authentication bypass or cross-request auth TTL is introduced.
Account/session provisioning and login throughput are measured separately from
the warmed API phase. Pin mailbox/message counts, body sizes, cache posture,
operation mix, selection seed and per-principal request distribution in reports.

Measure 10,000 authenticated WebSocket sessions separately from 40,000 active API
operations/sec. A connection test needs bounded handshake concurrency, refusals,
established/live counts, memory/socket usage, disconnects, session rechecks and
push-delivery completeness/latency. Ten thousand connections sharing one cookie
are a fan-out experiment, not ten thousand verified users. The combined capacity
target requires the intended distinct-principal population and sustained
successful API traffic while those sessions remain live.

## Implemented invocation

In the fixture application container set:

```sh
MW_LOAD_FIXTURE_EXTERNAL_DRIVER=1
MW_LOAD_FIXTURE_EXTERNAL_ROOT=/scratch/control
MW_LOAD_REPORT_DIR=/scratch/reports
MW_LOAD_FIXTURE_PROTECTION=off
```

The scratch directories must exist, be owned by the app/generator UID and not be
writable by other users. The app creates a 0700 `run-<UUID>` directory, publishes
its private handoff and `ready.json`, and waits for a nonce-matching completion.
Credentials are removed when that campaign returns, including failure. Allow
10–120 seconds for sidecar startup using
`MW_LOAD_FIXTURE_EXTERNAL_STARTUP_SECONDS` (default 60). The campaign itself
retains its independent bounded deadline.

Start the generator from the same versioned performance image, with the same
scratch volume and private network namespace but separate cgroup quotas:

```sh
docker run --rm --network container:FIXTURE_CONTAINER \
  --cpus=1 --memory=1g \
  --mount type=volume,source=OWNED_RUN_VOLUME,target=/scratch \
  --entrypoint node PERFORMANCE_IMAGE \
  /src/scripts/perf/http-external-driver.mjs \
  --root=/scratch/control --campaigns=1 --wait-seconds=1800
```

Replace the three uppercase placeholders with the specifically owned fixture
container/image/volume. Publish no ports and mount no Docker socket. If the
application runs both protection variants sequentially, use `--campaigns=2`;
separate single-variant processes are preferable for controlled comparisons.
The sidecar forwards only whitelisted campaign settings from the private control
record, not database credentials or application keys. Child output is not copied
to public logs. Heartbeats are atomic private files; thirty seconds without a
fresh heartbeat ends an already claimed campaign as incomplete.

The private native handoff may contain up to 10,000 distinct principal entries:
`cookie`, optional `csrfToken`, `accountId`, optional `ownerAccountId`, and seeded
`messageIds`. Request templates are prepared outside timed phases, and principals
rotate under five fixed endpoint labels. Reports include distinct attempted and
successfully validated principal counts without their identifiers. An HTTP
multi-user target and persistent WebSocket acceptance remain separate report
fields; the HTTP runner never invents successful WebSocket measurements.

## Debug and release images

The isolated performance Dockerfile defaults to the native multi-user fixture
and four build jobs. Release remains the default compiler profile:

```sh
docker build -f scripts/perf/Dockerfile -t mailwoman:perf-native .
# Debug correctness experiments use the same multi-user fixture:
docker build -f scripts/perf/Dockerfile --build-arg MW_FIXTURE_PROFILE=debug -t mailwoman:perf-native-debug .
```

Both images accept `MW_NATIVE_LOAD_PRINCIPALS=100` or `10000` at runtime.
Choose `1` explicitly for a single-user diagnostic. The loopback-trusted-proxy
fixture models distinct client addresses; it does not create physical hosts or
measure a real network between machines. Use the separate generator container
above to isolate its CPU and memory from the application. Debug results remain
correctness and contention diagnostics rather than release capacity estimates.

The older single-user proxy fixture is available explicitly with
`--build-arg MW_FIXTURE_KIND=proxy`. Baseline worktrees without the optional
native-engine fixture must use that option; their results cannot establish
multi-user capacity.

`prepare-fixture-overlay.mjs` now copies the external-control helper and sidecar
scripts as test-only harness files, while preserving the older backend and its
baseline diagnostics adapter.
