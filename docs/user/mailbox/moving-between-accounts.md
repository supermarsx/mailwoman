---
title: Moving messages between accounts
screen: true
---
# Moving messages between accounts

**Copy or move to another account** sends one message from the mailbox you are in to a folder of another [linked mailbox](/user/mailbox/linked-accounts). It copies the message; if you choose, it also moves the original to Trash once the copy has succeeded.

![The mailbox with the Copy or move to another account window open over it: the message subject, the note No other linked mailbox is ready, the Destination account list, and the buttons Cancel and Copy message.](/screenshots/user-mailbox/transfer-dialog.webp){.mw-shot}

The window shows the subject of the message, a list of destination accounts, and the buttons **Cancel** and **Copy message**.

## Before you start

- Turn on **Copy or move to another account** in Settings, in **Folders and sidebar**. It is off by default ([`mail.crossAccountTransfer`](/admin/settings#mail.crossAccountTransfer)). Its help text reads: Transfer messages between linked mailboxes that support export and import.
- Link at least one more mailbox, and make sure it is ready ([Linked mailboxes](/user/mailbox/linked-accounts)). Without one, the window says **No other linked mailbox is ready.**
- The destination needs a folder you can add messages to.

## Copy or move a message

1. Right-click the message in the message list.
2. Choose **Copy or move to another account**.

![The context menu of a message row with the entry Copy or move to another account marked.](/screenshots/user-mailbox/transfer-menu.webp){.mw-crop}

1. **Copy or move to another account**.
{.mw-callouts}

3. In **Destination account**, choose **Choose an account** and pick the mailbox.
4. In **Destination folder**, choose a folder. The list shows the folders of that mailbox where you may add messages.
5. To move the message, tick **Move the original to Trash after the copy succeeds**. Leave it unticked to copy.
6. Choose **Copy message**. When the box is ticked the button reads **Move message**.

The window closes and a notice says **Message copied to the other account.** or **Message moved to the other account.**

A draft can be copied but not moved. The same entry is on the message menu of the [unified mail view](/user/unified-mail), where it works on one message at a time.

## Troubleshooting

- **"Could not transfer the message. Check the account and folder, then try again."**: the account or the folder did not accept the message.
- **"The destination did not confirm the import. Check the destination folder before trying again."**: look in the destination folder before you try again, so you do not create a second copy.
- **"The message was copied. Check the source mailbox to confirm whether it moved to Trash."**: the copy succeeded but the move of the original was not confirmed.
- **"This account has no writable folders available for imports."**: the destination cannot take messages by this route. Choose another account.

## Related

- [Linked mailboxes](/user/mailbox/linked-accounts)
- [Unified mail](/user/unified-mail)
- [Selecting several messages](/user/mailbox/bulk-actions)
