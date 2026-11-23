---
screen: true
---
# Usage analytics

The **Usage analytics** screen of the console configures two separate, optional features: server-side usage counts, which the server sends to an analytics collector for people who agreed, and a browser analytics tag, which loads an analytics provider's own script in the mail app for people who agreed to that provider. Both are off by default.

![The Usage analytics screen with collection off: the collector configuration form and the exporter status.](/screenshots/admin-privacy/analytics-overview.webp){.mw-shot}

## Before you start

- Reading the screen needs `integrations.read`; saving needs `integrations.write` on all domains and your password again.
- Each person decides separately, under **Settings > Privacy > Optional usage counts** in the mail app: one switch for the usage counts, and one for the browser tag that names the provider. Nothing is collected for a person who has not agreed.
- The cookie consent banner does not control either feature (see [Cookie consent](cookies.md)), and the consent ledger's **Usage counts** and **Analytics provider script** purposes only keep receipts (see [Restrictions and consent](consent.md#purposes)).

## Server-side usage counts

1. Under **Collector configuration**, tick **Enable optional analytics collection**.
2. Choose the **Provider**: Umami, Matomo or Google Analytics 4. For Umami and Matomo enter the **Collector endpoint** and **Website or site ID**; for Google Analytics 4 choose the global or EU endpoint and enter the **Measurement ID** and **Measurement Protocol API secret**.
3. Select **Save configuration**.

What is sent: one event per screen the person opens, named mail, calendar, tasks, contacts, notes or settings. The server sends it, not the browser, with the user agent `Mailwoman/1.0 (consented application events)`, so the collector sees the server's address. For Google Analytics 4 each event has a new random client id, the ad consent values are `DENIED`, and only a `screen_view` event is sent.

When an event is dropped:

- the request carries Do Not Track or Global Privacy Control;
- the person has not agreed, or has withdrawn;
- the person is restricted with **Analytics and open tracking** paused;
- more than 60 events per account or 600 in total arrive in a minute, or more than 256 are waiting.

Consent, the session and the configuration are checked again before delivery. There is no retry. **Exporter status** counts events **Queued**, **Accepted by collector** and **Dropped** since the server process started; "accepted" means the collector answered with success, not that it kept the event.

A collector on a private network needs an entry under **Private collector destinations**, which requires `egress.write`. Analytics traffic uses its own outbound traffic class (see [Outbound connections](/security/egress)).

## Browser analytics tag

![The top of the Browser analytics tag form: where the tag runs, the switch that loads it, the provider and the provider address.](/screenshots/admin-privacy/analytics-tag.webp){.mw-crop}

1. Under **Browser analytics tag**, choose the **Tag provider**: Plausible, Matomo, Umami, Fathom, PostHog, Simple Analytics, Cloudflare Web Analytics, or **Custom tag (other provider)**. Fill in the provider's fields.
2. Check **Content Security Policy additions**: the origins the page will allow for scripts, connections and images.
3. Tick **Load this tag for consenting users** and **I understand this provider's script runs with full access to users' webmail pages.**
4. Select **Save browser tag**. **Test** fetches the script through the analytics traffic class and reports the result.

Turning a tag on, or changing an enabled tag, needs a full Super admin as well. The change is recorded in the audit log (`analytics-tag-configured`) with the provider, the script's hash and the allowed origins.

::: warning The provider's script runs inside the mail app
The provider's code runs in the person's browser with access to the mail app page. It sees the page address (including the folder and message route, unless the provider strips it), the browser and the person's IP address, and it can set its own identifiers. The person's agreement is tied to the provider and to the loader file this server generates; the provider can change its own script at any time without anyone being asked again.
:::

The tag is served only in the mail app, only to a signed-in person who agreed to this provider and loader, only without Do Not Track or Global Privacy Control, and never to a browser that carries an administrator session or has been used for the console (the `mw_admin_browser` cookie, kept about 400 days). Some presets switch off the provider's tracking features: Matomo without cookies, Umami without the address's hash and query, PostHog with autocapture, session recording and surveys off and all text masked. When a person withdraws, the tab is cut off from third-party requests and reloads without the tag.

The console is served from the same origin as the mail app, so these cases remain possible:

- an administrator signs in to the console in a browser where a tagged mail tab is already open;
- a shared browser where a consenting person's tag runs before any administrator has used that browser;
- windows and frames the tagged page opens itself.

Do not turn on a browser tag on a deployment whose administrators use the same browser for mail.

## Options

The configuration of this screen is stored as its own records, not as settings, so it does not appear in the settings reference. Related settings:

| Setting | What it does | Default |
| --- | --- | --- |
| [`cap.analytics`](/admin/settings#cap.analytics) | Feature switch for posting usage events; it does not by itself turn collection on | on |
| `restriction.scope.analytics` | Pauses analytics for restricted people | on |
| `consent.analytics.*`, `consent.analyticsTag.*` | Legal basis, version and wording shown to people; receipts only | consent |

## Related

- [Restrictions and consent](consent.md)
- [Cookie consent](cookies.md)
- [Outbound connections](/security/egress)
- [Impact assessments](compliance.md#impact-assessments)
