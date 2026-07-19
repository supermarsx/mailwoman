# Mailwoman — calendar module strings (source locale: en).
# Lazily loaded by the calendar area (loadCatalog('calendar')). ids are
# kebab-case + module-prefixed (calendar-*). User-controlled event titles are
# isolated at the call site with isolate() before interpolation.

# -- Toolbar / chrome --------------------------------------------------------
calendar-title = Calendar
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
calendar-resolver-update-note = Rescheduling or shortening an event with attendees sends them an update.
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
calendar-fb-caption = Attendee availability across the conflict window, by hour.
calendar-fb-attendee = Attendee
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
