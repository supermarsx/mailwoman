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
