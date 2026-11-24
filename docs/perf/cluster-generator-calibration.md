# Multiprocess generator calibration

This optional prototype measures a constant-response HTTP transport and its
generator. It exercises no Mailwoman authentication, policies, database, mailbox
operations or upstream server. Every report explicitly sets application capacity
and security claims to false, including a successful 40,000-RPS synthetic phase.

The server starts 1–16 independent Node data workers, each listening on an
ephemeral literal `127.0.0.1` port, plus a separate private control listener.
Only a newly created 0600 handoff carries the authority token and listener ports.
Worker configuration travels over private IPC, never command-line tokens or
inherited application/DSN environment. Its lifetime is bounded to 10–900 seconds.
The primary waits for every worker to acknowledge reset and collects every
worker's response/CPU/memory counters before publishing an aggregate snapshot.

The client starts 1–16 processes, distributes requests over all data listeners,
and splits the **total** integer offered rate and outstanding budget exactly.
Every phase uses a common future start and end, and reconstructs percentiles from
merged raw histograms. Missing/crashed workers, mismatched budgets, invalid
histograms or excessive start skew reject the result. A process count represents
simulated clients on the recorded generator hosts, never extra physical hosts.

Freeze the scripts used in a run, including their imported `http-load.mjs`,
`http-multi-protocol.mjs` and original calibration protocol dependencies. The
current release image may contain older scripts; overlay a reviewed immutable
copy in a private directory rather than changing the measured server binary.

Example commands **inside separate explicitly limited containers sharing only
their owned network namespace and private handoff directory**:

```sh
node /perf/http-cluster-calibration-server.mjs --handoff=/control/cluster.private.json --workers=4 --seconds=300
node /perf/http-cluster-calibration-client.mjs --handoff=/control/cluster.private.json --output=/reports/cluster.json --workers=4 --seconds=3 --concurrency=256 --rates=10000,20000,40000,60000,80000
```

Bound server and generator CPU/RAM independently (for example, each at 4 CPU and
1–2 GiB initially), record their actual cgroup observations, and run without
compilers or unrelated load. Publish no ports and mount no Docker socket or
application data. The scripts do not require privileged mode or host PID access.
The CLI limits each phase to 1–3,600 integer seconds, at most five rates up to
250,000 RPS, and a total outstanding budget of at most 50,000 requests.

Payloads are the same exact approximately 4 KiB JSON and 64 KiB binary samples as
the single-process calibration, with complete response validation. HTTP RPS
counts actual requests, not payload bytes or worker count. Server counters mean
response-handler attempts and offered payload bytes, rather than confirmed
delivery. Client acceptance, errors, scheduler/capacity drops and completions
inside the common window remain separate.

Server resource observations include the common-start lead and drain interval.
The report also divides all observed server CPU time by the scheduled duration,
an upper bound rather than a lower utilization claim. Per-process RSS sums are
not synchronized peaks. A saturated synthetic server can still conceal generator
headroom: compare independently measured quotas/CPU, scale the synthetic server
workers and resources, and repeat. An authenticated `POST /__stop` on the control
listener closes the owned listeners and confirms data-worker shutdown.

An isolated Docker correctness smoke passed with two data workers and two client
workers, each container limited to 1 CPU and 512 MiB. It validated exactly one
JSON and one binary request per data port, aggregate reset/snapshots, real client
readiness/prepare IPC and cancellation, and clean server-worker shutdown. No
client start command, rate phase or sustained measurement ran. Both containers
exited successfully and their private volume was removed. This validates the
runtime wiring, not generator headroom or application capacity.

## Relation to the coordinated application generator (t53s)

The calibration scripts now use the same ceilings as the coordinated application
generator: up to 64 client workers, 50,000 outstanding requests and 3,600-second
phases (at most five rates up to 250,000 RPS; the synthetic server keeps its own
16-worker bound). The application generator can also merge workers from several
hosts with measured clock offsets and supports a `late=dispatch` policy; see
"Scale-out of the coordinated generator" in `generator-calibration.md`. The
multi-process path has still not been run at rate.

The calibration scripts themselves speak HTTP/1.1 only. The HTTP/2 mode of the
coordinated generator (see "HTTP/2 mode" in `generator-calibration.md`) has no
synthetic-server counterpart yet; a calibration for HTTP/2 needs its own
constant-response h2 server and must be run and reported separately from the
HTTP/1.1 one. No numbers are claimed.

