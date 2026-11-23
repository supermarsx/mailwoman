---
title: Writing messages
screen: true
---
# Writing messages

Choose **Compose** in the sidebar to write a new message. It opens in its own tab. The same window opens when you reply to or forward a message. This page describes the window, the editor ribbon, and how a message is saved, sent or discarded.

![A new message with one recipient, a subject and a short text, with markers 1 to 8 on the Send button, the Send later arrow, the draft buttons, the recipients field, the subject, the ribbon tabs, the text area and the status line.](/screenshots/user-mailbox/compose-screen.webp){.mw-shot}

1. **Send**.
2. The arrow beside **Send**: **Send later options**.
3. The draft buttons **Discard**, **Save to Drafts**, **Drafts** and **Recall**. The marker sits on **Save to Drafts**.
4. The **To** field.
5. The **Subject** field.
6. The ribbon tabs: **Format text**, **Insert**, **Layout**, **Table layout**, **Security** and **Delivery**. The marker sits beside the last tab.
7. The text area.
8. The status line under the text area: the word, paragraph and character counts, the size, and the save state.
{.mw-callouts}

## Write and send a message

1. Choose **Compose**. A tab named **Compose** opens; it takes the name of the subject once you type one.
2. Type an address in **To** and press Enter. The address becomes a chip. See [Recipients](#recipients).
3. Type the **Subject**.
4. Click in the text area and write the message. The ribbon above it holds the formatting tools ([The editor ribbon](#the-editor-ribbon)).
5. Choose **Send**.

After you choose **Send** the message waits in your **Outbox** before it leaves, so you can still change your mind. The wait is the undo window, 10 seconds by default; you can change it in **Composing and replies** in Settings ([`mail.send.recallDelaySeconds`](/admin/settings#mail.send.recallDelaySeconds), [Mail settings](/user/settings/mail)). **Recall** at the top right of the window lists messages that are still held, and cancels them. Messages that have already been delivered cannot be recalled.

## Recipients

![The header of a new message: the To field with one recipient chip, the Cc Bcc and Reply-To links, and the Subject field.](/screenshots/user-mailbox/compose-header.webp){.mw-crop}

1. The **To** field. Each address becomes a chip with an **×** to remove it.
2. **Cc Bcc** shows the Cc and Bcc fields.
3. **Reply-To** adds a Reply-To address, so replies go to a different address.
4. The **Subject** field.
{.mw-callouts}

While you type in **To**, Cc or Bcc, Mailwoman suggests matching contacts. A suggestion matches the name or the organisation of a contact.

![The To field with the letters pri typed and a list of matching contacts below it.](/screenshots/user-mailbox/compose-suggest.webp){.mw-crop}

A chip marked with a warning symbol is not a valid address, and the status line reports it, for example **Invalid address: pri**. The status line also counts recipients outside your trusted domains, for example **External: 1**.

![The message header with the Cc and Bcc fields shown between To and Subject.](/screenshots/user-mailbox/compose-cc-bcc.webp){.mw-crop}

1. **Cc**.
2. **Bcc**.
{.mw-callouts}

An encrypted message lists the key of every recipient, so a Bcc recipient would be visible to the others. When both are set, the window asks you to remove the Bcc recipients or turn encryption off.

## The editor ribbon

The ribbon has six tabs. Choose a tab to see its tools. The pin at the right end of the tab row is named **Always show ribbon (Ctrl+F1)**. When it is pressed the ribbon stays open; when it is not, the ribbon hides while you are not using it.

### Format text

![The Format text tab of the editor ribbon: HTML and Visual editor, Clipboard, Format Painter, Font, Paragraph, Styles, Editing, View, Recipients and Message type.](/screenshots/user-mailbox/compose-ribbon-format.webp){.mw-crop}

**Format text** has the groups **HTML** (**Visual editor** and **HTML source**), **Clipboard**, **Format Painter**, **Font**, **Paragraph**, **Styles**, **Editing**, **View**, **Recipients** and **Message type**. The **Message type** button, shown as **HTML**, chooses between an HTML message and a plain text message.

### Insert

![The Insert tab: Response templates, Attachments, Illustrations, Links, Cards and calendar, Symbols, Insert, Table and Signature.](/screenshots/user-mailbox/compose-ribbon-insert.webp){.mw-crop}

**Insert** has **Response templates**, **Attachments**, **Illustrations** (**Picture from file…**, **Picture from messages…**, **Shapes**, **Picture from web address…** and **Screenshot**), **Links**, **Cards and calendar**, **Symbols**, **Insert**, **Table** and **Signature**.

To attach a file, open **Attachments** and choose **Attach a file**. The same menu has **Insert account item** and **Attach from messages…**.

![The Attachments menu of the Insert tab: Attach a file, Insert account item and Attach from messages.](/screenshots/user-mailbox/compose-attach-menu.webp){.mw-crop}

**Signature** lists the signatures of the identity you are sending from and has **Manage signatures** for creating one. See [Settings](/user/settings/account).

### Layout and Table layout

![The Layout tab: Columns, Content width, Spacing, Page, Themes, Font and Direction.](/screenshots/user-mailbox/compose-ribbon-layout.webp){.mw-crop}

**Layout** has **Columns**, **Content width**, **Spacing**, **Page**, **Themes**, **Font** and **Direction**. **Table layout** has the **Table** tools (rows, columns, merge, split, header row, delete) and **Cell shading**. Its buttons work when the cursor is in a table.

### Security

![The Security tab with the checkboxes Encrypt (end-to-end) and Sign (verify it's from you).](/screenshots/user-mailbox/compose-ribbon-security.webp){.mw-crop}

**Security** has the checkboxes **Encrypt (end-to-end)** and **Sign (verify it's from you)**. **Encrypt (end-to-end)** is available when keys for the recipients are found. See [Encryption and keys](/user/encryption-keys).

### Delivery

![The Delivery tab: Send later date and time, Importance, Request a read receipt, Attach my public key or certificate and the other options.](/screenshots/user-mailbox/compose-ribbon-delivery.webp){.mw-crop}

**Delivery** has:

- **Send later**: a date and time to send the message.
- **Importance**, with **Normal** as the default.
- **Request a read receipt**, **Request delivery notifications** and **Record when this message is opened**.
- **Attach my public key or certificate** and **Include SHA-256 hashes of attached files**.
- **Message options…**.
- **Sent copy**, for where the copy of the sent message is kept.

## Send later

The arrow beside **Send** opens **Send later options**.

![The Send later options menu under the Send button: In 1 hour, Later today, Tomorrow morning, Monday morning and Pick date and time.](/screenshots/user-mailbox/compose-send-later.webp){.mw-crop}

Choose **In 1 hour**, **Later today, 17:00**, **Tomorrow morning, 09:00** or **Monday morning, 09:00**, or choose **Pick date and time…** to set your own, then **Schedule send**. A time must be in the future, and it cannot be further ahead than your administrator allows. If your administrator has turned scheduled sending off, a message tells you so.

## Save a draft

Mailwoman saves what you write as a draft while you edit. The save state is on the status line:

- **Saved in this browser**: the window keeps a recovery copy in this browser.
- **Unsaved changes** and **Saving to Drafts…**: the draft is being saved to the **Drafts** folder of the account.
- **Saved to account Drafts**: the draft is in the **Drafts** folder.
- **Draft was not saved. Your local recovery copy remains available.**: the save failed; the recovery copy is kept.

**Save to Drafts** saves at once. **Drafts** opens a list of your drafts, with **Account** drafts and **Local drafts**; local drafts are kept in this browser only. An account keeps up to 100 drafts.

![The Drafts list opened from the composer: a search box, the tabs Account drafts and Local drafts, a Select button, and the empty Account drafts list; the draft being written is under Local drafts.](/screenshots/user-mailbox/compose-drafts.webp){.mw-crop}

The draft you are writing is listed under **Local drafts** until it is saved to the account. **Select** lets you choose drafts and discard them together.

To keep a draft, close the tab. To remove it, use **Discard**.

![The confirmation Discard this draft? with the buttons Keep editing and Discard.](/screenshots/user-mailbox/compose-discard.webp){.mw-crop}

**Discard** asks **Discard this draft?** and offers **Keep editing** and **Discard**. Closing the composer instead keeps the draft.

## Reply and forward

Choose **Reply**, **Reply all** or **Forward** in the [message toolbar](/user/mailbox/reading). A new tab opens with the recipients and subject filled in, and the original message quoted under the place where you write.

![A reply to Sam Okoye: the To field already holds the sender, the subject starts with Re:, and the quoted original sits below the cursor in the text area.](/screenshots/user-mailbox/compose-reply.webp){.mw-crop}

## Templates

**Response templates** on the **Insert** tab inserts text you saved. See [Templates](/user/mailbox/templates).

## Options

| Label | What it does | Default |
|---|---|---|
| Undo send ([`mail.send.recallDelaySeconds`](/admin/settings#mail.send.recallDelaySeconds)) | Seconds a sent message waits in the **Outbox** before it leaves; 0 sends at once | 10 |
| Draft save delay ([`mail.draftDebounceMs`](/admin/settings#mail.draftDebounceMs)) | Idle milliseconds before an edited draft is saved to the account | 1000 |
| Draft save interval ([`mail.draftIntervalSecs`](/admin/settings#mail.draftIntervalSecs)) | Seconds between periodic saves of a draft | 30 |

These can be changed in Settings and can be set or locked by an administrator ([Mail settings](/user/settings/mail)).

## Troubleshooting

- **The status line says "Invalid address"**: a chip in To, Cc or Bcc is not a complete address. Edit or remove it.
- **"Save confirmation is missing. Mailwoman will check again and retry safely."**: the mail server did not confirm the save to **Drafts**. Mailwoman retries, and your local recovery copy stays in this browser until the save is confirmed.
- **"This account already has 100 saved drafts."**: discard an account draft you no longer need, then save again.
- **Encrypt (end-to-end) is not available**: no keys were found for the recipients. Delivery is then protected in transit only.

## Related

- [Reading messages](/user/mailbox/reading)
- [Templates](/user/mailbox/templates)
- [Attachments](/user/mailbox/attachments)
- [Encryption and keys](/user/encryption-keys)
- [Settings](/user/settings/mail)
