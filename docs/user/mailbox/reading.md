---
title: Reading messages
screen: true
---
# Reading messages

Opening a message shows it in the reader, under a header with the subject, the sender, the recipients and the date, a toolbar of actions, and, when the message has files, an attachments bar.

![An opened message: the header with sender, recipient and date, the action toolbar, the attachments bar with one PDF, and the message text, with markers 1 to 4 on each.](/screenshots/user-mailbox/reading-screen.webp){.mw-shot}

1. The header.
2. The message actions toolbar.
3. The attachments bar.
4. The message text. Message content stays isolated from the application.
{.mw-callouts}

## Open a message

1. Click a row in the message list. The message opens in the reader. Where the reader sits (right of the list, below it, or not at all) depends on **Reading pane** in the [toolbar](/user/#the-toolbar).
2. To open it in its own tab instead, right-click the row and choose **Open in a new tab**.
3. To close it, use **Close message** on the toolbar.

## The header

![The header of an opened message: subject, From and To, the date, the Inbox folder chip, and the DMARC not present chip, with markers 1 and 2 on the folder chip and the DMARC chip.](/screenshots/user-mailbox/reading-header.webp){.mw-crop}

1. The folder chip, here **Inbox**. Its tooltip reads **Stored in Inbox · Go to folder**, and clicking it goes to that folder.
2. The authentication badge. It reports the DKIM, SPF and DMARC results your server recorded when the message arrived. When there is no DMARC result the badge reads **DMARC not present**. Click it to see the details.
{.mw-callouts}

The sender and the recipients are buttons. Click a name to see that person's contact details. Next to the date, **Spam score unavailable** appears when the server has no spam score for the message.

A message can carry chips that the sender asked for:

- **Mailing list** with the list name, for mail from a mailing list.
- **Replaces an earlier message**: the sender says this message replaces an earlier one. A message that has been replaced shows **Superseded by a newer message**.
- **Reply by** and **Expires** with a date, when the sender set them. An expired message shows **Expired** and a suggestion to archive it.
- **Personal**, **Private** or **Company confidential**: a label the sender put on the message. It does not protect the message.

![The header of the message Agenda for Thursday (room change) with the chip Replaces an earlier message and a Find it button.](/screenshots/user-mailbox/reading-superseded.webp){.mw-crop}

1. The **Replaces an earlier message** chip.
{.mw-callouts}

## Use the toolbar

![The message actions toolbar with markers 1 to 10 on its groups of buttons: reply, conversation, pin and flag, archive and delete, print, zoom, related search, view and colors, raw headers, and More actions.](/screenshots/user-mailbox/reading-actions.webp){.mw-crop}

1. **Reply**, **Reply all** and **Forward**. See [Writing messages](/user/mailbox/composing).
2. **View conversation** lists the messages of the thread, across folders.
3. **Pin**, **Mark unread** and **Flag**. **Flag** sets the flag on the message; it is not the same as a follow-up reminder.
4. **Archive**, **Move to…**, **Delete** and **Spam**.
5. **Print / Save as PDF**.
6. The zoom group: **Zoom out**, **Message zoom 100%. Reset to 100%** and **Zoom in**. Only the message text changes size; printing always uses 100%.
7. **Search related messages** ([Searching](/user/mailbox/search)).
8. **View: Full** and **Message colors**.
9. **Raw message headers**.
10. **More actions**.
{.mw-callouts}

Between the search button and the view buttons is **Clean up this sender**, which lets you review what to do with the other messages from that sender before anything is applied.

Each action can be placed on the toolbar, in **More actions**, or hidden, in the Message display settings ([Mail settings](/user/settings/mail)). Actions that do not fit on the toolbar move into **More actions**.

### Move a message

Choose **Move to…**, then a folder from the list. Type in **Search folders** to narrow the list.

![The Move to list under the toolbar: a Search folders box and the folders Drafts, Sent, Archive, Spam, Trash, Projects and Receipts.](/screenshots/user-mailbox/reading-move-menu.webp){.mw-crop}

### View a conversation

**View conversation** opens a window that lists the messages of the thread in all folders, each with its sender and folder. The open message is outlined. Opening a message from the window does not change your folder selection.

![The View conversation window listing the four messages of the thread, with the open message outlined.](/screenshots/user-mailbox/reading-conversation.webp){.mw-crop}

### View mode and colors

![The View mode menu with Full, No media and Plain text, Full checked.](/screenshots/user-mailbox/reading-view-menu.webp){.mw-crop}

**View: Full** opens the **View mode** menu:

- **Full**: sanitized HTML with images and styling.
- **No media**: sanitized HTML with all images and media removed.
- **Plain text**: no HTML is rendered; the message is shown as plain text only.

![The Message colors menu with four colour modes and the two entries for making a mode your default.](/screenshots/user-mailbox/reading-colors-menu.webp){.mw-crop}

**Message colors** chooses how the colors of a message are shown: **Automatic with theme** (the default), **Original message colors**, **Adapt to theme** and **Invert message colors**. **Make this my default** saves the mode you chose for every message. Both entries are unavailable when your administrator locks the setting.

### Raw headers

**Raw message headers** opens a window with the original header lines of the message. **Copy headers** copies them.

![The Raw message headers window with a Copy headers button and the original header lines of the message.](/screenshots/user-mailbox/reading-headers-dialog.webp){.mw-crop}

### More actions

![The More actions menu: Export, Properties, Save sender as contact, Save contacts from this message, Create task from message, View source, Create filter from this message and Block sender.](/screenshots/user-mailbox/reading-more-menu.webp){.mw-crop}

The menu holds **Export**, **Properties…**, **Save sender as contact**, **Save contacts from this message**, **Create task from message**, **View source**, **Create filter from this message** and **Block sender**. **Export** saves the message in the format you choose in the **Format** list, such as **EML (original message)**, which keeps the original, or a converted copy such as **Plain text**, **Markdown**, **MSG (Outlook)**, **DOCX (Word)** or **PDF**; your administrator's export and attachment limits apply. See [Contacts](/user/contacts), [Tasks](/user/tasks) and [Rules and filters](/user/rules-filters) for the contact, task and filter entries.

### Related searches from a message

![The Search related messages menu.](/screenshots/user-mailbox/reading-related-menu.webp){.mw-crop}

**Search related messages** offers searches based on the open message: the messages from the same sender, with the same subject, from the same domain, or in the same folder. The search then runs in the search box ([Searching](/user/mailbox/search)).

## Attachments

When a message has files, the attachments bar shows the number of files and their total size, for example **1 attachment · 688 bytes**. **Download all** downloads every file as one ZIP file. The arrow at the right shows or hides the list of files.

![The attachments bar of a message: the summary 1 attachment, the Download all button, the arrow that shows the files, and the file catalogue-proof.pdf with its download and More actions buttons.](/screenshots/user-mailbox/reading-attachments.webp){.mw-crop}

1. The summary of the attachments.
2. **Download all** (the ZIP file).
3. The arrow that hides the list (**Hide attachments**). When the list is hidden it shows the files again (**Show attachments**).
{.mw-callouts}

Click a file to download it. The button with three dots on a file opens its menu.

![The menu of the file catalogue-proof.pdf: Open, Open in split view, Download and Forward in a new email.](/screenshots/user-mailbox/reading-attachment-menu.webp){.mw-crop}

- **Open** opens the file in its own tab, with a viewer for the file type. See [Attachments](/user/mailbox/attachments).
- **Open in split view** opens the file beside the message.
- **Download** saves the file.
- **Forward in a new email** starts a new message with the file attached.

Files that your attachment policy blocks cannot be downloaded or opened; the bar says why.

## Images in a message

Images stored on other servers can tell the sender when and where you opened the message, so Mailwoman blocks them until you choose to show them. The bar above the message text says what was blocked.

![A newsletter with its remote images blocked: the Remote images blocked bar with a note that one tracker was removed, the Show images button (1) and the More image options arrow (2).](/screenshots/user-mailbox/reading-remote-images.webp){.mw-crop}

1. **Show images** loads the images of this message through the server, so the sender does not see your IP address.
2. **More image options** opens a menu: **Always show images from** the sender, **Always show images from** the sender's domain, **Always show images in all mail**, and **Privacy settings**.
{.mw-callouts}

The bar also counts the trackers it removed, for example **1 tracker removed**. Tracking images are removed before the message is shown. After you show images, **Undo** blocks them again.

## Options

| Label | What it does | Default |
|---|---|---|
| **Reading pane** | Where the open message appears: **Right**, **Bottom** or **Off** | **Right** |
| Message zoom ([`reader.zoom`](/admin/settings#reader.zoom)) | The size messages open at, from 50% to 200% | 100% |
| **Message colors** | How the colors of a message are shown | **Automatic with theme** |
| **Automatically hide message headers** | Opens each message with the sender and recipient details collapsed | off |
| Remote content ([`privacy.remoteContent`](/admin/settings#privacy.remoteContent)) | What happens to remote images: block, ask, load through the server, or load directly | ask |

These settings are in the Message display and Privacy sections of [Settings](/user/settings/mail).

## Troubleshooting

- **A message shows "Superseded by a newer message"**: a newer message names this one in its Supersedes header. Open the newer message from the list.
- **Images do not load**: they are blocked until you choose **Show images**. If the bar says they are blocked by your privacy settings, change that in the Privacy settings.
- **A file cannot be downloaded**: the bar or the file shows the reason, for example that downloads are turned off or the attachment policy blocks the file.

## Related

- [Writing messages](/user/mailbox/composing)
- [Searching](/user/mailbox/search)
- [Attachments](/user/mailbox/attachments)
- [Snooze messages](/user/snooze)
- [Keyboard shortcuts](/user/keyboard-shortcuts)
