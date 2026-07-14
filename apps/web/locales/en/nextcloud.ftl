# Mailwoman — Nextcloud module strings (attach / save-to / share-link).
# Source locale: en. Ids are kebab-case, MODULE-PREFIXED (`nextcloud-`).
#
# Untrusted, user/server-controlled values interpolated into these messages —
# file names, folder/destination paths, share paths — are wrapped in a bidi
# isolate (`isolate()`) AT THE CALL SITE (SPEC §24: the exe.png↔gnp.exe filename
# spoof is this module's risk). Static UI text is never isolated.

# -- File picker -------------------------------------------------------------
nextcloud-up = Up
nextcloud-loading = Loading…
nextcloud-list-error = Could not list this folder.
nextcloud-empty = This folder is empty.
nextcloud-file-list = Nextcloud files
nextcloud-open-folder = Open folder { $name }
nextcloud-select-file = Select { $name }

# -- Attach from Nextcloud ---------------------------------------------------
nextcloud-attach-title = Attach from Nextcloud
nextcloud-attach-action = { $count ->
    [0] Attach
    [one] Attach { $count } file
   *[other] Attach { $count } files
  }
nextcloud-error-select-file = select at least one file to attach
nextcloud-error-attach-failed = could not attach the selected files

# -- Save to Nextcloud -------------------------------------------------------
nextcloud-save-panel-label = Save to Nextcloud
nextcloud-save-title = Save “{ $name }” to Nextcloud
nextcloud-save-action = Save here ({ $dir })
nextcloud-saved = Saved to { $path }
nextcloud-error-save-failed = could not save to Nextcloud

# -- Share link --------------------------------------------------------------
nextcloud-share-panel-label = Create share link
nextcloud-share-title = Share link
nextcloud-share-intro = Create a public link to { $path } instead of attaching the file.
nextcloud-protect-password = Protect with a password
nextcloud-password-label = Password
nextcloud-share-password = Share password
nextcloud-set-expiry = Set an expiry date
nextcloud-expires-on = Expires on
nextcloud-expiry-date = Expiry date
nextcloud-create-link = Create link
nextcloud-password-protected = Password-protected
nextcloud-no-password = No password
nextcloud-expires = expires { $date }
nextcloud-no-expiry = no expiry
nextcloud-error-need-password = enter a password or turn password protection off
nextcloud-error-need-expiry = pick an expiry date or turn expiry off
nextcloud-error-share-failed = could not create the share link
