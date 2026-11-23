---
title: Nextcloud
---

# Nextcloud

The Nextcloud integration lets a mailbox user work with files, calendars, task lists and address books on a Nextcloud server that an administrator has approved. Each user links their own Nextcloud account with an app password. There is no shared service account.

What users can do:

- **Attach from Nextcloud**: pick files from Nextcloud to attach to a message.
- **Save to Nextcloud**: save a received attachment to Nextcloud.
- **Large-file share links**: for a file over the attachment limit, upload it to Nextcloud and insert a share link instead.
- **Contacts and calendars**: discover calendars, task lists and address books on the linked account, connect the ones you want, and synchronize them (native engine only).

![The Nextcloud accounts section of the settings. With no instance approved it reads: No enabled Nextcloud instances are available. Ask your administrator to configure one.](/screenshots/integrations/nextcloud-user-link.webp){.mw-shot}

This is the section before an administrator approves an instance. Approved instances appear in it, each with its status.

## Before you start

- The server must be able to reach the Nextcloud server over HTTPS.
- Calendar, task and address book sync need the native mail engine. In proxy mode the page shows **Contacts and calendar synchronization requires the native mail engine. File access remains available through your account link.**
- The administrator permission **Nextcloud** (`cap.integration.nextcloud`, default on) must be on for the user. Share links also need **Large-file share links** (`cap.compose.share_links`, default on).

## Approve an instance (administrator)

1. Open **Nextcloud** in the admin console.
2. Choose **Add Nextcloud instance**.
3. Enter a **Display name** that users see when they choose an instance.
4. Enter the **HTTPS installation URL**, the installation root such as `https://cloud.example.org/` or `https://example.org/nextcloud/`. The URL must use HTTPS, must not contain credentials, a query string, a fragment or an encoded path prefix, and its certificate must be trusted and match the hostname.
5. Optionally paste **Additional certificate authorities (PEM)** for a private CA. Public trust and hostname checks stay on. This does not change the certificate trust of an HTTPS forward proxy.
6. Turn on **Allow users to link this instance**.
7. Save.

![The Add Nextcloud instance form filled in with the display name Team cloud, the URL https://cloud.example.org/ and Allow users to link this instance ticked.](/screenshots/integrations/admin-nextcloud-form.webp){.mw-crop}

After saving, the **Nextcloud** page lists the instance with its **Display name**, **HTTPS installation URL** and **Status**. The pencil button edits it.

![The Nextcloud instances page of the administration console listing the instance Team cloud at https://cloud.example.org/ with the status Enabled.](/screenshots/integrations/admin-nextcloud-instance.webp){.mw-shot}

Saving any change to an instance asks users to verify their links again. Disabling an instance stops new operations at once; users can still remove their stale or disabled links.

## Link your account (user)

1. In Nextcloud, open your personal security settings and create an app password.
2. In Mailwoman open **Settings → Nextcloud accounts**. Instances your administrator approved are listed with a status: **Linked and ready**, **Not linked**, or **Instance configuration changed. Verify the link again.**
3. Choose **Link account** (or **Verify link again**).
4. Enter the **Nextcloud app-password username** and the **Nextcloud app password**.
5. Choose **Verify and link**.

Verification makes one authenticated request. Rejected credentials are not retried. If it fails, the password field is cleared and an existing working link is kept. The app password is stored encrypted for this mailbox account only. To remove it, choose **Unlink account**.

## Attach, save and share

### Attach from Nextcloud

In the composer, choose **Attach from Nextcloud**. The button appears only when a Nextcloud link is available. Choose a **Nextcloud instance**, browse the **Nextcloud files** list, select files and choose **Attach** (the label shows the count). The files are added to the message.

### Save to Nextcloud

In the message reader, open the menu of an attachment and choose **Save _name_ to Nextcloud**. Choose the folder and **Save here**. If your administrator requires confirmation for that attachment, tick the acknowledgement first. A blocked attachment is refused: saving to Nextcloud follows the same download rules as downloading.

### Large-file share links

When you add a file over the attachment limit and a Nextcloud link is available, the composer offers **Upload and insert link** instead of failing:

1. Optionally turn on **Protect with a password** and enter a **Share password**, and **Set an expiry date** with an **Expires on** date (it must be tomorrow or later).
2. Choose **Upload and insert link**, or **Not now** to dismiss.
3. The file uploads to the **Mail attachments** folder of your Nextcloud, with a progress bar, and a message line with the link is inserted in the body. The line states the expiry and that the link is password-protected when set.

Files up to 4 GiB are accepted. They upload in 5 MiB chunks, so the server holds one chunk at a time.

## Contacts, calendars and task lists

Open **Contacts and calendars** on a linked instance:

1. Choose a **Collection category**: **Calendars**, **Address books** or **Task lists**.
2. Choose **Discover collections**. Discovery changes nothing locally.
3. Choose **Connect** for each collection you want. The page then reads **Collection connected. Synchronize now to load its contents.**
4. Choose **Synchronize now** to pull current data.

Connected edits and deletions are written to Nextcloud with conditional requests. If Nextcloud refuses a write, the cached object is kept. **Disconnect** keeps cached data and stops remote writes.

To create a calendar or task list in Nextcloud, use **Name of the new collection** and **Create in Nextcloud** under **New collection in Nextcloud**. The new collection is connected here. Address books cannot be created from Mailwoman.

### Automatic refresh

Connected collections refresh in the background when the refresh interval is above zero. The setting is `nextcloud.syncIntervalSecs`, labelled **Nextcloud automatic refresh interval** for administrators and **Automatic refresh interval (seconds)** for users.

| Value | Meaning |
|---|---|
| 300 | Default. |
| 0 | Manual refresh only. |
| 1 to 59 | Treated as 60. |
| Up to 86400 | Allowed maximum, in seconds. |

Administrators can set it globally or per domain and lock it; users can choose their own value or **Use administrator default**. A refresh runs per account with a 30-second limit and backs off after failures. Rejected credentials stop further requests until you relink. Background refresh never signs in to mailbox accounts.

## Network and limits

- Nextcloud requests use the **Nextcloud files and DAV requests** traffic class. In **Egress**, a route set to that class and scoped `global` or `domain:<name>` carries them. A failed proxy never falls back to a direct connection. Redirects on requests that carry credentials are refused. See [Outbound connections](../security/egress.md).
- A private-network Nextcloud server needs an internal-destination permission for the Nextcloud class. Metadata addresses stay blocked.
- Each request or response is limited to 8 MiB, with four requests at a time. A request has 15 seconds; a chunk over 1 MiB has 60 seconds and assembling a chunked upload has 120 seconds.
- Do not use a certificate verification bypass to work around trust problems; add the CA to the instance instead.
- Scheduling (free/busy) requests to Nextcloud are not supported.
- In proxy mode, calendar, task and address book sync is not available; file attach, save and share work.

The environment variables `MW_NEXTCLOUD_URL`, `MW_NEXTCLOUD_USER` and `MW_NEXTCLOUD_APP_PASSWORD` are deprecated and ignored; they do not give any mailbox access to a shared Nextcloud account.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| **No enabled Nextcloud instances are available. Ask your administrator to configure one.** | No instance is approved and enabled. | Add and enable an instance in the admin console. |
| **Instance configuration changed. Verify the link again.** | An administrator changed the instance. | Choose **Verify link again** and enter the app password. |
| Attach from Nextcloud is missing in the composer. | No verified link, or the integration is off for you. | Link your account; ask an administrator to check **Nextcloud**. |
| Large files fail with a size error and offer no link. | Share links are off or no link exists. | Check **Large-file share links** and your Nextcloud link. |
| **No verified Nextcloud links. Add or verify a link in User settings → Nextcloud.** | The picker has nothing to choose from. | Link the account as above. |

## Related

- [Outbound connections](../security/egress.md)
- [Administrator settings](../admin/settings.md)
- [Plugins](../plugins/index.md): the older Nextcloud plugin is not loaded by the server
