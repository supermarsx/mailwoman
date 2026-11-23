---
title: Mail import and export
---

# Mail import and export

Users bring a mail archive into their account with **Import and export mail** in Settings, and download messages and folders with the **Export** dialog. This page is for the person running the server: what the server needs for import to work, the limits it applies, and which settings control export. For the user's side see [Settings: mail](/user/settings/mail#import-and-export-mail).

## Before you start

Import needs all of these:

- engine mode (`MW_MODE=engine`);
- an IMAP account that is connected, writable, and advertises UIDPLUS or IMAP4rev2. POP3 accounts and accounts that are not connected cannot import;
- an upload directory, set with `MW_UPLOAD_DIR`. Without it the server answers that import requires a configured upload backend rather than dropping data.

The import feature must also be on for the user; an administrator who turns it off gives users **Your administrator has disabled mail import.** The same restriction applies to the JMAP method `Email/import`.

## Import behaviour

Users choose a **Mail file**, a **File format** and a **Destination folder**, then **Import into selected folder**. The server accepts an EML file, an mbox file, or a Maildir packed as an uncompressed tar archive.

- One request carries at most 64 MiB, and at most 20,000 messages. The page shows the limits as **Maximum … and … messages per file.** A larger archive has to be split.
- At most two imports run at once on a server process.
- Each message is saved to the mail server with an IMAP `APPEND`, and the server counts it as imported only after the mail server has confirmed it with its UID. Valid keywords and dates in the file are kept.
- An imported message does not run arrival rules, vacation replies or forwarding.
- The result reads **Imported … of … messages. Failed: …**. If the connection drops part-way the page says **The import did not finish normally. Some messages may already be saved. Check the destination before trying again.** Importing the same archive twice creates duplicates.

## Export

The **Export** dialog is available from the message view, the message menus and the selection bar. A user can export one message, a conversation, a selection or a whole folder, in the formats the administrator allows: EML (the original message bytes), mbox, plain text, Markdown, Outlook MSG, Word DOCX, a CSV summary, JSON properties and PDF (made by the browser's print function). Text, Markdown, MSG and DOCX are converted copies of the message; only EML and mbox keep the original.

Export follows the administrator's attachment and export restrictions, and may ask the user to confirm their password. When the server connection cannot produce a format the dialog says **Export in these formats is unavailable for this server connection.**

## Options

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Export dialog** ([`export.dialog.enabled`](/admin/settings#export.dialog.enabled)) | The dialog in the message view and menus | On | Server, domain, user |
| **Export dialog formats** ([`export.formats`](/admin/settings#export.formats)) | Which formats the dialog offers | All nine | Server, domain |
| **Export dialog default format** ([`export.defaultFormat`](/admin/settings#export.defaultformat)) | The format the dialog starts with | `eml` | Server, domain, user |
| **Export dialog default scope** ([`export.defaultScope`](/admin/settings#export.defaultscope)) | What it exports by default, when the context offers it | `message` | Server, domain, user |
| **Message download** ([`export.message.download.enabled`](/admin/settings#export.message.download.enabled)) | Download a message as `.eml`, headers only or mbox | On | Server, domain |
| **Folder export** ([`export.folder.enabled`](/admin/settings#export.folder.enabled)) | Download a folder as a ZIP | On | Server, domain |
| **Folder export format** ([`export.folder.format`](/admin/settings#export.folder.format)) | ZIP of `.eml` files, ZIP with an mbox, or both | Both | Server, domain |
| **Folder export size limit** ([`export.folder.maxBytes`](/admin/settings#export.folder.maxbytes)) | Largest folder export, in bytes | 2,147,483,648 | Server, domain |
| **Folder export message limit** ([`export.folder.maxMessages`](/admin/settings#export.folder.maxmessages)) | Most messages in one folder export | 100,000, never applied above 65,000 | Server, domain |
| **Include subfolders in export** ([`export.folder.includeSubfolders`](/admin/settings#export.folder.includesubfolders)) | Folder export includes subfolders by default | On | Server, domain, user |
| **Import message count** ([`import.maxMessages`](/admin/settings#import.maxmessages)) | Appears in Settings but has no effect: the import limit is fixed at 20,000 | 20,000 | Server, domain (no effect) |
| **Import message size** ([`import.maxMessageBytes`](/admin/settings#import.maxmessagebytes)) | Appears in Settings but has no effect: the import limit is fixed at 64 MiB | 64 MiB | Server, domain (no effect) |

## Troubleshooting

- **Mail import is unavailable for this server connection.** The account is not an engine-mode IMAP account with a writable connection. POP3 and proxy-mode accounts cannot import.
- An import answers that it requires a writable IMAP connection with UIDPLUS or IMAP4rev2 and a configured upload backend: check the mail server's capabilities and that `MW_UPLOAD_DIR` is set.
- A large import is refused with a JSON `413`: the file is over 64 MiB. A refusal that is not JSON comes from a proxy in front of the server; raise its body limit.
- **Choose a nonempty file within the displayed size limit.** The file is empty or over the limit.

## Related

- [Settings: mail](/user/settings/mail#import-and-export-mail)
- [Settings reference](/admin/settings)
- [IMAP and POP3 mail servers](/deploy/imap-pop3)
