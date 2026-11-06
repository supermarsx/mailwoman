# Drag-and-drop arrangement of folders (sidebar and Settings > Folders and sidebar).
folder-dnd-moved = Folder moved: { $name }
folder-dnd-moved-many = { $count } folders moved
folder-dnd-undo = Undo
folder-dnd-cancel = Cancel
folder-dnd-handle = Drag { $name } to move it
folder-dnd-locked-hint = The folder order is fixed by your administrator.
folder-dnd-forced-hint = Pinned by your administrator. It cannot be moved or unpinned.
folder-dnd-disabled-hint = Folder drag and drop is turned off.
folder-dnd-system-hint = System folders cannot be moved.
folder-dnd-pinned-group = Pinned
folder-dnd-top-level = Move to top level
folder-dnd-problem-disabled = Moving folders is turned off.
folder-dnd-problem-locked = The folder order is fixed by your administrator.
folder-dnd-problem-self = A folder cannot be moved into itself.
folder-dnd-problem-descendant = A folder cannot be moved into one of its own subfolders.
folder-dnd-problem-system = System folders cannot be moved here.
folder-dnd-problem-no-children = That folder cannot contain subfolders.
folder-dnd-problem-nesting-off = Your administrator does not allow nesting folders.
folder-dnd-problem-depth = That would nest folders too deeply.
folder-dnd-problem-forced = This folder is pinned by your administrator.
folder-dnd-problem-unknown = That folder is no longer there.
folder-dnd-problem-noop = The folder is already there.
folder-dnd-problem-collision = A folder named { $name } already exists there. Rename it first.
folder-dnd-problem-server = The server did not move { $name }. Nothing was changed.
folder-dnd-announce-up = { $name } moved up
folder-dnd-announce-down = { $name } moved down
folder-dnd-announce-indent = { $name } moved into the folder above
folder-dnd-announce-outdent = { $name } moved up one level
folder-dnd-nest-title = Move { $name } here?
folder-dnd-nest-title-many = Move { $count } folders here?
folder-dnd-nest-body = Choose whether this changes the folder on the mail server or only how this app shows it.
folder-dnd-nest-legend = How to move it
folder-dnd-nest-local = Only arrange in this app
folder-dnd-nest-local-help = A view only. The folder stays where it is on the server and in other mail apps.
folder-dnd-nest-server = Move on the server
folder-dnd-nest-server-help = Changes the folder everywhere, in every mail app that uses this account.
folder-dnd-nest-remember = Remember my choice
folder-dnd-nest-confirm = Move
folder-dnd-arranged-only = Arranged only in this app
folder-dnd-arranged-only-help = Its place on the server is different: { $path }
folder-dnd-apply-server = Apply on server
folder-dnd-reset = Reset order to default
folder-dnd-reset-done = Folder order reset
folder-dnd-move-to = Move { $name } to...
folder-dnd-layout-link = Folder layout
folder-dnd-reorder-enable = Rearrange folders by dragging
folder-dnd-nest-mode = When I nest a folder
folder-dnd-nest-mode-ask = Ask me
folder-dnd-nest-mode-server = Move on the server
folder-dnd-nest-mode-local = Only arrange in this app
folder-dnd-new-placement = New folders appear
folder-dnd-new-placement-end = After their siblings
folder-dnd-new-placement-alphabetical = In alphabetical order
folder-dnd-handles = Show drag handles
folder-dnd-handles-hover = On hover
folder-dnd-handles-always = Always
folder-dnd-arrange-title = Arrange folders
folder-dnd-arrange-help = Drag a folder to reorder it, drop it on another folder to nest it, or use Alt and the arrow keys on a focused folder.
folder-dnd-admin-locked = Set by your administrator
folder-dnd-arrange-menu = Move to…
folder-star-add = Star folder
folder-star-remove = Unstar folder
folder-star-starred = Starred
folder-star-toggle = Star { $name }
folder-star-only = Show starred only
folder-star-only-help = Shows only starred folders and the folders that contain them.
folder-dnd-indent = Move { $name } into the folder above
folder-dnd-outdent = Move { $name } up one level
folder-dnd-more-actions = Arrange
folder-dnd-selected-move = Move selected to…
folder-dnd-locked-row = Locked
folder-dnd-problem-pin = Pinned folders stay above the others. Unpin it first to move it there.

folder-dnd-handle-hint = Drag to move this folder
folder-pinned-group = Pinned
folder-pinned-unpin-zone = Drop here to unpin

settings-folders-subscribe-selected = Subscribe
settings-folders-unsubscribe-selected = Unsubscribe
settings-folders-delete-selected = Delete…
settings-folders-subscribed = { $count ->
    [one] Subscribed to { $count } folder
   *[other] Subscribed to { $count } folders
}
settings-folders-unsubscribed = { $count ->
    [one] Unsubscribed from { $count } folder
   *[other] Unsubscribed from { $count } folders
}
settings-folders-subscribe-partial = Changed { $done } folders; { $failed } could not be changed.
settings-folders-subscribe-failed = The subscriptions could not be changed.
settings-folders-bulk-delete-none = None of the selected folders can be deleted (system folders and folders you may not delete are skipped).
settings-folders-bulk-delete-title = { $count ->
    [one] Delete { $count } folder?
   *[other] Delete { $count } folders?
}
settings-folders-bulk-delete-body = { $folders ->
    [one] { $folders } folder and the { $messages } messages in it will be deleted. This cannot be undone.
   *[other] { $folders } folders (with their subfolders) and the { $messages } messages in them will be deleted. This cannot be undone.
}
settings-folders-bulk-delete-confirm = { $count ->
    [one] Delete { $count } folder
   *[other] Delete { $count } folders
}
settings-folders-bulk-delete-list = Folders to delete
settings-folders-bulk-delete-skipped = { $count } selected folders are skipped because they cannot be deleted: { $names }
settings-folders-bulk-delete-done = { $count ->
    [one] Deleted { $count } folder
   *[other] Deleted { $count } folders
}
settings-folders-bulk-delete-partial = Deleted { $done } folders. Not deleted: { $names }
settings-folders-bulk-delete-failed = Nothing was deleted. Not deleted: { $names }
