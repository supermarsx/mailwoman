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
