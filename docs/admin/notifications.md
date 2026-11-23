---
title: Notification settings
---

# Notification settings

Administrators control notifications through values in the console's **Settings**: what users may turn on, whether background push is allowed at all, and a quiet window for the whole server. What users see and set for themselves is described in [Notifications](/user/notifications) (the list of account activity) and [Settings: interface](/user/settings/interface) (alerts for new mail). The server side of background push, including the endpoints it uses, is described in [Push notifications](/deploy/push).

Open **Settings** in the administration console and search for the label, or open `#/settings/<key>` directly. Each value can be set for the whole server, per domain and per user, and **Lock for users** makes a value final. Which of those apply to a setting is the last column below.

## Before you start

- You need the console permission to read and change settings. Changes apply within 30 seconds on every server process.
- Alerts for new mail are off until a user turns them on. Background push, which delivers while the app is closed, also needs the user to register a browser, and the browser's own notification permission.

## Allow or limit notifications

![The setting Background push notifications allowed in the console's Settings, with its value for the whole server, per domain and per user.](/screenshots/admin-ops/opsc-settings-push.webp){.mw-crop}

1. Open **Settings** and find **Background push notifications allowed**.
2. Under **For the whole server**, tick or untick the value and choose **Save**. Turn on **Lock for users** if domains and users must not differ.
3. To let some domains differ, set the value in the per-domain section below.

With **Background push notifications allowed** off, a browser cannot register for background push and the server sends no background wake for that account. The setting does not stop alerts shown while the app is open.

### Quiet hours for push

**Push quiet hours** (`push.quietHours`) is a window in UTC in which the server sends no background wake at all. The value is JSON with an hour from 0 to 23 for each end, for example `{"start": 22, "end": 6}`. The window includes `start`, excludes `end`, and wraps past midnight when `start` is later than `end`. Leave it empty for no quiet window.

![The setting Push quiet hours in the console's Settings.](/screenshots/admin-ops/opsc-settings-quiet.webp){.mw-crop}

If the quiet-hours setting cannot be read when a wake is about to be sent, the server sends nothing. If `push.quietHours` has never been set, the server reads the older value `push.quiet_hours` (a `"start-end"` string such as `"22-7"`); setting `push.quietHours`, even to empty, ends that fallback.

### Push contact

**Push contact** (`push.vapidContact`) is the contact address that Web Push services receive in the signed token with each background wake, so they can reach you about delivery problems. It must be `mailto:` followed by an address, or an `https://` URL. When it is set, the server uses it. When it is empty, the server uses the environment variable `MW_VAPID_CONTACT`, and when that is empty too it uses `mailto:push@mailwoman.invalid`, an address that cannot receive mail. Setting `MW_VAPID_CONTACT` pins the setting: the console then shows it as fixed.

## Options

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **New-mail notifications** ([`notifications.enabled`](/admin/settings#notifications.enabled)) | Whether the account's new-mail alerts are on | Off | Server, domain, user |
| **Background push notifications allowed** ([`notifications.pushAllowed`](/admin/settings#notifications.pushallowed)) | Ceiling for push registration and delivery | On | Server, domain |
| **Push quiet hours** ([`push.quietHours`](/admin/settings#push.quiethours)) | UTC window with no background wake | Not set | Server |
| **Push contact** ([`push.vapidContact`](/admin/settings#push.vapidcontact)) | Contact sent to Web Push services | Not set; `MW_VAPID_CONTACT`, then `mailto:push@mailwoman.invalid` | Server |
| **Persistent browser storage requests allowed** ([`offline.persistentStorageAllowed`](/admin/settings#offline.persistentstorageallowed)) | Ceiling for the browser's persistent-storage request. The browser decides whether to grant it. | On | Server, domain |
| **Offer notification permission** ([`notifications.askPermission`](/admin/settings#notifications.askpermission)) | Whether the app offers the browser's notification permission request | On | Server, domain, user |
| **Enable the Notifications view** ([`notifications.view.enabled`](/admin/settings#notifications.view.enabled)) | Makes the **Notifications** list available | On | Server, domain, user |
| **Show Notifications in the sidebar** ([`notifications.view.showInSidebar`](/admin/settings#notifications.view.showinsidebar)) | Shows that list in the sidebar | On | Server, domain, user |

The settings whose labels begin **New-mail**, **Native new-mail notification**, **Flash the browser tab**, **Highlight arriving messages**, **Show toasts** and similar (keys starting `notifications.incoming.`, `notifications.events.` and `notifications.toasts.`) are the choices users see under Notifications in their own settings. Set them here to change the default for a domain or to lock a choice; each is described in the [settings reference](/admin/settings).

## Limits to know

- Background push carries only an opaque wake marker, never message text. An account with content rules (notify only for certain senders or subjects) gets no background system notification, because the wake does not carry enough to evaluate the rules.
- Web Push and UnifiedPush deliveries use the **push** traffic class of the outbound connections: HTTPS only, public destination addresses, no redirects. The APNs transport is a stub that only logs; no push reaches Apple devices.

## Troubleshooting

- The **Register background notifications** button is disabled for a user: either new-mail notifications are off for them or **Background push notifications allowed** is off for their domain. The page also says the administrator does not allow push.
- No background notification arrives at certain hours: check **Push quiet hours**. It is read in UTC.
- A push service reports delivery problems and cannot reach you: set **Push contact**.
- The app never offers the browser notification permission: it offers it only when new-mail notifications are on for the account, **Offer notification permission** is on, and the browser has not yet allowed or blocked notifications. The browser's own prompt appears only after the user chooses to allow it in the app's offer.

## Related

- [Push notifications](/deploy/push)
- [Notifications](/user/notifications)
- [Settings reference](/admin/settings)
- [Outbound connections](/security/egress)
