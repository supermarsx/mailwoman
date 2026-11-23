---
title: Folders and labels
screen: true
---
# Folders and labels

Folders hold messages: a message is in one folder at a time. Labels are tags you add to a message; a message can have several, and they do not move it. This page covers the folder list, the folder menu, creating and arranging folders, and labels.

![The folder list: Inbox, Drafts, Sent, Archive, Spam and Trash, then the group User folders with Projects and Receipts, each with unread and total counts.](/screenshots/user-mailbox/folders-sidebar.webp){.mw-crop}

1. **Inbox**. The two numbers on a folder are its unread messages and its total messages.
2. **Spam**. Inbox, Drafts, Sent, Archive, Spam and Trash are the standard folders.
3. **User folders**, the group of folders you created.
4. A folder inside the group, **Projects**.
{.mw-callouts}

## Open a folder

Click a folder in the sidebar. Its messages appear in the message list. Choose **Open in new tab** from the folder's menu to open it in a tab of its own, where its messages are listed a page at a time, each with a check box.

![The folder Projects opened in its own tab: its four messages as rows with a check box each, the buttons Refresh, Select this page, Rename and Empty folder at the top, and Previous page and Next page at the bottom.](/screenshots/user-mailbox/folder-tab.webp){.mw-shot}

1. **Refresh** loads the folder again.
2. **Select this page** selects the messages on the page ([Selecting several messages](/user/mailbox/bulk-actions)).
3. **Next page**. **Previous page** is on the left.
{.mw-callouts}

## Create a folder

1. Right-click **User folders** in the sidebar and choose **Create folder**.
2. Type the **Folder name**.
3. Choose the **Parent folder**. **Top level** puts the folder at the first level.
4. Choose **Create folder**.

![The Create folder window with the name Clients typed, the Parent folder list set to Top level, and the buttons Cancel and Create folder.](/screenshots/user-mailbox/folders-create.webp){.mw-crop}

1. **Folder name**.
2. **Parent folder**.
{.mw-callouts}

To create a folder inside another folder, choose **New subfolder** in that folder's menu instead.

## The folder menu

Right-click a folder to open its menu.

![The context menu of the folder Projects: Open folder, Open in new tab, Mark all as read, Pin, Star folder, Folder layout, Move to, Refresh folder, New subfolder, Rename, Clean up folder, Empty folder, Properties, Folder settings, Retention, Hide from sidebar and More.](/screenshots/user-mailbox/folders-menu.webp){.mw-crop}

- **Open folder** and **Open in new tab**.
- **Mark all as read**. It is unavailable when the folder has no unread messages.
- **Pin** and **Star folder**. A starred folder can be shown on its own (**Show starred only**).
- **Folder layout** and **Move to…** arrange the folder in the list.
- **Refresh folder**.
- **New subfolder** and **Rename…**. You can also select a folder and press F2 to rename it. A name cannot be empty, cannot contain `*`, `%` or the character the mail server uses to separate folders, and cannot repeat the name of another folder in the same place.
- **Clean up folder…** finds messages older than an age you choose. You review the count before anything is deleted, and subfolders are excluded.
- **Empty folder…** moves every message in an ordinary folder to Trash, where you can restore them. In Spam and in Trash, emptying deletes the messages permanently. A window states the number of messages and asks you to confirm. Subfolders and the messages in them are not touched.
- **Properties…**, **Folder settings**, **Retention…** and **Hide from sidebar**.
- **More**: **Delete folder**, **Copy all items to…**, **Move all items to…**, **Move up** and **Move down**.

### Delete a folder

**Delete folder** asks **Delete** and the folder's name. It permanently deletes the folder. Subfolders are never deleted automatically: if the folder has subfolders you must remove or move them first. If the folder holds messages, tick **Also permanently delete all messages in this folder** to confirm; this cannot be undone.

### Arrange folders

Drag a folder to change its place, or drop it on another folder to put it inside that folder. With a folder focused, hold Alt and press an arrow key. When you nest a folder, Mailwoman asks how to move it: **Only arrange in this app** changes only how Mailwoman shows it, and **Move on the server** changes the folder everywhere, in every mail app that uses the account.

The standard folders cannot be moved unless your administrator allows it. When your administrator fixes the folder order, dragging is off and the sidebar says so.

**Sort folders by** (in the menu of **User folders**) offers **Name**, **Unread count**, **Total count**, **Last activity** and **Custom order**, each ascending or descending.

## Move a message to a folder

Do any of these:

- In the reader, choose **Move to…** and pick a folder ([Reading messages](/user/mailbox/reading)).
- Select messages and choose **Move to folder…** in **More selection actions** ([Selecting several messages](/user/mailbox/bulk-actions)).
- Drag a message from the list onto a folder in the sidebar.
- Choose **Archive** to move it to the Archive folder.

## Labels

A label is a coloured tag with a name and an icon. Point at a row in the message list and click **Label** to choose labels for that message.

![The Label menu opened from a message row, listing the labels Work, Personal, Important, Later and Receipts, each with its colour and icon.](/screenshots/user-mailbox/labels-menu.webp){.mw-crop}

Choose a label to add it. The row then shows the label as a tag next to the date. Choose it again to remove it. A notice at the bottom of the screen confirms **Label added** and offers **Undo**.

You can also add labels to many messages at once with **Labels…** in the selection bar. A label is stored on the message as a mail keyword, so it is separate from the folder the message is in. To see the messages that have a label, search for `tag:` and the label name ([Searching](/user/mailbox/search)).

### Manage labels

Label names, colours and icons belong to your account and are the same on every device. Open **Labels** in Settings to add a label (**New label**), edit its **Label name**, **Label colour** and **Label icon**, or remove it. Renaming a label keeps its keyword on the messages. Removing a label's definition keeps the keyword on the messages that have it, shown by its keyword name. An account can have up to 100 label definitions. See [Mail settings](/user/settings/mail).

## Share a folder

See [Sharing a folder](/user/mailbox/sharing).

## Options

| Label | What it does | Default |
|---|---|---|
| Rearrange folders by dragging ([`folders.reorder.enabled`](/admin/settings#folders.reorder.enabled)) | Lets you drag folders in the sidebar and in Settings | on |
| Drag to folder ([`mail.dragToFolder`](/admin/settings#mail.dragToFolder)) | Lets you move messages by dragging them to a folder | on |
| Hide empty folders ([`folders.hideEmpty`](/admin/settings#folders.hideEmpty)) | Hides folders with no messages in the sidebar | off |

These are in **Folders and sidebar** in Settings ([Mail settings](/user/settings/mail)). An administrator can set or lock them.

## Troubleshooting

- **"A folder name cannot contain …"**: the mail server uses that character to separate folders. Choose another name.
- **"There is already a folder called … here."**: pick another name or place.
- **"This folder has subfolders. Remove or move them first."**: delete or move the subfolders, then delete the folder.
- **"A folder cannot be moved into itself."** and **"A folder cannot be moved into one of its own subfolders."**: choose a different destination.
- **"The folder order is fixed by your administrator."**: moving folders is turned off for your account.

## Related

- [Selecting several messages](/user/mailbox/bulk-actions)
- [Searching](/user/mailbox/search)
- [Snooze messages](/user/snooze)
- [Rules and filters](/user/rules-filters)
- [Mail settings](/user/settings/mail)
