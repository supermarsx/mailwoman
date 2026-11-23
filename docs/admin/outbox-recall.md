---
title: Outbox, recall and scheduled sending
---

# Outbox, recall and scheduled sending

When someone sends a message from the web client, the server keeps it in a queue called the **Outbox** before it is handed to the outgoing mail server. While it waits, the sender can **Undo** the send, send it at once, or cancel it. A message can also be scheduled for later with **Send later**. This page explains what people see, which limits you can set, and what happens when delivery fails.

The queue is stored in the database, so a waiting message survives closing the browser and restarting the server. The queue and the limits apply to accounts that Mailwoman sends for itself, which are accounts on its built-in mail engine.

![The Outbox screen with one message waiting to send, marked Sending soon, with its Send now, Recall and edit and Cancel buttons, and the Outbox count beside the folder list.](/screenshots/admin-ops/outbox-screen.webp){.mw-crop}

## Before you start

- The limits in this page are administrator settings. They can be set globally or for a domain in **Settings**. The time a user waits is their own choice within your ceiling (see below).
- The time a user waits before a message leaves is the user's own choice, within your ceiling. The user sets it under **Undo send** in **Settings**, in **Composing and replies**.

## What the sender sees

1. After pressing **Send**, a notice says "Message queued — sends in" with the time left, an **Undo** button and a **Send now** button.

   ![The notice after pressing Send, with the time left to undo and the Undo and Send now buttons.](/screenshots/admin-ops/outbox-queued.webp){.mw-crop}

2. **Undo** takes the message back and says "Message recalled. It is back in your drafts." The composer reopens with the recipients, subject, body and attachments.
3. **Send now** sends the message without waiting. The notice says "Sending now."
4. Once the dispatcher has started sending, **Undo** is refused with "Too late to recall: the message is already being sent."

The waiting messages are listed in the **Outbox** in the folder list. The number beside it is the count of waiting messages.

![One message in the Outbox with markers 1 to 3 on Send now, Recall and edit and Cancel.](/screenshots/admin-ops/outbox-list.webp){.mw-crop}

1. **Send now** sends the message at once.
2. **Recall and edit** takes it back and reopens it in the composer.
3. **Cancel** drops it. The message stays a draft.
{.mw-callouts}

The countdown and the end of the undo period are decided by the server. They do not depend on the browser staying open. The composer also has a **Recall** button that lists the held and scheduled messages of the open mailbox.

### The status of a message

The Outbox labels each message:

| Status | Meaning |
| --- | --- |
| **Sending soon** | Waiting for its undo period to end |
| **Scheduled** | Waiting for the time chosen with **Send later** |
| **Held** | Never sent until the sender chooses **Release**. It can still be cancelled or discarded |
| **Sending** | The dispatcher has claimed it and is sending it |
| **Retrying** | An attempt failed before the mail server accepted the message. The next attempt time and the number of attempts are shown |
| **Not sent** | The attempts ran out. The message is kept in the drafts with the last error |
| **Delivery unknown** | The final answer of the mail server was lost after the message data was sent. It is never sent again by itself. Check the Sent folder or ask the recipient before sending again |
| **Sent** | The mail server accepted the message. The copy in Sent is filed after this |
| **Canceled** | The sender recalled or cancelled it |

## Send later

The arrow beside **Send** opens **Send later options**: **In 1 hour**, **Later today, 17:00**, **Tomorrow morning, 09:00**, **Monday morning, 09:00** and **Pick date and time…**. The picker shows the time zone it uses. The time is stored as an instant, and the zone is only used to show it.

![The Send later menu with the four preset times and Pick date and time.](/screenshots/admin-ops/compose-send-later.webp){.mw-crop}

- Only the server's clock decides when a message is due, so a browser with a wrong clock cannot send early. A time that has already passed is refused.
- A scheduled message shows in the Outbox with **Scheduled** and its send time. **Edit send time** changes it.
- Messages that fell due while the server was stopped are sent soon after it starts.
- Pre-send checks run before the message is queued, not when it is sent.

## Options

| Label | What it does | Default | Who can change it |
| --- | --- | --- | --- |
| **Undo-send window (default)** ([`mail.send.recallDelaySeconds`](/admin/settings#mail.send.recalldelayseconds)) | How many seconds a sent message waits before it leaves. 0 sends at once. Range 0 to 1800 | 10 | Administrators globally and per domain, and users, unless locked |
| **Undo-send window (limit)** ([`mail.send.recallDelayMaxSeconds`](/admin/settings#mail.send.recalldelaymaxseconds)) | The longest wait a user may choose. A longer request is shortened to it. Range 0 to 1800 | 1800 | Administrators globally and per domain |
| **Allow recalling sent messages** ([`mail.send.allowRecall`](/admin/settings#mail.send.allowrecall)) | Off sends immediately with no undo period. A message with a future send time is still scheduled | on | Administrators globally and per domain |
| **Scheduling horizon** ([`mail.send.maxDelayedSeconds`](/admin/settings#mail.send.maxdelayedseconds)) | How far ahead a message may be scheduled, in seconds. A later time is refused. 0 turns scheduling off. Range 0 to 31,536,000 (one year) | 7,776,000 (90 days) | Administrators globally and per domain |
| **Queued messages per mailbox** ([`mail.send.maxQueued`](/admin/settings#mail.send.maxqueued)) | How many messages a mailbox may have waiting or scheduled. Further sends that would wait are refused. Range 1 to 5000 | 200 | Administrators globally and per domain |
| **Sent and cancelled send records (days)** ([`retention.outbox.days`](/admin/settings#retention.outbox.days)) | How long the record of a delivered or cancelled message is kept. Waiting, retrying and failed records are never removed. Range 0 to 3650 | 30 | Administrators globally |

![The Settings screen searched for mail.send, with the five send settings and their default values.](/screenshots/admin-ops/settings-send-limits.webp){.mw-crop}

The limits are read when a message is queued, from the settings of the mailbox's domain and the global values. The engine applies them, not only the web client, so a modified client cannot go beyond them.

To stop delayed sending for a domain, set **Allow recalling sent messages** to off and **Scheduling horizon** to 0 for that domain.

::: warning No effect
**Sending attempts** ([`outbox.maxAttempts`](/admin/settings#outbox.maxattempts)) is shown in **Settings** with the help text "How many times a message is handed to the mail server before it is reported as not sent", but the server does not read it. The number of attempts is fixed at 8.
:::

## When sending fails

A message that the mail server did not accept is tried again, up to 8 attempts in all. The waits between attempts are 30 seconds, 1, 2, 4, 8 and 16 minutes, and then 30 minutes. The last attempt is about an hour after the first. After that the message is marked **Not sent**, stays in the drafts, and the Outbox shows the last error. Nothing is dropped silently.

A message whose final answer was lost is marked **Delivery unknown** and is not retried. There is no button to resend it. A missing copy in Sent does not prove that the message was not delivered, because filing the copy is a separate step after the mail server accepts the message.

In a setup with several server instances, all of them look at the same table, and a claim on each message makes sure only one of them sends it.

## Troubleshooting

- **Undo says "Too late to recall: the message is already being sent."** The undo period ended and the dispatcher claimed the message. It cannot be taken back. Recalling twice is harmless: the second request is refused and creates no duplicate.
- **Send later says "Scheduled sending is turned off by your administrator."** **Scheduling horizon** is 0 for the user's domain.
- **Send later says "That is too far ahead. Messages can be scheduled up to N days in advance."** The chosen time is beyond **Scheduling horizon**.
- **Send later says "Choose a time in the future."** The chosen time has already passed.
- **A send is refused with "too many queued or scheduled messages (200 allowed)".** The mailbox has reached **Queued messages per mailbox**. Wait for messages to leave or raise the limit.
- **Users cannot pick a long undo period.** Their choice is capped by **Undo-send window (limit)**, and the setting card says "Your administrator limits this to N."
- **The Undo send card says "Recalling sent messages is turned off by your administrator. Messages are sent immediately."** **Allow recalling sent messages** is off.
- **A message shows Delivery unknown.** See [When sending fails](#when-sending-fails).

## Related

- [Composer ribbon](/deploy/composer-ribbon)
- [Account drafts](/deploy/account-drafts)
- [Read receipts and delivery notifications](/deploy/read-receipts)
- [Settings](/admin/settings)
