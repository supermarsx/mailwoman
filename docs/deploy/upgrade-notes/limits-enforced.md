# Upgrade note: send limits and the storage limit are now enforced

Applies to every deployment that has set `limit.send_per_hour`,
`limit.send_per_day` or `limit.storage_bytes` in the admin console, globally,
for a domain, for a user or in a feature profile. Before this release the
console saved these values but the server did not apply them. From this
release on it does. A value of `0`, the default, still means no limit, so
deployments that never set them see no change.

## Before you upgrade

Open **Settings** in the admin console and search for `limit.send_per_hour`,
`limit.send_per_day` and `limit.storage_bytes`, and check the **Limits**
section of each feature profile. A value saved during testing or by mistake
takes effect on the first request after the upgrade. Set it to `0` to keep the
old behaviour.

## Send limits

- **What is counted.** Each submission a client creates with
  `EmailSubmission/set` counts as one, however many recipients it has. Saving a
  draft, cancelling a scheduled send and changing a scheduled send do not count.
  `limit.recipients_per_message` still limits recipients per message, as before.
- **Windows.** `limit.send_per_hour` counts per clock hour (UTC) and
  `limit.send_per_day` counts per UTC day. The windows are fixed, not rolling:
  at most twice the limit can go out across the boundary of two windows, for
  example N at 10:59 and N more at 11:00.
- **Refusal.** A batch that would go over a limit is refused before anything in
  it runs, with HTTP `429` and
  `{"code":"limit_exceeded","limit":"limit.send_per_hour","max":N}` (or
  `limit.send_per_day`). The `Retry-After` header gives the seconds until the
  current window ends.
- **Failed sends do not count.** The count is reserved before the batch runs.
  If the request is refused or fails (any non-2xx status), the whole reservation
  is given back. If the batch runs, every submission the response does not list
  as created is given back.
- **Several server instances.** The counters live in the database
  (`usage_counter`, metrics `send.hour` and `send.day`), and a reservation is a
  single conditional update. Instances that share the database cannot admit
  more than the limit together.
- **Both modes.** Send limits apply in engine mode and in proxy mode.

## Storage limit

- **Engine mode only.** `limit.storage_bytes` limits the bytes this server
  holds for a person: the sizes of the messages it has cached for the account
  plus the files uploaded through `/jmap/upload/`. An upload that would take the
  total over the limit is refused with HTTP `413` and
  `{"code":"limit_exceeded","limit":"limit.storage_bytes","max":N}`.
- **Proxy mode.** The setting has no effect in proxy mode. The mail server holds
  the mail there, and its own quota applies.
- **Only new uploads are checked.** Mail that arrives through synchronisation
  is never refused, so the bytes held can go over the limit. Uploads are then
  refused until the total is back under it.

## Retention

Counter rows are kept for `usage.retentionDays` and then deleted by the
retention sweeper.
