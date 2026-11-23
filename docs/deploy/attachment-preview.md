---
title: Attachment preview
---

# Attachment preview

When someone opens an attachment that is itself an email message, Mailwoman can show what is inside it: the headers, the text, and the files it contains. This applies to message files (`.eml`, `message/rfc822`) and to the `winmail.dat` files that some Microsoft mail programs send (TNEF, `application/ms-tnef`). This page describes what the preview does, where it runs, and what limits and settings apply.

## Before you start

- The preview is available when **Allow attachment previews** ([`attachments.previewAllowed`](/admin/settings#attachments.previewallowed)) and **Allow original attachment delivery** ([`attachments.downloadAllowed`](/admin/settings#attachments.downloadallowed)) are both on. Turning off delivery also turns off raw previews. Both default to on and can be set globally or per domain.
- The server needs its render worker program. When it is missing the preview is unavailable and the viewer says "This message attachment could not be previewed."

## What the preview shows

The preview lists:

- for a message: the subject, sender, recipients, date, and the text and HTML bodies;
- for a `winmail.dat` file: the message text from it, and the files it carries;
- under **Files inside this message**, each contained file with its size and a **Download file** button. A file that is itself a message is offered as a download.

Attachment bytes are not changed. File names are cleaned for display and download, and nothing is written to the server's file system. Embedded objects (OLE) are offered as opaque downloads and are never run. A message body in TNEF can be rich text. Mailwoman shows its plain text when there is no plain or HTML body, and it shows a warning when the rich text is invalid or unsupported while the attached files stay available. Remote images and other remote resources are not loaded in the preview.

A file listed in the preview is checked against the same attachment rules as a normal download, such as the allowed types. A file that the rules refuse is left out.

## How it is isolated

The server never parses these formats itself. It passes the attachment to a separate short-lived worker process. Inside the worker, a WebAssembly module with no imports does the parsing. The worker then cleans the HTML with the same sanitiser that is used for received mail, and the browser shows the result in a sandboxed frame. On Linux the worker also runs under a kernel jail made of seccomp, Landlock, a network namespace and resource limits. On other systems the process and WebAssembly isolation remain and the kernel jail is not available.

## Limits

| Limit | Value |
| --- | --- |
| Size of the attachment | 4 MiB. A larger one shows "Message attachment previews are limited to 4 MiB. You can still download the original attachment." |
| Size of one request | 6 MiB |
| Files returned from one preview | 64 |
| Size of one body | 1 MiB |
| Previews running at once | 2 workers on the server. A third waits briefly and is then refused |
| Time for one preview | 15 seconds |

A file that is damaged, has a failed checksum, has an unsupported property or has too many entries is refused as a whole. Rich text formatting, legacy font encodings and layout are not reproduced, and encrypted inner messages are not decrypted. The preview is not a complete Outlook viewer.

## Options

| Label | What it does | Default | Who can change it |
| --- | --- | --- | --- |
| **Allow attachment previews** ([`attachments.previewAllowed`](/admin/settings#attachments.previewallowed)) | Allows the preview | on | Administrators globally and per domain |
| **Allow original attachment delivery** ([`attachments.downloadAllowed`](/admin/settings#attachments.downloadallowed)) | Allows downloading the original. Off also disables previews | on | Administrators globally and per domain |

Users choose how the preview buttons look, and whether images show thumbnails, in the **Attachment previews** card of their settings. Thumbnails are off by default.

## Troubleshooting

- **"This message attachment could not be previewed. The file may be unsupported, damaged, or the preview worker unavailable. You can still download the original."** The file is not a message or TNEF file the worker could read, it is damaged, or the render worker is not running. Download the original instead.
- **"Attachment access is restricted by administrator policy" (error code `attachmentPreviewDisabled`).** Previews or deliveries are switched off for the user's domain. See [Options](#options).
- **The preview is refused with `attachmentTooLarge`.** The attachment is larger than the maximum attachment size set in the attachment policy.
- **"attachment preview workers are busy; retry shortly".** Both worker slots are in use. Try again.
- **A file inside a `winmail.dat` is missing.** The attachment policy refuses its type or size, so it is left out of the list.

## Related

- [Settings](/admin/settings) for the attachment settings
- [Outbound connections (egress)](/security/egress) for how remote images are handled in mail
- [PDF attachment preview](/security/pdf-preview) for how PDF attachments are shown
