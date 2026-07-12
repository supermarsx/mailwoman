# Pairing Mailwoman with IMAP / POP3 (engine mode)

In **engine mode** (`MW_MODE=engine`) Mailwoman stops being a JMAP proxy and
instead drives a real mailbox itself: it syncs an IMAP or POP3 account, parses
MIME locally, submits outgoing mail over SMTP, and presents the *same* JMAP
surface the SPA already speaks. The browser cannot tell the difference — the
login form is unchanged; its **server-URL** field (the one that held a JMAP URL
in proxy mode) is now read as a mail-server URL.

## How a user logs in

The login form posts `{ jmapUrl, username, password }`. In engine mode
`jmapUrl` is parsed as a mail-server URL:

| URL | Backend | Transport | Default port |
|-----|---------|-----------|--------------|
| `imaps://host` | IMAP | implicit TLS | 993 |
| `imap://host`  | IMAP | STARTTLS | 143 |
| `pop3s://host` | POP3 | implicit TLS | 995 |
| `pop3://host`  | POP3 | STARTTLS | 110 |
| `host` (bare)  | IMAP | implicit TLS | 993 |

An explicit `:port` overrides the default. `MW_ENGINE_TLS` overrides the
transport for *every* login (e.g. to force `plaintext` against a test server
without changing the URL). The username/password are whatever the mail server
expects; credentials are sealed at rest with `MW_SERVER_KEY`.

## The send path (SMTP)

IMAP/POP3 only cover receive. To be daily-drivable the engine also submits mail
over SMTP (`EmailSubmission/set` → build MIME → SMTP → append to Sent). Point it
at the provider's submission endpoint:

```sh
MW_SMTP_HOST=smtp.example.org   # defaults to the IMAP host if unset
MW_SMTP_PORT=587                # 587 STARTTLS / 465 implicit / 25 plaintext
MW_SMTP_SECURITY=starttls       # starttls | implicit | plaintext
```

SASL uses the same username/password as the receive account (PLAIN/LOGIN;
XOAUTH2 for providers that require it).

## Dovecot (self-hosted, production-fidelity)

Dovecot is the reference self-hosted target. A typical pairing:

- IMAP over implicit TLS on 993 (or STARTTLS on 143), plus a submission MTA
  (Postfix/Dovecot Submission) on 587.
- Log in with `imaps://mail.example.org`, the mailbox username, and password.
- Special-use folders (`\Sent`, `\Drafts`, `\Junk`, `\Trash`) are detected via
  the IMAP SPECIAL-USE / LIST extensions and mapped to JMAP roles; QRESYNC /
  CONDSTORE drive incremental sync where Dovecot advertises them (it does).

The repo ships a **containerised Dovecot** for conformance/dev under
`scripts/dovecot/` (see below) — a minimal, plaintext, seeded instance, **not** a
production template. For real deployments follow the upstream Dovecot docs and
keep TLS on.

## Gmail (IMAP + XOAUTH2)

Gmail works as an IMAP account (`imaps://imap.gmail.com`) with SMTP submission
via `smtp.gmail.com:587`. Interactive OAuth (XOAUTH2) is required for normal
accounts; the SASL framing is exercised by recorded fixtures in CI and a
documented nightly live job (no interactive consent runs in CI). Gmail also has
quirks the engine degrades around (e.g. missing UIDPLUS on some operations —
the engine re-derives by `Message-ID`).

## POP3-only hosts

For a POP3 account (`pop3s://host`) the engine syncs INBOX by UIDL diff (no
server-side folders or flags — those are tracked engine-side) and honours a
leave-on-server policy (keep / delete-after-N-days / delete-on-retrieval).
Sending still goes through the configured SMTP endpoint.

---

## Testing backends (docker-compose.dev.yml)

Two mail servers are wired for development and CI conformance. Neither is a
production template — both are plaintext, single-user, and seeded for tests.

### Greenmail — the deterministic gate

Greenmail is a pure-Java test mail server: fast, deterministic, always-green. It
is the backend the CI `imap-conformance` job **gates** on.

```sh
docker compose -f docker-compose.dev.yml up -d --wait greenmail
```

- Ports: IMAP **3143**, POP3 **3110**, SMTP **3025** (TLS: 3993 / 3995 / 3465).
- Preseeded account: email `testuser@example.org`, **IMAP/POP3 login name
  `testuser`** (the local part — *not* the full address), password `testpass`.
- Health: TCP probe on 3143/3110/3025. `scripts/greenmail/wait-for-greenmail.sh`
  additionally asserts an IMAP LOGIN succeeds before tests run.

Run mw-server against it in engine mode:

```sh
docker compose -f docker-compose.dev.yml up -d --wait greenmail mailwoman-engine
# SPA on http://localhost:8090 — log in with:
#   server URL = imap://greenmail:3143   username = testuser   password = testpass
```

### Dovecot — the fidelity target

A real Dovecot 2.4 (official image) trimmed to a minimal plaintext IMAP+POP3
host, seeded with the same `testuser`/`testpass` and two INBOX messages (one
multipart). Richer capabilities than Greenmail (QRESYNC/CONDSTORE/SPECIAL-USE/
MOVE/UIDPLUS…), so it is the production-fidelity leg — CI runs it
`continue-on-error` (plan §6 risk #11) until stabilised.

```sh
docker compose -f docker-compose.dev.yml up -d --wait dovecot
# IMAP host:2143 -> 143 · POP3 host:2110 -> 110
```

Config lives in `scripts/dovecot/`:

- `zz-mailwoman.conf` — a conf.d drop-in over the stock image: enables
  `imap pop3`, cleartext auth, no TLS, a passwd-file passdb/userdb, and the
  plaintext 143/110 listeners. **CI/dev only.**
- `mailwoman-users` — the passwd-file (`testuser` / `{PLAIN}testpass`).
- `seed/*.eml` — seed messages delivered into INBOX.
- `seed-and-serve.sh` — the container command: starts Dovecot, waits for its
  auth service, delivers the seeds with `doveadm save`, then runs the server in
  the foreground.

### Running the conformance tests locally

The live tests are `#[ignore]` + env-gated, so `cargo test --workspace` stays
green with no server. Point the `GREENMAIL_*` env at whichever backend:

```sh
# Greenmail (gate) — all four crates, including SMTP submission:
GREENMAIL_IMAP=127.0.0.1:3143 GREENMAIL_POP3=127.0.0.1:3110 \
GREENMAIL_SMTP=127.0.0.1:3025 GREENMAIL_USER=testuser GREENMAIL_PASS=testpass \
GREENMAIL_TLS=plaintext \
  cargo test -p mw-imap -p mw-pop3 -p mw-smtp -p mw-server --test '*' -- --ignored

# Dovecot (fidelity) — no SMTP submission, so omit mw-smtp:
GREENMAIL_IMAP=127.0.0.1:2143 GREENMAIL_POP3=127.0.0.1:2110 \
GREENMAIL_USER=testuser GREENMAIL_PASS=testpass GREENMAIL_TLS=plaintext \
  cargo test -p mw-imap -p mw-pop3 -p mw-server --test '*' -- --ignored
```
