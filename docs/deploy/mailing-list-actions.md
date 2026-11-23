---
title: Mailing list actions
---

# Mailing list actions

When a message comes from a mailing list, the reader shows a badge. Choosing it opens a dialog with what the message says about the list and the actions you can take. This page explains the dialog, how unsubscribing works, and the settings that control it.

![The Mailing list dialog for City Library News with markers 1 to 3 on the ways to unsubscribe, the Unsubscribe button and Always keep.](/screenshots/admin-ops/reader-list-badge.webp){.mw-crop}

1. **Ways to unsubscribe** lists what the message offers, such as a one-click request, a web page or an email address.
2. **Unsubscribe** starts the unsubscribe that the message offers.
3. **Always keep** exempts the list or sender from clean up and unsubscribe suggestions.
{.mw-callouts}

## Before you start

The badge appears when the message has a `List-Id` or a `List-Unsubscribe` header, and when **Mailing list badge** is not set to `never`. It reads "Mailing list · " followed by the list's name.

These headers are claims made by the message itself. Mailwoman does not verify them. Reading them and opening the dialog does not contact any address in them.

## What the dialog shows

- **List**, **List-Id** and **Sent from**.
- How many messages from the list, and from the sender, are in your mail, and when the last one arrived. Only messages that are loaded in the mailbox are counted.
- **Ways to unsubscribe**: "One-click request to <host>, sent by Mailwoman for you", "Web page on <host>, opened in a new tab after you review it", "Email to <address>, written as a draft you review", or "This message does not offer a way to unsubscribe."
- The links of the list, **Help**, **Post to list**, **Contact list owner** and **List archive**, as text. Only `http`, `https` and `mailto` addresses are shown. You can open each in a new tab or copy it. A list that says it takes no posts shows "This list says it does not accept posts by email."
- The buttons **Unsubscribe**, **Unsubscribe and clean up**, **Move future messages to a folder…**, **Mute this list**, **Always keep**, **Block sender**, **Create filter from this list…**, **Report as spam** and **Clean up this sender**.

## Unsubscribe

How **Unsubscribe** works depends on what the message offers.

- **One-click request.** The message has a `List-Unsubscribe-Post: List-Unsubscribe=One-Click` header and an `https` address in `List-Unsubscribe`. Mailwoman asks "Send an unsubscribe request to <host>? Mailwoman sends it for you, so the list host does not see your IP address." and, after **Send request**, sends it from the server. The request is a `POST` with the fixed body `List-Unsubscribe=One-Click` to that address, sent over HTTPS without cookies, referrer or redirects, to an address that passes the server's outbound address checks. A success says "Unsubscribe request sent to <host>. Mail can take a few days to stop."
- **Web page.** The address is shown first and opens in a new tab only after you confirm. It leaves Mailwoman without a referrer.
- **Email.** An ordinary editable new message opens with the recipient, subject and body from the `mailto` address. Mailwoman never sends it for you. Only one recipient, a subject and a plain-text body are accepted. Hidden recipients, attachments and header tricks in the address are refused.

A `List-Unsubscribe-Post` header alone does not prove who sent the message and does not give permission to send a request automatically.

### Remembering an unsubscribe

After a successful one-click request, or after you choose **I unsubscribed, remember it** for a web page or draft, Mailwoman records "Unsubscribed on <date>" for that list in the browser. If mail from the list arrives more than three days later, the dialog says "Mail has arrived since, so the list may be ignoring your request." **Forget** removes the record. The record is kept per account and `List-Id`, in the browser only, and is not sent anywhere.

## Options

| Label | What it does | Default | Who can change it |
| --- | --- | --- | --- |
| **Mailing list badge** ([`mail.badge.list`](/admin/settings#mail.badge.list)) | When the badge shows: `always`, `notable` or `never` | `always` | Administrators globally and per domain, and users, unless locked |
| **Mailing list Unsubscribe** ([`mail.list.unsubscribeMode`](/admin/settings#mail.list.unsubscribemode)) | `ask` confirms before sending a one-click request, `one-click-direct` sends it at once, `open-link` never sends a request and only opens the list's page or draft | `ask` | Same |
| **Always keep** ([`mail.list.keep`](/admin/settings#mail.list.keep)) | The senders and lists a user always keeps, as `sender:<address>` or `list:<List-Id>`. Clean up, unsubscribe suggestions and retention rules skip them | empty | Same |

## Troubleshooting

- **"The unsubscribe request failed (bad-url)."** The address in the message is not usable.
- **"The unsubscribe request failed (busy)."** The server is already sending four unsubscribe requests. Try again.
- **"The unsubscribe request failed (refused)."** The address is blocked by the server's outbound address checks, for example because it points at a private network. Use the list's own page.
- **"The unsubscribe request failed (timeout)."** The list host did not answer within 10 seconds.
- **"The unsubscribe request failed (rejected)."** The list host answered with an error.
- **"Kept on this page only: the choice could not be saved to your settings."** Always keep could not be written to the settings. The choice is held until the page is reloaded.
- **No badge shows.** The message has neither header, or **Mailing list badge** is `never`.

## Related

- [Delivery status reports](/deploy/delivery-status)
- [Outbound connections (egress)](/security/egress) for how the server checks outbound addresses
- [Settings](/admin/settings)
