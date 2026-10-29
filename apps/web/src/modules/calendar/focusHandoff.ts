// "Open in calendar" from outside the calendar module (t53: the reader's event
// card). The caller names a day; a calendar controller created later starts on
// it, and one already mounted moves to it. No URL carries the date, so a
// message never steers the calendar by itself: only the user's click does.

type Listener = (day: Date) => void;
let pending: Date | null = null;
const listeners = new Set<Listener>();

/** Ask the calendar to show `day`. */
export function requestCalendarFocus(day: Date): void {
  if (Number.isNaN(day.getTime())) return;
  if (listeners.size === 0) pending = day;
  else for (const listener of listeners) listener(day);
}

/** The day asked for before any calendar was mounted, once. */
export function takeCalendarFocus(): Date | null {
  const day = pending;
  pending = null;
  return day;
}

/** Follow later requests; returns the unsubscribe. */
export function onCalendarFocus(listener: Listener): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}
