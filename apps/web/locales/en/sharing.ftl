# Mailwoman — mailbox sharing / ACL editor strings (source locale: en, SPEC §24,
# t13 26.13 plan §Workstream-2 E8). RFC 4314 rights, in plain language.
#
# Lazily loaded by the sharing module (AclEditor). Ids are kebab-case, prefixed
# `sharing-`. Untrusted values (identifiers, mailbox names) are bidi-isolated at
# the call site via i18n `isolate()` before interpolation (SPEC §24).

# -- editor frame ------------------------------------------------------------
sharing-editor-label = Mailbox access
sharing-title = Who can access this mailbox
sharing-title-named = Who can access { $mailbox }
sharing-intro = Access is enforced by the mail server. Changes here take effect only if your account is allowed to administer this mailbox.
sharing-loading = Loading access…
sharing-load-failed = Could not load the mailbox access list.
sharing-op-failed = The change was not saved. The server rejected it or was unreachable.

# -- your own access (MYRIGHTS) ----------------------------------------------
sharing-your-access = Your access
sharing-no-access = You have no access rights on this mailbox.
sharing-readonly = You do not hold the administer right on this mailbox, so access is shown read-only. Ask an administrator to change it.

# -- grants (ACL entries) ----------------------------------------------------
sharing-grants = People and groups
sharing-no-grants = No access has been granted yet.
sharing-remove = Remove

# -- add a grant -------------------------------------------------------------
sharing-add-heading = Grant access
sharing-identifier-label = User or group
sharing-identifier-placeholder = user name, group, or anyone
sharing-rights-legend = Rights to grant
sharing-add-grant = Add access

# -- RFC 4314 rights bits: label + plain-language description -----------------
sharing-right-l-label = Look up
sharing-right-l-desc = See this mailbox in the folder list.
sharing-right-r-label = Read
sharing-right-r-desc = Open the mailbox and read its messages.
sharing-right-s-label = Keep read state
sharing-right-s-desc = Remember which messages have been read between sessions.
sharing-right-w-label = Write flags
sharing-right-w-desc = Change flags other than read and deleted, such as flagged or answered.
sharing-right-i-label = Insert messages
sharing-right-i-desc = Add messages to this mailbox by copying or moving them in.
sharing-right-p-label = Post
sharing-right-p-desc = Send mail to the address associated with this mailbox, where supported.
sharing-right-k-label = Create sub-mailboxes
sharing-right-k-desc = Create mailboxes nested inside this one.
sharing-right-x-label = Delete mailbox
sharing-right-x-desc = Delete or rename this mailbox itself.
sharing-right-t-label = Delete messages
sharing-right-t-desc = Mark messages in this mailbox as deleted.
sharing-right-e-label = Expunge
sharing-right-e-desc = Permanently remove messages that are marked as deleted.
sharing-right-a-label = Administer
sharing-right-a-desc = Change who has access to this mailbox.
