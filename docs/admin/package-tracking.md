---
title: Package tracking
screen: true
---

# Package tracking

**Package tracking** configures the parcel tracking provider that your users' **Package tracking** page talks to. Mailwoman uses one provider, AfterShip Tracking. The feature is off until you enable the provider here and each user opts in for their own account.

Open **Package tracking** in the **Configuration** group of the console navigation. The screen is titled **Package tracking provider**.

![The Package tracking provider screen with the AfterShip Tracking card: the checkbox to allow opted-in accounts to contact the provider, the provider API key, the minimum refresh interval and the save button.](/screenshots/admin-ops/package-tracking-screen.webp){.mw-shot}

## Before you start

- You need an AfterShip account and an API key that has permission to read and create trackings. You manage billing, provider retention and carrier access in AfterShip, not in Mailwoman.
- You need the console capability to change integrations. Without it the form is read-only. Saving also asks you to confirm that it is you (your password and second factor, or a passkey) if you have not done so recently.
- Mailwoman sends requests to the provider through the **package_tracking** outbound route. If your server needs a proxy to reach the internet, set one up for that route in [Outbound connections](/security/egress). This route only allows public HTTPS addresses, and it refuses redirects and proxies that resolve names themselves.

## Set up the provider

1. Open **Package tracking**.
2. Tick **Allow opted-in accounts to contact this provider**.
3. Enter the **Provider API key**. It is required the first time. The server stores it sealed and never sends it back to the browser. Later the field says "A key is stored. Leave empty to retain it, or enter a replacement. The existing key is never shown."
4. Set **Minimum refresh interval in seconds**, between 60 and 86400.
5. Choose **Save provider configuration**. The screen says "Provider configuration saved. Accounts still need their own opt-in before sending a tracking request."

![The AfterShip Tracking card with markers 1 to 3 on the allow checkbox, the provider API key and the refresh interval.](/screenshots/admin-ops/package-tracking-form.webp){.mw-crop}

1. **Allow opted-in accounts to contact this provider** turns the provider on or off for everyone.
2. **Provider API key** takes a key of up to 512 visible characters.
3. **Minimum refresh interval in seconds** is how long a package must wait before it can be refreshed again.
{.mw-callouts}

The key is sealed with the server's key. Keep that key with your backups, or a restored server cannot read the stored provider key.

## What users can do

Each user opens **Settings**, then **Package tracking**, ticks **Enable package tracking for my account**, and adds packages by tracking number and carrier code. See [Package tracking](/user/packages) in the user guide.

- Saving a package sends nothing to the provider.
- **Track or refresh** sends only the tracking number and the carrier code. Mailwoman does not read message bodies, does not send labels or email addresses, and does not check packages in the background.
- An account can hold up to 100 packages. The most recent 20 tracking events are kept for each.
- Each package can be refreshed once per **Minimum refresh interval in seconds**, counting failed attempts too. After a failed refresh the last good status stays.
- Only one refresh runs at a time on the whole server. A request waits briefly for its turn.
- Requests have a 15 second limit and are not retried automatically. A response larger than 1 MiB is refused.
- Removing a package in Mailwoman does not delete the tracking at the provider.

## Options

| Label | What it does | Default | Who can change it |
| --- | --- | --- | --- |
| **Allow opted-in accounts to contact this provider** | Turns the provider on | off | Administrators with the integrations capability, with a recent confirmation |
| **Provider API key** | Key used for every request. Stored sealed | none | Same |
| **Minimum refresh interval in seconds** | Shortest time between two refreshes of one package, 60 to 86400 | 900 | Same |
| **Package tracking allowed** ([`packages.allowed`](/admin/settings#packages.allowed)) | Allows or forbids the feature for a domain or an account. It does not replace each user's opt-in | on | Administrators globally, per domain and per user account |

## Troubleshooting

- **A user sees "Package tracking is disabled or has not been configured by your administrator."** The provider is not enabled or has no key, **Package tracking allowed** is off for the user's domain or account, or the user has not opted in.
- **A refresh fails with "The tracking provider could not complete the request. Check the carrier and number, then wait before retrying. Your administrator can check the provider configuration."** The provider could not be reached or refused the request. Check the key, that the server can reach `api.aftership.com` over HTTPS, and the **package_tracking** route. The message appears for any failure to reach or hear back from the provider within 15 seconds.
- **"The provider returned an unexpected response. The saved package status was not replaced."** The provider answered in a form Mailwoman does not understand. The earlier status is kept.
- **"This package was checked recently. Wait for the configured refresh interval."** The refresh interval has not passed yet. This is expected.
- **"Another tracking update is in progress. Please try again shortly."** Another refresh holds the single slot. Try again.
- **"Your account can save up to 100 packages."** The user has reached the limit and must remove one.
- **Some carriers cannot be tracked.** Carriers that need extra information, such as a postal code, are not supported by this integration.

## Related

- [Package tracking](/user/packages) for what users see
- [Outbound connections](/security/egress) for the route this feature uses
- [Settings](/admin/settings) for `packages.allowed`
