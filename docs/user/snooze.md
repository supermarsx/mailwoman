# Snooze messages

Choose **Snooze** in a message row or its context menu. **Choose date and time** opens a picker using the display timezone from Regional settings. Times must be in the future. Nonexistent daylight-saving times are rejected; repeated times require you to choose an occurrence. The message uses the existing snooze, resurfacing and undo mechanisms.

Manage shortcuts under **Settings → Message display → Snooze**. Add, rename, edit, reorder, hide or delete presets, then save the menu. Restoring built-in presets changes the draft menu; save to apply it. Removing every preset still leaves the custom picker available. Settings search opens this tab directly.

The built-in menu includes later today, tomorrow, next week, 2/6/24 hours, 3/7 calendar days, end of week/month and the first day of next month. Calendar options have an editable time of day. Next week means the next regional first day of the week. End of week uses the regional last day. If an end-of-period time has already passed, the next period is used. Hours represent elapsed time; calendar rules follow the configured display timezone and can differ across daylight-saving changes. A calendar preset whose wall time does not exist is omitted from the menu; the custom picker remains available.

Administrators manage `mail.snooze.presets` in the Mail settings category with the same preset editor. The normal global, domain, group and per-user defaults and locks apply. At most 32 presets are allowed, with stable unique IDs, positive whole intervals, valid times and names of up to 80 characters. The existing `cap.snooze` permission still governs message changes.
