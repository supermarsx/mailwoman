---
title: Selecting several messages
screen: true
---
# Selecting several messages

Select more than one message to archive, delete, move, mark, flag or label them together. While messages are selected, a selection bar floats at the bottom of the message list and the reading area shows a panel with the same kind of actions.

![The mailbox with two messages selected: check marks on their rows, the floating selection bar at the bottom of the list, and in the reading area the panel 2 conversations selected.](/screenshots/user-mailbox/bulk-screen.webp){.mw-shot}

1. **Select messages**, the toolbar button that turns on selection mode.
2. The selection bar, here showing **2 selected**.
3. The panel in the reading area, **2 conversations selected**.
{.mw-callouts}

## Select messages

Use any of these:

- Click **Select messages** in the toolbar, then click the rows you want.
- Hold Ctrl (Command on a Mac) and click rows one at a time.
- Click a row, then hold Shift and click another row to select everything between them.
- With the keyboard, move to a row and press Space to select or clear it, press Shift+Up or Shift+Down to extend the selection, press Ctrl+A (Command+A) to select every message in view, and press Esc to clear the selection.

A selected row shows a check mark. A message that heads a conversation stands for every message in it. With selection mode off, a plain click on a row opens that message and clears the selection.

## The selection bar

![The selection bar with markers on the count, Select all, Archive, Delete, More selection actions and Clear selection.](/screenshots/user-mailbox/bulk-bar.webp){.mw-crop}

1. The count of selected messages.
2. **Select all** selects every message loaded in the list.
3. **Archive**.
4. **Delete**.
5. **More selection actions**, for the actions that do not fit in the bar.
6. **Clear selection** (the **×**).
{.mw-callouts}

The bar does not move the rows or take over the keyboard. You can keep clicking, scrolling and using the arrow keys in the list underneath. The bar is one stop after the list for the Tab key: press F6 in the list to move to the bar, use the arrow keys, Home and End to move between its buttons, and press Esc or F6 to go back to the list.

### More selection actions

![The More selection actions menu: Move to folder, Mark read, Flag, Mark as spam, Labels, Forward selected messages as attachments, Download selected messages as mbox and Download attachments as ZIP.](/screenshots/user-mailbox/bulk-more-menu.webp){.mw-crop}

- **Move to folder…** moves the messages to a folder.
- **Mark read** or **Mark unread**.
- **Flag** or **Remove flag**.
- **Mark as spam**.
- **Labels…** adds or removes labels, with one request for the whole selection.
- **Forward selected messages as attachments** opens a new message with the original messages attached. It does not add recipients and does not send anything.
- **Download selected messages as mbox** downloads one mbox archive.
- **Download attachments as ZIP** downloads the attachments of the selected messages as one ZIP file.

Forwarding and the mbox download take at most 100 messages and 32 MiB in total. If a selection is too large, the window says: **Select up to 100 messages totaling at most 32 MiB, and check your account's export permissions.** Your administrator's export and attachment limits apply to every message in the selection.

## The panel in the reading area

When you select two or more messages, the reading area shows a panel with the number of conversations selected and buttons for the whole selection.

![The panel in the reading area for two selected conversations: the buttons Select all in Inbox, Delete, Flag, Mark unread, Mark read, a Move to folder list, Properties and Cancel.](/screenshots/user-mailbox/bulk-panel.webp){.mw-crop}

- **Select all in Inbox** selects every message in the folder, not only the ones loaded in the list. If the folder cannot be loaded, the panel says **The full folder could not be selected. Check your connection and try again.**
- **Delete**, **Flag**, **Mark unread** and **Mark read**.
- **Move to folder** is a list of folders.
- **Properties…** opens the properties of the selected messages.
- **Cancel** clears the selection.

## Selecting in a folder tab

A folder opened in its own tab ([Folders and labels](/user/mailbox/folders-labels)) lists its messages a page at a time, each with a check box. **Select this page** selects the messages on the page, and the line at the bottom shows the range and the number selected, as in **1–4 of 4 · 0 selected**. The actions on a folder tab are mark read, mark unread, archive, delete and move. In Trash, delete is permanent and asks you to confirm first.

## Options

| Label | What it does | Default |
|---|---|---|
| Ctrl and Shift selection ([`mail.modifierSelection`](/admin/settings#mail.modifierSelection)) | Select with Ctrl-click, Shift-click, Shift+arrow keys and Ctrl+A | on |
| Drag to folder ([`mail.dragToFolder`](/admin/settings#mail.dragToFolder)) | Move selected messages by dragging them to a folder | on |

An administrator can set or lock both values.

## Troubleshooting

- **Some messages were refused**: the mail server did not accept the change for them. On a folder tab the result line counts the messages **updated**, **refused**, **unchanged or no longer in this folder** and **unconfirmed**; refresh the folder to check any unconfirmed ones before you try again.
- **Download selected messages as mbox is refused**: the selection is over 100 messages or 32 MiB, the export is not allowed for the account, or the server could not prepare the archive. No partial archive is downloaded.

## Related

- [Folders and labels](/user/mailbox/folders-labels)
- [Reading messages](/user/mailbox/reading)
- [Unified mail](/user/unified-mail)
- [Keyboard shortcuts](/user/keyboard-shortcuts)
