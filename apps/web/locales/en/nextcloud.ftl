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

# Account-owned integration configuration
nextcloud-account-title = Nextcloud accounts
nextcloud-account-help = Link your own account to an instance approved by your administrator. Credentials are stored encrypted for this mailbox account only.
nextcloud-account-load-error = Could not load Nextcloud links. Reload to try again.
nextcloud-account-unlink-error = Could not remove the link. Reload before trying again.
nextcloud-account-reload = Reload
nextcloud-account-no-instances = No enabled Nextcloud instances are available. Ask your administrator to configure one.
nextcloud-account-ready = Linked and ready
nextcloud-account-stale = Instance configuration changed. Verify the link again.
nextcloud-account-unlinked = Not linked
nextcloud-account-relink = Verify link again
nextcloud-account-link = Link account
nextcloud-account-unlink = Unlink account
nextcloud-account-password-help = In Nextcloud personal security settings, create an app password and copy its username and password here. Verification makes one bounded request; rejected credentials are not retried.
nextcloud-account-username = Nextcloud app-password username
nextcloud-account-password = Nextcloud app password
nextcloud-account-verify = Verify and link
nextcloud-account-verify-error = Verification or saving failed. Check the instance, app-password username and password. The password field has been cleared; an existing working link was not replaced.
nextcloud-admin-title = Nextcloud instances
nextcloud-admin-help = Approve HTTPS instances that mailbox users may link individually. No shared service password is given to mailbox accounts. File operations are limited to 8 MiB. Native-engine users explicitly discover and connect calendar, task and address-book collections after linking.
nextcloud-admin-add = Add Nextcloud instance
nextcloud-admin-edit = Edit Nextcloud instance
nextcloud-admin-enabled = Enabled
nextcloud-admin-disabled = Disabled
nextcloud-admin-missing = This instance no longer exists. Return to the instance list.
nextcloud-admin-private-help = Private addresses require an explicit Nextcloud egress permission. Configured proxy routes remain authoritative; a refused proxy request never falls back to a direct connection.
nextcloud-admin-revalidation = Saving any instance change requires users to verify their links again. Disabling an instance immediately stops new operations.
nextcloud-admin-authority = Instance authority
nextcloud-admin-name = Display name
nextcloud-admin-name-help = A short name shown to users when choosing an instance.
nextcloud-admin-url = HTTPS installation URL
nextcloud-admin-url-help = Use the installation root, such as https://cloud.example.org/ or https://example.org/nextcloud/. The certificate must be trusted and match the hostname. Credentials, query parameters and fragments are refused.
nextcloud-admin-enable = Allow users to link this instance

nextcloud-admin-ca = Additional certificate authorities (PEM)
nextcloud-admin-ca-help = Optional private CA certificates for this Nextcloud server. Public trust remains available and hostname verification stays required. Changes require users to verify their links again; proxy certificate trust is separate.

nextcloud-instance-select = Nextcloud instance
nextcloud-instance-placeholder = Choose a linked instance
nextcloud-instances-empty = No verified Nextcloud links. Add or verify a link in User settings → Nextcloud.
nextcloud-instances-error = Linked instances could not be loaded. Reopen the picker to try again.

nextcloud-save-warning = Your administrator requires confirmation before this attachment can leave the mailbox.
nextcloud-save-acknowledge = I understand the attachment warning and want to save this file to the selected Nextcloud instance.
nextcloud-save-attachment = Save { $name } to Nextcloud
nextcloud-save-close = Close Nextcloud destination
nextcloud-save-options = Save attachments to Nextcloud
nextcloud-pim-title = Contacts and calendars
nextcloud-pim-help = Discover collections from this verified Nextcloud account, then explicitly connect the ones you want. Synchronize to pull current data; connected edits use the remote collection and fail if its link or permissions are revoked.
nextcloud-pim-category = Collection category
nextcloud-pim-calendar = Calendars
nextcloud-pim-contacts = Address books
nextcloud-pim-tasks = Task lists
nextcloud-pim-discover = Discover collections
nextcloud-pim-discovered = Available collections
nextcloud-pim-empty = No matching collections were returned.
nextcloud-pim-connect = Connect
nextcloud-pim-connected = Connected collections
nextcloud-pim-sync = Synchronize now
nextcloud-pim-sync-needed = Collection connected. Synchronize now to load its contents.
nextcloud-pim-synced = Synchronization completed.
nextcloud-pim-error = The operation failed. Check the account link and permissions, then retry explicitly. Existing cached data was not discarded.
nextcloud-pim-native-required = Contacts and calendar synchronization requires the native mail engine. File access remains available through your account link.
nextcloud-pim-disconnect = Disconnect
nextcloud-pim-disconnect-help = Disconnecting preserves cached data and stops remote writes. Relinked accounts require discovery and explicit connection again.
nextcloud-pim-ready = Ready
nextcloud-pim-disconnected = Disconnected or link changed
nextcloud-pim-unnamed = Unnamed collection
nextcloud-pim-back = Back to Nextcloud accounts
nextcloud-pim-interval = Automatic refresh interval (seconds)
nextcloud-pim-interval-help = Zero uses manual refresh only. Positive intervals have a 60-second minimum. Connected accounts refresh in the background; failures back off and rejected credentials stop further requests until you relink.
nextcloud-pim-interval-reset = Use administrator default
