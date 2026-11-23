---
title: Searching
screen: true
---
# Searching

The search box above the message list finds messages by sender, recipient, subject, text, attachment, date, size and more. You can type a query with operators, or build it with the **Filters** panel, which writes the query for you.

![The mailbox after a search for from:priya subject:spring: the query in the search box, the row of filter chips under it, and the single matching message in the list.](/screenshots/user-mailbox/search-results.webp){.mw-shot}

1. The search box. The query here is `from:priya subject:spring`.
2. The matching message. The row shows the folder the message is in.
{.mw-callouts}

Under the search box a row of chips shows how Mailwoman read the query: **Folder: Inbox**, **From: priya** and **Subject: spring**. A chip has an **×** to remove it. The row scrolls sideways when it is longer than the list.

## Search for messages

1. Click the search box (**Search mail**).
2. Type a query, for example `from:alice subject:invoice larger:1mb`, and press Enter, or click **Search**.
3. Open a result by clicking its row.
4. To clear the search, click the **×** in the search box, or press Esc.

By default a search covers the folder you are in. To search every folder, choose **All folders** for the **Folder** filter ([Search in other folders](#search-in-other-folders)).

## Words and operators

Words you type are matched against the message text. Put a phrase in quotes to match it exactly. Add an operator and a colon to limit a word to one field:

| Operator | Finds |
|---|---|
| `from:` | messages from a sender (name or address) |
| `from-domain:` | messages from a domain |
| `to:`, `cc:` | messages to or copied to someone |
| `subject:` | words in the subject |
| `body:` | words in the message text |
| `text:` | words in all the message text |
| `filename:` | an attachment file name |
| `tag:` | a label |
| `mentions:` | messages that mention someone |
| `in:` | messages in one folder |
| `has:attachment` | messages with an attachment (`has:calendar` for an event) |
| `is:` | state: `is:unread`, `is:read`, `is:flagged`, `is:answered`, `is:unanswered`, `is:forwarded`, `is:unforwarded`, `is:followup`, `is:finished`, `is:high-priority`, `is:low-priority` |
| `after:`, `before:` | a date, written `YYYY-MM-DD` |
| `larger:`, `smaller:` | a size such as `500k`, `2m` or `1g` |
| `pinned:` or `is:pinned` | pinned messages |
| `include:trash` | also search the Trash folder |

Combine them with spaces, which mean "and". Put `-` or `NOT` before a term to exclude it, `OR` between terms to accept either, and parentheses to group. A group or an `OR` stays one chip, **Expression**, because it is not a flat list of filters.

## Build a query with Filters

Click **Filters** (the funnel button) to open the **Guided search** panel.

![The Filters panel under the search box: the buttons From, To, Subject, Body, Date and Folder, the checkboxes Has attachment, Unread, Flagged, Marked for follow-up, Finished, Mentions me and Include Trash, and the Add filter button.](/screenshots/user-mailbox/search-filters.webp){.mw-crop}

1. The buttons **From**, **To**, **Subject**, **Body**, **Date** and **Folder**. Each opens a small editor for one filter.
2. The checkboxes **Has attachment**, **Unread**, **Flagged**, **Marked for follow-up**, **Finished** and **Mentions me**.
3. **Include Trash**. It applies when you search all folders; a selected folder is searched directly.
4. **Add filter** opens a menu with more filters, such as **Has event**.
{.mw-callouts}

Every change in the panel writes the query into the search box, and every change you type in the search box updates the chips.

### Choose a sender, a date or a folder

![The Filters panel after choosing From: a field for the sender, and the buttons Exclude matches, Apply and Cancel.](/screenshots/user-mailbox/search-from-filter.webp){.mw-crop}

**From**, **To**, **Subject** and **Body** each ask for a value. Tick **Exclude matches** to find messages that do **not** match, then choose **Apply**.

![The Filters panel after choosing Date, with fields for After and Before.](/screenshots/user-mailbox/search-date-filter.webp){.mw-crop}

**Date** asks for **After** and **Before**. Leave one empty for an open end. Dates are written as `YYYY-MM-DD`; a date that does not fit shows **Enter dates as YYYY-MM-DD.**

### Search in other folders

![The Filters panel after choosing Folder, with the choices All folders and Current folder (Inbox).](/screenshots/user-mailbox/search-scope.webp){.mw-crop}

**Folder** opens a list. Choose **All folders**, a single folder, or the current folder, and then **Apply**. **Remove** drops the folder filter.

## Search from a message

In the reader, **Search related messages** offers a ready-made search: the messages from this sender, with this subject, from this sender's domain, or in this folder. See [Reading messages](/user/mailbox/reading).

## Saved searches

A saved search can show up as a folder in your folder list. You create them in Settings, in **Saved searches**, as a search folder with its conditions. A search folder shows matching messages without moving or copying them. See [Mail settings](/user/settings/mail).

## Options

| Label | What it does | Default |
|---|---|---|
| Default search scope ([`mail.search.defaultScope`](/admin/settings#mail.search.defaultScope)) | Where new searches start: **Current folder** or **All folders** | **Current folder** |

The setting is in **Saved searches** in Settings. Saved searches keep their own scope.

## Troubleshooting

- **A search finds nothing in Trash**: Trash is not searched unless you add `include:trash` or tick **Include Trash** while searching all folders.
- **"Enter a size such as 500k, 2m or 1g."**: a `larger:` or `smaller:` value was not in that form.
- **Old messages are missing**: the search covers the current folder only. Choose **All folders**.

## Related

- [Reading messages](/user/mailbox/reading)
- [Folders and labels](/user/mailbox/folders-labels)
- [Attachments](/user/mailbox/attachments)
- [Mail settings](/user/settings/mail)
