---
title: Optional consistency trade-off modes
---
# Optional consistency trade-off modes

Three start-up switches let a single-process SQLite installation do less database work per request. Each one changes a consistency guarantee, which is stated below. All three are **off by default**. With none of the variables set, the server behaves as it did before these switches existed.

No throughput figures are published for these modes yet. Do not turn them on to reach a capacity target; turn one on only when you accept the trade described for it.

| Switch | Default | What it changes |
|---|---|---|
| [`MW_GUARD_SNAPSHOT`](configuration.md#mw-guard-snapshot) | `off` | Reuses a recent authority check instead of reading it again on every request. |
| [`MW_INDEX_PUBLICATION_ACK`](configuration.md#mw-index-publication-ack) | `commit` | `Email/set` returns after the database commit instead of after the search-index commit. |
| [`MW_REQUEST_PROTECTION_ACCOUNTING`](configuration.md#mw-request-protection-accounting) | `synchronous` | Request-protection counters are written in batches instead of on every request. |

## Before you start

- **Single process on SQLite.** These modes are built for one server process that owns one SQLite file. On PostgreSQL the guard snapshot and write-behind accounting do nothing (the server logs a warning and keeps the default behaviour), and journal acknowledgement stops start-up.
- **A restart applies a change.** All the variables are read once at start-up.
- **Invalid values stop start-up.** Before it opens the database, the server checks every variable on this page. An unknown word or a number out of range stops it with `refusing to start: <VARIABLE>…` naming the variable.

At start-up the server logs the modes it is using in one line, for example:

```text
t34 modes: guard snapshot off; index publication commit (read fence 5000 ms, shutdown drain 10 s); request-protection accounting synchronous
```

In proxy mode there is no search index, so the line says `index publication not used (proxy mode)`, and the index variables are not checked.

## Guard snapshot

`MW_GUARD_SNAPSHOT=bounded` lets an ordinary signed-in browser request reuse the result of the authority read that an earlier request of the same session already made: the session row, its account and principal, the policy and feature settings, and the guard configuration (allow-lists, client access, maintenance mode). The request still runs every policy, clock and process check on that data; only the read is skipped.

A background task polls the database for authority changes every quarter of the staleness bound (at least every 25 ms). A reused result is accepted only while the last complete poll is younger than the bound and shows no change to anything the result depended on. When polls fail or fall behind, every request reads fresh.

**The trade.** A change made **by another process** on the same database file (for example the `mailwoman admin` command, or a second server process) is honoured within the staleness bound, `MW_GUARD_SNAPSHOT_MAX_STALENESS_MS` (default 1000 ms, range 100 to 5000). During that window a request can still be admitted with the state from before the change. A revocation made **through this server** (sign-out, session revoke, password change, an administrator disabling an account) makes the next request read fresh.

What is never reused: bearer tokens, API keys, administrator requests, linked-mailbox selections, request-protection counters and the final check of mail data. Session clocks (idle and absolute expiry) are re-checked; when a reused session would come within two minutes of a deadline, the request reads fresh. A session's activity update (`last_seen`) no longer forces a fresh read: the request records it in memory and the server writes it within 5 seconds. So a revocation made **by another process** is honoured only through the staleness bound (and, for revocations made through this server, the immediate fresh read described above); a busy session is not otherwise sent back to a fresh read.

PostgreSQL: no poller is started and every request reads fresh.

## Journal acknowledgement for search-index publication

`MW_INDEX_PUBLICATION_ACK=journal` applies in engine mode. In the database transaction that changes a message, a trigger writes a journal row. `Email/set` returns after that transaction commits; the search index is updated afterwards by the index owner, which deletes the journal row only after its index commit.

**Reads wait for the writer's own changes.** A search or another index-backed read for an account waits until the publications that the account handed off before the read started are applied. It waits at most `MW_INDEX_READ_FENCE_TIMEOUT_MS` (default 5000 ms, range 100 to 30000). After that the read is refused: the method answers `serverFail`, or index-derived list fields are left out. It never answers from an index that is missing the change.

**Crash and restart.** If the process stops before the index caught up, the journal rows remain. At the next start the server replays them before it serves index reads; reads wait (and are refused after the timeout) until the replay finishes. If the replay does not complete, index reads stay refused until a later replay succeeds. A background sweeper retries rows that no publication covered.

**Queue limits.** At most 128 publication jobs are accepted and not yet applied at any time, and at most 32 of them for one account. An account at its limit waits for its own jobs (up to 30 seconds), after which its `Email/set` fails with `serverFail` before any database change. Other accounts are not held up by one account at its limit. Several accounts each at their limit can still fill the 128-job queue together; then every account waits.

**Shutdown.** On a graceful stop the server closes admission and waits for accepted jobs for up to `MW_INDEX_PUBLICATION_DRAIN_SECS` (default 10 s, range 0 to 60). Jobs still pending after that stay in the journal and replay at the next start.

**One process per database.** The index lives in each process, and journal order equals commit order only within one SQLite file. A journal-mode process takes an exclusive lock on the sidecar file `<database file>.index-journal.lock` for its lifetime and refuses to start when another process holds it. The lock has limits:

- It is keyed on the database path as configured. A second process that opens the same database through a symbolic link, a hard link or another path to the same file is not detected.
- A commit-mode process that started **before** any journal-mode process on the file is not detected. Its start-up would turn journal capture off. A commit-mode process that starts while a journal-mode process holds the lock leaves the journal alone and logs an error.

Do not run a second server process, in any mode, on a database used in journal mode.

**Returning to `commit`.** Rows left by a journal-mode run are replayed when the next commit-mode process starts. Reads are held only during that replay; reads that arrive before the replay begins, or after a replay that failed, are served as in commit mode, and the remaining rows replay at the following start.

PostgreSQL: refused at start-up (`MW_INDEX_PUBLICATION_ACK=journal is SQLite-only`).

## Write-behind request-protection accounting

`MW_REQUEST_PROTECTION_ACCOUNTING=write-behind` keeps request-protection counters (request rate and the adaptive anomaly windows) in memory and writes them to the database in batches every `MW_REQUEST_PROTECTION_FLUSH_MS` (default 250 ms, range 50 to 5000). Decisions are made from the in-memory state.

These are still written at once, before the request is answered: the first observation of a subject (so the subject-capacity limit is checked), and every change of decision (a throttle or block starting). A refused request is therefore refused durably.

**The trade.**

- **Crash.** Observations still queued when the process stops abruptly are lost: at most one flush interval of ordinary (non-refusing) observations, and never more than `MW_REQUEST_PROTECTION_MAX_PENDING` (default 4096, range 64 to 65536). A subject can therefore get up to that many extra requests counted as never made.
- **Full queue.** When the queue is full, the request is still decided from the cached state (an existing throttle or block still applies), but its observation is not recorded. If request protection is set to fail closed and enforcing, the request is answered with 503 instead.
- **Several processes.** Each process decides from its own cache between flushes. N processes sharing one SQLite file can together admit up to about (N − 1) times a subject's limit within one flush interval beyond the limit. The default `synchronous` mode keeps exact counting across processes.
- **Administrator views** of protection subjects commit queued observations before they read.

**Shutdown.** On a graceful stop, after the HTTP server and WebSocket actions have stopped and after the search-index drain, the server commits the queued observations, waiting at most 5 seconds. Observations still queued after that are lost, as in a crash, and a warning is logged.

PostgreSQL: the setting is ignored with a warning and accounting stays synchronous.

## Combining the modes

The switches are independent; each can be turned on alone. Their checks are the same whether one or all three are on. On shutdown the order is: stop accepting HTTP connections, finish WebSocket actions, drain index publication, flush write-behind accounting, save the request-history and browser-timing buckets (at most 5 seconds each), then the remaining telemetry and log flushes. Each step runs with every mode at its default too; with no queued work it returns at once.

## Related

- [Server configuration reference](configuration.md): every variable with its default and range.
- [SQLite start-up and recovery](database-startup.md): `MW_SQLITE_STARTUP_TIMEOUT_SECONDS`, which is also checked before the database opens.
- [Scaling and multi-location deployments](../deploy/scaling.md): what a second process or replica does and does not share.
- Engineering notes: [guard snapshot](../perf/guard-snapshot.md) and [index publication](../perf/index-publication.md).
