# Cookie consent controls

This page describes the cookie consent control, the cookie and storage inventory, the storage scan and the consent log. They are configurable controls. This documentation does not say that any configuration meets a particular law or regulation; deciding what your deployment needs is the operator's responsibility.

The admin screen is **Privacy > Cookies**. Every setting has the key `cookies.*` and is also listed in [Settings](settings.md#cookies.enabled), where it can be set globally or per domain, and locked.

## What the control does

When `cookies.enabled` is on, the sign-in page and the mail app show a banner, card or dialog (`cookies.layout`, `cookies.position`) with:

* **Accept all** and **Reject all**. With `cookies.rejectProminent` on they have the same size and style.
* **Choose categories**: one toggle per offered category and a list of what is stored in each, with purpose and duration.
* Links to the cookie policy and privacy policy (`cookies.policyUrl`, `cookies.privacyUrl`).

A **Cookie preferences** link on the sign-in page and under Settings > Privacy reopens the same dialog. The dialog is keyboard operable, names itself and its text for assistive technology, traps focus in the dialog layout, closes on Escape when reopened from a link, uses the theme tokens and keeps borders and focus outlines in forced-colors mode.

### Categories

| Category | Offered | What the application sets |
| --- | --- | --- |
| Strictly necessary | always on | Session, CSRF, admin session, client-access proof, the consent record, boot and reload guards. |
| Preferences and functions | `cookies.functional.enabled`, default on | Theme, language, layout memory, drafts kept in the browser, remembered sign-in addresses, offline data, and syncing appearance to the account. |
| Analytics | `cookies.analytics.enabled`, default off | Nothing set by the application itself. Use it for the analytics tag or a plugin. |
| Marketing | `cookies.marketing.enabled`, default off | Nothing set by the application itself. |

A category that is not offered is treated as not consented.

### Consent models

`cookies.mode` applies outside the strict regions, `cookies.regionMode` inside them.

* `optIn`: nothing optional is stored until the visitor accepts it.
* `optOut`: each category's `cookies.<category>.default` applies until the visitor rejects it.
* `noticeOnly`: the visitor is informed once and the defaults apply; there are no toggles.
* `cookies.regionMode = same` uses `cookies.mode` in the strict regions too.

The visitor's country comes from the request header named by `cookies.regionHeader` (default `X-Country-Code`, set by a proxy or CDN that you trust and that overwrites any value a client sends), else from the region subtag of the first `Accept-Language` entry. Countries in `cookies.strictRegions` (default: EU and EEA states, United Kingdom, Switzerland) use `cookies.regionMode`. When no country can be determined, `cookies.unknownRegion` decides.

### Privacy signals

With `cookies.honourGpc` on (default) a Global Privacy Control signal turns analytics and marketing off and disables their toggles, and the control says why. `cookies.honourDnt` does the same for Do Not Track (default off). The recorded decision carries the signal that applied.

### Re-prompting and lifetime

A decision is stored in the browser (`localStorage` key `mw.consent`) with the policy version and the time. The visitor is asked again when `cookies.lifetimeDays` (365 days by default) has passed or, with `cookies.repromptOnVersion` on, when `cookies.policyVersion` differs from the stored one. Raise the version when your cookie use or policy changes.

For a signed-in visitor the decision is also kept on the server, so another browser of the same account can adopt it when it has no decision of its own.

### Enforcement

Enforcement is in the browser, driven by the configuration the server sends:

* Writes to `localStorage`, `sessionStorage`, cookies set by page scripts and IndexedDB databases are blocked when their category is not consented. A blocked write is dropped as in a browser with storage turned off.
* Storage already present in a category that is not consented is removed when the configuration loads and whenever the choice changes.
* The appearance sync to the account is a functional feature and waits for consent to the preferences category.
* Server-set cookies (session, CSRF and similar) are strictly necessary and are not affected.

Names that match no inventory entry and no rule follow `cookies.unclassified`: `report` leaves them alone and lists them for review, any category name treats them as that category.

If the configuration cannot be loaded, nothing is blocked; the last configuration and decision kept in the browser still apply.

## Inventory and rules

The inventory is the list of cookies and browser storage this build sets, each with name pattern, kind (cookie, local storage, session storage, IndexedDB), category, purpose, duration, party and provider. It is part of the application and is shown read-only in the admin table and to visitors under the category toggles.

**Rules** are yours. A rule has a kind, a name pattern (`*` stands for any run of characters; a pattern of only `*` is refused), a category and a note. A rule overrides the built-in entry for the same name, and an exact name beats a wildcard. Up to 200 rules.

## Storage scan and unclassified names

The browser scans `document.cookie`, `localStorage`, `sessionStorage` and IndexedDB database names when it loads (at most every `cookies.scanIntervalHours`; 0 means only on demand), from **Check storage now** under Settings > Privacy, and from **Scan this browser** in the admin tab.

* Only names and kinds leave the browser. Values are never read for the scan and never sent.
* Names that no entry or rule classifies are reported to `POST /api/cookies/detected`. The server keeps the name, kind, first and last time seen and a count, at most 500 names, and only while `cookies.enabled` is on. Requests are rate limited and a body with extra fields is refused.
* In **Unclassified storage** you can classify a name (which adds an exact rule), ignore it or remove it.

## Consent log and statistics

With `cookies.logConsent` on (default) each decision is stored with: time, domain, policy version, decision (`acceptAll`, `rejectAll`, `custom`, `notice`), the category choices, a region code (`??` when unknown), the privacy signal and, only for a signed-in visitor, the account id. No address, IP address or user agent is stored. Consent records are kept without an age limit by default. A positive `cookies.logRetentionDays` enables age-based cleanup for that domain, checked at most once an hour. Browser consent renewal still follows `cookies.lifetimeDays`.

The tab shows decision counts, the share of visitors allowing each category, decisions per day (UTC) and the top regions, over 7, 30 or 90 days. **Export log (CSV)** downloads the rows of the period; the export is audited. Statistics cover only decisions recorded while the log was on.

## Endpoints

Public, resolved for the requesting domain:

* `GET /api/cookies/config` returns `{ "enabled": false }` when off, else the resolved model, region, texts, categories, inventory and rules. Not cached.
* `POST /api/cookies/consent` records a decision anonymously. Body: `version`, `decision`, `categories`, `signal`. Answers 204 also when the log is off.
* `POST /api/cookies/detected` reports names, as above. These two POST routes read no cookie and are exempt from the CSRF token check (not from the Origin check).

Signed in (CSRF-protected): `POST` and `GET /api/cookies/consent/account` record and read the newest decision of the account.

Admin (`settings.read` / `settings.write`, unscoped): `GET /admin/cookies/inventory`, `POST /admin/cookies/rules`, `DELETE /admin/cookies/rules/{id}`, `GET` and `POST /admin/cookies/detected`, `POST /admin/cookies/detected/{id}/classify`, `POST .../ignore`, `DELETE /admin/cookies/detected/{id}`, `GET /admin/cookies/stats`, `GET /admin/cookies/consents.csv`.

## Limits to know about

* Enforcement covers storage the page itself writes. A third-party script you add can set its own cookies from its own origin; load such scripts only after the matching category is consented.
* Per-user overrides are not offered: the user's own choice is their consent decision, not a setting.
* The consent control, region rules and statistics were exercised in automated tests; they have not been reviewed by a lawyer, and nothing here is legal advice.
