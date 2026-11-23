---
title: Calendar
screen: true
---

# Calendar

**Calendar** shows your events in a day, week, month or other view, lets you add and
edit events, subscribe to calendars published elsewhere, and share a calendar with
other people. Open it from **Calendar** in the sidebar.

![The Calendar screen in week view for 8 to 14 March 2026: the toolbar on top, the date navigator and calendar list on the left, and the week's events in the time grid.](/screenshots/user-features/cal-week.webp){.mw-shot}

## The toolbar

![The calendar toolbar with markers on Today, the Week view tab, New event, the quick add field, Import and Export.](/screenshots/user-features/cal-toolbar.webp){.mw-crop}

1. **Today** moves to today. The arrow buttons on either side, **Previous** and
   **Next**, move by one period of the current view.
2. The view tabs: **Day**, **3 Day**, **Work Week**, **Week**, **Month**,
   **Quarter**, **Schedule**, **Agenda** and **Year**. The selected tab is
   highlighted.
3. **New event** opens an empty event form.
4. **Quick add** takes a one-line description of an event. See
   [Add an event with a sentence](#add-an-event-with-a-sentence).
5. **Import** reads an event file from your computer.
6. **Export** downloads all your events as `mailwoman-calendar.ics`.
{.mw-callouts}

The title on the left names the period shown, for example **March 2026**. When some
events overlap, a button reading "Resolve 1 conflict" (or the number of conflicts)
appears beside **New event**.

![The calendar in month view for March 2026 with the week's events on their days.](/screenshots/user-features/cal-month.webp){.mw-crop}

**Schedule** and **Agenda** list events in order instead of drawing a grid. In every
view, click an event to open it. Right-click an event for **Open**, **Duplicate**,
**Export** and **Delete**, and right-click an empty day for **New event** on that
day, or to jump to that day's **Day** or **Week** view.

## Calendars, filter and subscriptions

The left pane holds the date navigator and the calendar list.

![The calendar side panel with markers on the date navigator, a calendar's visibility checkbox, its Share button, Add calendar, the category filter and the Subscribe button.](/screenshots/user-features/cal-sidebar.webp){.mw-crop}

1. The date navigator is a month grid. Click a day to go to it. The arrows move
   between months, and **Today** returns to today.
2. The checkbox beside a calendar shows or hides its events. The square beside it is
   the calendar's colour.
3. **Share** opens the sharing dialog for that calendar.
4. **Add calendar** creates a calendar.
5. **Category to show** filters the events by one category.
6. **Subscribe** adds the calendar at the address typed in **Subscribe by URL**
   (for example `https://example.com/calendar.ics`). A subscribed calendar is read
   only. If the address cannot be fetched, a message says so and nothing is added.
   Use the **Refresh** button on a subscribed calendar to fetch it again.
{.mw-callouts}

Under **Holidays**, **Add a region…** imports the public holidays of a region into
your calendar.

## Add an event

1. Choose **New event**. To start from a day, right-click it and choose **New event**.
2. Enter a **Title**. Set **Start** and **Duration (min)**, or tick **All day**.
   Optionally set a **Location**.
3. Choose **Save**.

![The New event form with the title Budget review and the location Meeting room 2 filled in, and markers on its main fields.](/screenshots/user-features/cal-editor.webp){.mw-crop}

1. **Title**.
2. **Calendar** chooses which calendar the event belongs to.
3. **All day** makes it an all-day event.
4. **Start** is the start date and time.
5. **Duration (min)** is the length in minutes.
6. **Repeats** makes the event recur.
7. **+ reminder** adds a reminder.
8. **name@example.com** adds an attendee.
{.mw-callouts}

The form also has:

- **Event time zone**, an IANA name such as `Europe/Lisbon`. The event keeps this
  zone when viewed elsewhere. It starts as **Use browser time zone**.
- **Shows as** **Busy** or **Free**, and a status of **Confirmed**, **Tentative** or
  **Cancelled**.
- **Reminders**: **5 min**, **15 min**, **30 min**, **1 hour** or **1 day** before.
  Each reminder has a remove button.
- **Attendees**: type an email address and choose **Add**. Each attendee has a role
  (**Chair**, **Required**, **Optional** or **Non-participant**) and a type
  (**Person**, **Group**, **Resource** or **Room**), and shows their reply: **No
  reply**, **Accepted**, **Declined** or **Tentative**.
- **Categories**, **Attachments** (a name and a link) and **Notes**.

To change an event, click it, edit the form, and choose **Save**. **Cancel** closes the
form, and **Delete** deletes the event.

### Repeating events

Tick **Repeats**. Choose a **Frequency** (**Daily**, **Weekly**, **Monthly** or
**Yearly**) and an **Interval**; for weekly events tick the weekdays. Under **Ends**
choose **Never**, **After N** occurrences or **On date**. A line below the fields
sums up the rule, for example "Every week".

![The New event form with Repeats ticked, showing the Frequency, Interval and Ends fields.](/screenshots/user-features/cal-repeat.webp){.mw-crop}

1. **Frequency**.
2. **Interval**.
3. **Ends**.
{.mw-callouts}

### Add an event with a sentence

Type a sentence in the quick add field and choose **Add**. The event goes into the
first visible calendar. Mailwoman reads these parts of the sentence and leaves the
rest as the title:

- the day: `today`, `tonight`, `tomorrow`, a weekday name with an optional `next`, a
  date such as `2026-03-13` or `3/13`, or a month name and a day;
- the time: `3pm`, `3:30pm`, `15:00`, `noon` or `midnight`, optionally after `at`, or
  a range such as `3pm-4pm`;
- a length: `for 90m`, `for 2h` or `for 1h30m`;
- a place: a word starting with `@`, for example `@Cafe`.

A day without a time makes an all-day event. For example, "Lunch with Sam tomorrow at
1pm @Cafe for 90m" creates a 90-minute event titled "Lunch with Sam" at the cafe.

## Share a calendar

Choose **Share** beside a calendar.

![The Share Calendar dialog: a note about access, the list of people it is shared with, and a field to add a person.](/screenshots/user-features/cal-share.webp){.mw-crop}

Type the person's email address, choose **Read only** or **Read & write**, and choose
**Add**. People you share with can see the calendar; with read-write access they can
also add and edit events. To change access or stop sharing, use the access menu and
**Remove** beside the person's name in **Shared with**. Choose **Close** when done.

## Import and export

**Import** accepts `.ics` calendar files and `.hol` holiday files. After an import the
calendar reports how many events were imported and how many were skipped because the
calendar already had them. A file that cannot be read as a calendar is refused with
"That file could not be read as a calendar. Nothing was imported." A holiday file for
several locations asks you to choose the locations to import.

## Invitations in messages

An invitation or update that arrives by email shows an event card in the message with
the time, place, organizer and attendees, and whether it conflicts with your calendar.
The card offers **Add to calendar**, **Apply event update**, **Apply cancellation** or
**Record attendee reply**, depending on the message. When you are an invited attendee,
the event form opened from your calendar shows **Accept**, **Tentative**, **Decline**
and **Counter…**, where you can propose a new start.

## Troubleshooting

- **The event was not saved. The server refused it.** The event form stays open. Check
  the fields, in particular the time zone and a start time that exists on that day.
- **Subscribing fails.** "The calendar at that address could not be fetched, so it was
  not added." Check the address.

## Related

- [Tasks](/user/tasks)
- [Contacts](/user/contacts)
- [Sync and offline access](/user/offline-sync)
