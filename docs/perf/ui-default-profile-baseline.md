# UI-default two-worker diagnostic

This is a successful **100-user diagnostic**, not the sustained 40k-RPS or
10,000-user acceptance result. Both the application fixture and external
generator exited zero, and Rust verified/exported the paired JSON and Markdown
reports. The earlier run with the missing Markdown artifact remains incomplete.

The release image is
`sha256:3eaa6ef1ca3067780ab328b06d2b7b378408c8691f83f58f67d2d7140f9a4748`.
Its server executable is the frozen `945bb8dd` plus `ad24da31` diagnostic-counter
snapshot, SHA-256
`0cbf656b5a1cf4ba38788cf10d4a806a2495eef8e2ec5fca4b509bed56913ad5`.
The Node controller includes the paired-artifact fix `7f0d6d36`. Request profiling
was enabled, so these are instrumented observations, not unprofiled capacity.

The application had 4 vCPUs/4 GiB/no swap on CPUs 0–3. The separate generator
cgroup had 2 vCPUs/1 GiB/no swap on CPUs 8–9, shared by its controller and two
workers. Workers had disjoint 50-user partitions and one common measurement
window. There were 5,000 indexed messages: 50 per account, one sender and no Sent
history. The latter does not exercise high sender/contact cardinality or Sent
correspondence classification. No TLS, reverse proxy, upstream network, sends or
simultaneous persistent WebSocket workload was measured. The Docker VM had more
memory than the target whole host; its application memory cap is not proof of a
16-GiB whole-host deployment.

The workload used the actual UI default 36 list properties and 38 reader
properties, with their fetch options recorded in `requestPropertySets`. This is
heavier than the historical five-field projection. Both sampled cross-account
reads returned only owned duplicates and foreign `notFound`. Locked protocol
policy revocation/restoration passed; the scanner probe was enforced with 429.

| Endpoint | In-window successful RPS | p95 / p99 | Users reached | Errors / throttles / timeouts |
| --- | ---: | ---: | ---: | ---: |
| Identity | 1,049.8 | 26 / 33 ms | 100 | 0 / 0 / 0 |
| UI-default read/write mix | 112.5 | 316 / 347 ms | 100 | 0 / 0 / 0 |

Each phase lasted ten seconds at 16 globally outstanding requests. Worst worker
start lag was 0.32 ms. Application observations were 2.94–3.18 logical CPUs and
271–314 MiB RSS; worker CPU observations were 0.71 and 0.18 cores respectively.
The latter exclude controller CPU, which the report records separately.

Request-local telemetry observed 25 reads per identity request and about 70.4 per
mixed request. The mixed phase performed about 10.9 legacy exact-document index
searches per HTTP request. These counts support the next bounded projection
batching work; timings include waiting and do not isolate CPU cost. The later
shared projection `12dfdd75` and background-download intent `99b89bfd` were not in
this measured server. Their gains and full HTTP behavior require later images.
The [subsequent projection diagnostic](list-projection-batching.md#measured-follow-up)
records fewer index searches without a measured throughput improvement.

Local evidence is under
`.orchestration/reports/perf-investigation/sqlite-scoped-ui-profile-100-workers2-complete/`.
Reports and infrastructure metadata contain no credentials; private handoffs and
generated keys remain separate. The 40k goal remains unachieved.
