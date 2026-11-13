# Unified mail

Enable **Unified mail** in Settings → Folders. The administrator can set or lock the same `mail.unifiedInbox` preference; it remains off by default.

There are two entry points: the Unified mail workspace in the sidebar and a virtual Unified mail entry in the account selector. Virtual-account mode replaces the physical folder tree with aggregate Inbox, All mail, Sent, Drafts, Archive, Junk and Trash folders and unread/total counters, without changing the underlying authenticated account. The separate workspace view keeps the normal account tree. Selecting a physical mailbox returns to its normal mail view.

The view merges the primary and ready linked mailboxes, with account/folder badges, the normal message-row presentation, total/unread/loaded counters, account and folder-role filters, text search, unread-only filtering, sorting, grouping and account-scoped conversations. A missing count stays unknown. Older pages load on scroll and when more rows are needed to fill the viewport. Visible idle lists at the top refresh each minute and on focus.

Checkboxes, modifier clicks, keyboard range selection and the normal floating selection bar support bulk marking, flags, Archive, Trash and Junk. Each confirmed mutation updates only its source account; failed items remain selected. Custom-folder moves require a selection from one source account. Cross-account copy/move uses the existing opt-in transfer action for an individual message. Message opening retains its source mailbox in a new tab.
