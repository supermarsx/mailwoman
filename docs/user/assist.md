---
title: AI assistant
---

# AI assistant

The AI assistant is an optional set of writing and mail helpers that send text to an AI
service your administrator chose. It is off unless your administrator turns it on, and
what it may do and see is set by them. This page explains what you will see and what
leaves your device. The administrator side is described in [AI assistant: data flows and controls](/assist).

::: warning Not shown here
The AI assistant is not available in the demonstration setup used for the screenshots in
this manual, so this page has no screenshots. The labels are the ones in the application.
:::

## Before you start

Nothing appears until your administrator has set up an AI service and granted the
assistant its capabilities. If the assistant is off, there is nothing to turn on in your
settings, and no message content leaves your device. Your administrator can also remove
the assistant for your account, or require your consent first.

## What can leave your device

When the assistant is on, a **What can leave this device** note is shown with the
assistant. It says, in the words of the application:

- the selected message text (subject and body) for the tool you chose;
- the name of the AI service's host it was sent to;
- never: end-to-end-encrypted content, attachments, your credentials or other accounts,
  unless your administrator has allowed encrypted content and attachments to be sent.

The note also says "Send is never automated — you always confirm before anything leaves
your Outbox."

::: warning Automatic label suggestions
If your administrator has granted automatic labels, the note adds: "Auto-tag is on: each
time you open a message, its subject and preview are sent to" the AI service "to suggest
labels, without you selecting anything." If that is not acceptable, ask your
administrator to turn the capability off.
:::

## Ask the assistant

When the **assistant** capability is on, a panel titled **AI assistant** appears in the
mailbox. Type in **Ask the AI assistant…** and choose **Ask**. The reply appears in the
conversation. While a message is open, its subject and a preview are sent along with
your question, so you can ask about that message.

The assistant can propose an action. A proposal is shown under **Proposed action**. If it
would create a message, it says "This would place a message in your Outbox. Nothing is
sent until you confirm it there." and the button is **Review in Outbox**. Choosing it opens
the message for you to check; the assistant never sends mail itself.

If the service cannot answer, the panel says "The AI assistant could not respond just
now."

## Writing help in the message editor

In the message editor, a **Writing assistant** group appears when your administrator has
granted writing tools. Its menu offers:

- **Fix grammar** corrects spelling and grammar.
- **Rewrite** makes the message clearer.
- **Adjust tone** rewrites it in a **Tone**: **Neutral**, **Friendly**, **Formal**,
  **Concise** or **Assertive**.
- **Translate** translates it into the **Language** you pick.

Write something first; with an empty message the tool says "Write something first." The
whole text of your draft is sent to the AI service. The result appears as a **Suggested
edit**. Choose **Apply to draft** to replace your text, or **Discard** to keep yours.

If a draft may contain text from an encrypted message, for example a reply that quotes
it, the tools do not send it and say "Not sent to the AI assistant: this draft may contain
text from an encrypted message, and decrypted content is not allowed to be sent."

If dictation is granted, **Hold to dictate** records while you hold the button. It uses
your browser's on-device speech recognition when there is one; otherwise "Audio is
transcribed by your AI assistant endpoint."

## Suggested labels

When automatic labels are granted, **Suggested labels** appear for an opened message,
each with a confidence percentage and an **Apply** button. After applying one, **Undo**
removes it. Tick **Apply automatically** to have suggested labels applied for you.

## Troubleshooting

- **The assistant is missing.** Your administrator has not turned it on for your account.
- **"The AI assistant could not complete that just now."** The AI service did not answer,
  or a limit was reached. Try again later.

## Related

- [AI assistant: data flows and controls](/assist)
- [Encryption keys](/user/encryption-keys)
- [Composing messages](/user/mailbox/composing)
