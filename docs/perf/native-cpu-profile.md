# Native mail-workload CPU diagnostic

The 2026-11-20 profile identifies costs to investigate; it does not establish the
40k-RPS or 10,000-user target. It uses the frozen `72d9f20e` source snapshot from
the [projection comparison](list-projection-batching.md#measured-follow-up), with
the separate retained-symbol release executable. Later admission and sync-history
fixes are absent from this runtime.

The workload had 100 users, 5,000 messages and the actual UI-default properties.
Two generator workers shared a 60-second window at 16 outstanding requests.
Request profiling instrumentation was disabled. Locked policy revocation and
restoration, account isolation and the scanner probe passed. Application and
generator exited zero, with no errors, throttles or timeouts.

| Run | Successful in-window RPS | p95 / p99 |
| --- | ---: | ---: |
| Without sampling | 116.53 | 261 / 304 ms |
| With sampling during part of the window | 109.03 | 286 / 329 ms |

These are individual diagnostics, not a controlled estimate of sampling overhead.
The app had 4 CPUs/4 GiB and the generator 2 CPUs/1 GiB. The sampling sidecar had
1 CPU/512 MiB, its own cgroup and only the owned fixture's PID namespace. It used
`PERFMON` and `SYS_PTRACE`, default seccomp, no-new-privileges, no network, a
read-only root filesystem and a private output volume. No host PID namespace,
privileged container or application-security bypass was used.

The exact fixture worker was selected using its executable and internal worker
marker without logging its environment. Its start-time identity remained stable.
`cpu-clock:u` sampling at 99 Hz captured 4,943 samples and zero lost samples.
Three thread-open warnings are retained; complete thread coverage is not claimed.
The requested recording was 20 seconds, the wrapper's wall span 21.061 seconds,
and perf's sample timestamp span 22.209 seconds. The wrapper interval fell inside
the coordinated workload window, from 11.817 to 32.878 seconds into that window.

| Function family | Self sample share |
| --- | ---: |
| SHA512 AVX2 compression | 16.06% |
| Tantivy FST registry entry | 8.01% |
| `malloc` | 2.75% |
| Tantivy FST stream traversal | 2.73% |
| Stored-document decompression | 2.27% |
| FST registry vector extension | 1.98% |
| AES256 decryption | 1.72% |

The flat report completed offline. Full and time-sliced DWARF caller reports
timed out, so these are self samples without caller attribution. Source inspection
shows SQLCipher's Rust provider uses SHA512 page HMAC; the crypto hotspot is
consistent with that implementation. Encryption integrity remains enabled.

Tantivy constructs an FST when weighting each term-set projection query, while
real flag and move operations also write and commit index documents. The current
50-MB writer budget selects three indexing workers on this four-CPU fixture;
Tantivy replaces indexing workers at every commit. These are separate potential
costs. Bounded release microbenchmarks compare query construction and indexing
worker counts before selecting production changes. A lower microbenchmark time
does not by itself prove an HTTP throughput gain.

Public aggregate evidence is in
`.orchestration/reports/perf-investigation/sqlite-projection-cpu-sampled-100-workers2/`.
Raw stack data remains private under `.orchestration/private/`. Dataset,
TLS/upstream/WebSocket and whole-host limits from the earlier diagnostic still
apply. The target remains unachieved.
