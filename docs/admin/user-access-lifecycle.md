---
screen: true
---
# User access lifecycle

The **Access and removal** tab of a user on **Users** controls whether that person can use Mailwoman at all: block and unblock their sign-in, or delete their local access for good. These actions change only Mailwoman. The mailbox on the mail server, the user's Mailwoman settings and their history stay where they are.

![The user alex@example.com on the Users screen with the Access and removal tab open, showing the Mailwoman access card.](/screenshots/admin-access/user-access-page.webp){.mw-shot}

## Before you start

- Blocking, unblocking and clearing the older block need the user flags permission; deleting needs the user delete permission. An admin limited to some domains can act only on users of those domains.
- Each action asks you to confirm your own identity again (step-up) before it is applied.
- These controls are about mailbox users. They do not change admin accounts or roles.

## Block or unblock sign-in

1. On **Users**, open the user and select the **Access and removal** tab.

   ![The Mailwoman access card on the Access and removal tab of alex@example.com, with the Block login and Delete local access buttons.](/screenshots/admin-access/user-access-tab.webp){.mw-crop}

   1. **Block login** holds the user's sign-in and ends their open sessions (**Unblock login** when blocked).
   2. **Delete local access** removes Mailwoman access permanently.
   {.mw-callouts}

2. Select **Block login**. The dialog **Block Mailwoman login** explains the effect.
3. Select **Block login** in the dialog.

![The Block Mailwoman login confirmation for alex@example.com, explaining that blocking revokes current sessions and does not change the upstream mailbox.](/screenshots/admin-access/user-access-block-dialog.webp){.mw-crop}

A blocked user:

- cannot sign in, even when the mail server accepts the password;
- is signed out of every open Mailwoman session at once, and their engine-mode mail connections are closed;
- has their API keys and tokens refused, because those are checked against the same block.

The mailbox on the mail server is not changed: other mail programs can still reach it until you disable the account there too.

To lift the block, select **Unblock login** and confirm. Other restrictions (the domain's sign-in lists, two-factor requirements, sign-in locks) still apply.

## The older login block

Accounts can also carry an older "disabled" flag, set elsewhere in the console. While it is set, the tab shows **Clear legacy login block** and the note "A legacy login flag also blocks this user. Clear it explicitly before changing the directory block." **Block login** is unavailable until it is cleared.

**Clear legacy login block** removes only that flag; the user's other flags and their block status stay as they are, so the user may still be unable to sign in.

## Delete local access

1. Select **Delete local access**. The dialog **Delete local Mailwoman access** opens.
2. Type the user's address to confirm and select **Delete local access**.

::: danger Cannot be undone here
A deleted access record stays in the database so that the next successful sign-in does not simply create the account again. The console has no way to restore it.
:::

What happens:

- The user is signed out everywhere and cannot sign in again.
- Their Mailwoman settings and history, and the mailbox on the mail server, are kept.
- A user added in advance who never signed in can be deleted the same way.

## How the block is matched

The block applies to the user's address however it is written: different letter case and Unicode or IDNA spellings of the same address are treated as one. A block on an address also covers sign-ins through the domain's aliases (see [Domains](./domains.md#domain-aliases)).

::: warning Limit of 5,000 blocked users
The sign-in check reads all blocked and deleted users at once and gives up when there are more than 5,000. Past that number, sign-in fails for everyone rather than letting a blocked user through.
:::

## Console API

`GET`, `PUT` and `DELETE <console>/users/{id}/lifecycle` read, block or unblock (`{"disabled": true|false}`) and delete; `GET` and `DELETE <console>/users/{id}/legacy-block` read and clear the older flag. Every write needs the `If-Match` revision from a fresh `GET`: without it the answer is `428`, with an outdated one `412`, and changing a deleted record `409`.

Each change is written to the admin audit log as `user-login-blocked`, `user-directory-block-cleared` or `user-local-access-deleted`, with the number of sessions it ended.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| **Block login** is greyed out | The older login flag is set, the access is deleted, or your role lacks the permission | Clear the legacy block first, or ask an admin with the permission |
| The user still receives mail in another program | Mailwoman does not change the mailbox on the mail server | Disable the account on the mail server as well |
| "Reload the user before changing access." | The request had no current revision | Reload the page and repeat |

## Related

- [First sign-in and pre-added users](./preprovisioned-users.md)
- [Sessions](./sessions.md)
- [Domains](./domains.md)
- [Security centre](./security-center.md)
