---
screen: true
---
# Country database

The **Country database** tab of the **Security centre** manages the local file that turns an IP address into a country. The sign-in country policies, the country check on signed-in requests, the country-change alerts, the unknown-country rule for the console, and the country labels in sign-in history read it. Lookups are made in this file on the server; no IP address is sent to another service.

![The Country database tab of the Security centre with no database installed, the warning that country features are on, and the status table.](/screenshots/admin-privacy/geoip-overview.webp){.mw-shot}

## Before you start

- Reading the tab needs `security.read` on all domains; every action needs `security.write`. Saving a change of source and uploading a file also ask for your password again.
- A fresh install has no country database. Two features that use it are on by default: the unknown-country block for the console (`security.trust.adminGeoEnabled`) and the session country-change alerts (`security.alerts.countryChangeEnabled`). The tab therefore shows **Country features are on but no country database is loaded** until you install one.
- When the environment variable `MW_GEOIP_DB` names a file, that file is used and the tab is read-only. Unset it to manage the database here.

## Install the database

1. Open **Security centre** and select **Country database**.
2. Under **Source and updates**, choose the provider and, for MaxMind or IPinfo, enter the licence key or token. Select **Save source**.
3. Select **Download now**. The server fetches the file, checks it and puts it in use at once; no restart is needed.
4. Check the status table: **Database**, **Source**, **Built**, **Networks with a country**, **Verification**, **Installed**.

To use your own file instead, use **Upload .mmdb** with a `.mmdb` or `.mmdb.gz` file of up to 128 MB (a reverse proxy in front may allow less). **Test lookup** shows the country the installed database gives for an address you type; the address is not stored.

| Provider | Account | Licence |
| --- | --- | --- |
| DB-IP IP to Country Lite (the default) | none | CC BY 4.0, which requires attribution |
| MaxMind GeoLite2 Country | free account and licence key | GeoLite2 EULA |
| IPinfo Lite | free account and token | CC BY-SA 4.0 |
| Custom URL | your choice | your source's |

The console shows the licence and attribution text of the provider in use. Read each provider's terms before you choose one.

The licence key or token is stored encrypted with a key derived from the server key and bound to the provider. It is never returned by the console or written to the logs. Changing provider deletes the stored key.

## Downloads and updates

Downloads use the `geoip` outbound traffic class (see [Outbound connections](/security/egress)): HTTPS only, at most 128 MB downloaded and 256 MB unpacked, three redirects, and only to the provider's own hosts. A MaxMind download must match the checksum MaxMind publishes; other sources are checked for structure: the file must be a valid MaxMind DB, at least one network must carry a country, and a lookup of that network must return it. A download, upload or rollback that fails leaves the database in use unchanged.

The data directory holds `active.mmdb` and `previous.mmdb`. A new database replaces `active.mmdb` and the old one becomes `previous.mmdb`; **Roll back** swaps the two.

**Automatic updates** are on by default, every 7 days (1 to 90). The schedule runs only when the database in use came from the provider you chose, so the server contacts no provider until you select **Download now** once. A database installed by `deploy.sh init --geoip` or `deploy.sh geoip` is a starting copy and does not start the schedule either. A failed check is retried after 1 hour, then 2, 4, 8, 16 and at most 24 hours. A database built more than 45 days ago is shown as stale.

## When the database is missing

While the instance sign-in country policy (`security.signin.geoScope`) is on and no usable database is loaded, what happens to sign-ins depends on `security.signin.geoNoDatabase`: with `block` (the default) they are refused with `503 signinGeographyUnavailable`; with `allow` they go through. Sign-ins from local network addresses are always allowed. The tab and the dashboard show a warning, and the server log reports it at most once every five minutes.

## Options

| Setting | Default |
| --- | --- |
| Provider, licence key or token, custom URL, automatic updates and interval (on this tab) | DB-IP, updates on, every 7 days |
| [`security.signin.geoScope`](/admin/settings#security.signin.geoScope) (instance sign-in country policy) | `off` |
| [`security.signin.accountGeoMode`](/admin/settings#security.signin.accountGeoMode) (per-account policy) | `off` |
| [`security.signin.geoEnforceSessions`](/admin/settings#security.signin.geoEnforceSessions) (country check on signed-in requests) | off |
| [`security.signin.geoNoDatabase`](/admin/settings#security.signin.geoNoDatabase) | `block` |
| [`security.trust.adminGeoEnabled`](/admin/settings#security.trust.adminGeoEnabled) (unknown-country console block) | on |
| [`security.alerts.countryChangeEnabled`](/admin/settings#security.alerts.countryChangeEnabled) (session country-change alerts) | on |

| Variable | Default | Meaning |
| --- | --- | --- |
| `MW_GEOIP_DIR` | `geoip/` beside the SQLite file | Where `active.mmdb` and `previous.mmdb` are kept; set it with Postgres |
| `MW_GEOIP_SEED` | unset | A file installed when the directory has no database |
| `MW_GEOIP_DB` | unset | A file you manage yourself; the tab becomes read-only |

## Related

- [Sign-in country policy](signin-geography.md)
- [Security centre](security-center.md)
- [Outbound connections](/security/egress)
