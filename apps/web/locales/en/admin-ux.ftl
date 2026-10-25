# Mailwoman — admin UX seam: page templates, source badges, impact preview,
# field history, revert, concurrency notice, stream view and the console's own
# appearance control (source locale: en). t32-cfg.md §3 (UX1–UX20), DC5.
# Ids are `admin-ux-*`. Shared words that existed before (Save, Cancel, Try again,
# Loading) stay in admin-shell.ftl.

admin-ux-close = Close

# -- Page states (UX11) ---------------------------------------------------------
# $reason — a sentence saying why the data is not there.
admin-ux-state-unavailable = Not available — { $reason }
admin-ux-unavailable-default = this server does not offer it yet.
# $code — the server's error code or "HTTP <status>".
admin-ux-state-code = Error code: { $code }

# -- List and detail ----------------------------------------------------------
admin-ux-list-open = Open
admin-ux-list-back = Back to the list

# -- Settings form ------------------------------------------------------------
admin-ux-form-no-changes = No unsaved changes
admin-ux-form-changes =
    { $count ->
        [one] 1 unsaved change
       *[other] { $count } unsaved changes
    }
admin-ux-form-discard = Discard changes
admin-ux-form-saving = Saving…
admin-ux-form-saved =
    { $count ->
        [one] Saved 1 change.
       *[other] Saved { $count } changes.
    }
admin-ux-form-fix =
    { $count ->
        [one] Nothing was saved. Fix the field marked below.
       *[other] Nothing was saved. Fix the { $count } fields marked below.
    }
admin-ux-learn-more = Learn more
# Added for screen readers after "Learn more". $label — the field label.
admin-ux-learn-more-field = about { $label }
admin-ux-propagation-hot = Applies within 30 seconds on every server process.
admin-ux-propagation-restart = Applies after the server restarts.
admin-ux-secret-set = A value is stored. Type a new one to replace it.
admin-ux-secret-unset = No value is stored.

# -- Field validation (UX5) ---------------------------------------------------
admin-ux-err-required = Enter a value.
admin-ux-err-number = Enter a number.
admin-ux-err-whole = Enter a whole number.
# $min, $max — the bounds the server declares for this field.
admin-ux-err-min = Enter { $min } or more.
admin-ux-err-max = Enter { $max } or less.
admin-ux-err-max-length = Use at most { $max } characters.
admin-ux-err-url = Enter a full address starting with https:// or http://.
admin-ux-err-email = Enter an email address such as name@example.org.
admin-ux-err-pattern = This value does not have the expected form.
admin-ux-err-option = Choose one of the listed options.
admin-ux-err-out-of-range = The server refused this value because it is outside the allowed range.
admin-ux-err-too-long = The server refused this value because it is too long.
admin-ux-err-invalid = The server refused this value. Check it and save again.
admin-ux-err-pinned = This value is set in the server environment and cannot be changed here.
admin-ux-err-locked = This value is locked at a wider scope and cannot be changed here.

# -- Source badge (UX9) -------------------------------------------------------
admin-ux-source-default = Default
admin-ux-source-stored = Stored
# $name — the environment variable, e.g. MW_LOG_LEVEL.
admin-ux-source-env-pin = Set by { $name }
admin-ux-source-env-fallback = Fallback from { $name }
admin-ux-source-build = Set at build time
# $scope — "global" or a domain name.
admin-ux-source-locked = Locked by { $scope }
admin-ux-source-read-only = Read-only
admin-ux-source-restart = Restart needed
admin-ux-source-env-pin-reason = The server environment sets this value with { $name }. Change it there and restart the server.
admin-ux-source-build-reason = This value is fixed when the server is built.
admin-ux-source-locked-reason = A lock at { $scope } scope fixes this value. Remove the lock there to change it.
admin-ux-source-read-only-reason = The console shows this value but cannot change it. The documentation says where it is set.

# -- History (UX8) ------------------------------------------------------------
# $who — an admin name; $time — date and time in the reader's locale.
admin-ux-history-changed-by-at = Last changed by { $who } on { $time }
admin-ux-history-changed-by = Last changed by { $who }
admin-ux-history-changed-at = Last changed on { $time }
admin-ux-history-never = Not changed from the console
admin-ux-history-link = History

# -- Revert to default (UX7) --------------------------------------------------
# $value — the default as shown, e.g. "587".
admin-ux-revert-default-is = Default: { $value }
admin-ux-revert = Revert to default
# Added for screen readers after "Revert to default". $label — the field label.
admin-ux-revert-field = ({ $label })
admin-ux-revert-done = Reverted to the default ({ $value }).

# -- Changed by someone else (UX19) -------------------------------------------
admin-ux-stale-by-at = { $who } changed this on { $time } while you were editing.
admin-ux-stale-by = { $who } changed this while you were editing.
admin-ux-stale-unknown = Someone else changed this while you were editing.
admin-ux-stale-choice = Reload to see their version and lose your edits, or keep yours and save them over theirs.
admin-ux-stale-reload = Reload
admin-ux-stale-keep = Keep mine

# -- Impact preview (UX6) -----------------------------------------------------
admin-ux-impact-checking = Checking what this change affects…
admin-ux-impact-heading = This change affects
# $label — what is affected (e.g. "Users refused"); $count — how many.
admin-ux-impact-count = { $label }: { $count }
admin-ux-impact-more =
    { $count ->
        [one] and 1 more
       *[other] and { $count } more
    }
admin-ux-impact-none = Nothing else changes.
admin-ux-impact-unavailable = Impact preview not available — the server did not report what this change affects.

# -- Stream view --------------------------------------------------------------
admin-ux-stream-filter = Filter
admin-ux-stream-pause = Pause
admin-ux-stream-resume = Resume
admin-ux-stream-clear = Clear
admin-ux-stream-announce = Read new entries aloud
admin-ux-stream-connecting = Connecting…
admin-ux-stream-live = Live
admin-ux-stream-ended = Ended
admin-ux-stream-lost = Connection lost
admin-ux-stream-reconnect = Reconnect
admin-ux-stream-paused =
    { $count ->
        [0] Paused
        [one] Paused — 1 new entry held
       *[other] Paused — { $count } new entries held
    }
admin-ux-stream-no-match = No entry matches the filter.

# -- Command palette ----------------------------------------------------------
admin-ux-palette-kind-field = Field

# -- Console appearance (DC5) -------------------------------------------------
admin-ux-appearance = Appearance
admin-ux-appearance-title = Console appearance
admin-ux-appearance-note = These choices change only the admin console, in this browser.
admin-ux-appearance-following = The console follows this browser's webmail theme until you choose here. These choices change only the admin console, in this browser.
admin-ux-appearance-mode = Mode
admin-ux-appearance-mode-light = Light
admin-ux-appearance-mode-dark = Dark
admin-ux-appearance-mode-system = Match the system
admin-ux-appearance-theme = Theme
admin-ux-appearance-accent = Accent colour
admin-ux-appearance-light = Light
admin-ux-appearance-dark = Dark
admin-ux-appearance-reset = Use the webmail theme

# -- List editor (f33) ----------------------------------------------------------
admin-ux-list-add = Add
admin-ux-list-remove = Remove { $item }
admin-ux-list-none = None yet.
admin-ux-list-duplicate = { $item } is already in the list.
admin-ux-list-empty-entry = Type an entry first.
