# mw-observability

Operational observation services independent from server application state:

- Resource capacity discovery (including Linux cgroups and fractional CPU quotas).
- Read-only process CPU/resident-memory and data-filesystem snapshots.
- Bounded latency histograms, task-local phase timing and cancellation accounting.
- Telemetry/analytics configuration and guarded analytics transport.
- Browser stack symbolication using bounded Vite source-map lookups.

The crate owns no HTTP routes. `latency::track` accepts a future and a status
classifier, while `latency::Persistence` lets the host connect durable metrics
without a reverse dependency on `mw-server`. The host installs that sink at startup.

`resource_snapshot::snapshot` receives an operator-owned data directory. CPU usage
is process CPU: 100% means one logical core. The first sample, counter resets and
intervals over two minutes produce no CPU percentage. Effective memory available
is conservative host/cgroup headroom, not exact container usage. Process resident
bytes and host memory are reported separately. Unknown values remain `None`.

No subprocesses, directory traversal, network calls or filesystem writes are used
to discover resources. The snapshot should run outside async executor threads;
server authentication and route capabilities remain the host's responsibility.
