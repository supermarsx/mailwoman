---
title: Sharing a folder
---
# Sharing a folder

Sharing lets other users or groups of your mail server open one of your folders. You choose who, and what they may do in the folder. The mail server keeps the access list and enforces it: a change takes effect only if your account is allowed to administer the folder.

## Before you start

- The mail server must support folder sharing. If it does not, the window says **This mail server does not support folder sharing.**
- You need the **Administer** right on the folder. Without it, **Share folder** is not offered, and an access list that is open is read-only. If you try anyway, the message says **You are not allowed to manage sharing for this folder. Ask its owner or administrator.**
- Mailwoman checks the rights of the selected folder when you select it. A folder that is not shareable has no **Share folder** entry. Saved searches shown as folders cannot be shared.

## Share a folder

1. Select the folder in the sidebar.
2. Choose **Share folder**. It is an entry in the sidebar for the selected folder, and it is also in the folder's menu (right-click the folder).
3. A tab opens for the folder with the heading **Who can access** followed by the folder's name.
4. Under **Grant access**, type a **User or group** (a user name, a group, or `anyone`).
5. Under **Rights to grant**, tick the rights you want to give.
6. Choose **Add access**.

The window lists **Your access** (your own rights on the folder) and **People and groups**, with everyone who has been given access. Choose **Remove** next to an entry to take its access away. If nobody has been given access yet, the list says **No access has been granted yet.**

If the change was not saved, the window says **The change was not saved. The server rejected it or was unreachable.**

## The rights

| Right | What it lets someone do |
|---|---|
| **Look up** | See this folder in the folder list. |
| **Read** | Open the folder and read its messages. |
| **Keep read state** | Remember which messages have been read between sessions. |
| **Write flags** | Change flags other than read and deleted, such as flagged or answered. |
| **Insert messages** | Add messages to this folder by copying or moving them in. |
| **Post** | Send mail to the address associated with this folder, where supported. |
| **Create sub-mailboxes** | Create folders inside this one. |
| **Delete mailbox** | Delete or rename this folder itself. |
| **Delete messages** | Mark messages in this folder as deleted. |
| **Expunge** | Permanently remove messages that are marked as deleted. |
| **Administer** | Change who has access to this folder. |

## Troubleshooting

- **"Folder sharing is unavailable or you no longer have permission to manage it."**: the server does not offer sharing for this folder, or your Administer right was removed. Ask the folder's owner or your administrator.
- **"Could not load the mailbox access list."**: reopen **Share folder** to try again.
- **A person you added cannot see the folder**: **Look up** is the right that shows the folder in their folder list, and **Read** is the right that lets them open it. Add the rights that are missing.

## Related

- [Folders and labels](/user/mailbox/folders-labels)
- [Linked mailboxes](/user/mailbox/linked-accounts)
