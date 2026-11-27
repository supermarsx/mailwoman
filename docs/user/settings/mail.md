---
title: Mail settings
screen: true
---

# Mail settings

The **Mail** category holds the settings that decide how messages are shown, written, filed and kept. It has nine sections.

![The Settings screen on Message display, with markers 1 to 9 on the nine sections of the Mail category in the section list.](/screenshots/user-settings/mail-overview.webp){.mw-shot}

1. **Message display**: text, actions, attachments, badges, pictures and the mail cache.
2. **Composing and replies**: addressing, checks before sending, images in replies, composer features, sending and templates.
3. **Folders and sidebar**: the folder list, counters, special folders, Focused inbox and Properties.
4. **Retention**: rules that move or delete your own mail after a number of days.
5. **Labels**: the labels you can put on messages.
6. **Mail rules**: filtering rules, Sieve scripts and forwarding.
7. **Saved searches**: search folders and the default search scope.
8. **Read receipts**: how requests for a read receipt are answered, and delivery notifications.
9. **Import and export mail**: bringing mail in, and where export lives.
{.mw-callouts}

Every setting in the tables on this page can be locked by an administrator, except the rows that name no setting key. A locked control is dimmed and shows **Set by your administrator**. The reset button beside a setting (**Use administrator default**) removes your own choice. See [Personal settings](./index.md) for how this works.

## Message display

Message display is split into eight tabs. Administrator defaults apply until you choose an override, and message content stays isolated from the application.

### Text and colors

![The Text and colors card with message colors, font size, line height, reading width and zoom, and below it the Loading and Message header cards.](/screenshots/user-settings/mail-reader-text.webp){.mw-crop}

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Message colors** | **Automatic with theme** adapts message colors in dark themes. **Original message colors** keeps the sender's colors. **Adapt to theme** uses the active theme. **Invert message colors** reverses colors while preserving image colors. | Automatic with theme | [`reader.colorMode`](/admin/settings#reader.colorMode) |
| **Reading font size (px)** | Base font size for message text, 12 to 24. Sender formatting can still set heading sizes. | 14 | [`reader.fontSize`](/admin/settings#reader.fontSize) |
| **Reading line height** | Space between lines, 1.2 to 2.2 times the font size. | 1.6 | [`reader.lineHeight`](/admin/settings#reader.lineHeight) |
| **Reading width** | **Comfortable** and **Wide** limit long lines; **Full pane** uses the whole message pane. | Full pane | [`reader.contentWidth`](/admin/settings#reader.contentWidth) |
| **Message zoom (%)** | The size messages open at, 50% to 200%. Only the message body scales, and printing always uses 100%. The zoom buttons beside the message actions change it too. | 100% | [`reader.zoom`](/admin/settings#reader.zoom) |
| **Loading appearance** | What you see while a message opens: **Placeholder lines**, **Text indicator** or **No visual indicator**. Screen readers still receive loading status. | Placeholder lines | [`reader.loading`](/admin/settings#reader.loading) |
| **Loading indicator delay (ms)** | Delays only the indicator, never the message, so fast requests do not flash it. 0 to 1000. | 120 | [`reader.loadingDelayMs`](/admin/settings#reader.loadingDelayMs) |
| **Automatically hide message headers** | Opens each message with the sender, recipient and metadata details collapsed. The subject and actions stay visible; **Show header** reveals the details. | Off | [`reader.autoHideHeader`](/admin/settings#reader.autoHideHeader) |

### Message reader actions

![The Actions and attachments card: collapse attachments by default, the message action ribbon style and the attachment density.](/screenshots/user-settings/mail-reader-actions.webp){.mw-crop}

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Collapse attachments by default** | Starts each message with a compact attachment summary. You can expand a single message without changing the default. | On | [`reader.attachmentsCollapsed`](/admin/settings#reader.attachmentsCollapsed) |
| **Message action ribbon** | **Compact icons** or **Icons and labels**. Every action keeps its keyboard name and tooltip. | Compact icons | [`reader.actionStyle`](/admin/settings#reader.actionStyle) |
| **Attachment density** | **Compact** shows small file buttons; **Comfortable** shows larger previews when expanded. | Compact | [`reader.attachmentsDensity`](/admin/settings#reader.attachmentsDensity) |

![The top of the Message reader actions card: a search box, a preview of the actions ribbon and the first rows that place each action on the toolbar, in the More menu or hide it.](/screenshots/user-settings/mail-reader-placement.webp){.mw-crop}

The **Message reader actions** card chooses where each action appears: **Toolbar**, **More menu** or **Hidden**. Toolbar actions move into More when space is short, and features, permissions and the message itself still decide which actions are available. Use **Find a ribbon action** to filter the list; the preview above it shows your ribbon. Each action has its own setting named `reader.actions.<action>`, for example [`reader.actions.reply`](/admin/settings#reader.actions.reply).

| Where | Actions at the default |
|---|---|
| Toolbar | Reply, Reply all, Forward, Edit draft, View reply draft, Open reply draft, Delete, Archive, Move, View conversation, Review links, Read / unread, Flag / unflag, Pin / unpin, Mark as spam, Security mode, Raw headers, Sweep sender, Print, Export, Message properties, Zoom controls, Related search, Color controls |
| More menu | Mark follow-up finished, Save sender, Save message contacts, Move to Focused, Move to Other, Not spam, View message source, Create filter, Create task from message, Block sender |
| Hidden | Delete all messages from this sender |

### Flag for follow-up and Snooze

![The Flag for follow-up card: a list of reminder presets, each with a checkbox, a name, the next reminder time and buttons to reorder and remove it.](/screenshots/user-settings/mail-reader-followup.webp){.mw-crop}

**Flag for follow-up** and **Snooze** each have a list of presets for the times they offer. Follow-up flags keep a message visible; snoozing hides it until the chosen time. A custom date and time is always available in the snooze menu. Tick a preset to offer it, move it with the arrows, remove it with the bin button, or select **Add preset** for your own. **Restore built-in presets** puts the built-in list back, and **Save** stores your changes. The list shows the time zone used to work out each time and, in the **Next reminder time** column, when each preset would fire now.

Only **Later today** (3 hours), **Tomorrow** (9:00) and **Next week** (9:00) are on by default; the other built-in presets are available to switch on. The settings are [`mail.followUp.presets`](/admin/settings#mail.followUp.presets) and [`mail.snooze.presets`](/admin/settings#mail.snooze.presets).

### Attachments bar

![The Attachments bar card and the Attachment previews card with their switches and choices.](/screenshots/user-settings/mail-reader-attachments.webp){.mw-crop}

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Click the bar to expand or collapse** | Clicking anywhere on the bar toggles it. Off means only the arrow button does. | On | [`attachments.bar.clickToToggle`](/admin/settings#attachments.bar.clickToToggle) |
| **Download buttons on attachments** | **Always**, **On hover or focus** or **Never**. | On hover or focus | [`attachments.bar.showDownloadButtons`](/admin/settings#attachments.bar.showDownloadButtons) |
| **Download all button** | Shows a button that downloads the attachments as one ZIP file. | On | [`attachments.bar.downloadAll`](/admin/settings#attachments.bar.downloadAll) |
| **Include inline images in the ZIP** | Images inside the message text are left out of Download all unless this is on. | Off | [`attachments.bar.zipInlineImages`](/admin/settings#attachments.bar.zipInlineImages) |
| **Preview action buttons** | Shows Download and File details as **Icons**, **Labels** or **Icons and labels**. | Icons | [`reader.attachmentActionStyle`](/admin/settings#reader.attachmentActionStyle) |
| **Show image attachment thumbnails** | Visible image attachments load small previews instead of file icons. Attachment security rules still apply. | Off | [`reader.attachmentThumbnails`](/admin/settings#reader.attachmentThumbnails) |
| **Attachment security warnings** | Warns about files that may run code or disguise their type. Five kinds can be switched separately: programs, macro documents, disguised file names, a file signature that does not match its declared type, and formats that can contain active content. An administrator can require them. | On, all five kinds | [`mail.attachmentWarnings`](/admin/settings#mail.attachmentWarnings) |

**File previews** chooses which file types can open in the app and which tools appear, up to a size limit (64 MiB by default); select **Save preview settings** to store them. Administrator attachment restrictions and download permissions still apply. The setting is [`reader.filePreviews`](/admin/settings#reader.filePreviews).

### Message badges

![The top of the Message badges card: each badge can be set to Always, Only when notable or Never.](/screenshots/user-settings/mail-reader-badges.webp){.mw-crop}

Badges appear in the header of an open message. Priority marks are set under [Folders and sidebar](#folders-and-sidebar). You can also right-click a badge to hide it. Each badge is set to **Always**, **Only when notable** or **Never**; notable highlights unusual values, active reminders, receipts, delivery events or approaching deadlines.

| Default | Badges |
|---|---|
| Always | Authentication (DMARC), Spam score, Folder, Mailing list, Read receipts, Delivery status, Message dates, Sensitivity, Reply deadline, Expiry, Replaced messages |
| Only when notable | Sanitized HTML (shown when something was blocked), Importance, Follow-up, Snooze |
| Never | Authentication on sent messages, Spam scores on sent messages |

**Folder** with **Only when notable** shows only for Junk and Trash. **Badge size** is **Compact** or **Comfortable** (default Compact), and **Badge labels** is **Icon and text** or **Icon only** (default Icon and text; the full text stays in the tooltip). The settings are named `mail.badge.<badge>`, for example [`mail.badge.auth`](/admin/settings#mail.badge.auth), plus [`mail.badge.density`](/admin/settings#mail.badge.density) and [`mail.badge.labels`](/admin/settings#mail.badge.labels).

### People and contact pictures

![The People and contact pictures card and the Save contacts from messages card.](/screenshots/user-settings/mail-reader-people.webp){.mw-crop}

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Sender and contact pictures** | **Hide pictures**, **Generate initials locally**, or **Use local contact pictures, with initials as fallback**. Contact pictures use only PNG or JPEG photos already saved in your address books, up to 512 by 512 pixels and 128 KiB. Remote picture URLs and external avatar services are never requested. | Generate initials locally | [`contacts.avatarMode`](/admin/settings#contacts.avatarMode) |
| **Show bundled sender-domain icons** | Recognised domains use icons bundled with the app. Your own contact pictures take priority. Icons are hints, not proof that a message is authentic. | On | [`contacts.senderIcons`](/admin/settings#contacts.senderIcons) |
| **Show brand logos beside verified senders** | A recognised company's logo replaces the initials beside the sender in the reader. Logos are bundled; nothing is requested from the sender. How strict the check is comes from the next row. The help text printed under this row describes the strictest choice, **Always**. | On | [`mail.brandAvatars`](/admin/settings#mail.brandAvatars) |
| **Authentication needed for brand logos** | **Never (match on domain only)** shows the logo whenever the sender's domain matches. **Only if a check failed** shows it unless the message failed DKIM or DMARC. **Always (require a passed check)** shows it only when the message passed DKIM and an aligned DMARC check for that company's own domain. | Only if a check failed | [`mail.brandAvatars.requireAuth`](/admin/settings#mail.brandAvatars.requireAuth) |
| **Show favorite contacts first in recipient suggestions** | Starred contacts are listed first when you type a recipient. Only the order changes. | On | [`contacts.favoritesFirst`](/admin/settings#contacts.favoritesFirst) |
| **Offer Save contacts from this message** | Adds the item to message menus and the reader More menu. | On | [`contacts.saveFromMessage.enabled`](/admin/settings#contacts.saveFromMessage.enabled) |
| **Offer Save sender as contact** | A one-click item in message menus. | On | [`contacts.saveSender.enabled`](/admin/settings#contacts.saveSender.enabled) |
| **Ticked first in the dialog** | Which people are ticked first: **Sender only**, **Sender and Reply-To**, or **Everyone on the message**. | Sender only | [`contacts.saveFromMessage.defaultScope`](/admin/settings#contacts.saveFromMessage.defaultScope) |
| **Skip automated senders** | Leaves out no-reply, notification and bounce addresses unless you tick them. | On | [`contacts.saveFromMessage.skipAutomated`](/admin/settings#contacts.saveFromMessage.skipAutomated) |
| **Add a note with the date** | The note reads "Saved from a message on" followed by the date. | On | [`contacts.saveFromMessage.addNote`](/admin/settings#contacts.saveFromMessage.addNote) |
| **Use Reply-To for list and notification senders** | Saves the Reply-To address instead of From for those senders. | Off | [`contacts.saveSender.useReplyTo`](/admin/settings#contacts.saveSender.useReplyTo) |
| **Address book** | The address book that saved contacts go to. | Default address book | [`contacts.saveFromMessage.defaultBook`](/admin/settings#contacts.saveFromMessage.defaultBook) |

### Cached mail

![The top of the Cached mail card: the session cache switches, age and size limits, and the Clear cached mail now button.](/screenshots/user-settings/mail-reader-cache.webp){.mw-crop}

Cached mail keeps recently visited folders and messages ready to open. By default, copies stay in the memory of the current tab. Preloading never marks mail as read and never loads remote images. **Clear cached mail now** empties the cache.

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Enable session cache** | Turns the cache on. | On | [`mail.cache.enabled`](/admin/settings#mail.cache.enabled) |
| **Refresh cached content when opening** | Refreshes a cached folder or message when you open it. | On | [`mail.cache.refreshOnOpen`](/admin/settings#mail.cache.refreshOnOpen) |
| **Cache folder message lists** | Caches message lists. | On | [`mail.cache.mailboxes`](/admin/settings#mail.cache.mailboxes) |
| **Cache opened messages** | Caches messages you have opened. | On | [`mail.cache.messages`](/admin/settings#mail.cache.messages) |
| **Keep cached mail indefinitely** | Keeps encrypted copies in this browser across reloads without age expiry. Online messages still refresh and the entry and byte limits still apply. Turning it off or clearing the cache removes saved copies. Attachments are not downloaded for offline use. | Off | [`mail.cache.keepIndefinitely`](/admin/settings#mail.cache.keepIndefinitely) |
| **Freshness interval (seconds)** | How long cached content counts as fresh, 0 to 3600. | 30 | [`mail.cache.freshSeconds`](/admin/settings#mail.cache.freshSeconds) |
| **Maximum cached age (seconds)** | Longest age of a cached entry, 1 to 86400. | 300 | [`mail.cache.maxAgeSeconds`](/admin/settings#mail.cache.maxAgeSeconds) |
| **Maximum cached entries** | 1 to 512. | 32 | [`mail.cache.maxEntries`](/admin/settings#mail.cache.maxEntries) |
| **Memory budget (bytes)** | 65536 to 268435456. | 16777216 | [`mail.cache.maxBytes`](/admin/settings#mail.cache.maxBytes) |
| **Folders excluded from preloading** | One rule per line: `role:trash`, `role:junk`, `role:inbox`, `role:drafts`, `role:sent`, `role:archive`, or `id:` followed by an exact folder ID. | `role:trash`, `role:junk` | [`mail.cache.excludedFolders`](/admin/settings#mail.cache.excludedFolders) |

The **Prefetching** card loads messages ahead of time, which uses more data. It needs the session cache. Body preloading is on by default: after the idle delay it warms a bounded set of visible messages, and pointer, keyboard focus and opening a message also warm likely next messages. Cached text appears immediately while the normal opening refresh runs. Administrators can set defaults or lock these controls for all users or a domain; users can adjust unlocked controls here.

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Enable speculative preloading** | Prepares likely next folders before you open them. | On | [`mail.cache.prefetchEnabled`](/admin/settings#mail.cache.prefetchEnabled) |
| **Preload message headers** | Fetches recipients, delivery addresses, dates and reply headers with message lists. Bodies and attachment files are not downloaded. | On | [`mail.cache.preloadHeaders`](/admin/settings#mail.cache.preloadHeaders) |
| **Preload when pointing at a folder** | Prepares a folder when the pointer rests on it for the delay below. | On | [`mail.cache.hoverEnabled`](/admin/settings#mail.cache.hoverEnabled) |
| **Preload when a folder receives keyboard focus** | Does the same for keyboard focus. | On | [`mail.cache.focusEnabled`](/admin/settings#mail.cache.focusEnabled) |
| **Pointer and focus delay (milliseconds)** | 0 to 10000. | 200 | [`mail.cache.dwellMs`](/admin/settings#mail.cache.dwellMs) |
| **Preload folders while idle** | Prepares nearby folders when you are not interacting with the app. | On | [`mail.cache.idleEnabled`](/admin/settings#mail.cache.idleEnabled) |
| **Idle delay (milliseconds)** | Wait after your last interaction, 100 to 60000. | 1500 | [`mail.cache.idleDelayMs`](/admin/settings#mail.cache.idleDelayMs) |
| **Maximum simultaneous preloads** | 1 to 4. Lower values use less bandwidth and server capacity. | 1 | [`mail.cache.maxConcurrency`](/admin/settings#mail.cache.maxConcurrency) |
| **Maximum queued preloads** | 1 to 64. Body preloads and pointer or focus hints take priority over idle folder preloads when the queue is full. Running jobs continue. | 8 | [`mail.cache.maxQueued`](/admin/settings#mail.cache.maxQueued) |
| **Allow speculative message bodies** | Preloads visible and nearby message bodies in the background for faster opening. Uses more data and cache space. | On | [`mail.cache.preloadMessages`](/admin/settings#mail.cache.preloadMessages) |

### Smart Archive

Smart Archive files a message where related mail has already been stored. It is **off by default**. Enable it in the **Smart Archive** card in Mail and reader settings. It appears as a separate quick action beside Archive. **Replace the main Archive action** is a separate switch, also off by default; it changes the toolbar, Archive shortcut and bulk Archive action. Ordinary Archive is always retained in context menus.

The context menu offers the latest message from the same sender, the latest message from the sender domain, the latest message in the conversation, and the folder most often used for that sender. Exact addresses and domain boundaries are checked; including subdomains is optional. Folder paths, matching-message counts and latest dates help identify the destinations.

By default, multiple matching folders ask you to choose. You can instead choose the most recent or most-used folder automatically. Confirmation previews can be always on, shown when needed, or off. Required folder choices still ask. Cancelling makes no changes. When history has no match, choose between the ordinary Archive folder, picking a folder, or skipping the message. Skipped messages stay where they are.

The history limit defaults to 100 recent matching messages, adjustable from 10 to 500. Only metadata is fetched. All default folders, including Archive, are always excluded from matching and folder suggestions. Edit additional exclusion rules as `role:name` or `id:folder-id`; current folders, search folders and non-writable folders are also excluded. Ordinary Archive remains available only as the separately configured fallback; adding `role:archive` also excludes that fallback. Selected messages are not used as evidence for their own destination. Bulk messages with the same matching history share a destination choice. Moves support Undo. No automatic filing rules are created.

Administrators can set and lock all `mail.smartArchive.*` defaults at server or domain scope. Users can change unlocked preferences. Existing quick-action and reader-action placement settings can move Smart Archive to More or hide it.

## Composing and replies

Composing and replies has seven tabs. How to write and send a message is described in [Composing a message](/user/mailbox/composing).

### Addressing and replies

![The Addressing and replies card: whether From is shown, which identity replies use, quick reply, recipient entry and the reply header style.](/screenshots/user-settings/mail-compose-addressing.webp){.mw-crop}

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Show From when composing** | Starts with the From field visible. You can also show it for one message from the ribbon. Only authorized sending addresses are accepted. | Off | [`mail.showFrom`](/admin/settings#mail.showFrom) |
| **Reply from** | Which identity a reply or forward uses: **The address the message was sent to**, **Always my default identity**, **Ask me when several identities match**, or **Keep the identity already selected**. If no identity matches, your default identity is used. Plus addresses such as `name+tag@example.org` match `name@example.org`. | The address the message was sent to | [`mail.replyFromMode`](/admin/settings#mail.replyFromMode) |
| **Headers that choose the identity** | One header name per line, tried in the order listed. The names you can use are Delivered-To, X-Original-To, Envelope-To, X-Forwarded-To, Resent-To, To and Cc; a header left out is not read. Save with **Save header order**. | Delivered-To, X-Original-To, To, Cc | [`mail.replyFrom.headerPriority`](/admin/settings#mail.replyFrom.headerPriority) |
| **Quick reply** | Offers a compact reply in the message reader with recipients already filled in. The normal safety checks and send options still apply. | Off | [`mail.quickReply`](/admin/settings#mail.quickReply) |
| **Finish a recipient when I type a space, comma or semicolon** | A valid address becomes a finished recipient and the separator is added. | On | [`mail.compose.autoChipRecipients`](/admin/settings#mail.compose.autoChipRecipients) |
| **Automatically add mentioned people as recipients** | Selecting a person from @mention suggestions adds them to To unless they are already in To, Cc or Bcc. | Off | [`mail.compose.autoAddMentionRecipients`](/admin/settings#mail.compose.autoAddMentionRecipients) |
| **Recipient separator** | **Comma** or **Semicolon**. | Comma | [`mail.compose.recipientSeparator`](/admin/settings#mail.compose.recipientSeparator) |
| **Message header (To, Cc, Subject)** | **Always show**, **Fold to a summary line**, or **Fold while I write the body**. A folded header shows one line such as To: Ana +2 and opens when you click it or a field needs attention. | Always show | [`mail.compose.headerCollapse`](/admin/settings#mail.compose.headerCollapse) |
| **Fold the header after writing (seconds)** | When the body has had focus for this long and the header has a recipient and a subject, it folds. 0 turns this off. | 0 | [`mail.compose.headerAutoCollapseSeconds`](/admin/settings#mail.compose.headerAutoCollapseSeconds) |
| **Reply and forward header style** | **Classic quoted reply** or **Outlook-style header**, which adds From, Sent, To, Cc and Subject above the original message. Existing drafts are unchanged. | Classic quoted reply | [`mail.quoteStyle`](/admin/settings#mail.quoteStyle) |

### Checks before sending

![The Checks before sending card: how to react to invalid recipients, a missing attachment and an empty subject, and the external recipient warning.](/screenshots/user-settings/mail-compose-checks.webp){.mw-crop}

For each check you choose what happens when it finds a problem: **Send without asking**, **Ask each time**, **Ask, and let me stop asking**, or **Block sending**. Only **Block sending** stops a message; the others always offer Send anyway.

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Invalid recipient addresses** | Checks the To, Cc and Bcc addresses. | Ask each time | [`mail.compose.checkInvalidRecipients`](/admin/settings#mail.compose.checkInvalidRecipients) |
| **Attachment mentioned but missing** | Checks a message that mentions an attachment but has none. | Ask each time | [`mail.compose.checkMissingAttachment`](/admin/settings#mail.compose.checkMissingAttachment) |
| **Empty subject** | Checks for a missing subject. | Ask each time | [`mail.compose.checkEmptySubject`](/admin/settings#mail.compose.checkEmptySubject) |
| **External recipient warnings** | Checks To, Cc and Bcc against the selected identity's domain and the trusted domain list. It protects sends from this composer; it is not a server delivery rule. Choices: **Off**, **Show a warning**, **Require confirmation before sending**, **Ask, and let me stop asking**, **Block sending**. | Show a warning | [`mail.externalRecipientWarning`](/admin/settings#mail.externalRecipientWarning) |
| **Trusted recipient domains** | Up to 100 exact domains, one per line. Subdomains must be listed separately; the sending identity's own domain is always trusted. URLs, email addresses and wildcards are not accepted. Save with **Save trusted domains**. | Empty | [`mail.trustedRecipientDomains`](/admin/settings#mail.trustedRecipientDomains) |

### Images in replies

![The Images in replies card: whether replies and forwards carry the original images, and the trusted sender domains.](/screenshots/user-settings/mail-compose-images.webp){.mw-crop}

**Images in replies and forwards** ([`mail.replyImages`](/admin/settings#mail.replyImages)) chooses whether a reply or forward carries the original message's images, which are copied into the draft as attachments: **Ask each time** (the default), **Always include**, **Include for trusted sender domains, ask otherwise**, or **Never include**. **Trusted image sender domains** ([`mail.replyImageTrustedDomains`](/admin/settings#mail.replyImageTrustedDomains)) holds up to 200 exact sender domains, one per line, used by the trusted-domains choice.

### Composer features

![The Defaults for new messages card: where the composer opens, receipts, record message opens, signature position and forwarding format.](/screenshots/user-settings/mail-compose-defaults.webp){.mw-crop}

Receipts are requests: the recipient or their mail server may decline or ignore them.

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Open new messages in** | **A workspace tab** or **A new window**. If your browser blocks a new window, the composer opens in a workspace tab. | A workspace tab | [`mail.composeWindow`](/admin/settings#mail.composeWindow) |
| **Always request read receipts** | New messages ask for a read receipt. | Off | [`mail.requestReadReceipt`](/admin/settings#mail.requestReadReceipt) |
| **Always request delivery receipts** | New messages ask for a delivery notification. | Off | [`mail.requestDeliveryReceipt`](/admin/settings#mail.requestDeliveryReceipt) |
| **Save a copy of replies in the original folder** | Offers the original folder for reply copies: when on, and no other folder is chosen for the message or by policy, the sent copy of a reply is filed in the folder of the message you answered. | Off | [`mail.replyCopyToOriginal`](/admin/settings#mail.replyCopyToOriginal) |
| **Record message opens by default** | Adds a tracking image to eligible formatted messages. Opens depend on the recipient loading images and do not prove the message was read. An administrator can disable tracking or lock this default. | Off | [`mail.openTracking`](/admin/settings#mail.openTracking) |
| **Signature position** | **After my text, before quoted messages**, or **End of the entire message**. New messages always put the signature at the end. | After my text, before quoted messages | [`mail.signaturePosition`](/admin/settings#mail.signaturePosition) |
| **Default forwarding format** | **Inline message text** or **Original message as an .eml attachment**. | Inline message text | [`mail.forwardMode`](/admin/settings#mail.forwardMode) |

![The top of the Composer features card: a profile choice with Apply profile, and the first group of features with a switch each.](/screenshots/user-settings/mail-compose-features.webp){.mw-crop}

**Composer features** chooses which composer controls are available. Features your administrator turned off or locked cannot be changed here. **Profiles** applies a set at once: **Everything**, **Simple (plain formatting only)** or **Locked-down (no inserts, no rich features)**; select **Apply profile** to use it, or **Reset to defaults**. You can change single features afterwards. Below it, four groups list one switch per feature, each group with an **All ... features** switch and a count such as 23 of 23 on:

- **Format** (23 features): for example font size steps, lists, indent, clear formatting, format painter, line spacing, text direction, paste options, text effects, the styles gallery and the Font dialog.
- **Insert** (19 features): for example attaching files, signatures, pictures from files or messages, screenshots, business cards, calendar blocks, events, symbols, links, and dropping files, images or messages into the composer.
- **Layout** (8 features): columns, content width, page and content colour, message themes, default message font, spacing, the collapsible message header and the ribbon auto-hide button.
- **Options** (12 features): signature position, where the sent copy is saved, Reply-To, Reply-By, return receipt (legacy), importance, sensitivity, expiry date, Auto-Submitted, conversation headers, Supersedes and custom headers. Return receipt, Auto-Submitted, conversation headers and custom headers also need your administrator's permission.

### Sending options

![The Sending options, Undo send and Account drafts cards.](/screenshots/user-settings/mail-compose-sending.webp){.mw-crop}

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Reply-To row** | When the composer shows the Reply-To row: **Only when there is an address or I ask for it**, **Always** or **Never**. | Only when there is an address or I ask for it | [`compose.replyTo.show`](/admin/settings#compose.replyTo.show) |
| **Where sent copies go** | **The Sent folder**, **One fixed folder**, **By recipient domain**, **By sending identity**, or **By rules (recipient domain or subject)**. A message can still choose its own folder in the Delivery tab. | The Sent folder | [`compose.sentFolder.mode`](/admin/settings#compose.sentFolder.mode) |
| **Fixed folder** | A folder role (for example `archive`) or folder id, used with the fixed-folder choice. | Empty | [`compose.sentFolder.fixedId`](/admin/settings#compose.sentFolder.fixedId) |
| **Folder per identity** | One line per identity, `identity-id=folder`. | Empty | [`compose.sentFolder.perIdentity`](/admin/settings#compose.sentFolder.perIdentity) |
| **Folder rules** | One rule per line, `domain:example.com=folder` or `subject:text=folder`. The first match wins. | Empty | [`compose.sentFolder.rules`](/admin/settings#compose.sentFolder.rules) |
| **Do not save sent copies** | Keeps no copy of sent messages; the Delivery tab says so before you send. | Off | [`compose.sentFolder.skipCopy`](/admin/settings#compose.sentFolder.skipCopy) |
| **Expired messages** | What to do with a message whose Expires date has passed: **Nothing**, **Show it dimmed**, **Suggest archiving it**, or **Hide it from message lists**. | Nothing | [`mail.expiry.hideExpired`](/admin/settings#mail.expiry.hideExpired) |
| **Hold sent messages for** (Undo send) | After you press Send the message waits in your Outbox for this long, from **Off (send immediately)** to **30 minutes**, or a custom value. Until then you can recall it and keep editing. The server holds it, so closing the browser does not send it early or lose it. | 10 seconds | [`mail.send.recallDelaySeconds`](/admin/settings#mail.send.recallDelaySeconds) |
| **Save automatically** (Account drafts) | Saves to your account's Drafts folder so drafts are available on another device: **After a typing pause**, **At regular intervals**, or **Only when I choose Save**. Only failures show a notification. | After a typing pause | [`mail.draftSaveMode`](/admin/settings#mail.draftSaveMode) |
| **Typing pause (milliseconds)** | 500 to 10000. | 1000 | [`mail.draftDebounceMs`](/admin/settings#mail.draftDebounceMs) |

The longest hold time you can choose, and whether recall is available at all, are limits set by your administrator ([`mail.send.recallDelayMaxSeconds`](/admin/settings#mail.send.recallDelayMaxSeconds) and [`mail.send.allowRecall`](/admin/settings#mail.send.allowRecall)). They are locked for you and are not shown as controls.

### Message notices

![The Message notices card with the Enable notices switch and the New notice button.](/screenshots/user-settings/mail-compose-notices.webp){.mw-crop}

A notice is reusable content added after the signature, independently of it. Notices belong to this account. **Enable notices** ([`mail.notices.enabled`](/admin/settings#mail.notices.enabled), on by default) turns them on; **New notice** creates one. Text is required for plain-text and encrypted mail; optional HTML is sanitized before sending.

### Response templates

![The Response templates card: use and sidebar switches, a search box, the template count and the Add template button.](/screenshots/user-settings/mail-compose-templates.webp){.mw-crop}

**Use response templates** ([`mail.templates.enabled`](/admin/settings#mail.templates.enabled)) and **Show Templates in the sidebar** ([`mail.templates.showInSidebar`](/admin/settings#mail.templates.showInSidebar)) are on by default. Personal templates are available only to you; shared templates are managed by your administrator. You can keep up to 20 templates with names up to 80 characters and plain message text up to 8,000 characters each; the library may use up to 64 KiB. **Add template** creates one and **Save templates** stores the changes.

## Folders and sidebar

Folders and sidebar has eight tabs. Choices here sync with your account and do not move messages or rename folders.

### Folder list and order

![The Folder list and order card: every folder with its visibility, unread and total counts, location and actions.](/screenshots/user-settings/mail-folders-list.webp){.mw-crop}

The list shows every folder with whether it is **Shown** in the sidebar, its unread and total counts, and its location. **Create folder** adds one. **Select visible**, **Select all folders** and **Select all user folders** select rows for the actions that follow. Drag folders to reorder them in the sidebar; pinned folders stay together.

### Folders and sidebar

![The Folders and sidebar card and the Counter bubbles card.](/screenshots/user-settings/mail-folders-sidebar.webp){.mw-crop}

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Collapse user folders in the sidebar** | Folders you created stay folded under the User folders heading until you open it. | Off | synchronized with your account; no settings key |
| **Default folder colour** | Colours folder icons and names. Edit a folder to give it its own colour. | Default | [`folders.colour`](/admin/settings#folders.colour) |
| **Unified virtual account** | Adds a separate virtual account to the account selector, with combined folders and counters. Independent of the unified message view. | Off | [`mail.unifiedAccount`](/admin/settings#mail.unifiedAccount) |
| **Unified message view** | Browse and search the primary and linked mailboxes together. | Off | [`mail.unifiedInbox`](/admin/settings#mail.unifiedInbox) |
| **Copy or move to another account** | Transfers messages between linked mailboxes that support export and import. | Off | [`mail.crossAccountTransfer`](/admin/settings#mail.crossAccountTransfer) |
| **Hide empty folders in the sidebar** | Folders reappear when they contain messages. Inbox, search folders, folders with unknown counts and parents of visible folders stay shown. | Off | [`folders.hideEmpty`](/admin/settings#folders.hideEmpty) |
| **Folder counts** | **Hide counts**, **Unread messages**, **Unread messages, only when there are any**, **Total messages**, or **Unread and total messages**. | Unread and total messages | [`mail.folderCounts`](/admin/settings#mail.folderCounts) |
| **Folders with counts** | **All folders** or **Main folders only**. Main folders are Inbox, Drafts, Sent, Archive, Junk, Trash, Outbox and other special-use folders. | All folders | [`mail.folderCountScope`](/admin/settings#mail.folderCountScope) |
| **Smaller badges outside Inbox** | Makes counters smaller except on Inbox. | On | [`mail.compactFolderBadges`](/admin/settings#mail.compactFolderBadges) |
| **Show unread count next to the mailbox name** | Adds the unread count beside each mailbox name. | Off | [`mail.accountUnreadBadge`](/admin/settings#mail.accountUnreadBadge) |

For the unified views see [Unified mail](/user/unified-mail) and [Moving mail between accounts](/user/mailbox/moving-between-accounts).

### Workspace view properties

![The top of the Workspace view properties card: visibility, display name and item count for each workspace view.](/screenshots/user-settings/mail-folders-views.webp){.mw-crop}

For each workspace view in the sidebar (Notifications, Events, Tasks, Notes, Contacts, Keys, Attachments, Outbox and Templates) you can set three things. **Visibility** is **Always show**, **Hidden** or **Only when notable**, where notable means, for example, unread notifications, events in the next seven days or unfinished tasks. **Display name** renames the view. **Show item count** shows or hides its count. Defaults are Always show, no custom name and count on. The settings are named `views.<view>.visibility`, `views.<view>.name` and `views.<view>.showCount`, for example [`views.tasks.visibility`](/admin/settings#views.tasks.visibility).

### Arrange folders

![The Arrange folders card: dragging, nesting behaviour, where new folders appear and drag handles.](/screenshots/user-settings/mail-folders-arrange.webp){.mw-crop}

Drag a folder to reorder it, drop it on another folder to nest it, or use Alt and the arrow keys on a focused folder.

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Rearrange folders by dragging** | Turns dragging folders on or off. | On | [`folders.reorder.enabled`](/admin/settings#folders.reorder.enabled) |
| **When I nest a folder** | **Ask me**, **Move on the server**, or **Only arrange in this app**. | Ask me | synchronized with your account; no settings key |
| **New folders appear** | **After their siblings** or **In alphabetical order**. | After their siblings | [`folders.newFolderPlacement`](/admin/settings#folders.newFolderPlacement) |
| **Show drag handles** | **On hover** or **Always**. | On hover | synchronized with your account; no settings key |

**Reset order to default** restores the default folder order.

### Special folders

![The Special folders card: one choice per mail role, each set to Automatic discovery.](/screenshots/user-settings/mail-folders-special.webp){.mw-crop}

Choose the real folder used for each mail role: **Inbox**, **Drafts**, **Sent mail**, **Trash**, **Junk and spam** and **Archive**. **Automatic discovery** uses what the server reports. A conflicting or missing destination disables that role instead of choosing another folder. The settings are [`mail.folderRole.inbox`](/admin/settings#mail.folderRole.inbox), [`mail.folderRole.drafts`](/admin/settings#mail.folderRole.drafts), [`mail.folderRole.sent`](/admin/settings#mail.folderRole.sent), [`mail.folderRole.trash`](/admin/settings#mail.folderRole.trash), [`mail.folderRole.junk`](/admin/settings#mail.folderRole.junk) and [`mail.folderRole.archive`](/admin/settings#mail.folderRole.archive); the default for each is automatic discovery.

### Focused inbox

![The Focused inbox card with its switch, and the Manage Focused senders list.](/screenshots/user-settings/mail-folders-focused.webp){.mw-crop}

**Focused inbox** ([`mail.focusedInbox`](/admin/settings#mail.focusedInbox), off by default) splits Inbox into **Focused** and **Other** tabs above the list. Messages from newsletters and automated senders go to Other. Other folders and search results show every message. **Manage Focused senders** lists senders you chose to always move to Focused or Other with **Always move to Focused** or **Always move to Other** on a message; remove one to let the automatic sorting decide again.

### Properties, priority and export

![The Properties, priority and export card: priority marks, the Properties dialogs and their keyboard shortcut.](/screenshots/user-settings/mail-folders-properties.webp){.mw-crop}

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Priority marks** | **High and low**, **High only** or **Off**. | High and low | [`mail.priorityMarks`](/admin/settings#mail.priorityMarks) |
| **Priority mark style** | **Icon**, **Icon and words** or **Colour bar**. | Icon | [`mail.priorityMarks.style`](/admin/settings#mail.priorityMarks.style) |
| **Priority selector in the composer** | Shows a priority choice in the composer. | On | [`mail.compose.priority.enabled`](/admin/settings#mail.compose.priority.enabled) |
| **Priority a new message starts with** | **Normal**, **High** or **Low**. | Normal | [`mail.compose.priority.default`](/admin/settings#mail.compose.priority.default) |
| **Folder Properties dialog** | Offers the Properties dialog for folders. | On | [`properties.folder.enabled`](/admin/settings#properties.folder.enabled) |
| **Message Properties dialog** | Offers the Properties dialog for messages. | On | [`properties.message.enabled`](/admin/settings#properties.message.enabled) |
| **Notes on folders** | Allows notes on folders. | On | [`properties.folder.notes.enabled`](/admin/settings#properties.folder.notes.enabled) |
| **Include subfolders when downloading a folder** | Folder downloads include subfolders. | On | [`export.folder.includeSubfolders`](/admin/settings#export.folder.includeSubfolders) |
| **Properties keyboard shortcut** | Turns the shortcut on. | On | [`properties.shortcut.enabled`](/admin/settings#properties.shortcut.enabled) |
| **Shortcut key** | The key combination. | Alt+Enter | [`properties.shortcut.key`](/admin/settings#properties.shortcut.key) |

### Storage

![The Storage card, which says the mailbox does not provide storage usage.](/screenshots/user-settings/mail-folders-storage.webp){.mw-crop}

**Storage** shows how much of your mailbox quota is used when the mail server reports it. When it does not, the card says so; select refresh to check again.

## Retention

![The top of Retention: the last run, the undo window, what this server allows and the list of your rules.](/screenshots/user-settings/mail-retention.webp){.mw-crop}

Retention rules move or delete your own mail after a number of days. A rule can look at a folder, a label, a sender, a size, whether the message is read, or whether it has an attachment. Flagged mail and mail labelled `keep` are left alone unless you change that. Deleted mail goes to Trash first and is purged after the undo window.

- **Run my retention rules** switches your rules on or off. The line above it shows when they last ran.
- **Move deleted mail to Trash first** is shown with the note that the server always does this.
- **Undo window in days** is how long mail a rule moved to Trash stays there before it is purged, 0 to 7 days. **Pause all rules for 30 days** stops rules for that long.
- **What this server allows** lists the administrator's limits for you: the shortest wait before a rule may delete, the number of rules you can keep, whether permanent deletion is allowed, whether removing attachments is allowed, and folders rules never touch.
- **Your rules** lists your rules. **New rule** adds one. Templates further down add ready-made rules (for example newsletters, receipts, notifications, Trash, Junk, or archiving read Inbox mail); check the labels and days before relying on them.
- **Run now** previews or runs your rules immediately: **Preview all rules**, **Run my rules now**.

![The Folder defaults card: one choice per folder, each set to Inherit, with the Drafts folder marked as protected by the administrator.](/screenshots/user-settings/mail-retention-folders.webp){.mw-crop}

**Folder defaults** sets how long mail stays in a folder before it is moved or deleted: **Inherit**, **Keep for ever**, **Move to Trash**, **Delete**, **Move to Archive** or **Mark as read**. A folder set to Inherit has no default of its own; the rules and the administrator's limits still apply, and subfolders follow their folder. Select **Save folder defaults** to store them. A folder the administrator protects shows **Protected by the administrator** and cannot be changed. **Calendar events** and **Notes** have their own retention cards, which say **Turned off by your administrator** when that applies.

Retention rules are stored as records and not as individual settings. See [Retention](/admin/retention) for the administrator's side.

## Labels

![The Labels list: a search box and the labels Work, Personal, Important, Later and Receipts with their keyword and colour.](/screenshots/user-settings/mail-labels.webp){.mw-crop}

Labels have a name, a colour and an icon. They belong to this account and synchronize across devices. A message label is a JMAP keyword, so the label **Work** is the keyword `work`. Removing a label definition keeps the existing keywords on messages, which are then shown by their keyword name. The list starts with five labels (Work, Personal, Important, Later and Receipts) and counts them against the limit of 100. **New label** opens a form with a **Label name**, a **Label colour** and a **Label icon** chosen from a set of native icons. Each label has buttons to edit and remove it.

## Mail rules

Mail rules has three tabs. What each tab can do depends on the account and on administrator settings; see [Rules and filters](/user/rules-filters).

![The Builder tab of Mail rules with a new rule open for editing, above the notice that vacation replies are unavailable.](/screenshots/user-settings/mail-rules-builder.webp){.mw-crop}

**Builder** lists your visual rules and **New rule** opens the editor: a **Rule name**, **When to run** (**Match all conditions** or **Match any condition**), **Conditions** (a field such as **From**, an operator such as **contains** and a value, with **Add condition**) and **Actions** (such as **Move to mailbox**, with **Add action**). Select **Save rule** to store it. A rule is applied when the engine syncs new inbox messages; replies and moves wait for the next sync. A notice tells you when vacation replies are not available for this account because its mail server does not offer them or your administrator has turned them off.

![The Raw Sieve tab: the script list, the source editor and the Validate script, Save, Activate and Delete buttons.](/screenshots/user-settings/mail-rules-sieve.webp){.mw-crop}

**Raw Sieve** lets you edit your server-side filtering script as text. Validate it before saving and activate a saved script to apply it to incoming mail. When your administrator has disabled raw script editing, the tab says so and the editing buttons are unavailable. **Export script** downloads the script. **Forwarding** has a switch, a **Forward to** address and **Keep a copy in this account**; save with **Save**. A forwarding address in a domain your administrator does not allow is refused.

## Saved searches

![The Saved searches section: the default search scope and the New search folder button.](/screenshots/user-settings/mail-searches.webp){.mw-crop}

A saved search can be shown as a folder in your mailbox list. A search folder shows matching messages without moving or copying them.

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Default search scope** | Search the **Current folder** by default, or start across **All folders**. You can still choose a different folder or All folders for an individual search. Saved searches keep their own scope. | Current folder | [`mail.search.defaultScope`](/admin/settings#mail.search.defaultScope) |

To add one, select **New search folder**, enter a **Folder name**, and fill in any of the conditions: **Search all message text**, **Sender contains**, **Recipient contains**, **Subject contains**, **Message body contains**, **Attachment filename contains**, a mailbox identifier, **Received after (UTC date and time)** and **Received before (UTC date and time)**, the **Attachments** choice (**Any message**, **With attachments**, **Without attachments**), **Unread messages only** and **Flagged messages only**. Messages must match all conditions; an empty condition is ignored. **Show as folder** puts the search in the folder list. See [Searching](/user/mailbox/search).

## Read receipts

![The Read receipts section: delivery notification defaults, how read receipt requests are answered and sender exceptions.](/screenshots/user-settings/mail-receipts.webp){.mw-crop}

A read receipt reveals that your mail program displayed a message. This section has three parts.

- **Delivery notification defaults**: whether new messages request delivery notifications, when supported. Each message can choose differently, and your administrator can lock the default or prohibit delivery notifications. A server delivery report does not prove anyone read your message. Select **Edit delivery notification default** to change it.
- **Read receipts**: how this account answers a sender's request. Select **Edit receipt responses** and choose the **Response policy**: **Never send read receipts**, **Ask before sending**, **Send automatically when safe**, **Send automatically to contacts only**, **Send automatically inside my organisation only**, or **Send inside my organisation; ask for other senders**. Administrator restrictions and safety checks always apply, and a different return address or a list message cannot silently receive an automatic receipt. The default is **Ask before sending**.
- **Sender exceptions**: **Add sender exception** matches one exact email address and sets the response for it to **Suppress prompts and automatic receipts** or **Send automatically when safe**. Your Never preference and administrator-held policies take precedence.

## Import and export mail

![The Import and export mail section with the Import mail button.](/screenshots/user-settings/mail-transfer.webp){.mw-crop}

**Import mail** brings a mail file into a folder of this account. Choose a **Mail file** and a **Destination folder**, then select **Import into selected folder**. Existing messages are preserved, and importing the same archive again can create duplicates. The dialog shows the maximum file size and number of messages. When your administrator has disabled mail import, the section says so.

Export starts from a message: open a message and use its **Export** action, where you choose a format: **EML (original message)**, **MBOX (mailbox file)**, **Plain text**, **Markdown**, **MSG (Outlook)**, **DOCX (Word)**, **CSV summary**, **JSON (message properties)** or **PDF**. EML preserves the original message; the others are converted. Your administrator's export and attachment restrictions apply.

## Troubleshooting

| What you see | Why | What to do |
|---|---|---|
| **Protected by the administrator** beside a folder in Retention. | The administrator keeps rules away from that folder. | Nothing to change; the folder is not touched by rules. |
| You cannot choose a longer hold time. | The longest hold time is an administrator limit, 30 minutes unless the administrator changed it. | Ask your administrator if you need a longer one. |

## Related

- [Personal settings](./index.md)
- [Reading messages](/user/mailbox/reading)
- [Composing a message](/user/mailbox/composing)
- [Folders and labels](/user/mailbox/folders-labels)
- [Rules and filters](/user/rules-filters)
