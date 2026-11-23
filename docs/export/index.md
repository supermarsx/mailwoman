---
title: Exporting messages
---

# Exporting messages

You can download a message, a conversation, a selection of messages or a whole folder in a format you choose. The server writes EML, mbox, plain text, Markdown, MSG and DOCX files. The web client builds CSV and JSON summaries from the message list, and a PDF comes from your browser's print function.

![The Export dialog opened over the mailbox for one message, with What to export, Format set to EML, Packaging, File name pattern, Privacy options and the size estimate.](/screenshots/integrations/export-dialog.webp){.mw-shot}

## Before you start

- Export needs the native mail engine. In proxy mode the message says **This account cannot be exported from here.**
- Your administrator can switch export off, limit the formats, and set size limits. See [Administrator settings](#administrator-settings).
- Export may ask you to confirm your identity again before it runs.

## Export messages

1. Open a message, then choose **Export** in the reader, or open the message's context menu and choose **Download message**. If your administrator has switched the dialog off, this downloads the message as an `.eml` file instead.
2. Under **What to export**, choose **This message**, **This conversation (n messages)**, **Selected n messages** or **Whole folder _name_**. Only scopes that apply are offered.
3. Choose a **Format**.
4. Under **Packaging**, choose **One file** or **ZIP with a file per message**. Formats that cannot hold several messages in one file (everything except MBOX, CSV and JSON) offer the ZIP only. A folder is always a ZIP.
5. Optionally change the **File name pattern**. Use `{date}`, `{from}`, `{subject}`, `{id}` and `{n}`; characters a file system does not allow are replaced. **Longest name (characters)** limits the length (20 to 200, default 80). The dialog shows an **Example**.
6. For EML and MBOX you can tick **Remove Received headers** or **Remove Bcc**. Removing headers changes the message, so a signature on it no longer verifies.
7. For a folder, tick **Include subfolders** if you want them.
8. Read the size estimate (**n messages, about size**) and choose **Export**. Choose **Stop** to cancel a running export, and **Download again** when it finishes.

## Formats

| Format | What you get |
|---|---|
| **EML (original message)** | The message exactly as stored. |
| **MBOX (mailbox file)** | One mbox file; several messages go into one file. |
| **Plain text** | Headers and the decoded text body. |
| **Markdown** | The message body converted to Markdown. |
| **MSG (Outlook)** | An Outlook message file. See [Outlook and Word formats](./msg-oft-docx.md). |
| **DOCX (Word)** | A Word document of the message. Attachments are listed, not included. See [Outlook and Word formats](./msg-oft-docx.md). |
| **CSV summary (date, from, to, subject, size)** | Built from the message list. It holds no message bodies. |
| **JSON (message properties)** | Message properties built from the list. |
| **PDF** | Not made by the server. Open the message, choose Print and then Save as PDF as the printer. |

The Outlook `.oft` template format exists in the server library but is not offered by the export dialog or the export route.

## Administrator settings

| Setting | Label | Default | Levels |
|---|---|---|---|
| `export.dialog.enabled` | Export dialog | on | global, domain, user |
| `export.message.download.enabled` | Message download | on | global, domain |
| `export.formats` | Export dialog formats | all nine | global, domain |
| `export.defaultFormat` | Export dialog default format | `eml` | global, domain, user |
| `export.defaultScope` | Export dialog default scope | `message` | global, domain, user |
| `export.folder.enabled` | Folder export | on | global, domain |
| `export.folder.format` | Folder export format | both | global, domain |
| `export.folder.maxBytes` | Folder export size limit | 2147483648 (2 GiB) | global, domain |
| `export.folder.maxMessages` | Folder export message limit | 100000 | global, domain |
| `export.folder.includeSubfolders` | Include subfolders in export | on | global, domain, user |

The master switch for the feature is **Export** (`cap.export`). The settings are described in [Administrator settings](../admin/settings.md).

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| **Exporting is switched off by your administrator.** | `export.message.download.enabled` is off. | Ask an administrator. |
| **This format is not available. Your administrator limits the formats.** | The format is not in `export.formats`. | Choose another format. |
| **Too many messages for one export.** / **This export is larger than your administrator allows.** | A folder limit was exceeded. | Choose fewer messages. |
| **This mail is under a legal hold and cannot be exported.** | A hold applies. | Contact your administrator. |
| **Processing of this account is restricted, so it cannot be exported.** | A restriction applies. | Contact your administrator. |
| **Your administrator's attachment rules do not allow this export.** | Attachment rules restrict content. | Contact your administrator. |
| **Too many exports just now. Wait a moment and try again.** | Rate limit. | Wait and retry. |

## Related

- [Outlook and Word formats](./msg-oft-docx.md)
- [Administrator settings](../admin/settings.md)
