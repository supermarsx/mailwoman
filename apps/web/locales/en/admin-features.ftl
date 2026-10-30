## Admin › Feature profiles (t39 N9)

admin-nav-features = Feature profiles
admin-features-title = Feature profiles
admin-features-intro = A profile is a named set of feature switches. Assign it to a domain, a group or a user; it then applies to the native accounts there. Accounts signed in through a proxy upstream follow that server's own policy.
admin-features-new = New profile
admin-features-search = Search profiles
admin-features-empty = No feature profiles yet.
admin-features-no-match = No profile matches the search.
admin-features-load-error = The feature catalogue could not be loaded.
admin-features-not-loaded = The feature catalogue is not loaded yet. Reload the page and try again.
admin-features-stale = Another administrator changed the feature catalogue. It has been reloaded; check the values and save again.
admin-features-not-found = No profile with the id { $id }.
admin-features-col-name = Name
admin-features-col-values = Switches
admin-features-col-assigned = Assigned to
admin-features-values-count = { $count ->
    [one] { $count } switch set
   *[other] { $count } switches set
}
admin-features-assigned-count = { $count ->
    [0] Not assigned
    [one] { $count } place
   *[other] { $count } places
}
admin-features-editor = Profile settings
admin-features-name = Profile name
admin-features-name-help = Shown to administrators only, up to 128 characters.
admin-features-name-invalid = Enter a name of 1 to 128 characters.
admin-features-switches = Feature switches
admin-features-switches-help = "Not set" leaves the feature to the next profile, the domain or the server default. A locked value cannot be changed by the user or by a lower level.
admin-features-col-feature = Feature
admin-features-col-value = Value
admin-features-col-lock = Lock
admin-features-value-for = Value for { $feature }
admin-features-unset = Not set
admin-features-on = Allowed
admin-features-off = Not allowed
admin-features-lock = Locked
admin-features-save = Save profile
admin-features-create = Create profile
admin-features-delete = Delete profile
admin-features-delete-title = Delete the profile { $name }?
admin-features-delete-body = { $count ->
    [0] The profile is not assigned anywhere. Its switches stop applying at once.
    [one] The profile is removed from the { $count } place it is assigned to, and its switches stop applying at once.
   *[other] The profile is removed from the { $count } places it is assigned to, and its switches stop applying at once.
}
admin-features-saved = Profile { $name } saved.
admin-features-deleted = Profile { $name } deleted.
admin-features-assigned = Profile { $name } assigned.
admin-features-unassigned = Assignment of { $name } removed.
admin-features-assignments = Assigned to
admin-features-assignments-help = Profiles apply in priority order, higher first. At equal priority a user assignment applies before a group, and a group before a domain.
admin-features-assignments-empty = This profile is not assigned yet.
admin-features-assign-form = Assign this profile
admin-features-col-scope = Applies to
admin-features-col-target = Name
admin-features-col-priority = Priority
admin-features-col-actions = Actions
admin-features-scope-domain = Domain
admin-features-scope-group = Group
admin-features-scope-user = User
admin-features-assign = Assign
admin-features-unassign = Remove
admin-features-no-domain = No domains are configured.
admin-features-no-group = No user groups exist.
admin-features-no-user = No native accounts have signed in yet.

## t98: grouped editor, presets, import and export, effective preview
admin-features-tools = Profile tools
admin-features-preset = Start from a preset
admin-features-preset-choose = Choose a preset
admin-features-preset-apply = Apply preset
admin-features-preset-applied = Preset { $name } applied. Review the values and save.
admin-features-preset-full = Full
admin-features-preset-standard = Standard
admin-features-preset-mail-only = Mail only
admin-features-preset-minimal = Minimal / kiosk
admin-features-import = Import JSON
admin-features-export = Export JSON
admin-features-exported = Profile exported as a JSON file.
admin-features-imported = { $count ->
    [one] Imported { $count } value. Review it and save.
   *[other] Imported { $count } values. Review them and save.
}
admin-features-import-json = The file is not valid JSON.
admin-features-import-format = The file is not a Mailwoman feature profile.
admin-features-import-version = This feature profile file comes from a newer version and cannot be imported.
admin-features-clone = Clone
admin-features-clone-name = Copy of { $name }
admin-features-filter = Find a feature
admin-features-filter-placeholder = Name, setting or key
admin-features-filter-none = No feature matches the search.
admin-features-diff-summary = { $differs ->
    [one] { $differs } value differs from the server default
   *[other] { $differs } values differ from the server default
} · { $same } repeat it
admin-features-differs = Differs from default
admin-features-default-is = Default: { $value }
admin-features-fix-values = Correct the values marked invalid before saving.
admin-features-allow-all = Allow all
admin-features-deny-all = Disallow all
admin-features-clear-all = Clear
admin-features-group-allow = Allow every feature in { $group }
admin-features-group-deny = Disallow every feature in { $group }
admin-features-group-clear = Clear every value in { $group }
admin-features-enforcement-ui = Hidden in the app only
admin-features-enforcement-partial = Some limits not yet counted
admin-features-other = Other settings a profile can hold
admin-features-other-help = Every other setting an administrator may set per profile. Leave a field empty to keep the server value.
admin-features-cat-compose = Mail and compose
admin-features-cat-reading = Reading and privacy
admin-features-cat-views = Views and search
admin-features-cat-organise = Organise
admin-features-cat-pim = Calendar, contacts, tasks, notes and keys
admin-features-cat-data = Data
admin-features-cat-accounts = Accounts and access
admin-features-cat-integrations = Integrations
admin-features-cat-developer = Developer access
admin-features-cat-personal = Personalisation
admin-features-feature-compose-sending = Sending limits
admin-features-feature-compose-sending-help = Recipients per message is refused by the server above the limit. Messages per hour and per day are stored for the profile but not yet counted.
admin-features-feature-storage = Storage quota
admin-features-feature-storage-help = The storage limit is stored for the profile; the mail server's own quota still applies.
admin-features-feature-access-protocols = Mail access protocols
admin-features-feature-access-protocols-help = Which mail server protocols these users may sign in with, separated by commas: imap, pop3, ews.
admin-features-feature-accounts-linked = Linked mailboxes
admin-features-feature-calendar = Calendar
admin-features-feature-contacts = Contacts
admin-features-feature-tasks = Tasks
admin-features-feature-notes = Notes
admin-features-preview = Effective values
admin-features-preview-help = What every feature resolves to for one user or for a user of one domain, after every profile, group, domain value and lock.
admin-features-preview-kind = Preview for
admin-features-preview-target = Who
admin-features-preview-show = Show
admin-features-preview-loading = Loading…
admin-features-preview-state = Effective
admin-features-preview-from = Decided by
admin-features-preview-profiles = Profiles that apply, first wins: { $names }
admin-features-preview-no-profiles = No profile applies; the domain and server values decide.
admin-features-source-default = Built-in default
admin-features-source-env-fallback = Environment default
admin-features-source-env-pin = Environment (pinned)
admin-features-source-global = Server value
admin-features-source-domain = Domain value
admin-features-source-profile = Profile
admin-features-source-profile-named = Profile { $name }
admin-features-source-group = Group value
admin-features-source-admin-user = Value set for this user
admin-features-source-user = The user's own choice
admin-features-source-locked = { $source } (locked)

## Admin › Observability › Audit log retention (t39 L5)

admin-retention-title = Audit log retention
admin-retention-intro = How long audit entries are kept, per category. A category without its own value follows the default; with no default every entry is kept. A legal hold stops pruning for its category, and a hold on the default stops it for all. Changes and prunes are recorded in the audit log.
admin-retention-loading = Loading the retention policy…
admin-retention-unavailable = Audit retention is not available on this server.
admin-retention-col-category = Category
admin-retention-col-kept = Kept
admin-retention-col-change = Change
admin-retention-cat-default = Default
admin-retention-cat-admin = Administrator actions
admin-retention-cat-security = Security events
admin-retention-cat-alert = User alerts
admin-retention-forever = Every entry
admin-retention-days = { $days ->
    [one] { $days } day
   *[other] { $days } days
}
admin-retention-follows-default = Default: { $kept }
admin-retention-hold-on = Legal hold
admin-retention-read-only = Changing retention needs the settings.write permission.
admin-retention-edit-for = Retention for { $category }
admin-retention-keep = Keep
admin-retention-mode-inherit = Follow the default
admin-retention-mode-forever = Every entry
admin-retention-mode-days = For a number of days
admin-retention-days-label = Days
admin-retention-hold = Legal hold
admin-retention-save = Save
admin-retention-bounds = Enter a whole number from { $min } to { $max }.
admin-retention-saved = Retention for { $category } saved.
admin-retention-task-none = No prune task yet. It is created, daily at 03:15 UTC, when a number of days is first saved.
admin-retention-task-disabled = The scheduled prune task is disabled; entries are pruned only with Prune now.
admin-retention-task-next = Next scheduled prune: { $when }.
admin-retention-task-daily = Pruned daily at 03:15 UTC.
admin-retention-last = Last prune that removed entries: { $when }.
admin-retention-never = No entries have been pruned yet.
admin-retention-prune = Prune now
admin-retention-prune-title = Prune the audit log now?
admin-retention-prune-body = Entries older than their category's retention are deleted. Categories that keep every entry or are on legal hold are not changed. This cannot be undone.
admin-retention-pruned = { $count ->
    [0] No entries were old enough to remove.
    [one] { $count } entry removed.
   *[other] { $count } entries removed.
}

## Admin › Logs › Authentication (t39 L5)

admin-auth-attempted = Attempted as { $name }
