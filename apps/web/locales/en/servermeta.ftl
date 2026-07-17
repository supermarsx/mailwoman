# Mailwoman — server / mailbox METADATA view strings (source locale: en, SPEC §24,
# t13 26.13 plan §Workstream-2 E8). RFC 5464 annotations.
#
# Lazily loaded by the servermeta module (MetadataView). Ids are kebab-case,
# prefixed `servermeta-`. Untrusted values (entry paths, mailbox names) are
# bidi-isolated at the call site via i18n `isolate()` before interpolation.

# -- view frame --------------------------------------------------------------
servermeta-view-label = Server annotations
servermeta-title = Server annotations
servermeta-title-named = Annotations on { $mailbox }
servermeta-intro = Metadata entries are stored on the mail server. Changes here take effect only if your account is allowed to set them.
servermeta-loading = Loading annotations…
servermeta-load-failed = Could not load the annotations.
servermeta-op-failed = The change was not saved. The server rejected it or was unreachable.
servermeta-readonly = Editing is turned off for you. Annotations are shown read-only.

# -- entries -----------------------------------------------------------------
servermeta-entries = Entries
servermeta-no-entries = No annotations are set.
servermeta-unset = Not set
servermeta-value-label = Value
servermeta-save = Save
servermeta-remove = Remove

# -- add an entry ------------------------------------------------------------
servermeta-add-heading = Add an annotation
servermeta-entry-label = Entry
servermeta-entry-placeholder = /shared/comment
servermeta-value-placeholder = value
servermeta-add-entry = Add annotation
