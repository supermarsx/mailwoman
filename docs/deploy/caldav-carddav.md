# CalDAV / CardDAV, sharing, and holidays (V3 PIM)

V3 turns Mailwoman into a personal-information manager: **calendars, tasks,
notes, and contacts**. Calendars/tasks sync over **CalDAV** and contacts over
**CardDAV** to whatever server you run; notes are Mailwoman-native (sealed at
rest). The PIM surface rides the same session-authed JMAP envelope the mail
client already speaks (`Calendar/*`, `CalendarEvent/*`, `Task/*`, `Note/*`,
`AddressBook/*`, `ContactCard/*`, `ContactGroup/*` under
`urn:mailwoman:{calendars,tasks,notes,contacts}`), so transport, offline, and
realtime push light up for PIM with no new plumbing.

CalDAV/CardDAV sync is an **engine-mode** feature: the local `mw-engine` store
holds the collections, drives the DAV clients (`mw-dav` / `mw-carddav`), and
persists per-collection `sync-token`/`ctag`/`etag`. In proxy mode the
Mailwoman-native sharing endpoints return `501 requires engine mode`.

## Pairing a CalDAV/CardDAV account

A calendar or address book is attached by its collection URL. The engine runs
RFC 6764 discovery for you — point it at the server root or the
`.well-known/caldav` / `.well-known/carddav` path and it walks
`current-user-principal → home-set → collection PROPFIND`. Sync is incremental
via `sync-collection` (RFC 6578) where the server advertises `sync-token`, and
degrades to a `getctag` + ETag-diff pull otherwise (feature-detected, the same
discipline as the V1 IMAP sync engine). Local writes `PUT` with
`If-Match:<etag>` (update) / `If-None-Match:*` (create); a `412` surfaces as a
conflict and re-pulls (last-write-wins with a conflict toast).

Verified server families:

| Server | Notes |
|--------|-------|
| **Radicale** | The reference testing target (below). Advertises `sync-token`; single Python process, trivial to seed/reset. |
| **Nextcloud** | CalDAV/CardDAV base `https://<host>/remote.php/dav/`. Advertises `sync-token` + `getctag`. Works with app-password auth. |
| **Baïkal** | sabre/dav-based; base `https://<host>/dav.php/`. `getctag` + ETag-diff fallback path. |
| **Google** | CalDAV `https://apidata.googleusercontent.com/caldav/v2/…`, CardDAV `https://www.googleapis.com/carddav/v1/…`. Needs **OAuth (XOAUTH2)**; quirks (absolute hrefs, `D:`-prefixed namespaces, weak `W/"…"` ETags) are covered by recorded fixtures. Live OAuth is manual/nightly, never in CI (plan §0). |
| **Microsoft 365 / Exchange** | No native CalDAV/CardDAV; a Graph/EWS bridge is a later milestone (V7). Recorded-quirk fixtures only. |

The parser is deliberately namespace-prefix and case tolerant, and stores the
raw `ical_raw`/`vcard_raw` body as the round-trip source of truth (unknown/`X-`
properties survive re-emit), so the Google/M365 quirks parse the same as
Radicale.

## Radicale — the testing / CI backend

`docker-compose.dev.yml` ships a **Radicale 3.x** service (`radicale`, host
port **5232**) with htpasswd auth (`testuser` / `testpass`, mirroring the
Greenmail/Dovecot test account) and filesystem collections. Config +
credentials live under `scripts/radicale/` (`config`, `users`); collections are
created over the DAV protocol by `scripts/radicale/seed.sh` (a VEVENT calendar,
a VTODO task list, and a CardDAV address book for `testuser`).

> **License posture.** Radicale is GPLv3, but it is **CI infrastructure, not a
> dependency** — a separate program invoked over the network only, never linked
> into, imported by, or shipped with any Mailwoman crate or binary (mere
> aggregation, the same posture as Greenmail/Dovecot in V1). It is excluded from
> the `cargo-deny` / JS-license dependency scope by design; the actual-dependency
> floor stays MIT/Apache/BSD/ISC/Zlib/MPL-2.0 (see `deny.toml`).

Bring up a self-seeding stack (the `radicale-seed` one-shot waits for health
then seeds):

```sh
docker compose -f docker-compose.dev.yml up -d --wait radicale radicale-seed
```

Or seed manually against an already-running server:

```sh
sh ./scripts/radicale/seed.sh 127.0.0.1 5232 60   # RADICALE_RESET=1 for a clean slate
```

Then run the env-gated live conformance tests (the `#[ignore]` create / sync /
round-trip / `412`-conflict transcripts) — this is exactly what the CI
`caldav-carddav-conformance` job does:

```sh
RADICALE_URL=http://127.0.0.1:5232 RADICALE_USER=testuser RADICALE_PASS=testpass \
  cargo test -p mw-dav -p mw-carddav -p mw-engine -- --ignored
```

`mw-dav` proves discovery → sync-collection, `mw-carddav` proves address-book
discovery → query → multiget projection, and `mw-engine` proves a full
`sync_pim` round-trip (engine A `PUT`s an event to the real collection, engine B
with a fresh store pulls it back). Log into the SPA against the stack with the
collection URL `http://localhost:5232/testuser/` and `testuser` / `testpass`.

## Calendar / address-book sharing endpoints

V3 does **Mailwoman-native (on-server) ACL sharing** plus **read-only overlay**
of a foreign CalDAV URL (bidirectional cross-server write-sharing is a
documented follow-up). A calendar's `shareWith` list (`{principal, access:
"read" | "readWrite"}`, set via `Calendar/set`) is enforced when another
principal fetches the collection:

| Endpoint | Meaning |
|----------|---------|
| `GET /dav/calendars/{accountId}/{calendarId}` | Serve a Mailwoman-native calendar to a grantee per `calendar_shares`. The owner always reads their own; a grantee with `read`/`readWrite` may fetch; everyone else gets `403`. Cookie-authed. |
| `GET /dav/addressbooks/{accountId}/{addressBookId}` | Serve a Mailwoman-native address book. **Owner-only** in V3 (the frozen model has no address-book share ACL yet). |

Both require engine mode (`501` otherwise) and pass the same CSRF / Origin /
security-header layers as every other route. A **read-only overlay calendar**
(`isReadOnlyOverlay`, backed by a foreign `caldavUrl`) is pull-only and never
written back.

## Holiday feeds

Bundled, subscribable holiday packs are compiled into the binary and served as
RFC 5545 iCalendar (each holiday an all-day `VEVENT` with `RRULE:FREQ=YEARLY`):

| Endpoint | Meaning |
|----------|---------|
| `GET /api/holidays` | The region index — `{id, name, url}` per bundled pack. |
| `GET /api/holidays/{region}` | The pack as a `text/calendar` feed (e.g. `/api/holidays/us`). |

Both are cookie-authed and read-only. V3 bundles **fixed-date** holidays only
(correct every year with no Easter / nth-weekday computation); richer regional
packs via `.hol` / ICS import (`mw_ics::parse_hol`, `CalendarEvent/import`) are
a follow-up. Users can also subscribe to any external ICS holiday URL as an
overlay calendar.

## At-rest notes (not zero-access)

Notes are Mailwoman-native and **always encrypted at rest** — bodies are sealed
BLOBs under the existing `mw-store` server-held XChaCha20-Poly1305 key (the same
seal that protects upstream credentials). This is **encrypted-at-rest, NOT
zero-access**: the server can decrypt (V6 swaps the key source for a
user-derived key hierarchy). Note title / tags / color / pinned are plaintext
columns so search and sort work without a plaintext body index.
