---
title: Linked mailboxes
screen: true
---
# Linked mailboxes

A linked mailbox is another mailbox you open from Mailwoman while you stay signed in with your primary account. Each linked mailbox keeps its own preferences, identities, signatures, folders and drafts. You manage them in Settings, in **Linked mailboxes**.

![Settings open on Linked mailboxes: the list of linked mailboxes, which is empty, and the Link a mailbox button.](/screenshots/user-mailbox/linked-settings.webp){.mw-shot}

## Before you start

- Your administrator must allow linked mailboxes ([`accounts.linkingEnabled`](/admin/settings#accounts.linkingEnabled), off by default). If it is off, **Linking mailboxes is disabled by your account policy.**
- Linked mailboxes work with verified native IMAP, POP3 and EWS accounts. Accounts that sign in through a proxy or through single sign-on cannot be linked.
- Your administrator can limit how many mailboxes you may link. At the limit the page says **You have reached the maximum number of linked mailboxes allowed by policy.**

## Link a mailbox

1. Open **Settings** and choose **Linked mailboxes**.
2. Choose **Link a mailbox**.

![The Linked mailboxes card: its description, the Link a mailbox button, and the note that there are no linked mailboxes yet.](/screenshots/user-mailbox/linked-list.webp){.mw-crop}

1. **Link a mailbox**.
{.mw-callouts}

3. Enter the **Username or email address** and the **Password** of the mailbox. The **Display name (optional)** can stay empty.
4. Choose **Verify and link**.

![The Link a mailbox form with the fields Username or email address, Password and Display name (optional), and the buttons Verify and link and Cancel.](/screenshots/user-mailbox/linked-form.webp){.mw-crop}

1. **Username or email address**.
2. **Password**.
3. **Display name (optional)**, up to 128 characters.
4. **Verify and link**. **Cancel** is beside it.
{.mw-callouts}

Mailwoman verifies the credentials. If the mailbox requires a second factor, you complete it in the next step, **Verify this mailbox**. Your primary sign-in stays unchanged.

The mailbox password is kept encrypted on the server, so access renews by itself and survives restarts. It is never shown again or sent to your browser. Mailboxes with two-step verification renew by signing in again when access ends. If zero-access mode is on, or your administrator does not allow saved mailbox passwords, the server does not keep the password and you renew access yourself.

## Use a linked mailbox

- Choose **Open mailbox** beside the mailbox in the list, or pick it from the **Mailbox account** list in the mailbox. Every switch checks the link on the server before it shows the mailbox. If it cannot open, the page says **The mailbox could not be opened. Your previous mailbox has not been silently substituted.**
- **Set as primary** makes a linked mailbox the one that opens first after sign-in. The sign-in account is labeled **Primary**.
- **Rename…** gives the mailbox a nickname only you see.
- **Change avatar** chooses a letter or an icon with colors for the mailbox.
- Drag a mailbox by its handle to change the order, or press Alt and the Up or Down arrow on a mailbox. The order is the same on every device.
- **Show in account selector** and **Hide from account selector** control whether the mailbox is in the **Mailbox account** list.
- A mailbox whose access ended shows **Access expired**. Choose **Renew access** and sign in to it again. The link, the nickname, the order and the primary choice stay as they were.

To see the mail of all your mailboxes together, see [Unified mail](/user/unified-mail). To copy or move a message from one mailbox to another, see [Moving messages between accounts](/user/mailbox/moving-between-accounts).

## Unlink a mailbox

Choose **Unlink mailbox**, then **Confirm unlink**. This removes access through your primary account. It does not delete the mailbox or its messages.

## Options

![The Mailbox preferences card with the checkboxes Use linked mailboxes, Preload linked mailboxes and Show mailbox connection icons.](/screenshots/user-mailbox/linked-preferences.webp){.mw-crop}

1. **Use linked mailboxes**.
2. **Preload linked mailboxes**.
3. **Show mailbox connection icons**.
{.mw-callouts}

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Use linked mailboxes** ([`accounts.useLinked`](/admin/settings#accounts.useLinked)) | Allows access to your linked mailboxes when your administrator permits it. Turning it off does not delete the links or their mailboxes. | on | you or your administrator |
| **Preload linked mailboxes** ([`accounts.preloadLinked`](/admin/settings#accounts.preloadLinked)) | Keeps a small session cache of the folders and recent messages of linked mailboxes for faster switching. Mail caching must also be on. | on | you or your administrator |
| **Show mailbox connection icons** ([`accounts.showConnectionStatus`](/admin/settings#accounts.showConnectionStatus)) | Shows a small status icon beside mailbox names. Connection status is always available in **Linked mailboxes**. | off | you or your administrator |

## Troubleshooting

- **"Enter a username and password."**: both fields are required. Keep the display name under 129 characters.
- **"Set up the required second factor by signing in to that mailbox separately, then try linking it again."**: the mailbox requires a second factor that is not set up. Set it up in that mailbox, then link it.
- **"Change this mailbox's password through its own sign-in, then try linking it again."**: the mailbox requires a password change first.
- **"Verification expired. Start linking the mailbox again."**: start the link again.
- **"This mailbox's sessions were signed out, so its access no longer renews by itself until you sign in again here."**: choose **Renew access** and sign in.
- **"That sign-in is for a different mailbox. Sign in as the mailbox this link was made for."**: you renewed access with another account's name.

## Related

- [Unified mail](/user/unified-mail)
- [Moving messages between accounts](/user/mailbox/moving-between-accounts)
- [Account settings](/user/settings/account)
