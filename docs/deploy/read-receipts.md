---
title: Read receipts and delivery notifications
---

# Read receipts and delivery notifications

Two different things can tell a sender what happened to a message. A **read receipt** is a reply from the recipient's mail program that says the message was displayed. A **delivery notification** is a report from a mail server about delivery. Neither proves that a person read the message. This page describes how Mailwoman handles both, the settings an administrator controls, and what the user sees.

![The Settings screen searched for receipt, listing the read receipt, delivery notification and scheduled receipt settings with their default values.](/screenshots/admin-ops/settings-receipts.webp){.mw-crop}

## Before you start

- Read receipt policy, scheduled receipts and delivery notifications are enforced by Mailwoman's mail engine, which handles accounts that Mailwoman reaches itself. For accounts that Mailwoman forwards to a JMAP server, the dedicated policy editor is not offered. Mailwoman still checks administrator prohibitions on the requests it forwards, see [Accounts behind a JMAP server](#accounts-behind-a-jmap-server).
- The user's own choices are in their **Settings**, in **Read receipts**.
- Two settings in the console are both labelled **Read receipts**. `mail.readReceipts` is the response policy. `mail.badge.receipts` only decides when a badge is shown in the reader.

## How a read receipt is answered

When a message asks for a read receipt, the reader shows "<sender> requests a read receipt. Sending it reveals that this message was displayed." with **Send read receipt** and **Not now**. What happens automatically depends on the policy.

| Policy (`mail.readReceipts`) | Label in **Settings** | What Mailwoman does |
| --- | --- | --- |
| `ask` (default) | **Ask before sending** | Shows the question |
| `never` | **Never send read receipts** | Shows nothing and sends nothing |
| `always` | **Send automatically when safe** | Sends automatically |
| `contactsOnly` | **Send automatically to contacts only** | Sends automatically when the sender's address is in the user's address books, otherwise does nothing |
| `orgOnly` | **Send automatically inside my organisation only** | Sends automatically when the sender is in the user's own domain or an alias of it, otherwise does nothing |
| `orgAlwaysAskOthers` | **Send inside my organisation; ask for other senders** | Sends automatically inside the organisation and asks otherwise |

A message whose return address is not the sender, or that comes from a mailing list, is always asked about, even when the sender would otherwise qualify. A message that already got a receipt, a draft, or a message in a folder that never answers receipts is not offered one.

The user can add sender exceptions for exact addresses, up to 500, under **Sender exceptions**: **Suppress prompts and automatic receipts** or **Send automatically when safe**. The user's own `never` beats any exception. When an administrator locks the policy, personal exceptions are ignored.

An automatic receipt is sent only when the message is open in a full message view, is displayed and visible, and has stayed so for 1.5 seconds. A preview pane, a background tab, or a display that could not be confirmed never sends one. An uncertain result is not retried by the browser.

### What an administrator controls

| Label | What it does | Default | Who can change it |
| --- | --- | --- | --- |
| **Read receipts** ([`mail.readReceipts`](/admin/settings#mail.readreceipts)) | The policy above | `ask` | Administrators globally and per domain, and users, unless locked |
| **Request read receipts by default** ([`mail.requestReadReceipt`](/admin/settings#mail.requestreadreceipt)) | Whether new messages ask for a read receipt | off | Same |
| **Delivery status requests** ([`receipts.dsnAllowed`](/admin/settings#receipts.dsnallowed)) | Whether messages may ask for delivery notifications at all | on | Administrators globally, per domain and per administrator-assigned user |
| **Request delivery notifications by default** ([`mail.requestDeliveryReceipt`](/admin/settings#mail.requestdeliveryreceipt)) | Whether new messages start with the delivery notification option ticked | off | Same as the first row |
| **Scheduled read receipts** ([`receipts.scheduleAllowed`](/admin/settings#receipts.scheduleallowed)) | Offers **Schedule receipt** on messages | off | Administrators globally and per domain |
| **Request read receipts** ([`cap.compose.read_receipts`](/admin/settings#cap.compose.read_receipts)) | Whether the composer can ask for read receipts | on | Administrators |
| **Allow Return-Receipt-To** ([`compose.headers.allowReturnReceiptTo`](/admin/settings#compose.headers.allowreturnreceiptto)) | Whether the old `Return-Receipt-To` header may be used | off | Administrators |

When an administrator locks **Read receipts** to `never`, the engine refuses every receipt: automatic ones, manual **Send read receipt**, and a raw disposition notification that a client submits. The check happens when the receipt is queued and again just before it is transmitted, including for delayed submissions. A failure to read the policy never grants permission.

Locking **Request delivery notifications by default** does not forbid a user from choosing a notification for one message. To forbid requests, set **Delivery status requests** to off. The engine checks that setting when the message is queued and again when it is sent. A value that cannot be read counts as off.

## Delivery notifications

The composer offers a delivery notification checkbox only when the account's mail server advertises the `DSN` extension and **Delivery status requests** allows it. The request asks for the headers of the original in the report (`RET=HDRS`) and a report for success, failure and delay (`NOTIFY=SUCCESS,FAILURE,DELAY`) for every recipient, including Bcc.

Mailwoman learns whether the server supports `DSN` by a short check of its greeting that sends no message. The check has a 2 second limit. A positive answer is remembered for 5 minutes and a negative one for 30 seconds. If a requested notification becomes unavailable before sending, the send fails before any mail is transferred. It is not sent as an ordinary message.

A draft keeps its choice. Losing the capability or the permission gives an error, and the request is not dropped silently. Incoming reports are shown as described in [Delivery status reports](/deploy/delivery-status).

## Scheduled read receipts

When **Scheduled read receipts** is on, an eligible message shows **Schedule receipt**. The user picks a date and time up to 30 days ahead, for this message or for every request from that sender that the user opens in a full view before then. **Stop scheduling for this sender** ends the sender schedule, and a queued receipt can be cancelled from the original message. The server keeps the schedule across sessions and checks the policy again before it sends.

## Sent messages

**Receipts** in the header of a sent message lists reports by recipient. See [Delivery status reports](/deploy/delivery-status#sent-messages-receipts).

## When delivery is uncertain

If a message's data has been sent but the final answer of the mail server is lost, Mailwoman records the delivery as unknown and does not send that attempt again. This applies to read receipts too: the receipt is marked as used and a retry is refused. A refusal before any data is sent, such as a failed login or an unsupported feature, frees the receipt so it can be tried again.

The Outbox shows **Delivery unknown** for such a message. Check **Sent** or ask the recipient before sending again. A missing copy in **Sent** does not prove non-delivery, because the copy is filed after the mail server accepts the message.

## Accounts behind a JMAP server

For an account that Mailwoman forwards to a JMAP server, the dedicated read receipt editor is not offered. Mailwoman still checks administrator prohibitions on requests that pass through it:

- With read receipts locked to `never`, it refuses an explicit receipt send and refuses submissions it cannot inspect. A normal message with an explicit, ordinary body and its submission is still allowed.
- With **Delivery status requests** off, it refuses delivery notification parameters.
- It cannot recall a message that the upstream server has already accepted, and it cannot see what a client connected directly to the upstream server does.

## Troubleshooting

- **A user sees no read receipt prompt.** The policy is `never`, or a message is not eligible, or the sender is on the user's suppress list. A preview pane never shows or sends receipts.
- **A receipt is refused with "read receipts are disabled by administrator policy".** An administrator locked **Read receipts** to `never` for that account.
- **A send is refused with "delivery status notifications are turned off by the administrator".** **Delivery status requests** is off for the account.
- **"The receipt outcome could not be confirmed. It will not be retried automatically."** See [When delivery is uncertain](#when-delivery-is-uncertain).
- **The delivery notification checkbox does not appear.** The mail server does not advertise `DSN`, or **Delivery status requests** is off.
- **"Your administrator does not allow this change. Reopen the editor to refresh its policy."** The user saved a receipt policy while an administrator lock was in force.

## Related

- [Delivery status reports](/deploy/delivery-status)
- [Outbox, recall and scheduled sending](/admin/outbox-recall)
- [Settings](/admin/settings)
