# Unified mail

Enable **Unified mail** in Settings → Folders. The administrator can set or lock the same `mail.unifiedInbox` preference; it remains off by default.

There are two entry points: the Unified mail workspace in the sidebar and a virtual Unified mail entry in the account selector. Virtual-account mode replaces the physical folder tree with aggregate Inbox, All mail, Sent, Drafts, Archive, Junk and Trash folders and unread/total counters, without changing the underlying authenticated account. The separate workspace view keeps the normal account tree. Selecting a physical mailbox returns to its normal mail view.

The view merges the primary and ready linked mailboxes, with account/folder badges, the normal message-row presentation, total/unread/loaded counters, account and folder-role filters, text search, unread-only filtering, sorting, grouping and account-scoped conversations. A missing count stays unknown. Older pages load on scroll and when more rows are needed to fill the viewport. Visible idle lists at the top refresh each minute and on focus.

Checkboxes, modifier clicks, keyboard range selection and the normal floating selection bar support bulk marking, flags, Archive, Trash and Junk. Each confirmed mutation updates only its source account; failed items remain selected. Custom-folder moves require a selection from one source account. Cross-account copy/move uses the existing opt-in transfer action for an individual message. Message opening retains its source mailbox in a new tab.


## Named virtual accounts

In Settings → Linked mailboxes, edit the default unified account or add a named
unified group. Each group has its own name, avatar, included physical accounts
and included folders. Up to 16 groups are supported. A group can be the virtual
mailbox opened first after sign-in; selecting its Inbox, Sent or other sidebar
folder keeps that group's account selection. Deleting a group preserves all
underlying accounts and messages.

Administrators can set or lock group definitions and the initial group using
`mail.unifiedAccount.groups` and `mail.unifiedAccount.primaryGroup`. The existing
`allowedAccounts` and `allowedFolders` bounds apply to every group and to the
separate unified workspace. Unified views require an enabled linked-mailbox
policy and more than one usable account.

For faster physical mailbox switching, Settings → Linked mailboxes offers
**Preload linked mailboxes** (`accounts.preloadLinked`, enabled by default).
It requires the mail cache and keeps verified folders and a first inbox page in
memory for up to two minutes, bounded by the configured cache entry/byte limits
and an additional 8 MiB cap. It does not enable persistent offline storage for
inactive accounts. Every switch still authorizes the link on the server before
showing cached data, then refreshes in the background. Clearing or disabling the
mail cache removes these snapshots too.
