send-delay-title = Undo send
send-delay-help = After you press Send, the message waits in your Outbox for this long. Until then you can recall it and keep editing. The server holds it, so closing the browser does not send it early or lose it.
send-delay-label = Hold sent messages for
send-delay-off = Off (send immediately)
send-delay-seconds = { $n } seconds
send-delay-minutes = { $n ->
    [one] 1 minute
   *[other] { $n } minutes
}
send-delay-custom = Custom…
send-delay-custom-label = Custom delay in seconds
send-delay-custom-help = Whole seconds, 0 to { $max }.
send-delay-capped = Your administrator limits this to { $max }.
send-delay-disabled = Recalling sent messages is turned off by your administrator. Messages are sent immediately.
send-delay-invalid = Enter a whole number of seconds from 0 to { $max }.

send-later-button = Send later
send-later-menu = Send later options
send-later-preset-hour = In 1 hour
send-later-preset-today = Later today, 17:00
send-later-preset-tomorrow = Tomorrow morning, 09:00
send-later-preset-monday = Monday morning, 09:00
send-later-custom = Pick date and time…
send-later-custom-label = Send at
send-later-zone = Time zone: { $zone }
send-later-schedule = Schedule send
send-later-cancel = Cancel
send-later-err-invalid = Enter a valid date and time.
send-later-err-past = Choose a time in the future.
send-later-err-horizon = That is too far ahead. Messages can be scheduled up to { $days } days in advance.
send-later-err-off = Scheduled sending is turned off by your administrator.

send-queued = Message queued — sends in
send-queued-undo = Undo
send-queued-now = Send now
send-queued-recalled = Message recalled. It is back in your drafts.
send-queued-too-late = Too late to recall: the message is already being sent.
send-queued-sent-now = Sending now.
send-queued-scheduled = Scheduled for { $when } ({ $zone }).

outbox-scheduled-badge = Scheduled
outbox-sends-in = Sends in { $time }
outbox-sends-at = Sends { $when }
outbox-recall-edit = Recall and edit
outbox-recall-too-late = Already being sent; it can no longer be recalled.
outbox-failed-drafts = Could not be sent and was kept in your drafts.
outbox-queued-count = { $count ->
    [one] 1 message waiting to send
   *[other] { $count } messages waiting to send
}

## Outbox reader, edit send time, queued-count chip
outbox-open = Open
outbox-reader-label = Queued message
outbox-reader-actions = Queued message actions
outbox-reader-loading = Loading the message…
outbox-reader-unavailable = The message could not be loaded.
outbox-reader-to = To: { $recipients }
outbox-sent-copy-folder = Will be saved in { $folder }
outbox-sent-copy-missing = Will be saved in Sent (the chosen folder is not available)
outbox-sent-copy-skip = No copy will be saved
outbox-reader-cc = Cc: { $recipients }
outbox-status-held = Held until you release it
outbox-status-queued = Queued
outbox-status-scheduled = Scheduled
outbox-status-sending = Sending
outbox-status-retrying = Retrying
outbox-status-failed = Not sent
outbox-status-uncertain = Delivery unknown
outbox-status-sent = Sent
outbox-status-canceled = Canceled
outbox-status-sends-at-in = Sends { $when } (in { $time })
outbox-status-next-attempt = Next attempt { $when }
outbox-status-attempts = { $count ->
    [one] 1 attempt so far
   *[other] { $count } attempts so far
}
outbox-status-last-error = Last error: { $error }
outbox-edit-time = Edit send time
outbox-edit-time-current = Currently { $when }
outbox-edit-time-save = Save send time
outbox-edit-time-conflict = The send time changed or the message was already sent. The Outbox has been reloaded.
outbox-edit-time-refused = The send time could not be changed. { $error }
outbox-recall-to-drafts = Recall to drafts
outbox-send-again = Send again
outbox-send-again-queued = The message is queued to send again.
outbox-send-again-failed = Could not queue the message again. { $error }
outbox-delete = Delete
outbox-delete-confirm = Delete message?
outbox-deleted = The queued message was deleted.
outbox-delete-failed = The message could not be deleted. { $error }
outbox-failed-toast = A message could not be sent. { $error }
outbox-failed-uncertain-toast = A message may or may not have been delivered. Check Sent before sending it again.
outbox-chip-title = Messages waiting to send
outbox-chip-next = Next send: { $when }
outbox-chip-view-all = Open Outbox
