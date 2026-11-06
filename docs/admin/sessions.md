# Sessions and server restarts

After the server process restarts (a deploy, a crash, a systemd restart, a
container recreate) people and administrators stay signed in and keep working
without typing a password again. This page lists what is stored, what is held in
memory only, what a restart does to each, and the settings that control it.

The same status is on the Security overview ("Sessions and restarts"): it reads
"Sessions survive restarts: yes" or "no" with the reason, and the number of
stored mailbox, app and console sessions.

## What is stored in the database

| State | Where | After a restart |
| --- | --- | --- |
| Mailbox (webmail) session | `sessions` table, cookie `mw_session` holds only the random id | Valid. Creation time, last activity (`activity_at`) and last-seen are columns, so the idle and absolute deadlines are unchanged. |
| Upstream credentials (IMAP, JMAP, DAV password) | `sessions.sealed_creds`, sealed with the server key | Opened lazily with the server key on the next request that needs them. Never written in plaintext. |
| Native app session | `native_sessions` (hash of the bearer token) plus its `sessions` row | Valid, same as a mailbox session. |
| Console (admin) session | `admin_sessions`: token hash, `expires_at` (idle), `absolute_expires_at` | Valid. The absolute deadline is stored when the session is created and never reset. The idle deadline is the stored one. |
| CSRF token | Not stored. It is derived from the server key and the session id (HMAC), and the `mw_csrf` cookie carries it | A token issued before the restart verifies after it, as long as the server key is the same. |
| Remembered devices (skip the second factor) | `remembered_device` (hash of the device token, its own expiry) | Still recognised. |
| New-device and sign-in history | `signin_event`, `remembered_device` | Unchanged. Serving a restored session records no sign-in, so a restart never triggers the "new device" email. |
| Pending single sign-on flows (OIDC state, nonce, PKCE verifier; SAML request id) | `ephemeral_state`, payload sealed with the server key, lookup key stored hashed, single use, 10 minute expiry | A sign-in that was in flight when the server restarted still completes. A callback that reaches a different instance finds the flow too. |
| Outbox queue, drafts, settings, push subscriptions | their own tables | Durable. |
| Session retention | The retention sweeper prunes `sessions` by `last_seen` | Unchanged. `last_seen` is written at most every `session.lastActivityFlushSeconds` seconds, so a pruned session is at most that many seconds staler than before. |

Session validity needs nothing from the process that created it. Several
instances behind a load balancer share one database and one server key, and any
of them can serve any session.

## What is held in memory only, on purpose

| State | Why | After a restart |
| --- | --- | --- |
| Step-up proof ("confirm it is you", five minutes) and its failure lockout counter | A short proof for sensitive actions; persisting it would let a database copy mint one. | The person confirms again. A proof never carries over to another session. Behind a load balancer the confirmation has to land on the instance that serves the action; use client affinity for `/api/security/step-up` or accept one extra confirmation. |
| Two-factor sign-in in progress (password accepted, code not yet entered), passkey challenges, TOTP enrolment secrets, console sign-in in progress | These hold a password-checked identity or a one-time secret for a minute or two. | The person starts that sign-in again. Not shared between instances; use client affinity during sign-in if you run several. |
| Sign-in rate limits, cooldowns, trust tiers | Abuse counters. Trust tiers are flushed to the database on a clean shutdown (SIGTERM or Ctrl+C). | Counters restart empty after a crash. |
| IMAP IDLE connections, engine caches, JMAP state strings, the search index handle | Connections and caches that are rebuilt on demand. | Reconnected lazily on the next request. |
| Sanitizer and remote-content caches | Derived data. | Rebuilt. |
| Undo history | Lives in the browser tab. | Survives, because the page is not reloaded. |
| Realtime connections (WebSocket, EventSource) | A connection cannot outlive its process. | The web app reconnects with exponential backoff and jitter and, once back, re-reads the lists once to catch up on changes missed while it was down. |

## What the web app does while the server is away

* A request that gets 502, 503 or 504, or no answer at all, is not a sign-out.
  Only a 401 is. The page-load session check waits (backoff from 0.5 s to 5 s with
  jitter, about a minute in all) and shows "Reconnecting…" instead of the sign-in
  form; after that it shows the usual retry card.
* The realtime connection backs off with jitter, and when it returns the app
  re-reads each datatype once.
* Writes that opted in to recovery are retried on 502/503/504 with the same
  backoff; an idempotency key makes a repeat safe.

## Settings

| Key | Default | Meaning |
| --- | --- | --- |
| `session.persistAcrossRestarts` | on | Off ends every stored session (mailbox, app, console) when the server starts. Use it where you want a restart to sign everyone out. Applies at the next start. |
| `session.lastActivityFlushSeconds` | 15 | A session's `last_seen` is written at most this often (0 to 300). It is request telemetry, not the idle clock, so it does not extend a session. Applies at the next start. |
| `server.shutdownGraceSeconds` | 30 | How long a stopping server lets in-flight requests finish before it closes connections (1 to 120). `MW_HTTP_SHUTDOWN_TIMEOUT_MS` overrides it. Applies at the next start. |

Idle and absolute limits keep their existing keys: `session.idleSecs`,
`session.absoluteSecs`, `admin.session.idleSecs`, `admin.session.absoluteSecs`.

A "memory-only" upstream credential mode (credentials sealed with a secret held
only in the cookie, so the database alone cannot open them) is not implemented:
background work such as sync and sending needs the credentials with no browser
present. To trade convenience for safety, turn `session.persistAcrossRestarts`
off.

## Shutdown

The server stops on SIGTERM (systemd, Docker, Kubernetes) or Ctrl+C. It stops
accepting connections, lets in-flight requests finish for up to the grace period,
then closes the rest, and writes the trust-tier state. Make the service manager's
stop timeout longer than the grace period.

## The server key

Sessions are sealed with the server key. It comes from, in order: `MW_SERVER_KEY`,
the file named by `MW_SERVER_KEY_FILE`, or `<database>.server-key` beside a SQLite
file. PostgreSQL requires the first two, shared by every instance.

* A **missing key file** with an existing database creates a new key. Stored
  sessions then cannot be opened: those people see the sign-in page, the server
  does not crash, and the Security overview shows "Sessions survive restarts: no
  (stored sessions do not open with the current server key)". Restore the original
  key file to bring them back; a damaged key file is never replaced silently.
* An **in-memory database, or a data directory that is wiped on restart** loses
  the key (or the database). Sessions cannot survive; the overview says so.
  Mount the data directory, or set `MW_SERVER_KEY`, on a container.
* A **clock jump** moves deadlines with it. A forward jump past a deadline ends the
  session. A backward jump lengthens live sessions by the size of the jump; a
  session that already ended is deleted and stays ended.

## Verified by tests

`crates/mw-server/tests/t_session_restart_survival.rs` builds a second app over
the same database and key and checks: a mailbox cookie still works and its
creation and activity times do not move, upstream credentials re-open, a restart
adds no sign-in event or device, the CSRF token still verifies (and a forged one
does not), a console session keeps its absolute deadline and is refused once that
passes, a step-up proof is per session and not carried over, turning persistence
off ends sessions at start, a different server key signs people out without a
crash and the overview says why, and a pending sign-on state is shared between
instances and single use. `crates/mw-server/tests/session_restart.rs` repeats the
mailbox case across a real process restart. The browser behaviour is covered by
`apps/web/src/state/bootRetry.test.ts`.
