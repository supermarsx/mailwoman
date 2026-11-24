# Bounded guard snapshot (`MW_GUARD_SNAPSHOT=bounded`)

Operator view, defaults and the consistency trade: [Optional consistency trade-off modes](../admin/performance-modes.md#guard-snapshot).
This note records the mechanism and its coverage. No throughput figures are published for it yet.

## The bound

`off` (default) is unchanged: every eligible request reads the native publication fence (one SQL statement) and the guard inputs fresh.

In `bounded` mode a request may reuse a fence that an earlier request of the same published entry read. Reuse requires the process `EpochMirror` (`crates/mw-store/src/authority_publication/mirror.rs`) to prove that no authority change applied since that fence was read: the caller takes the mirror generation `G` before the fence statement starts and stores it with the fence; `still_valid(fence, G)` fails once any scope the fence observed has `changed_gen > G`.

The mirror polls `authority_change_log` (migration 0115) every `bound / 4`, at least every 25 ms, in one SQL statement. It is fresh while the last complete poll started less than `MW_GUARD_SNAPSHOT_MAX_STALENESS_MS` ago (default 1000 ms, range 100 to 5000). A change committed at time C is refused by every check made at or after C + bound, in every process sharing the SQLite file. When polls fail or fall behind nothing is fresh and callers take the fresh SQL path.

## Invalidation sources

- Authority triggers (0111, 0112, 0114) write `authority_epoch`; 0115 appends one change-log row per epoch or `settings_revision` write, in the writer's transaction, including raw SQL writers that keep triggers enabled.
- Local revocations call `expedite`: no cached fence validates until a poll that started after the call completes. The request layer expedites after every credentialed state-changing request that did not answer 4xx (except `POST /jmap/api` and `POST /api/mail/refresh`). Writers that commit and then answer 4xx expedite at the write: sign-in session-cap eviction and the session-watch theft revoke.
- The mirror flushes everything (all cached fences go cold) on a full page, a cursor older than the retained horizon, a missing cursor row, a schema-stamp change or a SQLite generation change.

## Coverage

Covered (reused only while the mirror proves no change): the fence's sessions, accounts, `session_principal`, `identity_principal`, `identity_binding` and `settings_revision` reads; the published entry's account and session snapshots, linked-account existence, account names, zero-access and restriction rows, domains and aliases, directory holds, administrator flags, setting policy and values, groups, feature profiles, guard configuration (`security.inbound-allowlist.v1`, `security.clientAccess.v1`, `maintenance.mode.v1`), the parsed ban list and the session-watch settings.

Not covered, by design, and therefore re-checked or read fresh:

- Session clocks (`created_at`, `activity_at`, `last_seen`) have no trigger. Writers only move them forward, so a reused fence can only carry older clocks. A reused fence is accepted only when its touch is not due and its deadline is more than two minutes away; otherwise the request reads fresh.
- Request-protection subjects and events, native (bearer) sessions, administrator tables, and the final mail-data fence (`native_publication_fence_with_mail_data`) are read fresh on every request.
- Bearer, API-key, linked-mailbox selector and administrator requests never reach the reuse path.

Out of contract: a connection with triggers disabled, or a deletion of change-log rows strictly between the cursor and the head that keeps both ends.

## Multi-process behaviour

On one SQLite file every process runs its own mirror over the same change log, so a change made by any process is honoured everywhere within the bound. Only local revocations are expedited; a revocation made by another process takes up to the bound.

PostgreSQL: the log sequence is not commit-ordered, so the mirror is inert (`stamp_gen` is always `None`) and every request reads fresh. Migration 0118 drops the 0115 triggers there.

## Relationship to the other publication notes

[Published native inputs](published-native-inputs.md), [published guard inputs](published-guard-inputs.md) and the [authority publication foundation](authority-publication-foundation.md) describe the `off` behaviour, which stays the default. Their statements that there is no allow cache and that asynchronous delivery never permits zero SQL remain true for `off`. `bounded` replaces the per-request fence read with the bound stated here; it does not replace the final mail-data fence.
