# Mailwoman — calendar module strings (source locale: en).
# Lazily loaded by the calendar area (loadCatalog('calendar')). ids are
# kebab-case + module-prefixed (calendar-*). User-controlled event titles are
# isolated at the call site with isolate() before interpolation.

# -- Toolbar / chrome --------------------------------------------------------
calendar-title = Calendar
calendar-mini = Date navigator
calendar-prev = Previous
calendar-next = Next
calendar-today = Today
calendar-views = Calendar views
calendar-new-event = New event
calendar-import = Import
calendar-export = Export
calendar-import-file = Import calendar file

# -- View names --------------------------------------------------------------
calendar-view-day = Day
calendar-view-3day = 3 Day
calendar-view-work-week = Work Week
calendar-view-week = Week
calendar-view-month = Month
calendar-view-tri-month = Quarter
calendar-view-schedule = Schedule
calendar-view-agenda = Agenda
calendar-view-year = Year

# -- Sidebar -----------------------------------------------------------------
calendar-calendars-heading = Calendars
calendar-toggle = Toggle { $name }
calendar-color-for = Color for { $name }
calendar-add-calendar = Add calendar
calendar-new-calendar-name = New calendar
calendar-holidays-heading = Holidays
calendar-subscribe-holidays = Subscribe to holidays
calendar-add-region = Add a region…
calendar-synced = Synced calendar

# -- Grid / views ------------------------------------------------------------
calendar-all-day = all-day
calendar-all-day-full = All day
calendar-more = { $count } more
calendar-conflict = conflict
calendar-no-events-30 = No events in the next 30 days.
calendar-events-count = { $count ->
    [zero] No events
    [one] { $count } event
   *[other] { $count } events
}
# The month grid's accessible name, e.g. "Month of July 2026".
calendar-month-grid = Month of { $month }
# A time-grid's accessible name for the visible day range.
calendar-time-grid = Time grid
# A single day cell's accessible name: full date + its event count.
calendar-cell = { $date }, { $events }
# The live-region text announced as focus moves between day cells.
calendar-announce = { $date }, { $events }
# One event's accessible name: start time + (isolated) title.
calendar-event-at = { $time } { $title }
calendar-event-allday = All day: { $title }

# -- Event editor ------------------------------------------------------------
calendar-editor-new = New event
calendar-editor-edit = Edit event
calendar-invitation = Invitation
calendar-invited = You’re invited ({ $status }).
calendar-accept = Accept
calendar-tentative = Tentative
calendar-decline = Decline
calendar-counter = Counter…
calendar-propose-start = Propose new start
calendar-send-counter = Send counter
calendar-overlaps = This event overlaps { $count ->
    [one] { $count } other event
   *[other] { $count } other events
}.
calendar-field-title = Title
# Stored as the title of an event saved with the title left empty.
calendar-untitled = (no title)
calendar-field-calendar = Calendar
calendar-field-all-day = All day
calendar-field-start = Start
calendar-field-duration = Duration (min)
calendar-field-location = Location
calendar-field-shows-as = Shows as
calendar-shows-busy = Busy
calendar-shows-free = Free
calendar-field-status = Status
calendar-status-confirmed = Confirmed
calendar-status-tentative = Tentative
calendar-status-cancelled = Cancelled
calendar-repeats = Repeats
calendar-frequency = Frequency
calendar-freq-daily = Daily
calendar-freq-weekly = Weekly
calendar-freq-monthly = Monthly
calendar-freq-yearly = Yearly
calendar-every = every
calendar-interval = Interval
calendar-ends = Ends
calendar-end-mode = End mode
calendar-end-never = Never
calendar-end-count = After N
calendar-end-until = On date
calendar-occurrences = Occurrences
calendar-until-date = Until date
calendar-reminders = Reminders
calendar-reminder-before = { $min }m before
calendar-remove-reminder = Remove reminder { $min }
calendar-add-reminder = Add reminder
calendar-reminder-none = + reminder
calendar-reminder-5 = 5 min
calendar-reminder-15 = 15 min
calendar-reminder-30 = 30 min
calendar-reminder-60 = 1 hour
calendar-reminder-1440 = 1 day
calendar-attendees = Attendees
calendar-add-attendee = Add attendee
calendar-attendee-invalid = That is not an email address. Enter one address, like name@example.com.
calendar-save-failed = The event was not saved. The server refused it.
calendar-remove-attendee = Remove { $email }
calendar-notes = Notes

# -- Attendee roles / kind (iCal ROLE / CUTYPE) ------------------------------
calendar-attendee-role-for = Role for { $email }
calendar-attendee-cutype-for = Type for { $email }
calendar-role-chair = Chair
calendar-role-required = Required
calendar-role-optional = Optional
calendar-role-non-participant = Non-participant
calendar-cutype-individual = Person
calendar-cutype-group = Group
calendar-cutype-resource = Resource
calendar-cutype-room = Room
calendar-partstat-needs-action = No reply
calendar-partstat-accepted = Accepted
calendar-partstat-declined = Declined
calendar-partstat-tentative = Tentative

# -- Schedule view (distinct from Agenda) ------------------------------------
# Accessible name for one schedule row: start–end time + (isolated) title.
calendar-schedule-row = { $start } to { $end }: { $title }
calendar-schedule-free = { $gap } free
calendar-gap-hm = { $h }h { $m }m
calendar-gap-h = { $h }h
calendar-gap-m = { $m }m

# -- Conflict resolver -------------------------------------------------------
calendar-resolve-conflicts = Resolve { $count ->
    [one] { $count } conflict
   *[other] { $count } conflicts
}
calendar-resolver-title = Resolve conflicts
calendar-resolver-none = No conflicts to resolve.
calendar-resolver-pick = Conflict
calendar-resolver-pair-n = { $n } of { $total }: { $a } vs { $b }
calendar-resolver-time = Time
calendar-resolver-overlap = Overlap { $start } – { $end }
calendar-resolver-reschedule = Reschedule later event
calendar-resolver-shorten = Shorten earlier event
calendar-resolver-tentative = Mark later tentative
calendar-resolver-double-book = Double-book (mark free)
calendar-resolver-keep = Keep both
calendar-events-count-people = { $count ->
    [zero] No attendees
    [one] { $count } attendee
   *[other] { $count } attendees
}

# -- Free/busy grid ----------------------------------------------------------
calendar-fb-caption-people = Busy time on the day of the conflict, by hour, for you and each person invited.
calendar-fb-whose = Calendars
calendar-fb-own = Yours
# A grid row for an invited person whose busy time no source could give
# (no shared calendar on this server, no answer from the calendar server).
calendar-fb-unknown = Availability could not be retrieved for { $principal }.
calendar-fb-unavailable = Your busy time could not be loaded.
calendar-fb-loading = Loading availability…
calendar-fb-cell = { $principal } at { $hour }: { $status }
calendar-fb-busy = Busy
calendar-fb-tentative = Tentative
calendar-fb-free = Free

# -- Quick add (P3): natural-language event line ------------------------------
calendar-quick-add = Quick add event
calendar-quick-add-placeholder = Quick add, e.g. Lunch Friday 1pm
calendar-quick-add-btn = Add
calendar-quick-add-do = Quick add

# -- Categories (P4) ---------------------------------------------------------
calendar-categories = Categories
calendar-add-category = Add category
calendar-category-placeholder = Add a category
calendar-remove-category = Remove category { $name }
calendar-filter-heading = Filter
calendar-filter-category = Filter by category
calendar-filter-category-placeholder = Category to show

# -- Attachments (P5) --------------------------------------------------------
calendar-attachments = Attachments
calendar-attachment-title = Attachment name
calendar-attachment-title-placeholder = Name (optional)
calendar-attachment-uri = Attachment link
calendar-attachment-add = Add attachment
calendar-remove-attachment = Remove attachment { $name }

# -- Subscribe to a calendar URL (P6) ----------------------------------------
calendar-subscribe-heading = Subscribe by URL
calendar-subscribe-url = Calendar URL
calendar-subscribe-add = Subscribe
calendar-subscribe-url-placeholder = https://example.com/calendar.ics
calendar-subscribe-failed = The calendar at that address could not be fetched, so it was not added.
calendar-subscribe-hol-many-locations = This holiday file lists more than one location. Download it, then use Import and pick the locations you want.
calendar-subscribe-refresh = Refresh { $name }
calendar-subscribe-refresh-btn = Refresh
calendar-subscribe-refresh-failed = The calendar feed could not be fetched. Its events were left as they were.
calendar-import-failed = That file could not be read as a calendar. Nothing was imported.
# The file is an iTIP CANCEL, REPLY or COUNTER: a scheduling message that only
# means something with the e-mail it came in (whose sender it is checked against).
calendar-import-itip-message = This file is a reply or cancellation sent by e-mail. Open the message it came with to apply it. Nothing was imported.
# Appended to calendar-import-done when some events were already there.
calendar-import-skipped = { $count ->
    [one] { $count } event was already in the calendar and was left as it is.
   *[other] { $count } events were already in the calendar and were left as they are.
}
calendar-import-done = { $count ->
    [one] Imported 1 event.
   *[other] Imported { $count } events.
}
# The location choice shown for a .hol holiday file with more than one location.
calendar-import-locations-title = Choose locations to import
calendar-import-locations-intro = This file holds holidays for { $count } locations. Only the locations you select are imported.
calendar-import-locations-filter = Filter locations
calendar-import-locations-list = Locations in the file
calendar-import-locations-none = No location matches the filter.
calendar-import-locations-picked = { $count ->
    [one] 1 location selected.
   *[other] { $count } locations selected.
}
calendar-import-locations-import = Import selected
calendar-export-failed = The calendar could not be exported.
calendar-quick-add-failed = That event could not be added.
calendar-attachment-uri-placeholder = https://example.com/file

# -- Calendar sharing (P1) ---------------------------------------------------
calendar-share = Share
calendar-share-for = Share { $name }
calendar-share-title = Share { $name }
calendar-share-intro = People you share with can see this calendar. Give read-write access to let them add and edit events.
calendar-share-people = Shared with
calendar-share-empty = This calendar is not shared with anyone.
calendar-share-add = Add a person
calendar-share-new-access = Access for the new person
calendar-share-access-for = Access for { $principal }
calendar-share-remove = Remove { $principal }
calendar-share-read = Read only
calendar-share-readwrite = Read & write

# -- Received invitations ---------------------------------------------------
calendar-invite-title = Calendar invitation
calendar-invite-sender = From { $sender }
calendar-invite-add = Add to calendar
calendar-invite-update = Apply event update
calendar-invite-cancel = Apply cancellation
calendar-invite-reply = Record attendee reply
calendar-invite-new = This invitation can be added to your calendar.
calendar-invite-newer = The organizer sent an updated event.
calendar-invite-current = Your calendar already has this version.
calendar-invite-stale = This message is older than the event in your calendar.
calendar-invite-cancelled = The organizer cancelled this event.
calendar-invite-response = An attendee sent a response.
calendar-invite-not-organizer = The sender is not the stored organizer. Nothing will be changed.
calendar-invite-not-attendee = The sender is not a matching attendee. Nothing will be changed.
calendar-invite-unknown = This message refers to an event that is not in your calendar.
calendar-invite-unsupported = This proposal cannot be applied automatically.
calendar-invite-no-reply = This updates your calendar without sending a reply.
calendar-invite-applied = Your calendar was updated.
calendar-invite-unchanged = The event was not changed. Review the invitation's current status.
calendar-invite-apply-failed = The invitation could not be applied. Try again.
calendar-invite-load-failed = The invitation could not be checked.
calendar-invite-retry = Retry

calendar-field-time-zone = Event time zone
calendar-field-time-zone-help = Use an IANA name such as Europe/Lisbon. The event keeps this zone when viewed elsewhere.
calendar-time-invalid = Enter a valid time zone and local time. Some local times do not exist when clocks move forward.

calendar-mini-prev = Previous month
calendar-mini-next = Next month

## t53 — the Outlook-style event card in the reader and the .ics preview
calendar-card-region-event = Calendar event
calendar-card-kind-request = Meeting invitation
calendar-card-kind-update = Updated invitation
calendar-card-kind-cancel = Cancelled meeting
calendar-card-kind-reply = Meeting response
calendar-card-kind-counter = New time proposed
calendar-card-kind-publish = { $count ->
    [one] Calendar event
   *[other] { $count } calendar events
}
calendar-card-when = When
calendar-card-where = Where
calendar-card-organizer = Organizer
calendar-card-attendees = Attendees
calendar-card-all-day = { $when } (all day)
calendar-card-organizer-time = Organizer’s time: { $when } ({ $zone })
calendar-card-repeat-daily = { $interval ->
    [one] Every day
   *[other] Every { $interval } days
}
calendar-card-repeat-weekly = { $interval ->
    [one] Every week
   *[other] Every { $interval } weeks
}
calendar-card-repeat-weekly-on = { $interval ->
    [one] Every { $days }
   *[other] Every { $interval } weeks on { $days }
}
calendar-card-repeat-monthly = { $interval ->
    [one] Every month
   *[other] Every { $interval } months
}
calendar-card-repeat-monthly-day = { $interval ->
    [one] Every month on day { $day }
   *[other] Every { $interval } months on day { $day }
}
calendar-card-repeat-yearly = { $interval ->
    [one] Every year
   *[other] Every { $interval } years
}
calendar-card-repeat-until = { $rule } until { $date }
calendar-card-repeat-count = { $rule }, { $count ->
    [one] once
   *[other] { $count } times
}
calendar-card-repeat-except = { $rule } ({ $count ->
    [one] 1 date skipped
   *[other] { $count } dates skipped
})
calendar-card-repeat-custom = Repeats on a custom schedule
calendar-card-join = Join { $provider }
calendar-card-organizer-is = Organized by { $who }
calendar-card-attendee-summary = { $count ->
    [one] 1 attendee
   *[other] { $count } attendees
} · { $accepted } accepted, { $tentative } tentative, { $declined } declined
calendar-card-people-all = Show all { $count }
calendar-card-people-fewer = Show fewer
calendar-card-rsvp-accepted = Accepted
calendar-card-rsvp-tentative = Tentative
calendar-card-rsvp-declined = Declined
calendar-card-rsvp-none = No response yet
calendar-card-in-calendar = In your calendar
calendar-card-cancelled = Cancelled
calendar-card-open = Open in calendar
calendar-card-agenda = Your calendar on { $day }
calendar-card-agenda-title = Your day
calendar-card-agenda-this-event = This event
calendar-card-agenda-unavailable = Your calendar for this day could not be loaded.
calendar-card-conflicts = { $count ->
    [one] Conflicts with { $titles }
   *[other] Conflicts with { $count } events: { $titles }
}
calendar-card-no-conflicts = No conflicts in your calendar
calendar-card-invited-as = You were invited as { $address }.
calendar-card-not-invited = This invitation does not list any of your addresses. You can add it to your calendar, but you cannot answer it.
calendar-card-not-invited-short = This invitation does not list any of your addresses.
calendar-card-out-of-date = This invitation is out of date. A newer version is already in your calendar.
calendar-card-replied = { $who } { $answer ->
    [accepted] accepted.
    [tentative] tentatively accepted.
    [declined] declined.
   *[other] has not answered yet.
}
calendar-card-counter = { $who } proposed a new time. Proposals are not applied automatically.
calendar-card-todos-skipped = { $count ->
    [one] The file also holds 1 task, which is not added.
   *[other] The file also holds { $count } tasks, which are not added.
}
calendar-card-respond = Respond to the invitation
calendar-card-accept = Accept
calendar-card-tentative = Tentative
calendar-card-decline = Decline
calendar-card-propose = Propose new time
calendar-card-propose-start = New start
calendar-card-propose-send = Send proposal
calendar-card-propose-invalid = Enter a valid new start time.
calendar-card-send-response = Send a response to { $organizer }
calendar-card-comment = Comment (optional)
calendar-card-calendar = Calendar
calendar-card-events = Events in this file
calendar-card-add-one = Add
calendar-card-add-all = Add all ({ $count })
calendar-card-added = { $count ->
    [one] Added to your calendar.
   *[other] { $count } events added to your calendar.
}
calendar-card-remove = Remove from calendar
calendar-card-removed = Removed from your calendar.
calendar-card-record-reply = Update attendee status
calendar-card-answered-sent = { $answer }. Your response was sent to { $organizer }.
calendar-card-answered-not-sent = { $answer }. Your calendar was updated, but the response could not be sent.
calendar-card-answered-silently = { $answer }. No response was sent.
calendar-card-proposal-sent = Your proposal was sent to { $organizer }.
calendar-card-not-sent = The message could not be sent.
calendar-card-more = { $count ->
    [one] 1 more
   *[other] { $count } more
}
