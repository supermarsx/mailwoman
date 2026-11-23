---
title: Composer features
screen: true
---

# Composer features

**Composer features** lets you switch individual controls of the message composer on or off for everyone, or for one domain, and lock the choice so that users cannot change it. Every control that has a switch is one of about sixty features in four groups: **Format**, **Insert**, **Layout** and **Options**. A feature that is off is not drawn in the composer.

The card is part of the console's **Settings** screen. Users see a matching card, also called **Composer features**, in their own settings, where it shows "Set by your administrator" for anything you lock.

![The Settings screen searched for compose, with the Composer features card at the top of the list.](/screenshots/admin-ops/composer-features-card.webp){.mw-shot}

## Before you start

- You need a console role that can read and change settings. Without the change capability the card is read-only.
- A feature is stored as a setting with the key `compose.feature.<id>` ([`compose.feature.*`](/admin/settings#compose.feature.*)). Each is a yes or no value that is on unless set otherwise, and it can be set globally, for a domain or by the user. The card writes the global and domain values.

## Switch features on or off

1. Open **Settings** in the **Configuration** group.
2. Type `compose` in **Search settings**. The **Composer features** card appears above the list. It also appears when you filter by the **Interface** category.
3. Under **Applies to**, choose **Everyone (deployment default)** or **One domain**. For one domain, type the domain name in **Domain**. Everything below stays disabled until the domain name is valid.
4. Tick or untick **Value** for a feature to turn it on or off.
5. Tick **Lock for users** to stop users from changing the feature. The lock is stored with the value.
6. Choose **Use default** to remove your value and lock again, so the feature falls back to the next level.

Each change is saved as you make it, and the card says "Saved."

![The top of the Composer features card with markers 1 and 2 on the Applies to and Profiles selectors, marker 3 on Apply profile, and markers 4 to 6 on the Value checkbox, the Lock for users checkbox and the Use default button of the first feature.](/screenshots/admin-ops/composer-features-controls.webp){.mw-crop}

1. **Applies to** chooses everyone or one domain.
2. **Profiles** chooses a set of features, see [Profiles](#profiles).
3. **Apply profile** writes that set to every feature.
4. **Value** switches the feature.
5. **Lock for users** stops users from changing it.
6. **Use default** removes the value and the lock at the chosen level.
{.mw-callouts}

## Profiles

A profile sets every feature in one step, as the value at the level you chose in **Applies to**. It keeps each feature's lock as it is. Choose it in **Profiles** and then **Apply profile**.

| Profile | Result |
| --- | --- |
| **Everything** | Every feature on |
| **Simple (plain formatting only)** | Every feature on except those marked "no" in the **Simple** column below |
| **Locked-down (no inserts, no rich features)** | Only the **Format** features marked "yes" in the **Locked-down** column below. Every **Insert**, **Layout** and **Options** feature is off |

## Options

Each row is one feature. All of them default to on, can be set globally, per domain and by users, and can be locked.

### Format

| Label | Setting key | What it controls | On in **Simple** | On in **Locked-down** |
| --- | --- | --- | --- | --- |
| Font size steps | `compose.feature.format.fontSizeStep` | Buttons and shortcuts that grow or shrink the font size along the standard sizes. | yes | yes |
| Bulleted and numbered lists | `compose.feature.format.lists` | The two list buttons. | yes | yes |
| List types | `compose.feature.format.listTypes` | The dropdown on each list button: bullet shapes, numbering styles, checklists and numbering controls. | no | no |
| Indent | `compose.feature.format.indent` | Increase and decrease indent, and Tab and Shift+Tab in lists. | yes | yes |
| Clear formatting | `compose.feature.format.clearFormatting` | The clear formatting button, its options and Ctrl+Space. | yes | yes |
| Format painter | `compose.feature.format.painter` | Copy formatting and apply formatting. | no | no |
| Line spacing | `compose.feature.format.lineSpacing` | Line spacing and the space before and after paragraphs. | no | no |
| Text direction | `compose.feature.format.direction` | Left-to-right and right-to-left buttons. | no | no |
| Paste options | `compose.feature.format.pasteOptions` | Keep formatting, merge formatting or paste text only. | no | no |
| Strikethrough | `compose.feature.format.strikethrough` | The strikethrough button. | yes | yes |
| Double strikethrough | `compose.feature.format.doubleStrikethrough` | Double strikethrough in the text effects menu. Some mail apps show a single line. | no | no |
| Superscript | `compose.feature.format.superscript` | Superscript and Ctrl+Shift+=. | no | no |
| Subscript | `compose.feature.format.subscript` | Subscript and Ctrl+=. | no | no |
| Small caps | `compose.feature.format.smallCaps` | Small caps in the text effects menu. | no | no |
| All caps | `compose.feature.format.allCaps` | All caps as a style, without changing the typed text. | no | no |
| Underline styles | `compose.feature.format.underlineStyle` | Double, dotted, dashed, wavy and thick underlines. | no | no |
| Ligatures | `compose.feature.format.ligatures` | Turn ligatures off for the selected text. Many mail apps ignore this. | no | no |
| Text scale and spacing | `compose.feature.format.textScale` | Character width scale and condensed or expanded spacing. | no | no |
| Text outline and shadow | `compose.feature.format.textOutline` | Outline and shadow built from text shadows. Client dependent. | no | no |
| Styles gallery | `compose.feature.format.stylesRibbon` | The Styles group with headings, title, quote and code styles. | yes | yes |
| Sort selection | `compose.feature.format.sort` | Sort lines, list items or table rows. | no | no |
| Custom styles | `compose.feature.format.customStyles` | Create, update and keep your own styles in the Styles gallery. | no | no |
| Font dialog | `compose.feature.format.fontDialog` | The Font dialog and Ctrl+D. | no | no |

### Insert

| Label | Setting key | What it controls | On in **Simple** | On in **Locked-down** |
| --- | --- | --- | --- | --- |
| Attach file | `compose.feature.insert.attachFile` | Attach a file from this device. | yes | no |
| Signatures | `compose.feature.insert.signature` | Signatures | yes | no |
| Manage signatures | `compose.feature.insert.signatureEdit` | Manage signatures | yes | no |
| Attach from messages | `compose.feature.insert.attachFromMessages` | Pick attachments from other messages. | yes | no |
| Picture from file | `compose.feature.insert.pictureFile` | Insert pictures from files, with size presets. | yes | no |
| Picture from messages | `compose.feature.insert.pictureFromMessages` | Pick pictures from the attachments of other messages. | no | no |
| Shapes | `compose.feature.insert.shapes` | Insert drawn shapes as pictures. | no | no |
| Screenshot | `compose.feature.insert.screenshot` | Capture the screen or a window as a picture. | no | no |
| Picture from web address | `compose.feature.insert.onlinePicture` | Fetch a picture from a web address through the image proxy. | no | no |
| Business card | `compose.feature.insert.businessCard` | Insert a business card and its vCard. | no | no |
| Calendar blocks | `compose.feature.insert.calendar` | Insert availability, an agenda or a month snapshot. | no | no |
| Event | `compose.feature.insert.event` | Insert an event with an iCalendar invitation. | no | no |
| Symbols | `compose.feature.insert.symbols` | Special characters and emoji. | yes | no |
| Horizontal line | `compose.feature.insert.horizontalLine` | Insert a dividing line. | yes | no |
| Date and time | `compose.feature.insert.dateStamp` | Insert the current date and time. | yes | no |
| Link dialog | `compose.feature.insert.link` | Links with their own text, tooltip and safety check (Ctrl+K). | yes | no |
| Drop to attach | `compose.feature.insert.dragDropAttach` | Drop files on the composer to attach them. | yes | no |
| Drop images inline | `compose.feature.insert.dragDropInline` | Drop images into the body to place them inline. | no | no |
| Drop messages | `compose.feature.insert.dropMessages` | Drop a message from the list to attach it as .eml. | no | no |

### Layout

| Label | Setting key | What it controls | On in **Simple** | On in **Locked-down** |
| --- | --- | --- | --- | --- |
| Columns | `compose.feature.layout.columns` | Write in two or three columns using mail-safe tables. | no | no |
| Content width | `compose.feature.layout.contentWidth` | Limit the message to a fixed width and align it left or centred. | no | no |
| Page and content colour | `compose.feature.layout.pageColor` | Colour the page behind the message and the content card. These colours are sent. | no | no |
| Message themes | `compose.feature.layout.themes` | Apply a stationery theme with fonts, colours and spacing. | no | no |
| Default message font | `compose.feature.layout.defaultFont` | Set the font family, size and colour used for new text. | yes | no |
| Spacing | `compose.feature.layout.spacing` | Cell padding, column gutter and paragraph spacing presets. | no | no |
| Collapsible message header | `compose.feature.layout.headerCollapse` | The corner button, Alt+H and the folded summary line for To, Cc, Bcc and Subject. | yes | no |
| Ribbon auto-hide button | `compose.feature.layout.ribbonAutoHide` | The pin button in the ribbon's corner, Ctrl+F1 and double-click on a tab. | yes | no |

### Options

| Label | Setting key | What it controls | On in **Simple** | On in **Locked-down** |
| --- | --- | --- | --- | --- |
| Signature position | `compose.feature.options.signaturePosition` | Signature position | yes | no |
| Save sent item to a folder | `compose.feature.options.sentFolder` | Choose the folder that receives the copy of a sent message, or skip the copy. | no | no |
| Reply-To address | `compose.feature.options.replyTo` | Add a Reply-To header so replies go to a different address. | no | no |
| Reply-By | `compose.feature.options.replyBy` | Ask recipients to answer by a date (Reply-By header). | yes | no |
| Return receipt (legacy) | `compose.feature.options.returnReceipt` | The old Return-Receipt-To header, off unless an administrator allows it. | no | no |
| Importance | `compose.feature.options.importance` | Mark a message as high or low importance. | yes | no |
| Sensitivity | `compose.feature.options.sensitivity` | Mark a message as personal, private or company confidential. | no | no |
| Expiry date | `compose.feature.options.expires` | Say after which date the message is no longer relevant (Expires header). | no | no |
| Auto-Submitted | `compose.feature.options.autoSubmitted` | Mark a message as automatically generated (administrator gated). | no | no |
| Conversation headers | `compose.feature.options.inReplyTo` | Show In-Reply-To and References, and start a new thread. | no | no |
| Replaces an earlier message | `compose.feature.options.supersedes` | Name the message this one replaces (Supersedes header). | no | no |
| Custom headers | `compose.feature.options.customHeaders` | Add your own headers (administrator gated). | no | no |


## What the composer writes

The message that leaves is plain HTML: inline `style` attributes and a few legacy tags, no classes and no editor markers. Some of what the controls write:

| Control | Written as |
| --- | --- |
| Bullet shapes disc, circle, square | `list-style-type` on the list |
| Numbering 1. a. A. i. I. | `list-style-type` and the `type` attribute |
| Dash, arrow, tick, checklist, `1)`, `01.`, `1.1.1` | `list-style-type: none` and the marker as text in front of each item, because these have no dependable style in mail programs |
| Indent | `margin-left` in steps of 40 px, or a nested list |
| Text direction | the `dir` attribute on the paragraph |
| Line spacing and space before and after | `line-height`, `margin-top`, `margin-bottom` |

The text effects are written as follows. The menu in the composer shows the same two support levels.

| Effect | Written as | Support |
| --- | --- | --- |
| Superscript, subscript | `<sup>`, `<sub>` | most mail programs |
| All caps | `text-transform: uppercase`, the typed text is unchanged | most mail programs |
| Condensed or expanded spacing | `letter-spacing: -0.05em` or `0.1em` | most mail programs |
| Small caps | `font-variant: small-caps` | some programs show plain text |
| Double strikethrough | `text-decoration: line-through` with `text-decoration-style: double` | some programs show a single line |
| Underline style (double, dotted, dashed, wavy) | `text-decoration: underline` with `text-decoration-style`. Thick uses `text-decoration-thickness: 2px` | some programs show a single underline |
| Ligatures off | `font-variant: no-common-ligatures` | some programs ignore it |
| Text scale 50 to 200% | `font-stretch` and a matching `letter-spacing` | some programs ignore it |
| Outline | four 1 px `text-shadow` copies in the chosen colour | some programs show plain text |
| Shadow | `text-shadow: 1px 1px 2px rgba(0, 0, 0, 0.4)` | some programs show plain text |

## Styles

Three more settings belong to the **Styles** group of the **Format text** tab.

| Label | What it does | Default | Who can change it |
| --- | --- | --- | --- |
| **Style set** ([`compose.styles.defaultSet`](/admin/settings#compose.styles.defaultset)) | The style set the gallery starts with: `default`, `modern`, `classic` or `compact` | `default` | Administrators globally and per domain, and users, unless locked |
| **Custom styles per user** ([`compose.styles.maxCustom`](/admin/settings#compose.styles.maxcustom)) | The most custom styles a user keeps, 0 to 100. 0 turns **Custom styles** off | 20 | Same |
| [`compose.styles.custom`](/admin/settings#compose.styles.custom) | A user's own styles as JSON, written by the gallery | empty | Users, through the gallery |

The **Font dialog** writes its **Set as default** choice to **Default message font** ([`compose.layout.defaultFont`](/admin/settings#compose.layout.defaultfont)), which is the same setting the **Layout** tab uses.

## Troubleshooting

- **The Composer features card does not appear.** It is shown when the **Interface** category is selected, or when the search text contains `compos`, and only after the settings list has loaded. Search for `compose`.
- **The checkboxes are disabled.** Either your role cannot change settings, or **Applies to** is **One domain** and the domain name is not valid yet. Type the full domain name, for example `example.com`.
- **A user cannot switch a feature.** You locked it. Choose **Use default** to remove the lock.

## Related

- [Composer ribbon](/deploy/composer-ribbon) for the layout of the ribbon
- [Fonts](/deploy/fonts) for the fonts the editor offers
- [Settings](/admin/settings) for every setting key
- [Outbox, recall and scheduled sending](/admin/outbox-recall) for the delivery side of the composer
