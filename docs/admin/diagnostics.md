# Server diagnostics

Open **Operations → Diagnostics** to run the server suite and review saved runs.
Reading history requires the unscoped `diagnostics.read` grant; running checks
requires unscoped `diagnostics.run`. A session can start two runs per minute.

The server suite performs five checks:

- Query the configured database.
- Insert a synthetic settings row in a transaction, then roll it back.
- Compare the applied migration count with this server build.
- Seal and open a synthetic sample with the server key.
- Verify that the default address policy rejects representative private and
  loopback addresses.

Each check has a five-second deadline. A failed check or deadline reports a
failure. Reports contain fixed descriptions, status and elapsed time, without
database errors, credentials or sampled user data. Audit and history failures
are reported separately: a successful probe does not imply its report was saved.

History is paginated in groups of 50. `diagnostics.retentionDays` sets the global
retention period (1–365 days, default 30). Expired completed runs are removed when
history is read and after a suite runs; cleanup is not a background timer. A
cleanup failure is reported rather than silently claiming retention succeeded.

The API is `POST /admin/diag/run` with `{"suite":"server"}`, and
`GET /admin/diag/runs` with an optional opaque `cursor` from `nextCursor`.
Responses are marked private and no-store. Unsupported suite names are rejected.

This suite does not establish external service reachability, SMTP delivery,
certificate renewal, full database integrity, or decryption of all stored data.
Domain connection probes remain in domain administration. Other diagnostic
suites and automated repairs are not implemented by this page.
