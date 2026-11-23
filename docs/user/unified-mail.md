---
title: Unified mail
---

# Unified mail

**Unified mail** shows the mail of your main mailbox and your linked mailboxes in one
list. You can read, search and sort messages from all of them together, and handle them
in bulk. It is off until you turn it on in Settings, and it needs at least one linked
mailbox that is ready. See [Linked mailboxes](/user/mailbox/linked-accounts) for how to
link one.

::: warning No screenshot of the list
This manual has no screenshot of the unified list, because it needs a linked mailbox that
is ready. The labels below are the ones in the application.
:::

## Turn it on

Open **Settings**, then **Folders and sidebar**, then the **Folders and sidebar** tab.

![Two rows of the Folders and sidebar settings: Unified virtual account and Unified message view, each with a checkbox.](/screenshots/user-features/unified-settings.webp){.mw-crop}

1. **Unified virtual account** adds a separate virtual account to the account selector,
   with combined folders and counters. It is independent of the unified message view.
2. **Unified message view** adds **Unified mail** to the sidebar, to browse and search
   the main and linked mailboxes together.
{.mw-callouts}

Both are off at first, and an administrator can set or lock them. The settings take
effect only when linked mailboxes are allowed on your account and at least one is ready.

## Two ways in

- **Unified mail** in the sidebar opens the combined list while the normal folder tree
  stays in place.
- The virtual account in the account selector replaces the folder tree with combined
  **Inboxes**, **All mail**, **Sent**, **Drafts**, **Archive**, **Junk** and **Trash**
  folders and their counters. Your real sign-in does not change. Choose any real
  mailbox in the selector to go back to its normal view.

## Use the combined list

The list's header shows **Total**, **Unread** and **Loaded** counts. Under it:

- **Folder** chooses one of **Inboxes**, **All mail**, **Sent**, **Drafts**, **Archive**,
  **Junk** or **Trash**.
- **Account** chooses **All accounts** or one account. The buttons below the filters,
  one per account, show that account's unread and total counts and do the same.
- **Search mail** searches the list. The **Unread** checkbox shows unread messages only.

Every row shows a badge with the account it belongs to. Older messages load as you
scroll, and **Load more** loads the next page. If an account cannot be reached the list
says "Could not load mail from" that account and keeps showing the others. If nothing
matches, it says "No messages match this view."

Each row has buttons to mark a message read or unread, **Archive** it, **Delete** it, and
open **More actions**. To act on several messages, tick them, or use modifier-click or
keyboard range selection. The selection bar then acts on all of them, for example
marking read, **Archive**, **Delete** or marking as junk. Each change goes to the
message's own account. If some messages fail, the notice says "Could not update N
messages. They remain selected." Moving messages to a folder works only when the
selection is from one account. To copy or move one message to another account, use
**Copy or move to another account** when it is available; see
[Moving messages between accounts](/user/mailbox/moving-between-accounts).

Opening a message switches to the account it belongs to and opens it there.

## Named virtual accounts

Open **Settings**, then **Linked mailboxes**. Under **Unified account groups**, choose
**Add group** to make a named virtual account. Each group has a **Display name**, an
avatar, **Included accounts** and **Included folders**, and can be set to **Open
first**, so that it is the virtual mailbox that opens after sign-in. You can have up to
16 groups. Deleting a group keeps all of its accounts and messages.

**Preload linked mailboxes** in the same page keeps recently used linked mailboxes ready
for faster switching. It needs the mail cache to be on.

## Options

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Unified message view** ([`mail.unifiedInbox`](/admin/settings#mail.unifiedinbox)) | Adds Unified mail to the sidebar | Off | You, unless an administrator locks it |
| **Unified virtual account** ([`mail.unifiedAccount`](/admin/settings#mail.unifiedaccount)) | Adds the virtual account to the account selector | Off | You, unless locked |
| Groups ([`mail.unifiedAccount.groups`](/admin/settings#mail.unifiedaccount.groups)) | The named virtual accounts | None | You, unless locked |
| **Open first** ([`mail.unifiedAccount.primaryGroup`](/admin/settings#mail.unifiedaccount.primarygroup)) | The group opened after sign-in | The default virtual mailbox | You, unless locked |
| **Preload linked mailboxes** ([`accounts.preloadLinked`](/admin/settings#accounts.preloadlinked)) | Keeps linked mailboxes ready for faster switching | On | You, unless locked |

Administrators can also limit which accounts and folders unified views include.

## Troubleshooting

- **Unified mail is not in the sidebar.** Check that **Unified message view** is ticked,
  that linked mailboxes are allowed on your account, and that at least one is ready.
- **One account's messages are missing.** The list says "Could not load mail from" that
  account. Open the account's settings in **Linked mailboxes** to check its connection.

## Related

- [Linked mailboxes](/user/mailbox/linked-accounts)
- [Moving messages between accounts](/user/mailbox/moving-between-accounts)
- [Settings: mail](/user/settings/mail)
