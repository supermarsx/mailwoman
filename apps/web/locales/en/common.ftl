# Mailwoman — common UI strings (source locale: en).
#
# This is the ONE catalog that rides the critical entry bundle (see
# src/i18n/catalog.ts), so keep it to genuinely cross-cutting strings: generic
# buttons, states, and errors reused everywhere. Feature strings belong in their
# own module catalog (mail.ftl, calendar.ftl, …), lazily loaded.
#
# Fluent syntax primer for e1–e4:
#   message-id = Simple text
#   with-arg   = Hello, { $name }!
#   selects    = { $count ->
#       [one] { $count } message
#      *[other] { $count } messages
#   }
# Message ids are kebab-case and MODULE-PREFIXED (common-*, mail-*, calendar-*).

# -- Generic actions ---------------------------------------------------------
common-ok = OK
common-cancel = Cancel
common-close = Close
common-save = Save
common-delete = Delete
common-remove = Remove
common-edit = Edit
common-add = Add
common-back = Back
common-next = Next
common-done = Done
common-apply = Apply
common-confirm = Confirm
common-retry = Retry
common-copy = Copy
common-search = Search
common-more = More
common-yes = Yes
common-no = No

# -- Generic states ----------------------------------------------------------
common-loading = Loading…
common-saving = Saving…
common-empty = Nothing here yet
common-error = Something went wrong
common-error-network = Can’t reach the server. Check your connection and try again.
common-offline = You’re offline

# -- Generic labels ----------------------------------------------------------
common-required = Required
common-optional = Optional

# -- Attachments (global, cross-account view; rides the entry bundle so the
#    account-wide attachments screen needs no extra catalog fetch) ------------
common-attach-title = Attachments
common-attach-search = Search attachments
common-attach-search-placeholder = filename:report type:pdf larger:1mb from:alice
common-attach-filter-type = Filter by type
common-attach-loading = Loading attachments…
common-attach-empty = No attachments match.
common-attach-cat-all = all
common-attach-cat-image = image
common-attach-cat-pdf = pdf
common-attach-cat-text = text
common-attach-cat-audio = audio
common-attach-cat-video = video
common-attach-cat-other = other
