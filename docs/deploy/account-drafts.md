---
title: Account drafts
---

# Account drafts

The composer can save a message you are writing to the **Drafts** folder of your mail account, so you can continue on another device. These are called account drafts. This page explains when they are saved, which accounts support them, and the settings that control them. For the buttons in the composer see [Composer ribbon](/deploy/composer-ribbon).

## Before you start

Account drafts work only when all of the following are true:

- The account is one that Mailwoman reaches itself over IMAP. Accounts reached through POP3, through a JMAP server that Mailwoman forwards to, and through a plugin report "This account cannot currently confirm saves to its Drafts folder."
- The IMAP server confirms where it stored a message, which means it supports `UIDPLUS` or IMAP4rev2. Otherwise saving is refused with "draft saving requires confirmed APPENDUID".
- The account has one existing Drafts folder that Mailwoman can identify. If it cannot, saving is refused with "map an existing upstream Drafts folder before saving".

Mailwoman never replaces a draft save with a local copy that looks like one, and never sends the message to save it.

## How drafts are saved

Each user chooses **Save automatically** under **Account drafts** in their **Settings**.

| Label | What it does | Default | Who can change it |
| --- | --- | --- | --- |
| **Account draft save mode** ([`mail.draftSaveMode`](/admin/settings#mail.draftsavemode)) | `realtime` saves after a typing pause, `periodic` saves at an interval, `manual` saves only when the user chooses **Save to Drafts** | `realtime` | Administrators globally and per domain, and users, unless locked |
| **Draft typing pause (milliseconds)** ([`mail.draftDebounceMs`](/admin/settings#mail.draftdebouncems)) | The idle time before a save in `realtime` mode. 500 to 10000 | 1000 | Same |
| **Draft save interval (seconds)** ([`mail.draftIntervalSecs`](/admin/settings#mail.draftintervalsecs)) | The time between saves in `periodic` mode. 5 to 3600 | 30 | Same |

A successful save shows only a small status at the bottom of the composer, such as "Saved 09:30 AM". A failure shows an error notice.

## Open a draft

Choose **Drafts** in the composer. The list has two tabs, **Account drafts** and **Local drafts**. Choose an account draft to continue it on any device. Opening a draft that Mailwoman created from the **Drafts** folder's message list reaches the same editable draft. A draft that another mail program created stays an ordinary message that you can read, and is not treated as an editable draft.

![The composer's Drafts list with the Account drafts tab selected and one account draft marked Open now and Account.](/screenshots/admin-ops/compose-drafts-popup.webp){.mw-crop}

An account has at most 100 account drafts. At the limit a save fails with "This account already has 100 saved drafts. Discard an unwanted account draft before saving a new one."

## Close, discard and send

- Closing a composer keeps its draft.
- **Discard** in the composer asks "Discard this draft?". **Keep editing** is the safe choice. Discarding removes the copy in the Drafts folder first and then frees the slot. A small marker remains so that a late request cannot bring the draft back. If the removal fails, the draft stays and the composer says "Draft removal could not be confirmed. Its account record remains; retry Discard after reviewing the Drafts folder."
- A save writes the new copy and confirms it before it removes the previous one. If the old copy cannot be removed, the composer says "The new draft was saved, but its previous copy could not be removed. Review Drafts before saving again."
- Sending waits for a save that is under way and stops further automatic saves. A message that is queued keeps its saved draft. After delivery is confirmed, the draft is removed. A newer revision is never removed because an older one was sent.

## What is not saved

A message that quotes decrypted text, or that is set to be encrypted, is not uploaded as a plaintext draft. The composer says "Encrypted or decrypted quoted content is not saved as plaintext. Any earlier account draft remains until you explicitly discard it." Closing such a message discards the changes.

The content of an account draft is stored sealed on the Mailwoman server, tied to the signed-in user and the draft revision. This is not end-to-end encryption. The copy in the Drafts folder is an ordinary message on the mail server.

## Limits

| Limit | Value |
| --- | --- |
| Account drafts per account | 100 |
| Editor state of one draft | 1 MiB |
| Ordinary attachments per draft | 100 |
| Total size of attachments | 32 MiB |

## Troubleshooting

- **The status says "Draft was not saved. Your local recovery copy remains available."** The save failed. The browser keeps a local recovery copy. Check the connection and use **Save to Drafts**.
- **The status says "Waiting for server confirmation; saving will retry automatically when safe." or "Save confirmation is missing. Mailwoman will check again and retry safely."** The save may have reached the mail server but was not confirmed. Mailwoman does not repeat the write blindly, so it checks the Drafts folder first. Look at the folder before you save again.
- **"Another device changed this draft. Your edits have not overwritten its copy."** Two devices edited the same draft. Reopen it from **Drafts** to see the other copy.
- **"This account cannot currently confirm saves to its Drafts folder."** See [Before you start](#before-you-start).
- **Save mode is Only when I choose Save and nothing is saved.** In this mode the composer saves only when the user chooses **Save to Drafts**.

## Related

- [Composer ribbon](/deploy/composer-ribbon)
- [Outbox, recall and scheduled sending](/admin/outbox-recall)
- [Settings](/admin/settings)
