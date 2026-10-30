# Country database

Country policies (`security.signin.geo*`, see [Sign-in country policy](signin-geography.md))
and the country labels in the Security centre, sign-in history and Received hops read one
local MaxMind DB (`.mmdb`) file. Lookups never leave the server. The file comes from one of
two places:

1. **`MW_GEOIP_DB`** — a path set in the server's environment. When it is set, the
   environment owns the database: the console page is read-only ("managed by the
   environment"), downloads and the schedule do nothing, and the file is read once per
   path. Replace or repair the file and restart, or unset the variable to manage the
   database from the console. This is the behaviour before 26.20.
2. **The console** — Security → Country database (`#/security/geoip`). The server
   downloads the database from the source you choose, keeps it up to date on a schedule,
   or takes a file you upload. A new database is used at once; no restart.

## Fail closed

While a country policy is active (`security.signin.geoScope` is not `off`), a missing or
invalid database **refuses** every sign-in the policy covers, with `503
signinGeographyUnavailable`, even when `security.signin.geoUnknown` allows unmapped
addresses. That is deliberate: an attacker who can make the database disappear must not
switch the policy off. The console shows a red notice on the Country database page and the
dashboard lists "Needs attention: a country policy is active but no usable country database
is loaded". The server log says the same, at most once every five minutes.

A download, upload or rollback that fails leaves the database in use untouched.

## Sources and licences

Checked 2026-10-29. Each provider's terms are theirs to change; read them before you
choose one. The console shows the licence and the attribution text of the chosen provider
and of the database in use.

| Provider | Default | Account | Licence | Updates | Download | Verification |
|---|---|---|---|---|---|---|
| [DB-IP IP to Country Lite](https://db-ip.com/db/download/ip-to-country-lite) | yes | none | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) | monthly | `https://download.db-ip.com/free/dbip-country-lite-YYYY-MM.mmdb.gz` (about 4 MB, 8 MB unpacked) | structural (see below) |
| [MaxMind GeoLite2 Country](https://dev.maxmind.com/geoip/geolite2-free-geolocation-data) | | free account and licence key | [GeoLite2 EULA](https://www.maxmind.com/en/geolite2/eula) | twice weekly | `https://download.maxmind.com/app/geoip_download?edition_id=GeoLite2-Country&license_key=KEY&suffix=tar.gz`, redirected to `mm-prod-geoip-databases.a2649acb697e2c09b632799562c076f2.r2.cloudflarestorage.com` | SHA-256 from `…&suffix=tar.gz.sha256`, then structural |
| [IPinfo Lite](https://ipinfo.io/lite) | | free account and access token | [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/) | daily | `https://ipinfo.io/data/ipinfo_lite.mmdb?token=TOKEN` (about 40 MB, uncompressed) | structural |
| Custom URL | | your choice | your source's | your choice | any HTTPS `.mmdb`, `.mmdb.gz` or `.tar.gz` | optional SHA-256 file, then structural |

**Attribution.**

- DB-IP Lite is licensed under CC BY 4.0, which requires attribution. DB-IP asks web
  applications that display or use its results to link back:
  `<a href='https://db-ip.com'>IP Geolocation by DB-IP</a>`. The console shows this link
  next to the database status whenever a DB-IP database is in use. If you show country
  results elsewhere (for example in pages you build on the API), show the same link there.
- MaxMind's GeoLite2 EULA asks for: "This product includes GeoLite2 data created by
  MaxMind, available from https://www.maxmind.com". It also requires an account, limits
  redistribution and expects you to use a current release (MaxMind asks that outdated
  copies be deleted within 30 days of a new release); keep automatic updates on.
- IPinfo Lite is CC BY-SA 4.0: credit IPinfo ("IP address data powered by IPinfo", linking
  https://ipinfo.io); a database you derive from it and share must carry the same licence.
  IPinfo Lite uses its own schema (a flat `country_code`), which the server reads as well.
  The download address was confirmed to answer at `ipinfo.io` (an invalid token gets
  `401`); a redirect off `ipinfo.io` would be refused, because only the starting host is
  allowed for this provider.

**The MaxMind licence key and the IPinfo token** are write-only. They are stored sealed
(XChaCha20-Poly1305 under a key derived from `MW_SERVER_KEY`, bound to the provider, so a
key saved for MaxMind does not open for IPinfo), never returned by the API, never written to
the audit log or the server log, and the download URL that carries one is never logged,
stored or shown (the status shows the URL without its query). Changing provider drops the
stored key.

## Downloads

Every download is one request of the egress class `geoip` (see
[egress](../security/egress.md)):

- HTTPS only; the egress address policy applies to every hop, and an egress route that
  lists `geoip` carries it (a deployment that must use a proxy adds `geoip` to that route);
- redirects are followed only to `https` on the starting host, or to the hosts the
  provider names (MaxMind's storage host) or you list for a custom source;
- at most 128 MB downloaded and 256 MB unpacked; timeouts of 15 s to connect, 30 s for the
  first byte, 30 s between chunks and 5 minutes in all;
- gzip and gzip+tar (the first `*.mmdb` member) are unpacked; anything else is taken as a
  bare `.mmdb`.

**Verification.** Where the provider publishes a checksum (MaxMind) the download must match
it. DB-IP shows checksums only on its HTML page, so DB-IP, IPinfo and custom sources without a
checksum URL are checked structurally: the whole file passes the MaxMind DB format check
(search tree and data section), at least one network carries a country code, and a lookup of
that network's first address returns the same country. The same check applies to uploads.

**Replacement and rollback.** The data directory holds `active.mmdb` and `previous.mmdb`,
each with a `.json` file that records its source, release, digest and who installed it. A
new database is written to a temporary file, renamed over `active.mmdb` (an atomic replace),
and the file it replaced is kept as `previous.mmdb`. **Roll back** swaps the two, so a second
rollback undoes the first. The reader in memory is swapped at the same moment; lookups that
are running finish on the database they started with, and no lookup sees "no database"
during the swap.

## Automatic updates

On by default, every 7 days (1 to 90). The schedule starts only after the first download
from the chosen provider: the server never contacts a provider until an administrator presses
**Download now** (or `deploy.sh init --geoip` fetched the first copy). A check that finds the
provider's current release already installed changes nothing (DB-IP: same month; MaxMind:
same SHA-256; others: same file digest). A failed check is retried after 1 hour, then 2, 4,
8, 16 and at most 24 hours. With several server instances on one database, one instance
claims each run. Every scheduled run writes a security audit row
(`geoip-database-update`, actor `system`) with its result.

A database built more than 45 days ago is shown as stale, and the dashboard warns while a
country policy is active.

## Console

Security → Country database shows the state (loaded, missing or invalid, with the reason),
who manages it, the source and release, build date, file size, the number of networks with a
country, how it was verified, when it was installed and by whom, the last and next check, the
last error and the data directory. Actions:

- **Download now** — fetch from the source and install, even if that release is installed;
- **Check for updates** — install only a newer release;
- **Roll back** — swap with the previous copy;
- **Upload .mmdb** — a `.mmdb` or `.mmdb.gz` file (up to 128 MB through the console; a
  reverse proxy in front may allow less);
- the source: provider, licence key or token, custom URL, checksum URL, redirect hosts and an
  extra CA certificate for a mirror behind a private CA, the automatic update switch and
  interval;
- **Test lookup** — the country the database in use gives an address you type (not stored).

Reading needs unscoped `security.read`; every action needs unscoped `security.write`.
Saving a change of source and uploading also ask for a fresh step-up (the same proof as
other admission changes). Every action is audited: `geoip-configured`,
`geoip-database-downloaded`, `geoip-database-checked`, `geoip-database-rolled-back`,
`geoip-database-uploaded`, with the result and never the key.

API, under the console path: `GET /security/geoip`, `PUT /security/geoip/config`
(`If-Match`), `POST /security/geoip/download`, `/check`, `/rollback`, `/upload` (raw
body), `/lookup` (`{"ip": "…"}`).

## Data directory and volume

| Variable | Default | Meaning |
|---|---|---|
| `MW_GEOIP_DIR` | `geoip/` next to the SQLite file (`/data/geoip` in the Docker image) | where `active.mmdb` and `previous.mmdb` live; required with PostgreSQL |
| `MW_GEOIP_SEED` | unset | a file installed (after validation) when the directory has no database |
| `MW_GEOIP_DB` | unset | an operator-managed file that overrides the console |

In the Docker image the database is at `/data/mailwoman.db`, so the country database is in
`/data/geoip` on the same `data` volume and is included in `deploy.sh backup`. Several
instances share a database only if they share this directory (a shared volume); an instance
notices a file another one installed within a minute. With PostgreSQL, set `MW_GEOIP_DIR` to
a persistent, writable path (a volume in Docker, a PVC in Kubernetes).

`deploy.sh init --geoip` (or `deploy.sh geoip` later) downloads DB-IP Lite on the host into
`<state>/geoip/dbip-country-lite.mmdb` with curl (HTTPS only, size-capped, checked for the
MaxMind DB marker), mounts that directory read-only into the container and sets
`MW_GEOIP_SEED`, so a new installation starts with a database. After that the console and the
schedule keep it current.
