---
title: Delivery status reports
---

# Delivery status reports

When a mail server cannot deliver a message, or wants to report on a delivery, it sends back a delivery status report. It is a message of a special kind, commonly known as a bounce. Mailwoman recognises these reports and shows them as a structured card above the original text, instead of as a block of machine text. This page describes what the card shows, how Mailwoman finds the reports, and what limits apply.

A delivery status report is separate from a read receipt, see [Read receipts and delivery notifications](/deploy/read-receipts). Opening a report sends nothing and changes no other message.

## What the card shows

The card is titled **Delivery status report**. It says that the report describes the sender's delivery attempt, that its claims are not independently verified, and that delivery does not mean the message was read.

For each recipient in the report the card shows:

- the action the server reported: **Reported failure**, **Reported delay**, **Reported delivery**, **Reported relay** or **Reported list expansion**;
- the status code, such as `5.1.1`, with a plain description under **What this code means** and a suggestion under **What to do**;
- the **Server diagnostic** text from the reporting server;
- when the report gives them, **Reporting server**, **Original recipient**, **Remote server**, **Last attempt** and **Sender plans to retry until**.

The action is read from the report. It is not guessed from the numeric code. A failure report can carry a temporary `4.x.x` code after the sending server gave up retrying.

For some providers the card adds an interpretation of the diagnostic, under **Interpretation based on the server diagnostic or hostname:**, with a link to **Provider documentation**. The providers it knows are Microsoft Exchange / Microsoft 365, Gmail / Google Workspace, Mimecast, Proofpoint, Barracuda, FortiMail, Yahoo / AOL and generic SMTP relays.

## What counts as a report

Mailwoman treats a message as a delivery status report only when the root of the message is a `multipart/report` with a machine-readable delivery status part that follows the report format (RFC 3464, and RFC 6533 for internationalised reports). Anything else is left as an ordinary message and its original body stays readable. In particular these are left alone:

- reports that were forwarded inside another message;
- read receipts (MDNs);
- reports with duplicate or ambiguous fields, with required fields missing or damaged, or larger than the limits below;
- informal "your message could not be delivered" texts that do not follow the report format.

Text from the report is shown as plain text. No HTML or links from the report are rendered.

## How the reports are found

- For accounts on the built-in mail engine, the server reads the report from its cached copy of the message.
- For accounts reached through a JMAP server that Mailwoman forwards to, the browser first checks the top-level headers of the message. If it looks like a report, the browser downloads the message and sends it to the server at `POST /api/mail/delivery-status`, which parses it with the same code. The server accepts only the bytes, never an address to fetch.

A response that contains a parsed report is sent with `Cache-Control: private, no-store`.

### Limits

| Limit | Value |
| --- | --- |
| Whole message | 8 MiB |
| Status part | 256 KiB |
| Recipients in one report | 64 |
| One field | 4 KiB |
| Parsing at once on the server | 2 tasks, with a 5 second response limit |
| Browser wait for the server | 15 seconds |

## Sent messages: Receipts

In **Sent**, the **Receipts** button in a message header lists the read and delivery reports for that message, by recipient, under **Recipient receipts**. A report is matched to a sent message by the Message-ID in the original headers that the report returns, and by the recipient. Time shown for a delivery is the report's **Last attempt** when it has one, and otherwise the time the report arrived.

- A recipient with no report yet shows **Awaiting report**. A missing report does not mean the message failed or is unread.
- The lookup checks recent messages first. Choose **Check older messages** to look further back. If some candidates could not be checked it says "Some candidate reports could not be checked."
- A message without exactly one Message-ID says "This message has no single Message-ID to match against reports."

## Troubleshooting

- **A bounce shows as an ordinary message.** It does not follow the report format, or it is over a limit above. The text of the original body is still readable.
- **A delivery report says delivered but the recipient did not answer.** A delivery report is not a read receipt. It is only a claim by a mail server.
- **"The receipt lookup could not finish."** The search of the mailbox was interrupted. Open **Receipts** again.

## Related

- [Read receipts and delivery notifications](/deploy/read-receipts)
- [Outbox, recall and scheduled sending](/admin/outbox-recall)
- [Mailing list actions](/deploy/mailing-list-actions)
