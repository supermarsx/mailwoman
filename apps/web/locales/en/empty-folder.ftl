# Empty a whole folder (t45): the folder menu, the folder tab and the Trash toolbar.
empty-folder-action = Empty folder…
empty-folder-title = Empty “{ $name }”?
# Ordinary folder: the messages move to Trash.
empty-folder-body-trash = { $count ->
    [one] The message in { $name } will be moved to Trash, where you can restore it.
   *[other] All { $count } messages in { $name } will be moved to Trash, where you can restore them.
} Subfolders and the messages in them are not touched.
# Trash itself: permanent.
empty-folder-body-destroy-trash = { $count ->
    [one] The message in Trash will be deleted permanently. This cannot be undone.
   *[other] All { $count } messages in Trash will be deleted permanently. This cannot be undone.
} Subfolders and the messages in them are not touched.
# Junk / Spam: permanent.
empty-folder-body-destroy-junk = { $count ->
    [one] The message in { $name } will be deleted permanently. This cannot be undone.
   *[other] All { $count } messages in { $name } will be deleted permanently. This cannot be undone.
} Subfolders and the messages in them are not touched.
# An ordinary folder in an account that has no Trash folder: permanent.
empty-folder-body-destroy-no-trash = This account has no Trash folder, so { $count ->
    [one] the message in { $name } will be deleted permanently.
   *[other] all { $count } messages in { $name } will be deleted permanently.
} This cannot be undone. Subfolders and the messages in them are not touched.
empty-folder-confirm-trash = Move to Trash
empty-folder-confirm-destroy = Delete permanently
empty-folder-cancel = Cancel
empty-folder-already-empty = “{ $name }” is already empty.
empty-folder-prepare-failed = “{ $name }” could not be counted, so nothing was changed. Check your connection and try again.
empty-folder-syncing = “{ $name }” is still being synchronised, so it cannot be counted yet. Nothing was changed; try again in a moment.
empty-folder-running = A folder is already being emptied. Stop it from its notification before starting another.
empty-folder-progress-trash = Moving messages from { $name } to Trash: { $done } of { $total }.
empty-folder-progress-destroy = Deleting messages from { $name }: { $done } of { $total }.
empty-folder-stop = Stop
empty-folder-stopping = Stopping after the current batch. { $done } of { $total } done so far.
empty-folder-done-trash = { $done ->
    [one] Moved 1 message from { $name } to Trash.
   *[other] Moved { $done } messages from { $name } to Trash.
}
empty-folder-done-destroy = { $done ->
    [one] Deleted 1 message from { $name } permanently.
   *[other] Deleted { $done } messages from { $name } permanently.
}
empty-folder-stopped = Stopped. { $done } of { $total } messages done; { $remaining } still in { $name }.
empty-folder-refused = Stopped. { $done } of { $total } messages done; the mail server refused { $refused }. { $remaining } still in { $name }.
empty-folder-failed = Emptying { $name } stopped with an error after { $done } messages. The last batch may or may not have completed; check the folder before trying again.
