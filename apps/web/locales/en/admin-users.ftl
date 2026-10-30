## Admin › Users list (t107): the table, its search, filters, saved views and bulk actions.

admin-userlist-caption = Users. Column headers sort the list.
admin-userlist-search = Search users
admin-userlist-search-placeholder = Name, address, alias or domain
admin-userlist-count = { $count ->
    [one] { $count } user
   *[other] { $count } users
}
admin-userlist-count-filtered = { $count ->
    [one] { $count } of { $total } users matches
   *[other] { $count } of { $total } users match
}
admin-userlist-range = { $from }–{ $to } of { $total }
admin-userlist-loading = Loading users…
admin-userlist-retry = Try again
admin-userlist-empty-filtered = No users match this search and these filters.
admin-userlist-compact = Compact rows
admin-userlist-columns = Columns
admin-userlist-columns-legend = Show columns
admin-userlist-actions = Actions
admin-userlist-paging = Pages of users
admin-userlist-page-size = Rows per page
admin-userlist-prev = Previous
admin-userlist-next = Next
admin-userlist-sort-by = Sort by
admin-userlist-sort-asc = { $column }, ascending
admin-userlist-sort-desc = { $column }, descending

## Columns
admin-userlist-col-user = User
admin-userlist-col-domain = Domain
admin-userlist-col-status = Status
admin-userlist-col-2fa = Two-factor
admin-userlist-col-role = Console role
admin-userlist-col-last-signin = Last sign-in
admin-userlist-col-last-seen = Last seen
admin-userlist-col-created = Created
admin-userlist-col-quota = Quota
admin-userlist-col-profile = Feature profile
admin-userlist-col-linked = Linked mailboxes

## Cell values
admin-userlist-state-active = Active
admin-userlist-state-pending = Never signed in
admin-userlist-state-blocked = Blocked
admin-userlist-state-disabled = Disabled
admin-userlist-state-deleted = Deleted
admin-userlist-2fa-passkey = Passkey
admin-userlist-2fa-totp = Authenticator app
admin-userlist-2fa-none = None
admin-userlist-signin-never = Never
admin-userlist-signin-at = { $relative } ({ $exact })
admin-userlist-seen-never = No activity recorded
admin-userlist-seen-at = Last seen { $relative } ({ $exact })
admin-userlist-quota-unlimited = Unlimited
admin-userlist-profile-default = Default
admin-userlist-profile-via-user = Assigned to this user
admin-userlist-profile-via-group = From a group
admin-userlist-profile-via-domain = From the domain
admin-userlist-more = More actions
admin-userlist-more-for = More actions for { $account }
admin-userlist-select = Select { $account }
admin-userlist-select-page = Select every user on this page

## Filters
admin-userlist-filters = Filters
admin-userlist-add-filter = Add filter
admin-userlist-clear-all = Clear all
admin-userlist-chip-edit = Edit filter { $label }
admin-userlist-chip-remove = Remove filter { $label }
admin-userlist-filter-domain = Domain
admin-userlist-filter-status = Status
admin-userlist-filter-tfa = Two-factor
admin-userlist-filter-role = Console role
admin-userlist-filter-profile = Feature profile
admin-userlist-filter-signin = Last sign-in
admin-userlist-filter-created = Created
admin-userlist-filter-no-choices = Nothing to choose from yet.
admin-userlist-filter-remove = Remove filter
admin-userlist-apply = Apply
admin-userlist-cancel = Cancel
admin-userlist-role-any = Any console role
admin-userlist-role-none = No console role
admin-userlist-profile-none = No profile (default)
admin-userlist-signin-older = { $days ->
    [one] More than { $days } day ago
   *[other] More than { $days } days ago
}
admin-userlist-signin-within = { $days ->
    [one] In the last { $days } day
   *[other] In the last { $days } days
}
admin-userlist-signin-custom = More than this many days ago:
admin-userlist-signin-days = Days
admin-userlist-days-invalid = Enter a whole number of days from 1 to 36500.
admin-userlist-created-from = From
admin-userlist-created-to = To
admin-userlist-created-between = { $from } to { $to }
admin-userlist-created-since = since { $date }
admin-userlist-created-until = until { $date }
admin-userlist-dates-invalid = The start date is after the end date.

## Saved views
admin-userlist-views = Saved views
admin-userlist-view-save = Save this view…
admin-userlist-view-name = Name of the view
admin-userlist-view-save-confirm = Save view
admin-userlist-view-delete = Delete view “{ $name }”

## Row and bulk actions
admin-userlist-bulk = Actions on the selected users
admin-userlist-selected = { $count ->
    [one] { $count } selected
   *[other] { $count } selected
}
admin-userlist-clear-selection = Clear selection
admin-userlist-block = Block login
admin-userlist-unblock = Unblock login
admin-userlist-sign-out = Sign out everywhere
admin-userlist-delete = Delete local access
admin-userlist-bulk-profile = Assign feature profile
admin-userlist-bulk-profile-clear = Remove the user’s own profile
admin-userlist-confirm-skipped = { $count ->
    [one] { $count } selected user is skipped: the action does not apply to it.
   *[other] { $count } selected users are skipped: the action does not apply to them.
}
admin-userlist-confirm-block = { $count ->
    [one] Block login for { $count } user?
   *[other] Block login for { $count } users?
}
admin-userlist-confirm-block-one = Block login for { $account }?
admin-userlist-confirm-block-body = New sign-ins are refused and current Mailwoman sessions end. The upstream mailbox is not changed.
admin-userlist-confirm-block-action = Block login
admin-userlist-confirm-unblock = { $count ->
    [one] Unblock login for { $count } user?
   *[other] Unblock login for { $count } users?
}
admin-userlist-confirm-unblock-one = Unblock login for { $account }?
admin-userlist-confirm-unblock-body = The local login block is removed. Other security restrictions still apply.
admin-userlist-confirm-unblock-action = Unblock login
admin-userlist-confirm-signout = { $count ->
    [one] Sign out { $count } user everywhere?
   *[other] Sign out { $count } users everywhere?
}
admin-userlist-confirm-signout-one = Sign out { $account } everywhere?
admin-userlist-confirm-signout-body = Every open Mailwoman session of these users ends. They can sign in again.
admin-userlist-confirm-signout-action = Sign out
admin-userlist-confirm-delete = { $count ->
    [one] Delete local access for { $count } user?
   *[other] Delete local access for { $count } users?
}
admin-userlist-confirm-delete-one = Delete local access for { $account }?
admin-userlist-confirm-delete-body = A deleted access record is kept so the user is not enrolled again automatically. Mailwoman settings, history and the upstream mailbox stay stored. Type the address to confirm.
admin-userlist-confirm-delete-action = Delete local access
admin-userlist-confirm-profile = { $count ->
    [one] Change the feature profile of { $count } user?
   *[other] Change the feature profile of { $count } users?
}
admin-userlist-confirm-profile-one = Change the feature profile of { $account }?
admin-userlist-confirm-profile-body = The profile is assigned to each user directly and applies from their next request. Accounts that are not native mailboxes cannot take a profile and are reported as failed.
admin-userlist-confirm-profile-action = Change profile
admin-userlist-bulk-done-block = { $count ->
    [one] Blocked login for { $count } user.
   *[other] Blocked login for { $count } users.
}
admin-userlist-bulk-done-unblock = { $count ->
    [one] Unblocked login for { $count } user.
   *[other] Unblocked login for { $count } users.
}
admin-userlist-bulk-done-signout = { $count ->
    [one] Signed out { $count } user.
   *[other] Signed out { $count } users.
}
admin-userlist-bulk-done-delete = { $count ->
    [one] Deleted local access for { $count } user.
   *[other] Deleted local access for { $count } users.
}
admin-userlist-bulk-done-profile = { $count ->
    [one] Changed the feature profile of { $count } user.
   *[other] Changed the feature profile of { $count } users.
}
admin-userlist-bulk-failed = { $done } done, { $failed } failed: { $reason }
