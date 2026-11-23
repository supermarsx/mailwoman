# Composition formats and local drafts

This page explains the three body formats the composer offers, where an unsent message is kept while you write it, how named signatures are stored and inserted, and how message notices are added after the signature. It is for administrators who set defaults and locks, and for users who want to know what is sent.

## Message type

The **Message type** group on the **Format text** tab of the composer ribbon switches the body between three formats:

<!-- SCREENSHOT deploy/compose-message-type: composer, Format text tab open with the Message type menu expanded, crop of the Message type group -->

| Option | What you edit | What is sent |
| --- | --- | --- |
| **Plain text** | A plain text area. | A `text/plain` body only. Angle brackets, ampersands and line breaks stay literal text. |
| **Rich text** | The formatting editor. | An HTML body built from the editor's content. |
| **HTML** | The HTML editor, with a **Visual editor** view and an **HTML source** view. | An HTML body. Only supported email formatting is kept; scripts, external images and unsafe markup are removed. |

A new message opens in **HTML**. A resumed draft opens in the format it was saved in.

- Switching from **Plain text** to **Rich text** or **HTML** converts your text to HTML, escaping any markup characters you typed.
- Switching from **Rich text** or **HTML** to **Plain text** keeps the text and drops the formatting.

::: tip
If an administrator turns off rich text (`cap.compose.rich_text`, see [Configuration](../admin/configuration.md#cap.compose.rich-text)), the **Message type** group is hidden, the composer switches to **Plain text**, and the server refuses messages with an HTML body.
:::

### Recipients and open tracking

- Cc and Bcc recipients all receive the message. The message itself does not carry a `Bcc:` header, so recipients cannot see who was blind-copied.
- **Record when this message is opened** is offered only for **Rich text** and **HTML** messages that are neither encrypted nor signed, and only when the server has open tracking turned on. A plain-text, encrypted or signed message never gets a tracking image.

## Drafts while you write

The composer keeps two kinds of draft:

| Kind | Where it is kept | Shown as |
| --- | --- | --- |
| Local recovery copy | This browser's offline storage, encrypted. Up to 25 per account; the most recently saved are kept. | **Local drafts** in the **Drafts** list ("Local drafts are kept in this browser only.") |
| Account draft | The account's Drafts folder on the mail server, so you can resume it on another device. Up to 100 per account. | **Account drafts** in the **Drafts** list |

A local recovery copy is saved each time the composition changes, once it has a recipient, subject, body text or attachment. It keeps the selected identity, body format, inserted signatures, notice choice, recipients, reply threading, message options and references to uploaded attachments and inline images.

Local recovery copies:

- belong to the signed-in account; another account signing in at the same browser does not see them;
- are deleted when you send the message or discard the draft;
- are all deleted when you sign out;
- are not saved when encrypted storage is unavailable. The composer then warns: "Encrypted local recovery is unavailable. Keep this editor open or save the draft to the server before closing."

A message that quotes decrypted text from an encrypted message is not saved as a draft at all, locally or to the account. The composer shows **Not saved as plaintext**.

Account drafts are saved according to **Save automatically** in **Settings** > **Mail** > **Composing and replies** > **Sending options** > **Account drafts**: **After a typing pause**, **At regular intervals** or **Only when I choose Save** (then use **Save to Drafts** in the composer). When you send a message that was saved as an account draft, its copy is removed from the Drafts folder. Requirements and failure states are described in [Account drafts](./account-drafts.md).

| Setting | What it does | Default |
| --- | --- | --- |
| [`mail.draftSaveMode`](../admin/configuration.md#mail.draftsavemode) | `realtime`, `periodic` or `manual` | `realtime` |
| [`mail.draftDebounceMs`](../admin/configuration.md#mail.draftdebouncems) | Typing pause before a save, 500 to 10000 ms | `1000` |
| [`mail.draftIntervalSecs`](../admin/configuration.md#mail.draftintervalsecs) | Interval for periodic saves, 5 to 3600 s | `30` |

The composer caches the account draft list for 15 seconds, shared by every open composer and reader of the same account. A successful save or discard clears the cache. If the server refuses a list request, the client waits at least 30 seconds, or longer if the server sends `Retry-After`, before asking again.

## Signatures

Signatures are kept per account under **Settings** > **Account** > **Signatures**. Choose **New signature** to create one.

<!-- SCREENSHOT deploy/signature-editor: Settings > Account > Signatures, editing a signature in Rich text format with the Plain-text alternative field visible, full -->

The signature editor uses the same editor and ribbon as the composer, without recipients, attachments or delivery options:

1. Enter a **Name** (required, unique, at most 200 bytes).
2. Choose the **Signature format**: **Plain text**, **Rich text** or **HTML**.
3. Write the signature. In **HTML**, you can edit the source directly; it is cleaned before it is shown or stored.
4. For **Rich text** and **HTML**, check the **Plain-text alternative**. It follows your rich edits until you change it yourself; after that, your text is kept.
5. Select **Use as default** if this is the account's default signature, then **Save**.

To assign a signature to a sending address, open **Settings** > **Account** > **Identities** and choose the **Default signature** for **New messages** and for **Replies**. See [Sending identities](./sending-identities.md).

### How signatures are inserted

- A **Rich text** or **HTML** message gets the signature's HTML. A **Plain text** message gets the plain-text alternative (or text derived from the HTML if the alternative is empty).
- The composer **Signature** group (on the **Insert** tab) can remove the signature, insert the default one, choose another, or set the **Signature position**: **Above the quoted message**, **Below the quoted message** or **End of message**. The default position is the **Signature position** setting in **Settings** > **Mail** > **Composing and replies** > **Composer features** > **Defaults for new messages**.
- An identity always uses the current version of its signature: edits apply to the next message, and deleting a signature removes it from the identities that used it.

### Images and limits

- Signatures can contain images: PNG, JPEG, GIF or WebP. They are stored inside the signature HTML and sent as inline images of each message.
- The signature HTML, images included, is limited to 2 MiB. The plain-text alternative is limited to 64 KiB.
- HTML is cleaned when you save it and again when an identity's signature is read: scripts, event handlers, remote image sources and any embedded data other than a PNG, JPEG, GIF or WebP image are removed.

## Message notices

Message notices are reusable blocks of text, such as a confidentiality notice, added after the signature and kept separate from it. Manage them in **Settings** > **Mail** > **Composing and replies** > **Message notices**.

<!-- SCREENSHOT deploy/message-notices-settings: Settings > Mail > Composing and replies > Message notices, one notice open with Applicability rules visible, full -->

1. Make sure **Enable notices** is on.
2. Choose **New notice**.
3. Enter the **Notice name** and the **Notice text**. The text is required: plain-text and encrypted messages use it. You can add an HTML version; it is cleaned before sending.
4. Leave **Apply automatically by default** selected to let the rules below add the notice, or clear it to offer the notice only when chosen by hand.
5. Set the **Applicability rules**, then **Save**.

| Rule | Matches when |
| --- | --- |
| **From identities (emails or IDs, separated by commas)** | The message is sent from one of these identities or addresses. |
| **New messages**, **Replies**, **Forwards** | The message is of a selected kind. |
| **Recipients** | **All recipients**; **Internal recipients only** (every recipient is in the sender's domain or in one of the **Trusted recipient domains**); or **Includes external recipients**. |
| **Recipient domains (separated by commas)** | At least one recipient is in one of these exact domains. |
| **Subject contains**, **Message body contains** | The subject or body contains this text, ignoring case. |

All configured rules must match; an empty field matches any value. You can keep up to 20 notices. A notice name is at most 80 characters, the text at most 8000, the HTML version at most 32000.

**Use administrator default** discards your own notices and uses the ones your administrator set. When an administrator locks the notice settings ([`mail.notices.enabled`](../admin/configuration.md#mail.notices.enabled), [`mail.notices.templates`](../admin/configuration.md#mail.notices.templates)), the list is read-only.

### Notices in the composer

The **Message notices** control (on the **Insert** tab, or below the body when the ribbon is not used) applies to the current message only and is kept in its draft:

- **Automatic**: the notices whose rules match.
- **No notices**: none.
- **Choose notices**: the notices you tick.

The composer lists the notices that will be added. When the message is sent:

- In **Rich text** and **HTML**, each notice is placed directly after the signature, before the quoted message. Without a signature it goes at the end.
- In **Plain text**, the notice text is appended at the end.
- In an encrypted or signed message, the notice text is part of the encrypted or signed content.

::: warning
Notices are added by the Mailwoman composer only. Messages sent from other mail clients through the same account do not get them.
:::

## Troubleshooting

| Symptom | Cause | Fix |
| --- | --- | --- |
| "This signature is over 2 MiB of HTML. Remove an image or some formatting." | The signature HTML, including embedded images, is over the limit. | Remove or shrink an image. |
| "{name} is not a PNG, JPEG, GIF or WebP image." | A signature image of another type was inserted. | Convert the image to one of those types. |
| "Your signed-in account changed. Reopen this signature in the correct account." | You signed in to a different account while the editor was open. The server refuses the save. | Sign in to the account that owns the signature and open it again. |
| The composer status shows **No local recovery**. | The composer has no local storage for this session's account, so no recovery copy is kept. | Keep the composer open until you send, or use **Save to Drafts**. |
| **Save** stays unavailable in **Message notices**. | A notice has no name or text, or a field is over its limit. | Fix the notice; **Save** becomes available when every notice is valid. |

## Related

- [Account drafts](./account-drafts.md)
- [Sending identities](./sending-identities.md)
- [Composer ribbon](./composer-ribbon.md)
- [Configuration](../admin/configuration.md)
