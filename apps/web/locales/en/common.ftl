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
# Desktop and mobile apps only: under the error shown when the server the app
# was built or provisioned for does not answer at start. The user cannot change
# the server; an administrator can.
common-contact-administrator = If this keeps happening, contact your administrator.
# Desktop and mobile apps only: shown instead of the sign-in screen when the app
# was built without a server address and no provisioning file names one.
common-shell-not-configured = This app has not been set up for a server. Contact your administrator.
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

# -- Connection banner (realtime/ConnectionToast.tsx). Shown for a signed-in
#    session only; mounted in the app shell, so it rides the entry bundle. ------
common-conn-connecting = Connecting…
common-conn-degraded = Realtime updates paused — reconnecting
common-conn-offline = You are offline — changes sync when you reconnect
common-conn-auth-expired = Your session expired — please sign in again
common-conn-reconnect = Reconnect
common-conn-sign-in-again = Sign in again
common-conn-reconnected = Reconnected

# -- Toasts (components/toast/ToastRegion.tsx), in the webmail and the admin
#    console. The seconds are the time left on an undo window. -----------------
common-toast-close = Close notification
common-toast-seconds = { $seconds } s

# -- New-mail notification (App.tsx). The native desktop or mobile notification
#    shown while the app is in the background; the body is the message subject. --
common-notify-new-message = New message
common-notify-no-subject = (no subject)

# -- Generic labels ----------------------------------------------------------
common-required = Required
common-optional = Optional

# -- Narrow-viewport shell (the top bar renders with the entry bundle, before
#    any module catalog has loaded) -------------------------------------------
common-nav-open = Open folders and apps
common-nav-close = Close folders and apps

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

# -- Password fields (components/PasswordInput.tsx): the button at the end of
# every password field, on both sign-in screens and in settings. It is a toggle;
# its name stays "Show password" and screen readers announce pressed or not
# pressed. The hide text is the pointer tooltip while the password is shown.
common-password-show = Show password
common-password-hide = Hide password

# -- App modules: the nav-rail names of the modules beside mail (shell/modules.ts)
# and the title of the pinned mail tab (screens/Mailbox.tsx).
common-module-mail = Mail
common-module-calendar = Calendar
common-module-tasks = Tasks
common-module-notes = Notes
common-module-contacts = Contacts
common-module-keys = Keys

# -- Attachment viewers (viewers/): the in-app preview of one attachment and the
# thumbnail strip under a message. The MIME type itself is shown untranslated.
common-viewer-loading = Loading viewer…
common-viewer-loading-audio = Loading audio…
common-viewer-loading-image = Loading image…
common-viewer-loading-video = Loading video…
common-viewer-rendering-pdf = Rendering PDF…
common-viewer-pdf-failed = Could not render this PDF.
common-viewer-no-preview = No inline preview for this file type.
common-viewer-unknown-type = unknown type
common-viewer-download = Download
common-viewer-attachments = Attachments

common-message-preview-too-large = Message attachment previews are limited to 4 MiB. You can still download the original attachment.
common-message-preview-failed = This message attachment could not be previewed. The file may be unsupported, damaged, or the preview worker unavailable. You can still download the original.
common-message-preview-attachments = Files inside this message
common-message-preview-download = Download file
common-attach-cat-message = Messages / Outlook attachments
