---
title: Interface settings
screen: true
---

# Interface settings

The **Interface** category covers how Mailwoman looks, which keys it answers to, what notifies you, and the language, region and time zone it uses. It has four sections.

![The Settings screen on Appearance, with markers 1 to 4 on the four sections of the Interface category in the section list.](/screenshots/user-settings/interface-overview.webp){.mw-shot}

1. **Appearance**: theme, colours, density, fonts, message list and toolbars.
2. **Keyboard & display**: shortcuts, the offline cache, interface direction and a few mail behaviours.
3. **Notifications**: new-mail alerts, toasts, background notifications and quiet hours.
4. **Regional settings**: language, date and time formats, and time zones.
{.mw-callouts}

Every setting in the tables on this page can be locked by an administrator, except where noted. A locked control is dimmed and shows **Set by your administrator**. See [Personal settings](./index.md).

## Appearance

The Appearance section is one long page with five cards.

### Theme and colours

![The Theme and switching card with the theme choices, and the Colours card with accent and unread badge colour.](/screenshots/user-settings/interface-appearance-theme.webp){.mw-crop}

The **Theme and switching** card has two rows. **When to switch** chooses **One theme**, **Follow the system** or **By time of day**; the theme you pick stays on until you change it. **Theme** lists 25 themes, each with a short description and a Dark or Light tag: Dark, Light, Black, Paper, Blueprint, Terminal, Primary blocks, Slate Light, Slate Dark, Slate Mist, Slate Midnight, Grove Light, Grove Dark, Plum Light, Plum Dark, Ocean Light, Ocean Dark, Cyberpunk, Retrowave, Fight Club, 1984, Pop Art, AMOLED, High contrast (light) and High contrast (dark). Choosing a theme there switches to one fixed theme. The theme is saved to your account and other devices pick it up when they start. **Forget the appearance saved for my account** removes the saved choice, and the server's default applies again. The built-in default theme is Dark ([`appearance.theme`](/admin/settings#appearance.theme)).

The **Colours** card sets the **Accent** used for actions and selection (**Theme default** or one of 16 named colours), a **Custom accent colour** (with **Use named or theme accent** to go back) and the **Unread badge colour** (**Theme default**, **Same as accent**, **Lighter accent**, **Darker accent**, or a named colour).

| Label | Default | Setting |
|---|---|---|
| **Accent** | Theme default | [`appearance.accent`](/admin/settings#appearance.accent) |
| **Unread badge colour** | Theme default | [`appearance.badgeColor`](/admin/settings#appearance.badgeColor) |

### Text and layout

![The top of the Text and layout card: density, interface font, font size and weight, colours, and message list choices.](/screenshots/user-settings/interface-appearance-text.webp){.mw-crop}

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Density** | **Compact**, **Cozy** or **Relaxed** spacing of the interface. | Cozy | [`appearance.density`](/admin/settings#appearance.density) |
| **Interface font** | Default, System, Serif, Mono, Inter, Roboto, Open Sans, Jost or Manrope. | Default | [`appearance.font`](/admin/settings#appearance.font) |
| **Interface scale (%)** | Resize the whole interface, including controls and spacing, from 50–200%. Message zoom is separate. | 100% | [`appearance.uiScale`](/admin/settings#appearance.uiScale) |
| **Interface font size** | 12 to 24 pixels. | 14 | [`appearance.fontSize`](/admin/settings#appearance.fontSize) |
| **Interface font weight** | Light, Regular, Medium or Semibold. | Regular | [`appearance.fontWeight`](/admin/settings#appearance.fontWeight) |
| **Interface font color** | A custom text colour. Empty uses the theme. | Theme colour | [`appearance.fontColor`](/admin/settings#appearance.fontColor) |
| **Low contrast mode** | Softens interface text contrast. | Off | [`appearance.lowContrast`](/admin/settings#appearance.lowContrast) |
| **Message list style** | **Cards** or **Gmail-style rows**. | Cards | [`mail.messageListStyle`](/admin/settings#mail.messageListStyle) |
| **Message list order** | The order of messages within each group, or across the whole list when grouping is off: newest or oldest first, sender, subject, sent date, importance or size, in either direction. | Newest first | [`mail.messageListOrder`](/admin/settings#mail.messageListOrder) |
| **Message grouping** | Keeps related messages together under headings: **No grouping**, **Received date**, **Sent date**, **Sender**, **Subject**, **Importance** or **Message size**. | No grouping | [`mail.messageListGroup`](/admin/settings#mail.messageListGroup) |
| **Group messages into conversations** | Folds related replies into expandable conversations, independently of grouping. Only messages loaded in the current folder or search are included. An administrator can make it unavailable. | Off | [`mail.conversations.enabled`](/admin/settings#mail.conversations.enabled) |
| **Show message body previews** | Shows a preview line in the list. | On | [`mail.messageListPreview`](/admin/settings#mail.messageListPreview) |
| **Messages loaded per page (20 to 200)** | How many messages load at a time. | 50 | [`mail.list.pageSize`](/admin/settings#mail.list.pageSize) |
| **After moving a message** | **Keep message open**, **Open next message**, **Open previous message** or **Close message**. Failed moves leave the reader unchanged. | Keep message open | [`mail.afterMove`](/admin/settings#mail.afterMove) |
| **After deleting a message** | **Open next message**, **Open previous message** or **Close message**. | Open next message | [`mail.afterDelete`](/admin/settings#mail.afterDelete) |
| **Show attachments at the end of messages** | Places the attachments after the message text. | Off | [`reader.attachmentsAtEnd`](/admin/settings#reader.attachmentsAtEnd) |
| **Tell me when a new version of Mailwoman is available** | Shows a notice when an update is ready. | On | [`ui.notifyNewVersion`](/admin/settings#ui.notifyNewVersion) |
| **Text style** | **Normal text** or **Leetspeak (1337)** for interface labels and message reading. Editing and sent messages keep their original text. | Normal text | [`appearance.leetspeak`](/admin/settings#appearance.leetspeak) |
| **Layout** | **Focus** or **Command Ribbon**. The Command Ribbon shows a ribbon of commands above the mailbox and moves the composer's commands into it. | Focus | no settings key |

### Toolbars, and importing and exporting preferences

![The Toolbars card with a pinned or auto-hide choice for four bars, and the Import and export preferences card.](/screenshots/user-settings/interface-appearance-toolbars.webp){.mw-crop}

Each of the **Mail toolbar**, **Workspace tabs**, **Mail ribbon** and **Editor ribbon** can be pinned or auto-hide. Pinned bars stay visible; an auto-hide bar shows only its tabs or a thin handle until you point at, tap or tab to it. All four are pinned by default ([`ui.bar.mailTopBar`](/admin/settings#ui.bar.mailTopBar), [`ui.bar.workspaceTabs`](/admin/settings#ui.bar.workspaceTabs), [`ui.bar.mailRibbon`](/admin/settings#ui.bar.mailRibbon), [`ui.bar.composerRibbon`](/admin/settings#ui.bar.composerRibbon)).

**Export preferences** saves your personal choices to a file as a backup. **Import preferences** lets you review a settings file before restoring selected preferences.

## Keyboard & display

### Shortcut preset

![The Shortcut preset card: the Mailwoman, Gmail, Outlook, Vim and Off presets, and the View preset bindings list.](/screenshots/user-settings/interface-shortcuts-preset.webp){.mw-crop}

**Shortcut preset** chooses a set of keys: **Mailwoman**, **Gmail**, **Outlook**, **Vim** or **Off (no shortcuts)**. Changes apply on this device. **View preset bindings** lists the keys of the chosen preset, for example compose, archive, reply and next message; custom bindings and administrator policy can change the active shortcuts.

### Offline cache

![The top of the Offline cache card: offline access, persistent storage, header window, bodies, attachments and cache budget.](/screenshots/user-settings/interface-shortcuts-offline.webp){.mw-crop}

The Offline cache card sets how much mail to keep on this device for offline use and how to reclaim space. See [Sync and offline access](/user/offline-sync).

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Offline access** | Keeps encrypted copies of cached mail, calendars, tasks, notes and contacts on this device. Only cached content is available offline; queued changes wait for a verified connection. | Off | [`offline.enabled`](/admin/settings#offline.enabled) |
| **Offer persistent browser storage** | When encrypted offline mail is on, offers a browser request that protects the cache from automatic eviction. **Request persistent storage** sends the request. | On | [`offline.requestPersistence`](/admin/settings#offline.requestPersistence) |
| **Offline header window** | How far back message headers are synchronized: **30 days**, **90 days**, **365 days** or **All available mail**. Older mail stays available online. | 90 days | [`sync.headersWindow`](/admin/settings#sync.headersWindow) |
| **Message bodies** | **When opened**, or **Prefetch for offline use**. | When opened | [`sync.bodies`](/admin/settings#sync.bodies) |
| **Attachments** | **When opened**, **Inline images**, or **All permitted attachments** (each up to 10 MiB). Risk warnings are never accepted automatically. | When opened | [`sync.attachments`](/admin/settings#sync.attachments) |
| **Cache budget (MB)** | How much the cache may use. | 250 | [`offline.budgetMb`](/admin/settings#offline.budgetMb) |
| **Keep for (days)** | How long cached items are kept. | 30 | [`offline.retentionDays`](/admin/settings#offline.retentionDays) |
| **When the budget is reached** | **Remove least recently used**, **Remove oldest first** or **Only when I clear it**. | Remove least recently used | [`offline.eviction`](/admin/settings#offline.eviction) |

The card shows how much is cached, and **Clear the offline cache now** empties it. Whether offline storage is allowed on this device is also an administrator limit ([`offline.persistentStorageAllowed`](/admin/settings#offline.persistentStorageAllowed)).

### Interface direction

**Interface direction** is **Automatic** (follow the language), **Left to right** or **Right to left**, which mirrors the layout. A preview line below shows the effect.

### Keyboard shortcuts

![The top of the Keyboard shortcuts card: two switches, a search box and a list of actions with their bindings.](/screenshots/user-settings/interface-shortcuts-keys.webp){.mw-crop}

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Enable application shortcuts** | Turns application shortcuts on or off. | On | [`shortcuts.enabled`](/admin/settings#shortcuts.enabled) |
| **Show keyboard shortcut toasts** | Briefly shows the action and keys when a shortcut is used. It confirms activation, not completion. | Off | [`shortcuts.activationToast`](/admin/settings#shortcuts.activationToast) |

Below the switches, each action (Compose, Archive, Reply, Reply all, Forward, Next message, Previous message, Search, Refresh messages, Open settings, Move to trash, Mark read, Mark unread, and more) has a binding. Use **inherit** for your selected preset, **disabled** for no shortcut, or a chord such as Ctrl+Shift+K; Ctrl means Cmd on macOS. Conflicting bindings are inactive. Typing fields and dialogs keep their usual keys, and browser-reserved combinations may be unavailable. Each action's setting is named `shortcuts.<action>`, for example [`shortcuts.compose`](/admin/settings#shortcuts.compose), and is `inherit` by default. See [Keyboard shortcuts](/user/keyboard-shortcuts).

### Message selection, mail features and undo

![The Message selection and moving card and the Mail features card.](/screenshots/user-settings/interface-shortcuts-mail.webp){.mw-crop}

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Resume my open tabs** | Restores account-owned tab references after reopening Mailwoman. Message bodies and attachment URLs are not saved in the workspace record. | On | [`mail.resumeTabs`](/admin/settings#mail.resumeTabs) |
| **Select messages with modifier keys** | Ctrl or Command toggles separate messages; Shift selects a range. The visible selection button stays available when this is off. | On | [`mail.modifierSelection`](/admin/settings#mail.modifierSelection) |
| **Drag messages to folders** | Dragging a selected message moves the selection; dragging an unselected one moves just that message. Only folders in this account are accepted. | On | [`mail.dragToFolder`](/admin/settings#mail.dragToFolder) |
| **Message list quick actions** | How the quick actions on each message row appear: **Always open**, **Show on hover**, **Show on click** or **Off**. Off removes them from the rows; the context menu and toolbar still have every action. | Always open | [`mail.rowActionsMode`](/admin/settings#mail.rowActionsMode) |
| **Allow sending messages** | When off, Send and its shortcut are disabled. Drafts can still be saved. | On | [`mail.allowSend`](/admin/settings#mail.allowSend) |
| **Allow reply and reply all** | When off, Reply and Reply all and their shortcuts are not offered. | On | [`mail.allowReply`](/admin/settings#mail.allowReply) |
| **Allow forwarding** | When off, the Forward actions and their shortcut are not offered. | On | [`mail.allowForward`](/admin/settings#mail.allowForward) |
| **Allow downloading messages and attachments** | When off, downloads are not offered. | On | [`mail.allowDownload`](/admin/settings#mail.allowDownload) |
| **Allow printing** | When off, the Print action is not offered. | On | [`mail.allowPrint`](/admin/settings#mail.allowPrint) |
| **Show the Spam folder** | When off, the Spam folder and messages marked as spam are hidden from the folder list and search results. Nothing is deleted. | On | [`mail.showSpam`](/admin/settings#mail.showSpam) |
| **Show sender authentication results** | Shows the SPF, DKIM and DMARC results of a message in the reader. | On | [`mail.showAuthStatus`](/admin/settings#mail.showAuthStatus) |
| **Close a message when it is deleted** | Closes the open message, tab or window when that message is deleted or moved to Trash. | On | [`mail.closeOnDelete`](/admin/settings#mail.closeOnDelete) |
| **Show a Go to top button in the message list** | Adds a button that scrolls the list back to the first message. | Off | [`mail.showGoToTop`](/admin/settings#mail.showGoToTop) |
| **Show a Create folder button next to Compose** | Adds an icon-only button in the sidebar that creates a folder. | Off | [`mail.showCreateFolderButton`](/admin/settings#mail.showCreateFolderButton) |
| **Enable undo and redo history** | Shows Undo and Redo buttons in the mailbox toolbar and answers Ctrl+Z and Ctrl+Shift+Z outside text fields. The history lasts until you sign out or switch account. | On | [`ui.undoHistorySize`](/admin/settings#ui.undoHistorySize) |
| **Actions remembered** | How many recent mail actions Undo and Redo remember, 0 to 200; 0 turns them off. The administrator can set a ceiling. | 50 | [`ui.undoHistorySize`](/admin/settings#ui.undoHistorySize) |

The **Quick action layout** card places each row action (View draft, Pin, Snooze, Label, Flag for follow-up, Archive, Delete) on the toolbar, in the More menu or hidden, and arranges their order. Hidden actions stay available in the message context menu. Its setting is [`mail.rowActions`](/admin/settings#mail.rowActions).

## Notifications

Notifications has four tabs. For what the notification centre shows, see [Notifications](/user/notifications).

### New mail

The **Mail since your last check** card controls a sign-in toast with separate Inbox and Spam counts. It is enabled by default and counts received messages rather than your entire unread backlog. The first successful check establishes a baseline without showing a toast. The server stores one small checkpoint per verified mailbox, so it follows you between devices; concurrent tabs cannot show the same summary twice.

You can independently include Inbox, Spam and linked mailboxes, count unread mail only, respect quiet hours, or show an empty result. Defaults are a **15-second** display, **1 minute** away, and a **60-day** maximum lookback. The toast names the shorter period after a long absence. Linked mailboxes use their own preferences and policies. Successful foreground folder checks renew the active mailbox checkpoint at most once a minute; failed or offline reads preserve the previous checkpoint. A checkpoint expires after a year without use.

Administrators can set and lock all ten `notifications.loginSummary.*` controls at global, domain or user scope. These preferences are independent of live spam-folder alerts and browser notification permission; the general toast preferences still apply.

![The top of the New mail tab: the Notifications and Permission on this device cards and the first New-mail channels.](/screenshots/user-settings/interface-notifications-mail.webp){.mw-crop}

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Enable the Notifications view** | Controls the notification centre view. Security notices, mail toasts and browser alerts keep their own settings. | On | [`notifications.view.enabled`](/admin/settings#notifications.view.enabled) |
| **Show Notifications in the sidebar** | Shows the Notifications entry in the sidebar. | On | [`notifications.view.showInSidebar`](/admin/settings#notifications.view.showInSidebar) |
| **Show notifications for new mail** | The master switch for new-mail alerts. | Off | [`notifications.enabled`](/admin/settings#notifications.enabled) |
| **Offer notification permission on new devices** | Shows a permission prompt when this device has not allowed or blocked notifications. Saves immediately. | On | [`notifications.askPermission`](/admin/settings#notifications.askPermission) |
| **New-mail toast** | An in-app alert when matching mail arrives. | On | [`notifications.incoming.toast`](/admin/settings#notifications.incoming.toast) |
| **Native new-mail notification** | Browser or operating-system notifications. Browser permission is required. | On | [`notifications.incoming.native`](/admin/settings#notifications.incoming.native) |
| **Flash the browser tab** | Alternates the tab title until focused or the alert timeout expires. Reduced motion uses a steady indicator. | Off | [`notifications.incoming.tab`](/admin/settings#notifications.incoming.tab) |
| **Highlight arriving messages** | Briefly highlights newly received rows without changing the selection. | On | [`notifications.incoming.highlight`](/admin/settings#notifications.incoming.highlight) |
| **Native notifications while focused** | Also shows native alerts while this window is in use. | Off | [`notifications.incoming.foreground`](/admin/settings#notifications.incoming.foreground) |
| **Only notify matching rules** | Notifies only mail matching a Notify rule. Matching mute rules, quiet hours and muted folders still take priority. | Off | [`notifications.incoming.onlyMatchingRules`](/admin/settings#notifications.incoming.onlyMatchingRules) |
| **Allow spam-folder alerts** | Spam is muted unless this is on. | Off | [`notifications.incoming.includeJunk`](/admin/settings#notifications.incoming.includeJunk) |
| **New-mail attention duration** | Seconds before tab attention and row highlights stop, 2 to 120. | 15 | [`notifications.incoming.durationSeconds`](/admin/settings#notifications.incoming.durationSeconds) |
| **New-mail folder scope** | **All received folders**, **Inbox only** or **Selected folders**. Drafts, Sent and Trash never trigger new-mail alerts. | All received folders | [`notifications.incoming.scope`](/admin/settings#notifications.incoming.scope) |
| **Selected alert folders** and **Muted folders** | A checkbox per folder. Muted folders take priority, and matching mute rules suppress all new-mail channels. Folder-scoped alerts are evaluated while Mailwoman is open; content-free background alerts are suppressed for these scopes so they cannot bypass your choices. | None selected, none muted | [`notifications.incoming.folders`](/admin/settings#notifications.incoming.folders), [`notifications.incoming.mutedFolders`](/admin/settings#notifications.incoming.mutedFolders) |

The notification permission itself belongs to the browser on this device. When it is blocked, the card says so and tells you to change the site's permission in the browser settings.

### Events and toasts

![The Other notification events card and the Toast appearance and behaviour card.](/screenshots/user-settings/interface-notifications-events.webp){.mw-crop}

Each event can be set to **No alert**, **In-app toast**, **Native notification** or **Toast and native notification**. The default for all five is **In-app toast**.

| Event | Setting |
|---|---|
| **Message queued or scheduled** | [`notifications.events.queued`](/admin/settings#notifications.events.queued) |
| **Message accepted by the mail server** (this does not prove recipient delivery) | [`notifications.events.sent`](/admin/settings#notifications.events.sent) |
| **Sending failed or uncertain** (the Outbox keeps the status) | [`notifications.events.failed`](/admin/settings#notifications.events.failed) |
| **New security notices** | [`notifications.events.security`](/admin/settings#notifications.events.security) |
| **New application version** | [`notifications.events.update`](/admin/settings#notifications.events.update) |

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Show toasts** | Enables transient in-app alerts. Persistent progress and undo controls stay available. | On | [`notifications.toasts.enabled`](/admin/settings#notifications.toasts.enabled) |
| **Success toasts**, **Information toasts**, **Warning toasts**, **Error toasts** | Show each kind of toast. Errors still remain in their owning views. | On | [`notifications.toasts.success`](/admin/settings#notifications.toasts.success), [`notifications.toasts.info`](/admin/settings#notifications.toasts.info), [`notifications.toasts.warning`](/admin/settings#notifications.toasts.warning), [`notifications.toasts.error`](/admin/settings#notifications.toasts.error) |
| **Toast duration** | Seconds an ordinary alert stays, 1 to 60. Timed actions keep their own deadlines. | 5 | [`notifications.toasts.durationSeconds`](/admin/settings#notifications.toasts.durationSeconds) |
| **Visible toast limit** | How many ordinary toasts show together, 1 to 8. | 3 | [`notifications.toasts.limit`](/admin/settings#notifications.toasts.limit) |
| **Swipe to dismiss** | Allows dismissing toasts with a horizontal touch or pen gesture. | On | [`notifications.toasts.swipe`](/admin/settings#notifications.toasts.swipe) |

Completed actions keep separate toasts, including identical moves. Bursts wait in a bounded queue rather than replacing earlier feedback. Waiting ordinary toasts receive their full display time when shown; externally timed actions keep their original deadlines. Repeated warnings/errors and explicit progress IDs still coalesce.

### Browser and storage

![The Background notifications on this browser card: the mailbox chooser and the Register background notifications and Remove for this account buttons.](/screenshots/user-settings/interface-notifications-browser.webp){.mw-crop}

**Background notifications on this browser** registers this browser for background notifications. Enable notifications for new mail first; the browser may ask for permission. Choose the mailbox to configure: browser permission is shared, but registration and removal apply only to the selected account. **Register background notifications** is unavailable until new-mail notifications are on and push is allowed for your account; **Remove for this account** is available once this browser is registered. The tab also has the **Offer persistent browser storage** switch, the same setting as in Offline cache, and a **Browser storage** card. Persistent storage protects this site's local data from automatic eviction when the browser grants it; it does not change server retention.

### Quiet hours and rules

![The Quiet hours card with its switch and the Rules card with Manage notification rules.](/screenshots/user-settings/interface-notifications-quiet.webp){.mw-crop}

**Silence notifications during a set time** turns on quiet hours; the card then asks for a **From** and a **To** time. **Manage notification rules** opens the rules, which match text in a sender, subject or folder name. Matching mute rules take priority, and rules are saved only after validation. Quiet hours and rules are saved with your account as records, not as individual settings.

## Regional settings

![The Language and region, Region date and time formats and Time zones cards.](/screenshots/user-settings/interface-regional.webp){.mw-crop}

| Label | What it does | Default | Setting |
|---|---|---|---|
| **Language** | The language of menus, buttons and messages in Mailwoman. | Same as the browser | [`ui.language`](/admin/settings#ui.language) |
| **Region** | The region used for calendar and number conventions. Automatic follows your language. | Automatic | [`time.region`](/admin/settings#time.region) |
| **Date format** | How dates are displayed, or a custom format for a specific layout. Display formats do not change stored dates. | Automatic | [`time.dateFormat`](/admin/settings#time.dateFormat) |
| **Time format** | How times are displayed. | Automatic | [`time.timeFormat`](/admin/settings#time.timeFormat) |
| **Clock** | Your preferred 12-hour or 24-hour style. Custom H or h pattern tokens take precedence. | Automatic | [`time.hourCycle`](/admin/settings#time.hourCycle) |
| **First day of the week** | Automatic follows regional calendar conventions. | Automatic | [`time.firstDayOfWeek`](/admin/settings#time.firstDayOfWeek) |
| **Show time zone helpers** | Shows time zone helpers when scheduling. | On | [`time.showHelpers`](/admin/settings#time.showHelpers) |
| **Display time zone** | The time zone for message times: search for a city, region or UTC offset, or use the browser time zone to follow this device. Locations are never inferred from email addresses. | Use browser time zone | [`time.displayZone`](/admin/settings#time.displayZone) |
| **Comparison time zone** | An optional second zone to compare local times while scheduling. | None | [`time.comparisonZone`](/admin/settings#time.comparisonZone) |

A **Format preview** line shows how a sample date and time look with your choices.

## Troubleshooting

| What you see | Why | What to do |
|---|---|---|
| New mail does not notify you. | **Show notifications for new mail** is off by default, and a native notification also needs browser permission. | Turn it on, then allow notifications for this site in the browser. |
| A notification setting is dimmed with **Set by your administrator**. | An administrator fixed it. | Ask your administrator. |
| **Register background notifications** is unavailable. | New-mail notifications are off, your administrator has not allowed push for your account, or this browser is already registered. | Turn on **Show notifications for new mail** first. If it is still unavailable, ask your administrator. |

## Related

- [Personal settings](./index.md)
- [Keyboard shortcuts](/user/keyboard-shortcuts)
- [Notifications](/user/notifications)
- [Sync and offline access](/user/offline-sync)

### Changes on other devices

Saved account settings, appearance, and workspace/folder preferences update other open sessions through the existing WebSocket stream (EventSource fallback). Administrator setting defaults and locks also trigger a fresh read of each session's effective settings. Only an opaque invalidation is pushed; settings are read again using the receiving session's verified account and storage authority.

Rapid changes are combined. Unsaved local edits are retained, and applying a remote appearance does not save it back to the server. Reconnecting or bringing a browser tab back into focus checks for changes missed while it was disconnected. This is live session propagation within a server process; a separate server process catches up on reconnect/focus rather than through this process-local channel.


Message previews can be **Off** or **1–5 lines**. Row heights and virtual scrolling
adapt to the selected count. Account-level message fonts, spacing, highlighting,
default filters and reading-pane dimensions can be changed here; individual
folders may override them within administrator policy.
