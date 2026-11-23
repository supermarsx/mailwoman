---
screen: true
---
# Cookie consent

The **Cookies** tab of **Compliance** configures the cookie consent control shown on the sign-in page and in the mail app, lists the cookies and browser storage the application uses, and keeps a log of the choices people make. The control works in the browser and covers the storage the page itself writes. Deciding whether your deployment needs the control, and in which form, is for you.

![The Cookies tab of Compliance with the setting groups for the consent control, starting with Control and regions.](/screenshots/admin-privacy/cookies-tab.webp){.mw-shot}

## Before you start

- The tab needs `settings.read` (and `settings.write` to change anything) on all domains.
- The control is off until you turn on **Cookie consent control** ([`cookies.enabled`](/admin/settings#cookies.enabled)). While it is off, nothing is shown, nothing is enforced and nothing is logged.

## What people see

With the control on, the sign-in page and the mail app show a banner, card or dialog (**Layout**, **Position**) with **Accept all**, **Reject all** and **Choose categories**. **Choose categories** shows one switch per offered category, the storage in each, and **Save choices**. With **Reject as prominent as accept** on (the default), the two buttons have the same size and style. In the `noticeOnly` model, the control shows only **OK**. Links go to your **Cookie policy URL** and **Privacy policy URL**, or to the server's own `/cookies-policy` and `/privacy` pages when those are empty.

A **Cookie preferences** link on the sign-in page and the **Cookies and browser storage** card under **Settings > Privacy** open the same choices again. **Preview the control** on the tab shows the control as it is configured.

![The cookie consent banner at the bottom of the sign-in page, with Accept all, Reject all and Choose categories.](/screenshots/admin-privacy/cookie-banner.webp){.mw-shot}

## Categories

| Category | Offered | Storage in the built-in inventory |
| --- | --- | --- |
| Strictly necessary | always | Session and security cookies, the stored choice itself, start-up guards |
| Preferences and functions | **Offer functional cookies**, on by default | Theme, language, layout, drafts kept in the browser, remembered sign-in addresses, the offline database, and syncing appearance to the account |
| Analytics | **Offer analytics cookies**, off by default | Nothing |
| Marketing | **Offer marketing cookies**, off by default | Nothing |

A category that is not offered is treated as declined.

::: warning Analytics and marketing block nothing by default
The application stores nothing in the analytics and marketing categories, so declining them blocks nothing unless you add rules that put names in them. The analytics provider script of [Usage analytics](analytics.md) is not controlled by this banner: it follows each person's own consent switch for that script.
:::

## How choices are applied

Enforcement happens in the browser, from the configuration the server sends:

- Writes to `localStorage`, `sessionStorage`, cookies set by page scripts, and IndexedDB databases are dropped when their category is declined.
- Storage already present in a declined category is removed when the configuration loads and whenever the choice changes.
- Syncing appearance to the account waits for the functional category.
- Cookies the server sets as `HttpOnly` cannot be read or removed by the page and are not affected.
- A name that matches no inventory entry and no rule follows **Unclassified storage**. The default, `report`, lets the write through and lists the name for review.
- If the configuration cannot be loaded, nothing is blocked.

**Consent models.** **Consent model** applies outside the strict regions and **Consent model in strict regions** inside them. `optIn`: nothing optional is stored until the visitor accepts. `optOut`: each category's default applies until the visitor declines. `noticeOnly`: the visitor is told once and the defaults apply. The visitor's country comes from the request header named in **Country header** (default `X-Country-Code`), else from the region of the browser's first language. Only trust that header if your proxy sets it and overwrites any value a client sends. **Strict regions** lists the countries that use the strict model (EU and EEA states, the United Kingdom and Switzerland by default); **Unknown country** decides when there is none.

**Privacy signals.** With **Honour Global Privacy Control** on (the default), a Global Privacy Control signal turns analytics and marketing off. **Honour Do Not Track** does the same for Do Not Track (off by default).

**Lifetime.** The choice is stored in the browser with the policy version. The visitor is asked again after **Consent lifetime** days (365) and, with **Ask again on a new policy version** on, when **Policy version** changes. For a signed-in person, the choice is also kept on the server so another browser of the same account can use it.

## Check the inventory

1. On the **Cookies** tab, scroll to **Cookie and storage inventory**.

   ![The top of the cookie and storage inventory: the built-in entries with their kind, category, purpose, duration, provider and source.](/screenshots/admin-privacy/cookies-inventory.webp){.mw-crop}

2. Compare it with what your deployment sets. The built-in entries are a list in the server's code, checked against the cookies the server sets (see below).
3. To classify a name the list does not cover, use **Add a rule**: a kind, a name pattern (`*` matches any run of characters; a pattern of only `*` is refused), a category and a **Note**. A rule overrides the built-in entry for the same name, and an exact name beats a pattern. Up to 200 rules.
4. **Scan this browser** lists the names in your own browser's storage. Names no entry or rule covers appear under **Unclassified storage**, where you can **Classify** (adds an exact rule), **Ignore** or remove them. People's browsers report such names too, at most once per **Storage scan interval** hours (24); only names and kinds are sent, never values. The server keeps at most 500 names.

### Cookies the server sets

These are the cookies the server sets itself, from the same list the consent control, the generated cookie policy and the inventory above use. A test starts the server, runs the flows that set cookies (mailbox sign-in, session refresh, remembering a device, admin sign-in, the client-access check, OIDC authorize) and fails when a cookie is set that this list does not cover or when its `HttpOnly` flag differs. Another test fails when this table and the list differ.

| Cookie | Category | Purpose | Lifetime | HttpOnly |
| --- | --- | --- | --- | --- |
| `mw_admin_browser` | necessary | Marks a browser used for the admin console so the analytics tag is not loaded in it. | 400 days | Yes |
| `mw_admin_session` | necessary | Keeps an administrator signed in to the admin console. The name can be changed with MW_ADMIN_SESSION_COOKIE. | Until the admin session ends (30 minutes without activity, 12 hours at most, by default) | Yes |
| `mw_browser_device` | necessary | Recognises this browser at sign-in so a sign-in from a new device can be reported to you. | 365 days | Yes |
| `mw_client_access*` | necessary | Holds the proof of a client-access check when the administrator requires one. | Until the check expires | Yes |
| `mw_csrf` | necessary | Protects requests made while you are signed in against forgery. Page script reads it to echo it back. | Until the sign-in session ends (7 days without activity by default); a copy set without an expiry lasts until the browser closes | No |
| `mw_oidc_flow` | necessary | Ties an in-progress sign-in through this server's identity provider to the browser that started it. | 10 minutes | Yes |
| `mw_remembered_device` | necessary | Marks this browser as a remembered device, so a second-factor prompt is skipped while the period lasts. | The remembered-device period the administrator sets (off by default) | Yes |
| `mw_session` | necessary | Keeps you signed in to the mail app. | Until the sign-in session ends (7 days without activity by default) | Yes |

How the figures arise:

- `mw_session` and `mw_csrf` are set with a `Max-Age` equal to the time left in the mailbox session, at sign-in and whenever the session is refreshed. The idle timeout is [`session.idleSecs`](/admin/settings#session.idleSecs) (7 days) and the optional absolute limit is [`session.absoluteSecs`](/admin/settings#session.absoluteSecs) (off). Some responses issue `mw_csrf` without an expiry, and then it lasts until the browser closes.
- `mw_admin_session` carries [`admin.session.absoluteSecs`](/admin/settings#admin.session.absoluteSecs) (12 hours) as `Max-Age`; the server also ends it after [`admin.session.idleSecs`](/admin/settings#admin.session.idleSecs) (30 minutes) without a request. Set `MW_ADMIN_SESSION_COOKIE` to give it another name: the inventory then lists that name in place of `mw_admin_session`, and a name you choose is classified as necessary the same way.
- `mw_remembered_device` is set only when a person remembers a device, and only while [`twofa.rememberDays`](/admin/settings#twofa.rememberDays) is above 0 (the default is 0). Its `Max-Age` is that many days.
- `mw_client_access*` covers `mw_client_access` (the proof) and `mw_client_access_pending` (a check in progress).

All but `mw_csrf` are `HttpOnly`, so a browser scan cannot see them and the consent control cannot remove them. The client has no translated text for `mw_browser_device`, `mw_remembered_device` and `mw_oidc_flow` yet; it shows the English purpose above.

## Consent log and statistics

With **Keep a consent log** on (the default), each choice is stored with the time, domain, policy version, decision (accept all, reject all, custom or notice), the category choices, a country code (`??` when unknown), the privacy signal and, for a signed-in person only, the account id. No address, IP address or browser details are stored. **Consent log retention** is 0 by default, which keeps the log without limit; a positive number of days removes older rows, checked at most once an hour.

**Consent statistics** shows the decisions, the share of visitors allowing each category, decisions per day and the top regions over 7, 30 or 90 days. **Export log (CSV)** downloads the rows of the period and is recorded in the audit log.

## Options

All keys can be set per domain unless noted. See the [settings reference](/admin/settings#cookies.enabled).

| Setting | Default |
| --- | --- |
| **Cookie consent control** ([`cookies.enabled`](/admin/settings#cookies.enabled)) | off |
| **Consent model** ([`cookies.mode`](/admin/settings#cookies.mode)) / **Consent model in strict regions** ([`cookies.regionMode`](/admin/settings#cookies.regionMode)) | `optIn` / `optIn` |
| **Strict regions** ([`cookies.strictRegions`](/admin/settings#cookies.strictRegions)), **Country header** ([`cookies.regionHeader`](/admin/settings#cookies.regionHeader)), **Unknown country** ([`cookies.unknownRegion`](/admin/settings#cookies.unknownRegion)) | EU/EEA, GB, CH; `X-Country-Code`; `strict` |
| **Offer functional cookies** / **Functional default** (`cookies.functional.*`) | on / on |
| **Offer analytics cookies** / **Analytics default** (`cookies.analytics.*`) | off / off |
| **Offer marketing cookies** / **Marketing default** (`cookies.marketing.*`) | off / off |
| **Layout** ([`cookies.layout`](/admin/settings#cookies.layout)), **Position** ([`cookies.position`](/admin/settings#cookies.position)) | banner, bottom |
| **Reject as prominent as accept** ([`cookies.rejectProminent`](/admin/settings#cookies.rejectProminent)) | on |
| **Cookie policy URL**, **Privacy policy URL** (`cookies.policyUrl`, `cookies.privacyUrl`) | empty (server pages) |
| **Policy version** ([`cookies.policyVersion`](/admin/settings#cookies.policyVersion)) | 1 |
| **Consent lifetime** ([`cookies.lifetimeDays`](/admin/settings#cookies.lifetimeDays)) | 365 days |
| **Ask again on a new policy version** ([`cookies.repromptOnVersion`](/admin/settings#cookies.repromptOnVersion)) | on |
| **Honour Global Privacy Control** ([`cookies.honourGpc`](/admin/settings#cookies.honourGpc)) / **Honour Do Not Track** ([`cookies.honourDnt`](/admin/settings#cookies.honourDnt)) | on / off |
| **Keep a consent log** ([`cookies.logConsent`](/admin/settings#cookies.logConsent)) | on |
| **Consent log retention** ([`cookies.logRetentionDays`](/admin/settings#cookies.logRetentionDays)) | 0 (no limit) |
| **Storage scan interval** ([`cookies.scanIntervalHours`](/admin/settings#cookies.scanIntervalHours)) | 24 hours; 0 scans only on request |
| **Unclassified storage** ([`cookies.unclassified`](/admin/settings#cookies.unclassified)) | `report` |
| Texts of the control and the category names and descriptions (`cookies.text.*`, `cookies.label.*`, `cookies.description.*`) | built-in text |

## Troubleshooting

| Symptom | Cause | Fix |
| --- | --- | --- |
| The banner never appears | **Cookie consent control** is off (the default). | Turn it on, for all domains or the domain concerned. |
| Every visitor gets the strict model | No country is known and **Unknown country** is strict, or your proxy does not set the country header. | Set the header at the proxy, or change **Unknown country**. |

## Related

- [Restrictions and consent](consent.md)
- [Usage analytics](analytics.md)
- [Compliance: policy pages](compliance.md#privacy-policy-and-cookie-policy)
- [Cookies and consent in the mail app](/user/settings/account#cookies-and-consent)
