---
screen: true
---
# Privacy and remote content

The **Privacy** screen of the console sets how the mail app treats remote content in messages (images and other resources loaded from the sender's servers), tracking pixels and tracked links, and shows how the image proxy that fetches remote images is doing. This page also describes the **Attachment policy** screen, which decides which attachments are handed out for download and preview.

![The Overview tab of Privacy and remote content: the image proxy status, the key figures and the time range buttons.](/screenshots/admin-privacy/privacy-overview.webp){.mw-shot}

## Before you start

| Tab | Address | Needs |
| --- | --- | --- |
| **Overview** | `#/privacy/overview` | `egress.read`, all domains |
| **Remote content and trackers** | `#/privacy/remote` | `settings.read` |
| **Image proxy** | `#/privacy/image-proxy` | `egress.read` or `settings.read`, all domains |
| **Fetch limits** | `#/privacy/fetch-limits` | `settings.read` |
| **Traffic** | `#/privacy/traffic` | `egress.read`, all domains |

Changing a setting needs `settings.write`. The settings on these tabs are server-wide defaults with locks; most can also be set per domain with **Per-domain values**.

## How remote images are loaded

When a person allows remote images in a message, the browser asks this server for them; the server fetches each image itself through the image proxy:

- only for a signed-in person, and only within the permission to load images for that message;
- it resolves the address itself and refuses private, loopback and cloud metadata addresses, and checks every redirect again;
- it sends the user agent `Mailwoman-Image-Proxy` and forwards no cookie, referrer or authorization header from the browser;
- it re-encodes each image to PNG in a sandbox, which drops the image's metadata;
- it caches images in memory, and on disk when `privacy.imageDiskEnabled` is on (off by default).

The remote server therefore sees the server's address, not the person's, and it still sees when an image is fetched, which can tell it that the message was opened. A permission is per message: a session that may load images for a message can have any public URL fetched under that message.

## Set the remote content policy

1. Open **Privacy** and select **Remote content and trackers**.

   ![The top of the Remote-content policy settings: Remote content set to ask, with the switch that lets people change it and the lock.](/screenshots/admin-privacy/privacy-remote.webp){.mw-crop}

2. Choose **Remote content**: `block`, `ask` (the default), `proxy` or `direct`. `direct` lets the browser load images from the sender's servers itself, exposing the person's address; it applies only when **Allow direct images** is on and the person is allowed direct content, and falls back to `ask` otherwise.
3. Set the other switches (see [Options](#options)) and select **Save**.

## Watch the image proxy

The **Overview** and **Image proxy** tabs show the proxy's figures with a range of **15 minutes**, **1 hour**, **6 hours** or **24 hours**: images served, cache hit rate, tracking pixels replaced, data served, fetches in flight and waiting, and busy answers. The server samples every 15 seconds and keeps 24 hours in memory; the history is empty after a restart and each server process answers for itself. The page refreshes every 5 seconds while it is visible.

The **Speed** section of **Image proxy** shows the time from a request arriving to the response being ready, split into queue, cache, upstream fetch and decode, with median and 95th percentile. With **Image proxy speed tracking** on (the default), each request adds to a histogram; no URL, host, account or message is recorded.

The **Traffic** tab counts outbound requests per traffic class (allowed, refused by policy, failed, total time) since the server process started.

## Attachment policy

The **Attachment policy** screen (`#/attachments`) decides which attachments the server hands out, using the file name and MIME type stated in the message.

![The Attachment policy screen on its Allow and block lists tab, with empty lists of allowed and blocked file name extensions and MIME types.](/screenshots/admin-privacy/attachments-lists.webp){.mw-shot}

- **Allow and block lists**: allowed, blocked and warning lists of file name extensions and MIME types, up to 100 rules each. A blocked rule wins over an allowed one. With an allow list set, a file that matches none of its rules is not delivered.
- **Limits**: **Maximum attachment bytes**; 0 is no limit.
- **Download and preview**: **Allow original attachment delivery** and **Allow attachment previews**.
- **Warnings**: files that match are delivered only after the person confirms a warning.
- **Test a file**: asks the server how the saved policy treats a file with a given name, type and size. It uses the server-wide policy, not a domain's values, and stores nothing.

::: warning What the attachment policy does not do
The policy is applied when the server hands an attachment out: download, preview, whole-message export and saving to Nextcloud. Messages with blocked attachments are still received and stored. When sending, only the mail app's composer refuses a blocked file; a client that talks to the server's JMAP interface directly can attach and send it. **Maximum attachment bytes** limits what is delivered, not what is received or sent. It is not malware scanning, and a preview that receives the original bytes cannot prevent saving them.
:::

When any restriction is set, nested messages (`message/rfc822`, `.eml`) and TNEF files are refused, because their contents cannot be checked.

## Options

| Setting | What it does | Default |
| --- | --- | --- |
| **Remote content** ([`privacy.remoteContent`](/admin/settings#privacy.remoteContent)) | `block`, `ask`, `proxy` or `direct` | `ask` |
| **Block trackers** ([`privacy.trackers`](/admin/settings#privacy.trackers)) | Replaces known tracking pixels | on |
| **Offer per-message tracker consent** ([`privacy.trackerOverrideAllowed`](/admin/settings#privacy.trackerOverrideAllowed)) | Lets a person load trackers for one message | on |
| **Clean links** ([`privacy.cleanLinks`](/admin/settings#privacy.cleanLinks)) | Removes tracking parameters from links | on |
| **Allow direct images** ([`privacy.allowDirect`](/admin/settings#privacy.allowDirect)) | Permits `direct` | off |
| **Manual remote image downloads** ([`privacy.remoteImageDownloads`](/admin/settings#privacy.remoteImageDownloads)) | Offers downloading a remote image | off |
| **Loading-images notification** ([`privacy.remoteImagesToast`](/admin/settings#privacy.remoteImagesToast)) | Shown in the mail app while images load | on |
| **Remove outgoing mailer headers** ([`privacy.stripMailerHeaders`](/admin/settings#privacy.stripMailerHeaders)) | Removes mailer identification headers from sent mail | on |
| **Remote image limit** ([`privacy.maxBytes`](/admin/settings#privacy.maxBytes)) | Largest remote image fetched | 8 MiB |
| **Remote image redirects** ([`privacy.maxRedirects`](/admin/settings#privacy.maxRedirects)) | Redirects followed | 3 |
| Timeouts and concurrency (`privacy.fetch*TimeoutMs`, `privacy.maxConcurrent*`, `privacy.maxQueuedRequests`, `privacy.imageDecodeTimeoutMs`) | On **Fetch limits** | see the reference |
| **Remote image cache** (`privacy.cacheBytes`), **Successful image cache lifetime** (`privacy.cacheTtlSecs`), **Remote image request rate** (`privacy.ratePerMin`), **Image request burst allowance** (`privacy.rateBurst`) | Used only when the matching mode on **Image proxy** is manual; the default is automatic, which ignores these four values | 32 MiB, 3600 s, 1200, 1024 |
| **Allow original attachment delivery** ([`attachments.downloadAllowed`](/admin/settings#attachments.downloadAllowed)), **Allow attachment previews** ([`attachments.previewAllowed`](/admin/settings#attachments.previewAllowed)) | Delivery and preview | on, on |
| **Maximum attachment bytes** ([`attachments.maxBytes`](/admin/settings#attachments.maxBytes)) | Largest attachment delivered | 0 (no limit) |
| Allow, block and warning lists (`attachments.*Extensions`, `attachments.*MimeTypes`) | See above | empty |

## Related

- [Outbound connections](/security/egress)
- [Restrictions and consent](consent.md)
- [Cookie consent](cookies.md)
- [Message content settings in the mail app](/user/settings/account#message-content)
