---
title: Composer ribbon
screen: true
---

# Composer ribbon

The composer has a row of buttons for sending and saving, and a ribbon of formatting controls below the recipient fields. This page describes both, and the settings that decide whether the ribbon is always shown or hides itself.

![A new message in the composer: the Send button and the row of actions at the top, the To and Subject fields, the editor ribbon open on the Format text tab, and an empty message body.](/screenshots/admin-ops/compose-ribbon.webp){.mw-shot}

## Before you start

- The controls in the ribbon are switched with the [composer features](/admin/composer-features). A control that an administrator turned off is not shown.
- Sending, delayed sending and recall are described in [Outbox, recall and scheduled sending](/admin/outbox-recall).

## The row of actions

The row above the recipient fields holds the buttons that act on the whole message.

![The top row of the composer with markers 1 to 6 on Send, the arrow beside Send, Discard, Save to Drafts, Drafts and Recall.](/screenshots/admin-ops/compose-ribbon-actions.webp){.mw-crop}

1. **Send** is at the top left. It sends the message, or queues it for the undo period, see [Outbox, recall and scheduled sending](/admin/outbox-recall).
2. The arrow beside **Send** opens **Send later options**.
3. **Discard** asks "Discard this draft?" with **Keep editing** as the safe choice. Closing the composer instead keeps an account draft.
4. **Save to Drafts** saves the message to the account's Drafts folder now, see [Account drafts](/deploy/account-drafts).
5. **Drafts** opens the list of account drafts and local drafts.
6. **Recall** lists messages that are still held for undo or scheduled, and lets the sender cancel them. Messages that were already delivered cannot be recalled.
{.mw-callouts}

Below the row, **Message header** folds and unfolds the recipient and subject fields.

## The ribbon tabs

The ribbon has six tabs: **Format text**, **Insert**, **Layout**, **Table layout**, **Security** and **Delivery**. **Format text** is open when a message starts. Its groups are **HTML**, **Clipboard**, **Format Painter**, **Font**, **Paragraph**, **Styles**, **Editing**, **View**, **Recipients** and **Message type**.

- **Undo** and **Redo** each have a small arrow beside them that opens the **Undo history**. It lists up to 20 steps, so you can go back or forward several at once.
- **Visual editor** and **HTML source** in the **HTML** group switch between editing the message and editing its HTML.
- **Message type** chooses **Rich text**, **HTML** or **Plain text**.

### Dark background

The **View** group has a **Dark background** button and an **Editor zoom** selector.

![The View group of the ribbon opened, with the Dark background button and the Editor zoom selector set to 100%.](/screenshots/admin-ops/compose-view-group.webp){.mw-crop}

**Dark background** changes only what the writer sees. The writing area switches between a dark and a white background, and text whose own colour would be hard to read there is drawn in a readable shade. Neither the background nor those shades are saved in the message, so the message that is sent, saved as a draft or saved as a signature is the same with the button on or off. The message is sent without a background colour. The choice is saved with the user's other appearance settings. The signature editor has the same button.

::: tip
Page and content colours chosen on the **Layout** tab are different: they are part of the message and are sent.
:::

## Show or hide the ribbon

The pin button at the right end of the tab row, also reachable with Ctrl+F1, switches between keeping the ribbon open and showing only the tab names until you click one.

On screens narrower than 600 px the ribbon starts hidden, so the message keeps most of the height. That is only the starting state: once the user pins or unpins the ribbon, or an administrator sets or locks **Editor ribbon**, that value applies at every width.

## Options

| Label | What it does | Default | Who can change it |
| --- | --- | --- | --- |
| **Editor ribbon** ([`ui.bar.composerRibbon`](/admin/settings#ui.bar.composerribbon)) | `pinned` keeps the ribbon open, `auto` shows only the tab names | `pinned` | Administrators globally and per domain, and users, unless locked |
| **Ribbon auto-hide button** ([`compose.feature.layout.ribbonAutoHide`](/admin/settings#compose.feature)) | Shows the pin button, and enables Ctrl+F1 and a double-click on a tab | on | Administrators globally and per domain, and users, unless locked |

## Troubleshooting

- **The ribbon is only tab names.** The width is under 600 px and nobody has pinned it, or **Editor ribbon** is `auto`. Click the pin button or set **Editor ribbon** to `pinned`.
- **A button is missing from the ribbon.** The matching [composer feature](/admin/composer-features) is off, or locked off.
- **The editor has a dark background but the recipient sees white.** This is intended, **Dark background** is not saved in the message.

## Related

- [Composer features](/admin/composer-features)
- [Account drafts](/deploy/account-drafts)
- [Outbox, recall and scheduled sending](/admin/outbox-recall)
- [Fonts](/deploy/fonts)
