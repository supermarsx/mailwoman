# Scaling and multi-location deployments

Mailwoman has shared durable storage and some distributed coordination primitives.
It does **not** yet provide a complete horizontally scaled or geographically
sharded mail service. Increasing the replica count is not sufficient to make
engine synchronization, live notifications and local search behave as one service.

This is a source audit, not a load-test result. Targets such as **20,000 sustained,
50,000 burst and 100,000 peak** must first specify whether they mean registered
accounts, simultaneously connected browsers, active mailbox sessions, requests per
second, or message operations per second. No throughput or user-count capacity is
claimed for the current implementation or for the default 4 GiB/four-core instance.

## Current capabilities and boundaries

| Area | Implemented | Boundary for multiple replicas |
|---|---|---|
| Durable account, mail and session storage | SQLite or shared PostgreSQL, with transactional writes. | Use PostgreSQL for a shared database. Transactions do not establish upstream connection ownership or distribute in-memory events. See [`mw-store`](https://github.com/supermarsx/mailwoman/blob/master/crates/mw-store/src/lib.rs). |
| Shared cache | Optional Redis/Valkey tier behind process-local memory, with store/loader fallback. | This is a cache, not an account coordinator or WebSocket event bus. Invalidating one process's memory cache does not actively evict another process's memory entries. See [`mw-cache`](https://github.com/supermarsx/mailwoman/blob/master/crates/mw-cache/src/lib.rs). |
| Scheduled jobs | Database task leases, owner renewal and stale-run recovery. | These coordinate registered scheduler tasks; they are not a blanket lease around every background loop. See [`jobs`](https://github.com/supermarsx/mailwoman/blob/master/crates/mw-server/src/jobs.rs). |
| Outgoing submissions | Durable compare-and-set attempt claims and explicit uncertain-delivery state. | Preserve those claims and outcome semantics through any worker separation; do not independently replay a send after a lost acknowledgement. See [`dispatcher`](https://github.com/supermarsx/mailwoman/blob/master/crates/mw-engine/src/dispatcher.rs), [`jmap`](https://github.com/supermarsx/mailwoman/blob/master/crates/mw-engine/src/jmap.rs) and [`store v2`](https://github.com/supermarsx/mailwoman/blob/master/crates/mw-store/src/v2.rs). |
| Upstream synchronization | Each process has its own account-runtime map, watchers and startup account registration. | No distributed account-owner lease or account-shard routing was found in that production path. Replicas can each register the same account and establish upstream connections. See [`engine`](https://github.com/supermarsx/mailwoman/blob/master/crates/mw-engine/src/engine.rs) and [`engine mode`](https://github.com/supermarsx/mailwoman/blob/master/crates/mw-server/src/engine_mode.rs). |
| Realtime browser notifications | Bounded Tokio broadcast channels feed local WebSocket/SSE clients and the local push dispatcher. | No cross-replica event transport was found in the push path. A change handled by one process is not automatically delivered to clients connected to another. See [`push`](https://github.com/supermarsx/mailwoman/blob/master/crates/mw-server/src/push.rs). |
| Admission and abuse controls | Durable request-protection budgets and sign-in failures serialize relevant transitions in the store. | Other token buckets, resource gates and diagnostic history remain process-local. Replicas can multiply those local budgets. With `MW_REQUEST_PROTECTION_ACCOUNTING=write-behind` (SQLite only), N processes on one file can together admit up to about (N − 1) times a subject's limit within one flush interval; see [optional modes](../admin/performance-modes.md#write-behind-request-protection-accounting). See [`store protection`](https://github.com/supermarsx/mailwoman/blob/master/crates/mw-store/src/request_protection.rs), [`rate limits`](https://github.com/supermarsx/mailwoman/blob/master/crates/mw-server/src/rate_limit.rs), and [`request budgets`](https://github.com/supermarsx/mailwoman/blob/master/crates/mw-server/src/request_budget.rs). |
| Search | Per-process index, in memory or a configured local directory, with startup reconciliation. | PostgreSQL does not make the search index shared. A single index directory must not simply be shared by independent writers. `MW_INDEX_PUBLICATION_ACK=journal` is single-process SQLite only and is refused on PostgreSQL; see [optional modes](../admin/performance-modes.md#journal-acknowledgement-for-search-index-publication). See [`engine mode`](https://github.com/supermarsx/mailwoman/blob/master/crates/mw-server/src/engine_mode.rs). |
| Uploaded attachments | Metadata in the database; sealed objects on the configured upload backend. | Replicas need access to the same objects, through a suitable shared backend. Migrating only database rows does not copy attachment objects. See [`upload storage`](https://github.com/supermarsx/mailwoman/blob/master/crates/mw-store/src/upload.rs). |
| Kubernetes deployment | Configurable replica count, default SQLite storage on one ReadWriteOnce PVC, Recreate rollout strategy. | The chart is not a ready-made HA or sharding topology. Review the database, objects, index and ownership boundaries before adding replicas. See [chart deployment](https://github.com/supermarsx/mailwoman/blob/master/packaging/helm/mailwoman/templates/deployment.yaml). |
| Connection pools | Each store opens its own bounded SQLx pool. | Replica counts multiply total connections. Budget foreground and worker pools against database capacity; shared PostgreSQL does not remove this limit. |

A sticky load balancer can reduce local-state surprises while a browser stays on
one process. It does not provide account ownership, cross-process event delivery,
consistent cache invalidation, or failover recovery by itself.

## Required coordination before broad horizontal scaling

1. **Account ownership.** Assign each account to an upstream-sync owner using a
   durable renewable lease with fencing. Stop the old owner's work when it loses
   ownership, and route account operations to the current owner. Preserve the
   existing durable submission claims; a sync lease is not permission to resend
   uncertain mail.
2. **Cross-replica events.** Add an authenticated, account-scoped event transport,
   replay/cursor handling, reconnect reconciliation and duplicate suppression.
   Connect WebSocket/SSE and native push consumers to it. Redis/Valkey cache
   configuration alone does not enable this transport.
3. **Shared-state boundaries.** Define cache invalidation, security-policy refresh,
   cluster-wide budgets, object storage and search ownership. Coordinate every
   background loop rather than assuming scheduler leases cover it.
4. **Failure qualification.** Exercise owner loss, network partitions, database
   failover, event replay, expired sessions and rolling upgrades while checking
   for lost notifications, duplicate upstream work and duplicate sends.

## Per-location and account sharding

A future multi-location deployment should have an explicit account-to-home-shard
mapping. A shard owns the account's synchronization, mutations, durable data and
search partition. Gateways route to that home rather than allowing every location
to write every account independently. Linked accounts and unified views need
account-scoped fan-out with authorization and partial-failure handling across
those homes.

This requires implementation and tests for routing, ownership handoff, account
migration, data-location policy, cross-shard operations and recovery. It is not a
current deployment switch. Replicating PostgreSQL or placing a load balancer in
front of several instances does not implement these application guarantees.

## Measuring a capacity target

Use the [load-testing tools](load-testing.md) on isolated production-profile
builds. Define mailbox counts, retained body sizes, connected WebSocket clients,
IMAP/Graph/JMAP provider behavior, search and upload traffic, and the duration of
bursts and peaks. Record accepted throughput, p95/p99 latency, errors, throttles,
missed notifications, queue age, upstream connections, database wait time and
resource usage. Include cold starts, reconnect storms and a failed replica.

Start with a small measured deployment and increase one dimension at a time.
A passing functional suite, a 4 GiB/four-core default, or a source-level crate
extraction is not evidence that a 20k/50k/100k target has been reached.
