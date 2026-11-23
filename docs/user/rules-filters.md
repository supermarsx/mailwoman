---
title: Rules and filters
screen: true
---

# Rules and filters

**Rules & filters** sorts incoming mail for you. A rule has conditions, such as "the
sender is sam@example.org", and actions, such as "move it to Projects". You build rules
in a form; people who know the Sieve filtering language can edit scripts directly. The
same screen also sets up forwarding and, where the mail server offers it, automatic
vacation replies.

Open **Settings**, then **Mail rules**. The screen is titled **Rules & filters**.

![The Rules and filters screen with the Builder, Raw Sieve and Forwarding tabs, and the rule builder open with a new rule named Print shop mail.](/screenshots/user-features/rules-screen.webp){.mw-shot}

## Before you start

Rules need support from your mail server. If it has none, the screen says "This mail
server does not support mail rules." What is available also depends on administrator
settings, and the screen says so under its title.

## Create a rule

1. Choose **New rule**.
2. Enter a **Rule name**. A rule cannot be saved without one.
3. Choose **Match all conditions** or **Match any condition**.
4. Set a condition (a new rule starts with **From** **contains**): pick the **Field**, the **Operator** and the **Match value**.
   Choose **Add condition** for more.
5. Set an action. Choose **Add action** for more.
6. Choose **Save rule**. **Cancel** closes the form without saving.

![The rule builder opened from a message, with the rule name, the match choice, one condition on the sender, one Move to mailbox action, and the Save rule button.](/screenshots/user-features/rules-from-message.webp){.mw-crop}

1. **Rule name**.
2. **Match all conditions** / **Match any condition**.
3. The condition's **Field**: **From**, **To**, **Subject** or **Mailing list
   (List-Id)**.
4. The **Operator**: **contains** or **is**.
5. The **Match value**, the text to look for.
6. **Add condition**. The button with a cross beside a condition removes it.
7. The **Action**: **Move to mailbox**, **Add tag**, **Archive**, **Forwarding**,
   **Keep a copy in this account** or **Stop processing**.
8. The **Action value**: the mailbox name for **Move to mailbox**, the tag for **Add
   tag**, the address for forwarding.
9. **Add action**. The button with a cross beside an action removes it.
10. **Save rule**.
{.mw-callouts}

**Stop processing** ends the rule list for that message: rules below it are not applied.

### Start from a message

Right-click a message and choose **Create filter from this message**. The builder opens
with the rule named after the sender and a condition "From is" the sender's address.
Choose the action, then **Save rule**.

For a mailing list, the list's badge in the message header has **Mute this list**. It
saves a rule that matches the list's List-Id, marks the messages read and archives them.

## Where a rule runs

Under the form, a badge says where the rule will be applied:

- **Server (Sieve)**: "Uploaded to the mail server and applied there."
- **Engine**: "Applied when the engine syncs new inbox messages; replies and moves wait
  for the next sync."

## Manage your rules

Saved rules are listed under **Mail rules**, each with its name, the badge above, an
**Enabled** checkbox, **Edit** and **Delete**. Untick **Enabled** to switch a rule off
without deleting it.

## Raw Sieve

Choose the **Raw Sieve** tab to work with scripts for the server.

![The Raw Sieve tab of Rules and filters with the script list and editor.](/screenshots/user-features/rules-raw.webp){.mw-shot}

- **Sieve scripts** lists your scripts; the one in use is marked **Active**.
- **New script** starts an empty script, and a template list offers a starting point.
  **Export script** downloads the script as a `.sieve` file, and **Import script** loads
  one.
- Edit the **Raw Sieve source**, then choose **Validate script**. A script with no
  errors reports "No problems found." Choose **Save** to store it, and **Activate** to
  apply it to incoming mail, or **Deactivate** to stop it. A script that is active
  cannot be deleted.

If your administrator has turned this off, the tab says "Your administrator has disabled
raw script editing." and the editor is read-only.

## Forward your mail

Choose the **Forwarding** tab.

![The Forwarding tab of Rules and filters with the Forward to field and the Keep a copy option.](/screenshots/user-features/rules-forward.webp){.mw-shot}

1. Tick **Enabled**.
2. Enter the address in **Forward to**.
3. Tick **Keep a copy in this account** if you want to keep the original too. The option
   appears when the mail server can keep a copy.
4. Choose **Save**.

If your administrator does not allow forwarding outside your own domains, the tab says
"Your administrator does not allow forwarding to this domain." and **Save** stays off.
Adding a forwarding rule also creates a notice in
[Notifications](/user/notifications).

## Vacation replies

When your mail server, or Mailwoman, can send automatic replies, a **Vacation reply** tab
appears. Tick **Enable automatic replies**, set **Start (UTC)** and **End (UTC)**, write
the **Reply text**, and save. The tab says who sends the reply: your mail server when mail
arrives, or Mailwoman when it syncs new inbox mail, so the reply can wait until the next
sync. When neither can, the screen says "Vacation replies are not available for this
account: its mail server does not offer them, or your administrator has turned them off."
and the tab is not shown.

## Options

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Enabled** on a rule | Switches a rule on or off | On for a new rule | You |
| Raw script editing | Whether the Raw Sieve tab can be edited | Set by your administrator | Administrator |
| Forwarding outside your domains | Whether **Forward to** may name an outside address | Set by your administrator | Administrator |

## Troubleshooting

- **"The rule was not saved:" followed by a reason.** The server refused the rule. Read
  the reason, fix the rule and save again.
- **A condition on a thread cannot be saved.** The form says "Thread … conditions are not
  supported, so the rule cannot be saved until this condition is removed." Remove the
  condition.
- **"The rules could not be loaded."** Choose **Retry**.
- **A rule marked Engine does not act at once.** It runs when new inbox mail is
  synced; replies and moves wait for the next sync.

## Related

- [Settings: mail](/user/settings/mail)
- [Notifications](/user/notifications)
- [Folders and labels](/user/mailbox/folders-labels)
