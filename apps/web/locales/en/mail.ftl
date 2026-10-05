# Mailwoman — mail core UI strings (source locale: en).
#
# Owned by t8-e1 (web mail core: mailbox screen, message list, reader, composer,
# ribbon, outbox, sweep, tabs). Lazily loaded via `loadCatalog('mail')`. Message
# ids are kebab-case, `mail-` prefixed. User-controlled values interpolated into
# these strings (subjects, sender/display names, filenames) are wrapped with
# `isolate()` at the call site — see docs/i18n.md "RTL & bidi".

# -- Mailbox shell -----------------------------------------------------------
mail-brand = Mailwoman
mail-nav-settings = Settings
mail-compose = Compose
mail-nav-mailboxes = Mailboxes
mail-nav-attachments = Attachments
mail-nav-outbox = Outbox
mail-nav-sharing = Share folder
mail-nav-apps = Apps
mail-logout = Log out
mail-offline = Offline
mail-module-loading = Loading { $module }…

# -- Search ------------------------------------------------------------------
mail-search = Search
mail-search-label = Search mail
mail-search-placeholder = Search mail — from:alice subject:invoice larger:1mb
mail-search-clear = Clear

# -- Message list ------------------------------------------------------------
mail-list-label = Messages
mail-loading = Loading messages…
mail-empty = No messages
mail-unknown-sender = (unknown sender)
mail-no-subject = (no subject)
mail-pinned = Pinned
mail-has-attachment = Has attachment
mail-unread = Unread
# Announced row summary for screen readers (position within the whole list).
mail-row-position = Message { $pos } of { $total }

# -- Conversation threading (W2) ---------------------------------------------
mail-thread-count = { $count ->
    [one] { $count } message
   *[other] { $count } messages
}
mail-thread-expand = Expand conversation, { $count ->
    [one] { $count } message
   *[other] { $count } messages
}
mail-thread-collapse = Collapse conversation, { $count ->
    [one] { $count } message
   *[other] { $count } messages
}

# -- View options / reading pane (W3) ----------------------------------------
mail-view-options = View options
mail-reading-pane = Reading pane
mail-reading-pane-right = Right
mail-reading-pane-bottom = Bottom
mail-reading-pane-off = Off

# -- Inbox tabs --------------------------------------------------------------
mail-inbox-focused-enable = Focused inbox
mail-inbox-filter = Inbox filter
mail-inbox-focused = Focused
mail-inbox-other = Other
mail-inbox-turn-off = Turn off

# -- Sub-tab strip -----------------------------------------------------------
mail-subtabs-label = Open tabs
mail-subtab-pin = Pin { $title }
mail-subtab-unpin = Unpin { $title }
mail-subtab-tearoff = Open { $title } in a new window
mail-subtab-close = Close { $title }

# -- Per-row actions ---------------------------------------------------------
mail-pin = Pin
mail-unpin = Unpin
mail-snooze = Snooze
mail-snooze-menu = Snooze until
mail-snooze-later = Later today
mail-snooze-tomorrow = Tomorrow
mail-snooze-next-week = Next week
mail-unsnooze = Unsnooze
mail-label = Label
mail-labels-menu = Labels
mail-flag = Flag for follow-up
mail-clear-flag = Clear follow-up
mail-archive = Archive
mail-delete = Delete
# Opens the row's actions where they cannot be revealed by hovering (touch, phone).
mail-more-actions = More actions

# -- Reader ------------------------------------------------------------------
mail-reader-label = Message
mail-reader-empty = Select a message to read
mail-back = Back
mail-reader-from = From: { $addr }
mail-reader-to = To: { $addr }
mail-reader-actions = Message actions
mail-spam = Spam
mail-export = Export
mail-sweep-sender = Sweep sender
mail-message-body = Message body
mail-no-content = No content
mail-sanitizing = Sanitizing…
mail-attachments = Attachments
mail-attachment-unnamed = (unnamed)
mail-attachment-open = Attachment { $name }
mail-attachment-close = Close attachment
mail-attachment-loading = Loading attachment…

# -- Reader: decrypt ---------------------------------------------------------
mail-encrypted-region = Encrypted message
mail-encrypted-note = 🔒 This message is end-to-end encrypted. Unlock it on this device to read it.
mail-key-passphrase = Key passphrase
mail-decrypt = Decrypt
mail-decrypting = Decrypting…
mail-decrypt-no-key = No private key is available to decrypt this message.
mail-decrypt-failed = Decryption failed

# -- Composer ----------------------------------------------------------------
mail-compose-label = Compose message
mail-compose-title = New message
mail-compose-close = Close
mail-compose-from = From
mail-compose-from-default = Default
mail-compose-to = To
mail-compose-to-placeholder = someone@example.org
mail-compose-contact-suggestions = Contact suggestions
mail-compose-subject = Subject
mail-compose-body = Body
mail-compose-attachments = Attachments
mail-compose-remove-attachment = Remove { $name }
mail-compose-attach-nextcloud = Attach from Nextcloud
mail-compose-close-nextcloud = Close Nextcloud
# New-file blob upload (attach a local file from this device)
mail-compose-attach-file = Attach a file
mail-compose-uploading = Uploading…
mail-compose-upload-unavailable = File upload is not available right now.
mail-compose-upload-failed = Could not upload { $name }.
mail-compose-upload-too-large = { $name } is { $size } MB. The maximum upload size is { $max } MB.
mail-compose-send-later = Send later
mail-compose-cancel = Cancel
mail-compose-send = Send
mail-compose-schedule = Schedule
mail-compose-sending = Sending…
mail-compose-encrypted-subject = Encrypted message
mail-compose-dlp-blocked = Sending is blocked by a data-loss-prevention rule (see the warning above).
mail-compose-send-failed = Send failed
mail-compose-send-later-past = Choose a send time in the future, or clear the field to send now.
# Refusals raised by the send itself. The composer stays open and shows them.
mail-send-not-signed-in = You are not signed in.
mail-send-no-draft-folder = This account has no folder to hold the draft.
mail-send-no-recipient = Add at least one recipient.
mail-send-bad-recipient = { $count ->
    [one] This is not an email address: { $addresses }
   *[other] These are not email addresses: { $addresses }
}
mail-send-later-past = The send time is not in the future. Nothing was sent.
mail-send-draft-refused = The server refused this message: { $reason }. Nothing was sent.
mail-send-submission-refused = The server refused to send this message: { $reason }. Nothing was sent.
# Signing-key unlock (sign-on-send)
mail-compose-sign-unlock-title = Unlock signing key
mail-compose-sign-unlock-note = Enter your key passphrase to sign this message. It stays unlocked for this composer only.
mail-compose-sign-unlock = Unlock
mail-compose-sign-unlocking = Unlocking…
mail-compose-sign-no-key = No signing key is available on this device.
mail-compose-sign-unlock-failed = Could not unlock the signing key.
mail-compose-sign-unlock-required = Unlock your signing key to send this signed message.

# Rich-text editor (W1)
mail-compose-format-plain = Plain text
mail-compose-format-rich = Rich text
mail-compose-rt-toolbar = Formatting
mail-compose-rt-bold = Bold
mail-compose-rt-italic = Italic
mail-compose-rt-underline = Underline
mail-compose-rt-strike = Strikethrough
mail-compose-rt-bullet = Bulleted list
mail-compose-rt-ordered = Numbered list
mail-compose-rt-quote = Quote
mail-compose-rt-link = Link
mail-compose-rt-link-placeholder = https://example.org
mail-compose-rt-link-apply = Apply

# Signature picker (W12)
mail-compose-signature = Signature
mail-compose-signature-none = Add a signature…

# Recall (W10)
mail-compose-recall = Recall
mail-compose-recall-action = Recall
mail-compose-recall-holding = Held, waiting to send
mail-compose-recall-scheduled = Scheduled for { $when }

# Drafts drawer (W9)
mail-compose-drafts = Drafts
mail-compose-drafts-empty = No saved drafts
mail-compose-drafts-delete = Delete draft
mail-compose-drafts-no-recipient = (no recipient)

# -- Outbox ------------------------------------------------------------------
mail-outbox-label = Outbox
mail-outbox-refresh = Refresh
mail-outbox-empty = Nothing waiting to send
mail-outbox-send-now = Send now
mail-outbox-cancel = Cancel
mail-outbox-scheduled = Scheduled
mail-outbox-holding = Sending soon
mail-outbox-sent = Sent
mail-outbox-canceled = Canceled
# A held submission is one the server will not send until the mailbox owner
# releases it here (EmailSubmission `mailwomanHold: "manual"`). $name is the
# API key's prefix, or the connected app's client id.
mail-outbox-held = Held
mail-outbox-failed = Not sent
mail-outbox-release = Release
mail-outbox-discard = Discard
mail-outbox-held-note = Held. Not sent until released.
mail-outbox-held-note-api-key = Held — created by API key { $name }. Not sent until released.
mail-outbox-held-note-oauth = Held — created by the connected app { $name }. Not sent until released.
mail-outbox-origin-api-key = Created by API key { $name }
mail-outbox-origin-oauth = Created by the connected app { $name }
mail-outbox-to = To { $recipients }
mail-outbox-no-subject = (no subject)
mail-outbox-attempts = Tried { $count } times. Last error: { $error }
mail-outbox-error = Error: { $error }
mail-outbox-toast-canceled = Send canceled
mail-outbox-toast-discarded = Discarded. The message was not sent.
mail-outbox-toast-sent = Sent
mail-outbox-toast-retrying = Not sent yet: { $error }. It will be tried again.
mail-outbox-toast-failed = Not sent: { $error }
mail-outbox-toast-not-released = Could not send: { $error }
mail-outbox-toast-not-canceled = Could not cancel: { $error }

# -- Network status and the offline queue ------------------------------------
# Raised by the store (state/store.ts, state/slices/offline.ts) and listed by
# the notice above the message list (components/OfflineQueueNotice.tsx).
mail-net-lost = Connection lost — retrying…
mail-net-back = Back online
mail-queue-sent = { $count ->
    [one] Sent { $count } queued change
   *[other] Sent { $count } queued changes
}
mail-queue-failed = { $count ->
    [one] { $count } queued change failed
   *[other] { $count } queued changes failed
}
mail-queue-notice-label = Changes that could not be applied
mail-queue-notice-title = { $count ->
    [one] { $count } change made offline could not be applied
   *[other] { $count } changes made offline could not be applied
}
mail-queue-kind-send = Send “{ $subject }”
mail-queue-kind-send-untitled = Send a message with no subject
mail-queue-kind-move = Move a message
mail-queue-kind-flag = Change a label or flag
mail-queue-kind-draft = Save a draft
mail-queue-kind-pim = Calendar, task, note or contact change
mail-queue-reason = Reason: { $reason }
mail-queue-reason-unknown = The server did not say why.
mail-queue-retry = Retry
mail-queue-discard = Discard
mail-queue-retry-item = Retry: { $what }
mail-queue-discard-item = Discard: { $what }
mail-queue-discarded = Discarded. The change was not applied.

# -- Sweep dialog ------------------------------------------------------------
mail-sweep-label = Sweep messages
mail-sweep-title = Sweep { $sender }
mail-sweep-all = Delete all from this sender
mail-sweep-keep-latest = Keep the latest, delete the rest
mail-sweep-older-than = Delete older than N days
mail-sweep-block = Delete all and block this sender
mail-sweep-days = Days
mail-sweep-count = { $count ->
    [one] { $count } message will move to Trash
   *[other] { $count } messages will move to Trash
}
mail-sweep-cancel = Cancel
mail-sweep-run = Sweep { $count }
mail-sweeping = Sweeping…

# -- Undo toast --------------------------------------------------------------
mail-undo-dismiss = Dismiss
mail-undo-action = Undo
mail-undo-send-cancel = Cancel
mail-undo-failed = Could not undo

# -- Toasts raised by mail actions (state/slices/mail.ts) ----------------------
mail-toast-label-added = Label added
mail-toast-label-removed = Label removed
mail-toast-pinned = Pinned
mail-toast-unpinned = Unpinned
mail-toast-snoozed = Snoozed
mail-toast-follow-up-set = Follow-up set
mail-toast-follow-up-cleared = Follow-up cleared
mail-toast-archived = Archived
mail-toast-trashed = Moved to Trash
mail-toast-marked-spam = Marked as spam
mail-toast-moved-to = Moved to { $folder }
mail-toast-moved = Moved to folder
mail-toast-no-archive-folder = No Archive folder
mail-toast-no-trash-folder = No Trash folder
mail-toast-no-spam-folder = No Spam folder
mail-toast-sweep-nothing = Nothing to sweep
mail-toast-swept = { $count ->
    [one] Swept { $count } message
   *[other] Swept { $count } messages
}
mail-toast-load-more-failed = Could not load more messages
mail-toast-export-nothing = Nothing to export
mail-toast-exported = Exported .eml
mail-toast-export-failed = Export failed
mail-toast-send-queued = Queued — will send when back online
mail-toast-send-scheduled = Scheduled to send
mail-toast-sent = Message sent
mail-toast-send-canceled = Send canceled

# -- Ribbon (Outlook-style layout preset) ------------------------------------
mail-ribbon-label = Ribbon
mail-ribbon-tab-home = Home
mail-ribbon-tab-view = View
mail-ribbon-tab-folder = Folder
mail-ribbon-expand = Expand
mail-ribbon-collapse = Collapse
mail-ribbon-group-new = New
mail-ribbon-compose = Compose
mail-ribbon-group-session = Session
mail-ribbon-logout = Log out
mail-ribbon-group-theme = Theme
mail-ribbon-group-density = Density
mail-ribbon-group-settings = Settings
mail-ribbon-more = More…
mail-ribbon-group-folders = Folders
mail-density-compact = compact
mail-density-cozy = cozy
mail-density-relaxed = relaxed
