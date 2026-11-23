---
title: Server metadata
screen: true
---
# Server metadata

**Server metadata** (under **Operations** in the console navigation) shows and edits the server-level annotations of one provisioned account, as defined for IMAP by RFC 5464 (the METADATA extension). An annotation is a named value, for example `/shared/comment`, that the mail server stores. Changes are written straight to that account's mail server, and the mail server decides whether the account may set them.

![The Server metadata page: an Account selector and the prompt to select an account before its annotations are shown.](/screenshots/admin-ops/opsa-server-metadata.webp){.mw-shot}

## Before you start

- You need the `server_metadata.write` permission. Without it the page is not in the navigation.
- The account must be provisioned, and its mail server must support METADATA. The editor talks to the mail server for the account you choose, through the console's own route for this page, which works only when the server runs in engine mode. In proxy mode the server refuses that request.
- With no account provisioned the page says **No accounts are provisioned.** and offers **Add a user**, which opens **Users**.

## View and change annotations

1. Open **Server metadata**.
2. Under **Account**, choose **Select an account…** and then the account. Until you choose one the page says **Select an account to view and edit its server annotations.** If the list cannot be read, it says **Could not load the account list.**
3. The editor **Server annotations** lists the **Entries** of that account. An entry that has no value is shown as **Not set**; if there are none, the editor says **No annotations are set.**
4. To change a value, edit it under **Value** and choose **Save**. To remove an entry, choose **Remove**.
5. To add an entry, under **Add an annotation** enter the **Entry** (for example `/shared/comment`) and the **Value**, then choose **Add annotation**.

If the mail server rejects a change or cannot be reached, the editor says **The change was not saved. The server rejected it or was unreachable.** and nothing is changed.

## What the editor shows when annotations are not available

- **Could not load the annotations.** with **Retry**: the mail server did not return its annotations. This is also what the editor shows when the account's mail server does not support METADATA at all.
- **This mail server does not provide annotations.**: the server answered that it does not implement the method.

## Options

The page has no settings of its own. The account selector is the only control that is not part of the editor.

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Account** | The provisioned account whose annotations are shown. | None selected | Administrators with `server_metadata.write` |

## Troubleshooting

- **The editor says "Could not load the annotations." for every account.** The mail server does not support METADATA, or the server is in proxy mode. Use **Retry** once, then check the mail server's capabilities.
- **A value does not save and the editor reports the change was not saved.** The mail server refused it; it may not allow this account to set that entry.
- **The account list is empty.** Provision users in **Users** first.

## Related

- [Users](/admin/user-access-lifecycle)
- [Maintenance](/admin/maintenance)
