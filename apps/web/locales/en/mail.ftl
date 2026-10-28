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
mail-answered = Answered
mail-forwarded = Forwarded
mail-unread = Unread
mail-flagged = Flagged
# Announced on a selected row; the row also shows a check mark.
mail-selected = Selected
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

# -- Selection and bulk actions (the list toolbar while messages are selected) --
mail-bulk-label = Selected messages
mail-bulk-count = { $count } selected
mail-mark-read = Mark read
mail-mark-unread = Mark unread
# Sets the message's IMAP flag (\Flagged). Not the same as a follow-up reminder.
mail-flag-message = Flag
mail-unflag-message = Remove flag
mail-move-to = Move to folder
mail-move-to-placeholder = Move to…
mail-select-all = Select all
mail-clear-selection = Clear selection

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
mail-reader-cc = Cc: { $addr }
mail-reader-actions = Message actions
mail-reply = Reply
mail-reply-all = Reply all
mail-forward = Forward
mail-reply-failed = The message could not be prepared. Try again.
mail-reply-unthreaded = The ID of the original message could not be read. This reply will not be threaded with it.
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
mail-attachment-failed = This attachment could not be downloaded. Try again when the connection is available.

# -- Reader: decrypt ---------------------------------------------------------
mail-encrypted-region = Encrypted message
mail-encrypted-note = This message is end-to-end encrypted. Unlock it on this device to read it.
mail-key-passphrase = Key passphrase
mail-decrypt = Decrypt
mail-decrypting = Decrypting…
mail-decrypt-no-key = No private key is available to decrypt this message.
mail-decrypt-failed = Decryption failed

# -- Reply and forward: text written into the outgoing message --------------
# The line above the quoted original of a reply.
mail-quote-attribution = On { $date }, { $name } wrote:
mail-quote-attribution-undated = { $name } wrote:
# The header block above a forwarded original.
mail-forward-heading = ---------- Forwarded message ----------
mail-forward-from = From: { $addr }
mail-forward-date = Date: { $date }
mail-forward-subject = Subject: { $subject }
mail-forward-to = To: { $addr }
mail-forward-cc = Cc: { $addr }

# -- Composer ----------------------------------------------------------------
mail-compose-label = Compose message
mail-compose-title = New message
mail-compose-title-reply = Reply
mail-compose-title-reply-all = Reply all
mail-compose-title-forward = Forward
mail-compose-close = Close
mail-compose-from = From
mail-compose-from-default = Default
mail-compose-to = To
mail-compose-to-placeholder = someone@example.org
mail-compose-contact-suggestions = Contact suggestions
mail-compose-cc = Cc
mail-compose-bcc = Bcc
mail-compose-show-cc-bcc = Add Cc or Bcc
mail-compose-bcc-encrypted = An encrypted message lists the key of every recipient, so a Bcc recipient would be visible to the others. Remove the Bcc recipients or turn encryption off.
mail-compose-plaintext-confirm-title = Send decrypted text without encryption?
mail-compose-plaintext-confirm-body = This message quotes text from an encrypted message. It is not set to be encrypted, so the quoted text would be sent unprotected.
mail-compose-plaintext-confirm-send = Send unencrypted
mail-compose-plaintext-confirm-cancel = Go back
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
mail-compose-options = Send options
mail-compose-request-receipt = Request a read receipt
mail-compose-request-receipt-hint = The recipient's mail program may ask them, send one without asking, or ignore the request.
mail-compose-track-opens = Record when this message is opened
mail-compose-track-opens-hint = A 1×1 image that loads from this server is added to the message. Many mail programs block images or load them in advance, so a recorded open does not prove the message was read, and no record does not prove it was not. The recipient is not told.
mail-compose-image-type = { $name } cannot be placed in the text. Use a PNG, JPEG, GIF or WebP image.
# Refusals raised by the send itself. The composer stays open and shows them.
mail-send-not-signed-in = You are not signed in.
mail-send-no-draft-folder = This account has no folder to hold the draft.
mail-send-no-recipient = Add at least one recipient.
mail-send-bad-recipient = { $count ->
    [one] This is not an email address: { $addresses }
   *[other] These are not email addresses: { $addresses }
}
mail-send-later-past = The send time is not in the future. Nothing was sent.
mail-send-tracking-failed = Opens cannot be recorded right now, so nothing was sent. Clear "Record when this message is opened" to send without it.
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
mail-compose-rt-image = Insert an image

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
mail-toast-pinned = Pinned message “{ $target }”
mail-toast-unpinned = Unpinned message “{ $target }”
mail-toast-snoozed = Snoozed
mail-toast-follow-up-set = Follow-up set
mail-toast-follow-up-cleared = Follow-up cleared
mail-toast-archived = Archived
mail-toast-trashed = Moved to Trash
mail-toast-marked-spam = Marked as spam
mail-toast-moved-to = Moved to { $folder }
mail-toast-moved = Moved to folder
mail-toast-archived-count = { $count } messages archived
mail-toast-trashed-count = { $count } messages moved to Trash
mail-toast-marked-spam-count = { $count } messages marked as spam
mail-toast-moved-count = { $count } messages moved
mail-toast-moved-to-count = { $count } messages moved to { $folder }
mail-toast-marked-read = { $count ->
    [one] Marked as read
   *[other] { $count } messages marked as read
}
mail-toast-marked-unread = { $count ->
    [one] Marked as unread
   *[other] { $count } messages marked as unread
}
mail-toast-flagged = { $count ->
    [one] Flagged
   *[other] { $count } messages flagged
}
mail-toast-unflagged = { $count ->
    [one] Flag removed
   *[other] Flag removed from { $count } messages
}
mail-toast-move-failed = Could not move the messages. Nothing was moved.
mail-toast-update-failed = Could not change the messages. Nothing was changed.
mail-toast-update-refused = { $count ->
    [one] The server refused the change for one message
   *[other] The server refused the change for { $count } messages
}
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
mail-toast-tracking-not-bound = The message was sent, but its opens will not be recorded.
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

# -- Message security level: the three-position switch in the reader toolbar
# (viewers/MaxSecuritySwitch.tsx). The hint is the tooltip of each position.
mail-maxsec-label = Message security level
mail-maxsec-full = Full
mail-maxsec-full-hint = Sanitized HTML with images and styling.
mail-maxsec-no-media = No media
mail-maxsec-no-media-hint = Sanitized HTML with all images and media removed.
mail-maxsec-plain-text = Plain text
mail-maxsec-plain-text-hint = No HTML rendered — message shown as plain text only.

mail-toast-tab-pinned = Pinned tab “{ $target }”
mail-toast-tab-unpinned = Unpinned tab “{ $target }”

mail-workspace-message = Message
mail-workspace-layout = Workspace layout
mail-workspace-tabs = Tabs
mail-workspace-side-by-side = Side by side
mail-workspace-stack = Stacked
mail-workspace-refresh = Refresh folder
mail-workspace-load-error = This message could not be loaded. Close the tab and try again.

mail-resize-folders = Resize folders sidebar
mail-resize-list = Resize message list
mail-resize-tiles = Resize workspace panes
mail-show-folders = Show folders sidebar
mail-hide-folders = Hide folders sidebar
mail-compact-folders = Compact folder icons
mail-toggle-list = Show or hide message list
mail-toggle-reader = Show or hide reading pane

mail-folder-counts = { $unread } unread, { $total } messages

mail-print = Print / Save as PDF
mail-print-help = Print this message or choose Save as PDF in the print dialog. External images stay blocked.

mail-file-details = File details and fingerprint
mail-file-type = File type
mail-file-extension = Extension
mail-file-detected = Detected signature
mail-file-unknown = No recognized signature
mail-file-size = Size
mail-file-size-bytes = { $count } bytes
mail-file-hash-unavailable = Fingerprint unavailable (requires a secure browser and a file up to 64 MiB).
mail-file-fingerprint-help = A fingerprint identifies these exact bytes. It does not prove that the file is safe or who created it.
mail-file-warning-executable = This file may execute programs or change system settings. Only download it if you trust its source.
mail-file-warning-macro = This document can contain macros. Do not enable macros unless you have verified the sender and purpose.
mail-file-warning-disguisedName = The filename may disguise its real extension. Check the file details before downloading.
mail-file-warning-typeMismatch = The detected file signature does not match its declared type. The automatic preview has been blocked.
mail-file-warning-activeDocument = This format can contain active content. Its preview is isolated; opening a downloaded copy may behave differently.
mail-file-preview-blocked = Automatic preview is disabled for this file type or type mismatch.

mail-quote-header-from = From:
mail-quote-header-sent = Sent:
mail-quote-header-to = To:
mail-quote-header-cc = Cc:
mail-quote-header-subject = Subject:

mail-spam-score = Spam analysis
mail-spam-score-loading = Checking spam headers…
mail-spam-score-unavailable = Spam score unavailable
mail-spam-score-unverified = Reported by message headers, not independently verified. Sender-supplied headers can be forged.
mail-spam-score-positive = The header reports spam.
mail-spam-score-negative = The header reports a non-spam result.
mail-spam-score-unknown = No verdict was supplied with this score.

mail-external-recipients = External recipients
mail-external-count = { $count } recipient(s) outside trusted domains
mail-external-help = Check the To, Cc and Bcc recipients before sending confidential information. Trust is based on the selected sending identity and your effective domain policy.
mail-external-reviewed = I have checked these external recipients.
mail-external-confirm-required = Review and confirm the external recipients before sending.
mail-quick-reply = Quick reply
mail-quick-reply-help = Offer a compact reply tab with recipients already filled in. Off by default. The normal safety checks and send options still apply.
mail-quick-reply-expand = Edit recipients and subject

mail-attachment-policy-loading = Checking attachment policy…
mail-attachment-policy-unavailable = Attachment policy could not be checked. Try again before opening this file.
mail-attachment-policy-blocked = Your administrator has blocked this attachment or its original download.
mail-attachment-policy-warning = Your administrator recommends caution with this attachment. Only continue if you trust its source.
mail-attachment-policy-acknowledge = I understand — continue
mail-attachment-policy-no-preview = Attachment previews are disabled by your administrator. Original downloads may still be available.

mail-time-sent = Sent
mail-time-received = Received
mail-time-unknown = Not provided
mail-time-comparison = Time in { $zone }
mail-time-privacy = The comparison zone is your selection. Message timestamps do not identify anyone's location.
mail-time-event-comparison = Event time comparison
mail-time-ambiguous = This local time occurs twice when clocks change. Both possible instants are shown; check the event's intended offset before saving.
mail-bulk-add-label = Add label
mail-select-messages = Select messages
mail-bulk-remove-label = Remove label
mail-mark-spam = Mark as spam
mail-bulk-forward = Forward selected messages as attachments
mail-bulk-download = Download selected messages as mbox
mail-bulk-forward-subject = Forwarded messages ({ $count })
mail-bulk-transfer-failed = The selected messages could not be prepared. Select up to 100 messages totaling at most 32 MiB, and check your account's export permissions.
mail-toast-labels-added = Added label { $label } to { $count } messages
mail-toast-labels-removed = Removed label { $label } from { $count } messages

mail-action-help-reply = Reply to the sender in a new compose tab.
mail-action-help-reply-all = Reply to the sender and the other recipients in a new compose tab.
mail-action-help-forward = Review and forward this message to recipients you choose.
mail-action-help-pin = Keep this message pinned for easy access, or remove its pin.
mail-action-help-read = Change whether this message is marked as read. You can undo the change.
mail-action-help-flag = Set or remove the message flag for follow-up.
mail-action-help-archive = Move this message to the configured archive folder.
mail-action-help-delete = Move this message to the trash folder. You can undo the move.
mail-action-help-spam = Mark this message as junk and move it to the configured spam folder.
mail-action-help-export = Open the export page to download this message in a format you choose.
mail-action-help-sweep = Review actions for other messages from this sender before applying them.

mail-empty-trash = Empty trash
mail-trash-help = Review and permanently delete messages currently in Trash. This cannot be undone.
mail-trash-preparing = Preparing a snapshot of the messages in Trash…
mail-trash-prepare-failed = The Trash preview could not be prepared. Check your connection and reopen it. No messages were deleted.
mail-trash-confirm = Permanently delete { $count } messages from Trash? This cannot be undone. Messages arriving after this preview are excluded; messages moved out of Trash are skipped.
mail-trash-limit = This operation includes up to { $count } messages. More remain in Trash; review another operation afterward to remove them.
mail-trash-permanent = Delete permanently
mail-trash-stop = Stop after the current request
mail-trash-progress = Deleted { $deleted }; skipped { $skipped } messages no longer in Trash; { $remaining } still in this preview.
mail-trash-stopped = Deletion stopped. Confirmed deleted: { $deleted }; skipped: { $skipped }; remaining: { $remaining }; outcome unknown: { $uncertain }. Check Trash before starting another operation.
mail-links-title = Message links
mail-links-review = Review links ({ $count })
mail-links-help = Message links open only after you review their destination here. No destination is fetched automatically.
mail-links-inert = Use Review links above the message to inspect and open this destination.
mail-links-limit = The first 100 distinct supported destinations are listed. Additional links remain inactive in the message.
mail-links-destination = Destination to open
mail-links-external = This destination comes from the message sender. Opening it leaves Mailwoman; review the address before continuing.
mail-links-open = Open in a new tab
mail-links-compose = Review a new message
mail-links-warning-http = This address uses unencrypted HTTP.
mail-links-warning-address = This destination uses an IP address instead of a domain name.
mail-links-warning-international = This domain contains an internationalized name. Check its ASCII spelling carefully.
mail-links-warning-mismatch = The visible web address names a different host from the actual destination.
mail-links-warning-direction = The link text contains direction-changing characters that may disguise its address.

mail-outbox-uncertain = Delivery status unknown
mail-outbox-uncertain-note = Sending began, but delivery could not be confirmed. This message will not be resent automatically. Check Sent or confirm with the recipient before sending again.

mail-conversation-title = View conversation
mail-conversation-help = Browse this account's available conversation messages across folders. Opening a message does not change your folder selection.
mail-conversation-count = Showing { $loaded } of { $total } conversation messages.
mail-conversation-other-folder = Another folder
mail-conversation-loading = Loading conversation…
mail-conversation-missing = { $count } message(s) are no longer available.
mail-conversation-error = The conversation could not be loaded. Try again.
mail-conversation-more = Load more messages
mail-conversation-limit = This view is limited to { $count } messages.

mail-compose-request-delivery = Request delivery notifications
mail-compose-request-delivery-hint = Ask receiving mail servers to report delivery, failure, or delay. This does not request or prove that anyone read the message. Returned reports include headers, not the full body.
mail-compose-delivery-unavailable = Delivery notifications are unavailable or disabled by your administrator for this account. An explicitly requested notification will not be silently removed.

mail-compose-account-changed = This composition belongs to a different sign-in. Sign in to the correct account and reopen the draft.
mail-context-open-message = Open message
mail-context-open-folder = Open folder
mail-context-download = Download message
mail-context-refresh = Refresh folder
mail-context-folder-settings = Folder settings
mail-context-failed = The action could not be completed. Try again.
mail-sharing-unavailable = Folder sharing is unavailable or you no longer have permission to manage it.

mail-trash-cancelling = Stopping after the current request. Confirmed deleted: { $deleted }; skipped: { $skipped }; remaining: { $remaining }. Deletions already accepted cannot be undone.
mail-trash-already-running = Trash deletion is already running. Use its progress notification to stop future batches.

mail-context-search-sender = Search this sender
mail-context-search-subject = Search the same subject
mail-drag-too-many = Drag up to { $count } messages at a time. Select a smaller group before moving.

mail-subtab-reorder-help = Drag to reorder tabs, or press Ctrl+Shift+Left or Right on the active tab.

mail-compose-ribbon = Editor ribbon
mail-compose-ribbon-format = Format text
mail-compose-ribbon-insert = Insert
mail-compose-font = Font
mail-compose-font-size = Font size
mail-compose-font-inherit = Default
mail-compose-undo = Undo
mail-compose-redo = Redo
mail-workspace-limit = Close a tab before opening another. Up to 32 tabs can be retained.
mail-compose-private-storage-unavailable = Local draft recovery is unavailable for this session. Keep this editor open until you send or copy your message.
mail-offline-storage-unavailable = Offline changes are unavailable until this session has verified storage authority. Reconnect and sign in again.

mail-draft-status-idle = Not saved to account yet
mail-draft-status-pending = Unsaved changes
mail-draft-status-saving = Saving to Drafts…
mail-draft-status-saved = Saved to account Drafts
mail-draft-status-error = Draft was not saved. Your local recovery copy remains available.
mail-draft-status-uncertain = Save confirmation is missing. Check Drafts before creating another copy; automatic saving is paused.
mail-draft-status-conflict = This draft changed on another device. Reopen its account copy before continuing.
mail-draft-status-unsupported = This account cannot currently confirm saves to its Drafts folder.
mail-draft-protected = Encrypted or decrypted quoted content is not saved as plaintext. Any earlier account draft remains until you explicitly discard it.
mail-draft-save-now = Save to Drafts
mail-draft-save-failed = Could not save this draft to your account. Check the composer before closing it.
mail-draft-uncertain = The draft may have been saved. Automatic retries are paused; check your Drafts folder.
mail-draft-conflict = Another device changed this draft. Your edits have not overwritten its copy.
mail-draft-cleanup-pending = The new draft was saved, but its previous copy could not be removed. Review Drafts before saving again.
mail-draft-account-list = Account drafts
mail-draft-loading = Loading account drafts…
mail-draft-close-recovery = Close and keep local recovery only
mail-draft-discard-account = Discard account draft
mail-draft-discard-pending = Draft removal could not be confirmed. Its account record remains; retry Discard after reviewing the Drafts folder.
mail-draft-limit = This account already has 100 saved drafts. Discard an unwanted account draft before saving a new one.
mail-draft-quota-help = Up to 100 account drafts are retained. Discard removes the confirmed upstream copy before releasing its slot.
mail-draft-discard-confirm = Confirm draft removal
mail-draft-discard-confirm-detail = Permanently remove “{ $subject }” from account Drafts? Closing the composer instead keeps the draft.
# The discard confirmation dialog (t42): Keep editing is the safe default.
mail-draft-discard-title = Discard this draft?
mail-draft-keep-editing = Keep editing
mail-draft-discard-action = Discard
mail-draft-close-title = Discard unsaved changes?
mail-draft-close-protected = This message holds encrypted or decrypted content, so it is not saved as a draft. Closing it discards these changes. An earlier account draft, if any, stays in Drafts.
mail-draft-close-unstored = Drafts cannot be saved in this session, so closing this message discards it.

mail-workspace-view-mode = View arrangement
mail-workspace-tabs-help = Show one active view while keeping the other tabs open.
mail-workspace-side-by-side-help = Show two selected views next to each other. Drag a tab onto either view to replace it.
mail-workspace-stack-help = Show two selected views above and below each other. Drag a tab onto either view to replace it.
mail-workspace-masonry-side-by-side = Masonry side by side
mail-workspace-masonry-side-by-side-help = Tile all open views in staggered columns. Small screens show the active tab.
mail-workspace-masonry-stack = Masonry stacked
mail-workspace-masonry-stack-help = Tile all open views in staggered rows. Small screens show the active tab.
mail-workspace-view-first = First view
mail-workspace-view-second = Second view
mail-workspace-view-empty = Drag an open tab here to choose this view.

mail-popup-blocked = Allow pop-up windows for Mailwoman to detach this tab. Your tab remains here.
mail-popup-unavailable = The detached window could not be loaded. Your tab and unsaved changes remain in the main window. Try again after refreshing the app.
mail-popup-reattach = Reattach to main window
mail-popup-action-reattached = Returned to the main window to complete this action.
mail-popup-detached = This tab is open in a separate window. Closing that window returns it here.

mail-compose-from-custom = Another authorized address…
mail-compose-from-alias = From address for this message
mail-compose-from-alias-help = Use an alias you are authorized to send from. This changes only this draft; authorization is checked before sending.
mail-compose-from-refused = This address is not authorized for sending from this account.
mail-compose-from-unavailable = Could not verify this sender address. Your message has not been sent.
mail-compose-from-invalid = Enter one valid email address for the sender.

mail-compose-from-toggle = Show or hide the sender for this message

mail-compose-ribbon-security = Security
mail-compose-ribbon-delivery = Delivery
mail-compose-ribbon-recipients = Recipients
mail-compose-ribbon-message-type = Message type

mail-compose-recall-help = Cancel messages still held for undo or scheduled delivery. Messages already delivered cannot be recalled.
mail-compose-recall-loading = Checking queued messages…
mail-compose-recall-empty = No queued messages are available to cancel.
mail-compose-recall-failed = Could not confirm the queue status. Close and reopen this panel to refresh it before another action.

mail-compose-importance = Importance
mail-compose-importance-high = High
mail-compose-importance-normal = Normal
mail-compose-importance-low = Low

mail-insert-account-title = Insert account item
mail-insert-account-help = Attach a calendar export, contact business card, or original message from this account.
mail-insert-account-close = Close account item picker
mail-insert-account-kind = Item type
mail-insert-account-event = Calendar event (.ics)
mail-insert-account-calendar = Calendar (.ics)
mail-insert-account-contact = Contact business card (.vcf)
mail-insert-account-message = Original message (.eml)
mail-insert-account-search = Filter listed items
mail-insert-account-selection = Select an item
mail-insert-account-scope = Only items from the current account are listed. Original messages use the server export policy.
mail-insert-account-limited = Showing up to 100 items. Filtering searches this list only.
mail-insert-account-unavailable = No supported account exports are available.
mail-insert-account-empty = No matching items.
mail-insert-account-attach = Attach selected item
mail-insert-account-loading = Loading account item…
mail-insert-account-error = The item could not be attached. Check your access and export policy, then try again. Large calendars may need individual event exports.

mail-compose-html-source = HTML source
mail-compose-html-help = Edit HTML directly. Only supported email formatting is saved and sent; scripts, external images, and unsafe markup are removed.

mail-compose-check-names = Check names
mail-compose-check-names-help = Resolve exact names from your loaded contacts and check address formatting. This does not verify mailbox delivery.
mail-compose-names-unresolved = Choose a full email address for these unresolved or ambiguous names: { $names }
mail-compose-names-checked = Recipient formatting checked; { $count } contact name(s) resolved. Mailbox delivery has not been verified.

mail-compose-ribbon-table = Table
# Captions of the composer ribbon's 3-row groups (t38); also the folded group's dropdown text.
mail-compose-group-clipboard = Clipboard
mail-compose-group-paragraph = Paragraph
mail-compose-group-editing = Editing
mail-compose-group-assist = Writing assistant
mail-editor-counts = Editor statistics
mail-editor-words = { $count } words
mail-editor-paragraphs = { $count } paragraphs
mail-editor-characters = { $count } characters
mail-editor-selection-required = Select suitable text or table cells before using this action.
mail-editor-clipboard-unavailable = Clipboard access was unavailable. Use your browser’s Copy, Cut or Paste shortcut.
mail-editor-selection-changed = The selection changed. Select the intended text and try again.
mail-editor-clipboard = Clipboard and formatting
mail-editor-copy = Copy
mail-editor-cut = Cut
mail-editor-paste = Paste
mail-editor-copy-format = Copy formatting
mail-editor-apply-format = Apply formatting
mail-editor-style = Paragraph style
mail-editor-text-color = Font color
mail-editor-clear-color = Default color
mail-editor-highlight = Highlight
mail-editor-clear-highlight = Clear highlight
mail-editor-paragraph = Paragraph options
mail-editor-alignment = Alignment
mail-editor-line-height = Line height
mail-editor-space-before = Space before
mail-editor-space-after = Space after
mail-editor-indent = Indent
mail-editor-shading = Paragraph shading
mail-editor-clear-shading = Clear shading
mail-editor-case = Change case
mail-editor-sort = Sort selected paragraphs
mail-editor-sort-alpha = Alphabetical
mail-editor-sort-numeric = Numeric order
mail-editor-table-rows = Table rows
mail-editor-table-columns = Table columns
mail-editor-table-insert = Insert table
mail-editor-table-limits = Up to 20 rows, 10 columns and 200 cells per table; at most 10 tables. Nested tables are not supported.
mail-editor-table-row-before = Row above
mail-editor-table-row-after = Row below
mail-editor-table-row-delete = Delete row
mail-editor-table-column-before = Column before
mail-editor-table-column-after = Column after
mail-editor-table-column-delete = Delete column
mail-editor-table-merge = Merge cells
mail-editor-table-split = Split cell
mail-editor-table-header = Toggle header row
mail-editor-table-delete = Delete table
mail-editor-table-shading = Cell shading
mail-editor-find-replace = Find and replace
mail-editor-find = Find text
mail-editor-replacement = Replace with
mail-editor-match-case = Match case
mail-editor-previous = Previous match
mail-editor-next = Next match
mail-editor-replace = Replace selected match
mail-editor-replace-all = Replace all matches
mail-editor-matches = { $count } matches (up to 1,000)
mail-editor-select-all = Select all
mail-editor-clear-selection = Clear selection
mail-editor-history = Undo history
mail-editor-undo-groups = Undo last { $count } change groups
mail-editor-zoom = Editor zoom
mail-editor-markers = Paragraph marks
mail-editor-style-normal = Normal
mail-editor-style-no-spacing = No spacing
mail-editor-style-title = Title
mail-editor-style-subtitle = Subtitle
mail-editor-style-heading1 = Heading 1
mail-editor-style-heading2 = Heading 2
mail-editor-style-heading3 = Heading 3
mail-editor-style-heading4 = Heading 4
mail-editor-style-heading5 = Heading 5
mail-editor-style-heading6 = Heading 6
mail-editor-align-left = Left
mail-editor-align-center = Center
mail-editor-align-right = Right
mail-editor-align-justify = Justify
mail-editor-case-lower = lowercase
mail-editor-case-upper = UPPERCASE
mail-editor-case-title = Title Case
mail-editor-case-sentence = Sentence case

mail-editor-action-history = { $action } history
mail-editor-history-empty = No actions available
mail-editor-redo-groups = Redo { $count } changes
mail-compose-identities-failed = Sending identities could not be loaded. Close and reopen this draft to retry.

# First sync after sign-in (shown only when an empty folder could not be indexed).
mail-folders-loading = Loading folders…
mail-initial-sync-error = Your messages could not be synchronized yet.
mail-initial-sync-error-help = The connection to your mail server was interrupted. Try again, or ask your administrator to check the connection logs if this keeps happening.
