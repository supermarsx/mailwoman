---
title: Outlook and Word formats
---

# Outlook and Word formats

The export dialog can save a message as an Outlook message file (`.msg`) or a Word document (`.docx`). This page says what each file contains. For the dialog itself, see [Exporting messages](./index.md).

## MSG (Outlook)

An `.msg` file is a compound file in the Outlook message format (MS-OXMSG). Outlook and other readers of the format open it as a mail item of class `IPM.Note`.

The file contains:

- the subject, sender and recipients (To and Cc; Bcc is not written),
- the plain text body and, when the message has one, the HTML body,
- the internet headers, which carry the date and the rest of the original header block,
- each attachment, including its data,
- messages attached as `message/rfc822`, stored as embedded messages,
- custom `X-*` headers, stored as named properties in the internet-headers namespace, as Outlook does.

A message with no `X-*` headers and no attached message is written without those parts.

What is not reproduced: the file is a rendition of the message as received, not a copy of an Outlook-authored item. Other embedded objects and Outlook-specific properties that have no counterpart in the original message are not created.

## DOCX (Word)

A `.docx` file is a Word document of the message:

- one line each for **Subject**, **From**, **To**, **Cc** and **Date**, when the message has them, with bold labels,
- the body as paragraphs. An HTML-only message is converted to plain text first, and script and style content is dropped, so no code text appears in the document,
- an **Attachments** line listing the attachment file names.

Attachment data is not included in a DOCX file, and formatting, images and layout of an HTML message are not reproduced. Use EML or MSG if you need the attachments.

## OFT (Outlook template)

The server library can also write an Outlook template (`.oft`), which has the same structure as an `.msg` file. The export dialog and the export route do not offer it.

## Importing an `.oft` or `.msg`

The route `POST /api/import/oft` accepts the base64 content of an `.oft` or `.msg` file from a signed-in user and returns the subject and the sanitized body, which a client can use to fill a new draft. The web client does not call this route.

Reading an untrusted compound file is done only inside the server's isolated render worker, in a sandbox without network or secrets, and the input is limited to 4 MiB. Only the subject and the plain text body are read. If the render worker is not installed, the route refuses with an error rather than reading the file inside the server.

## Related

- [Exporting messages](./index.md)
- [Administrator settings](../admin/settings.md)
