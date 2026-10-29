import { describe, expect, it, vi } from 'vitest';
import { Suspense } from 'solid-js';
import { fireEvent, render, screen, waitFor, within } from '@solidjs/testing-library';
import type { JmapRequest, JmapResponse } from '../../api/jmap-types.ts';
import { EventCard } from './EventCard.tsx';
import { conflictTitles } from './DayAgenda.tsx';
import type { ItipView } from './api.ts';

const ORG = 'olga@example.com';
const ME = 'me@example.org';

function view(over: Partial<ItipView> = {}): ItipView {
  const event = {
    uid: 'mtg@example.com', title: 'Weekly sync', start: '2026-11-03T10:00:00', timeZone: 'UTC', duration: 'PT1H',
    description: 'Agenda\nhttps://example.com/doc', locations: [{ name: 'https://teams.microsoft.com/l/meetup-join/abc' }],
    recurrenceRules: [{ rrule: 'FREQ=WEEKLY;BYDAY=TU;UNTIL=20261203T235959Z' }], sequence: 1,
    participants: {
      [ORG]: { name: 'Olga', email: ORG, role: 'organizer', participationStatus: 'accepted' },
      [ME]: { name: 'Me', email: ME, role: 'attendee', participationStatus: 'needs-action', expectReply: true },
      'pat@example.net': { name: 'Pat', email: 'pat@example.net', role: 'attendee', participationStatus: 'tentative' },
    },
  };
  return {
    accountId: 'a', emailId: 'e', method: 'REQUEST', sender: ORG, organizer: ORG, me: ME, answer: null,
    event, items: [{ event, eventId: null, calendarId: null }], todos: 0, eventId: null, stored: null,
    outcome: 'new', applied: false, ...over,
  };
}

type Handler = (args: Record<string, unknown>) => object;

/** A JMAP stub answering by method name; records every call. */
function stub(handlers: Record<string, Handler>) {
  const calls: Array<[string, Record<string, unknown>]> = [];
  const jmap = vi.fn(async (request: JmapRequest): Promise<JmapResponse> => {
    const [name, args, id] = request.methodCalls[0]!;
    calls.push([name, args as Record<string, unknown>]);
    const fallback: Record<string, Handler> = {
      'Calendar/get': () => ({ accountId: 'a', list: [
        { id: 'c1', name: 'Work', color: '#2563eb', role: 'default', component: 'VEVENT' },
        { id: 'c2', name: 'Home', color: '#16a34a', role: null, component: 'VEVENT' },
        { id: 't1', name: 'Tasks', color: '#000', role: null, component: 'VTODO' },
      ] }),
      'CalendarEvent/expand': () => ({ accountId: 'a', list: [] }),
    };
    const handler = handlers[name] ?? fallback[name];
    return { sessionState: 's', methodResponses: [[name, handler ? handler(args as Record<string, unknown>) as Record<string, unknown> : { type: 'unknownMethod' }, id]] };
  });
  return { jmap, calls, of: (name: string) => calls.filter(([n]) => n === name).map(([, a]) => a) };
}

function card() {
  return screen.getByRole('region', { name: /Calendar (invitation|event)/ });
}

describe('the reader event card', () => {
  it('shows an invitation the Outlook way and accepts it with a comment and a reply', async () => {
    let answered = false;
    const s = stub({
      'CalendarEvent/itip': () => answered
        ? view({ outcome: 'current', eventId: 'ev1', stored: { calendarId: 'c1', status: 'confirmed', sequence: 1, participationStatus: 'accepted' } })
        : view(),
      'CalendarEvent/respond': () => { answered = true; return { accountId: 'a', emailId: 'e', action: 'accept', participationStatus: 'accepted', eventId: 'ev1', calendarId: 'c1', from: ME, to: ORG, sendResponse: true, sent: true }; },
    });
    render(() => <EventCard accountId="a" emailId="e" client={s} zone="UTC" />);
    expect(await screen.findByRole('heading', { name: 'Weekly sync' })).toBeInTheDocument();
    const region = card();
    expect(within(region).getByText('Meeting invitation')).toBeInTheDocument();
    expect(region.querySelector('[data-card="when"]')?.textContent).toContain('10:00');
    expect(region.querySelector('[data-card="recurrence"]')?.textContent).toContain('Every Tuesday until');
    expect(within(region).getByRole('link', { name: /Join Microsoft Teams/ })).toHaveAttribute('href', 'https://teams.microsoft.com/l/meetup-join/abc');
    expect(within(region).getByText(/You were invited as/)).toBeInTheDocument();
    expect(region.querySelector('[data-card="attendee-summary"]')?.textContent).toContain('2 attendees');
    // Nothing was applied or sent by opening the message.
    expect(s.of('CalendarEvent/respond')).toHaveLength(0);
    expect(s.of('CalendarEvent/itip')[0]).toMatchObject({ apply: false });
    await waitFor(() => expect(within(region).getByRole('combobox')).toBeInTheDocument());

    fireEvent.input(within(region).getByLabelText('Comment (optional)'), { target: { value: 'See you there' } });
    fireEvent.click(within(region).getByRole('button', { name: 'Accept' }));
    await waitFor(() => expect(s.of('CalendarEvent/respond')).toHaveLength(1));
    expect(s.of('CalendarEvent/respond')[0]).toEqual({ accountId: 'a', emailId: 'e', action: 'accept', sendResponse: true, comment: 'See you there', calendarId: 'c1' });
    expect(await within(region).findByText(`Accepted. Your response was sent to ⁨${ORG}⁩.`)).toBeInTheDocument();
    await waitFor(() => expect(region.querySelector('[data-card="badge"]')?.textContent).toBe('Accepted'));
    expect(within(region).getByRole('link', { name: 'Open in calendar' })).toHaveAttribute('href', '#/calendar/day');
    expect(within(region).getByRole('button', { name: 'Accept' })).toHaveAttribute('aria-pressed', 'true');
  });

  it('answers without sending when the response is turned off, in the chosen calendar', async () => {
    const s = stub({
      'CalendarEvent/itip': () => view(),
      'CalendarEvent/respond': () => ({ accountId: 'a', emailId: 'e', action: 'decline', participationStatus: 'declined', eventId: null, calendarId: null, from: ME, to: ORG, sendResponse: false, sent: false }),
    });
    render(() => <EventCard accountId="a" emailId="e" client={s} zone="UTC" />);
    await screen.findByRole('heading', { name: 'Weekly sync' });
    const region = card();
    await waitFor(() => expect(within(region).getByRole('combobox')).toBeInTheDocument());
    expect(within(region).getAllByRole('option').map((o) => o.textContent)).toEqual(['Work', 'Home']);
    fireEvent.change(within(region).getByRole('combobox'), { target: { value: 'c2' } });
    fireEvent.click(within(region).getByRole('checkbox', { name: /Send a response to/ }));
    expect(within(region).queryByLabelText('Comment (optional)')).not.toBeInTheDocument();
    fireEvent.click(within(region).getByRole('button', { name: 'Decline' }));
    await waitFor(() => expect(s.of('CalendarEvent/respond')).toHaveLength(1));
    expect(s.of('CalendarEvent/respond')[0]).toMatchObject({ action: 'decline', sendResponse: false, calendarId: 'c2' });
    expect(await within(region).findByText('Declined. No response was sent.')).toBeInTheDocument();
  });

  it('proposes a new time as a COUNTER in the event zone', async () => {
    const s = stub({
      'CalendarEvent/itip': () => view(),
      'CalendarEvent/respond': () => ({ accountId: 'a', emailId: 'e', action: 'counter', participationStatus: 'tentative', eventId: null, calendarId: null, from: ME, to: ORG, sendResponse: true, sent: true }),
    });
    render(() => <EventCard accountId="a" emailId="e" client={s} zone="Europe/Lisbon" />);
    await screen.findByRole('heading', { name: 'Weekly sync' });
    fireEvent.click(within(card()).getByRole('button', { name: 'Propose new time' }));
    fireEvent.input(within(card()).getByLabelText('New start'), { target: { value: '2026-11-03T15:30' } });
    fireEvent.click(within(card()).getByRole('button', { name: 'Send proposal' }));
    await waitFor(() => expect(s.of('CalendarEvent/respond')).toHaveLength(1));
    // Lisbon is UTC+0 in November; the event is in UTC.
    expect(s.of('CalendarEvent/respond')[0]).toMatchObject({ action: 'counter', counter: { start: '2026-11-03T15:30:00', duration: 'PT1H' } });
  });

  it('says an older invitation is out of date and offers no answer', async () => {
    const s = stub({ 'CalendarEvent/itip': () => view({ outcome: 'stale', eventId: 'ev1', stored: { calendarId: 'c1', status: 'confirmed', sequence: 3, participationStatus: 'accepted' } }) });
    render(() => <EventCard accountId="a" emailId="e" client={s} zone="UTC" />);
    expect(await screen.findByText('This invitation is out of date. A newer version is already in your calendar.')).toBeInTheDocument();
    expect(within(card()).queryByRole('button', { name: 'Accept' })).not.toBeInTheDocument();
    expect(within(card()).getByRole('link', { name: 'Open in calendar' })).toBeInTheDocument();
  });

  it('lets an invitation to none of my addresses be added but not answered', async () => {
    const s = stub({ 'CalendarEvent/itip': (a) => view({ me: null, applied: a['apply'] === true }) });
    render(() => <EventCard accountId="a" emailId="e" client={s} zone="UTC" />);
    expect(await screen.findByText(/does not list any of your addresses/)).toBeInTheDocument();
    expect(within(card()).queryByRole('button', { name: 'Accept' })).not.toBeInTheDocument();
    fireEvent.click(within(card()).getByRole('button', { name: 'Add to calendar' }));
    await waitFor(() => expect(s.of('CalendarEvent/itip').some((a) => a['apply'] === true)).toBe(true));
  });

  it('offers no action for a forged update', async () => {
    const s = stub({ 'CalendarEvent/itip': () => view({ outcome: 'notOrganizer', sender: 'mallory@example.net', eventId: 'ev1' }) });
    render(() => <EventCard accountId="a" emailId="e" client={s} zone="UTC" />);
    expect(await screen.findByText('The sender is not the stored organizer. Nothing will be changed.')).toBeInTheDocument();
    expect(within(card()).queryAllByRole('button')).toHaveLength(0);
  });

  it('removes a cancelled meeting only on a click', async () => {
    let removed = false;
    const s = stub({
      'CalendarEvent/itip': (a) => removed
        ? view({ method: 'CANCEL', outcome: 'unknownEvent' })
        : view({ method: 'CANCEL', outcome: 'cancel', eventId: 'ev1', applied: a['apply'] === true, event: { ...view().event, status: 'cancelled' } }),
      'CalendarEvent/set': (a) => { removed = true; return { accountId: 'a', destroyed: a['destroy'] }; },
    });
    render(() => <EventCard accountId="a" emailId="e" client={s} zone="UTC" />);
    expect(await screen.findByText('The organizer cancelled this event.')).toBeInTheDocument();
    expect(within(card()).getByText('Cancelled meeting')).toBeInTheDocument();
    expect(s.of('CalendarEvent/set')).toHaveLength(0);
    fireEvent.click(within(card()).getByRole('button', { name: 'Remove from calendar' }));
    await waitFor(() => expect(s.of('CalendarEvent/set')).toEqual([{ accountId: 'a', destroy: ['ev1'] }]));
    expect(s.of('CalendarEvent/itip').filter((a) => a['apply'] === true)).toHaveLength(1);
    expect(await within(card()).findByText('Removed from your calendar.')).toBeInTheDocument();
  });

  it('shows an organizer who answered what, and records it on a click', async () => {
    const reply = view({
      method: 'REPLY', sender: 'pat@example.net', outcome: 'reply', answer: 'accepted', me: null, eventId: 'ev9',
      event: { ...view().event, participants: { [ME]: { email: ME, role: 'organizer' }, 'pat@example.net': { name: 'Pat', email: 'pat@example.net', role: 'attendee', participationStatus: 'accepted' } } },
    });
    const s = stub({ 'CalendarEvent/itip': (a) => ({ ...reply, applied: a['apply'] === true }) });
    render(() => <EventCard accountId="a" emailId="e" client={s} zone="UTC" />);
    expect(await screen.findByText('⁨Pat⁩ accepted.')).toBeInTheDocument();
    expect(within(card()).getByText('Meeting response')).toBeInTheDocument();
    fireEvent.click(within(card()).getByRole('button', { name: 'Update attendee status' }));
    await waitFor(() => expect(s.of('CalendarEvent/itip').some((a) => a['apply'] === true)).toBe(true));
  });

  it('shows a counter-proposal without applying it', async () => {
    const s = stub({ 'CalendarEvent/itip': () => view({ method: 'COUNTER', sender: 'pat@example.net', outcome: 'unsupported', me: null }) });
    render(() => <EventCard accountId="a" emailId="e" client={s} zone="UTC" />);
    expect(await screen.findByText(/proposed a new time/)).toBeInTheDocument();
    expect(within(card()).queryAllByRole('button')).toHaveLength(0);
  });

  it('adds one or all events of a published file', async () => {
    const ev = (n: number) => ({ uid: `pub-${n}`, title: `Talk ${n}`, start: `2026-11-1${n}T09:00:00`, timeZone: 'UTC', duration: 'PT30M' });
    const s = stub({
      'CalendarEvent/itip': () => view({
        method: 'PUBLISH', outcome: 'publish', me: null, todos: 1, event: ev(1),
        items: [{ event: ev(1), eventId: null, calendarId: null }, { event: ev(2), eventId: null, calendarId: null }, { event: ev(3), eventId: 'held', calendarId: 'c1' }],
      }),
    });
    render(() => <EventCard accountId="a" emailId="e" client={s} zone="UTC" />);
    const region = await screen.findByRole('region', { name: 'Calendar event' });
    expect(within(region).getByText('3 calendar events')).toBeInTheDocument();
    expect(within(region).getByText('In your calendar')).toBeInTheDocument();
    expect(within(region).getByText('The file also holds 1 task, which is not added.')).toBeInTheDocument();
    await waitFor(() => expect(within(region).getByRole('combobox')).toBeInTheDocument());
    fireEvent.click(within(region).getAllByRole('button', { name: 'Add' })[1]!);
    await waitFor(() => expect(s.of('CalendarEvent/itip').filter((a) => a['apply'])).toHaveLength(1));
    expect(s.of('CalendarEvent/itip').find((a) => a['apply'])).toMatchObject({ uids: ['pub-2'], calendarId: 'c1' });
    await waitFor(() => expect(within(region).getByRole('button', { name: 'Add all (2)' })).toBeEnabled());
    fireEvent.click(within(region).getByRole('button', { name: 'Add all (2)' }));
    await waitFor(() => expect(s.of('CalendarEvent/itip').filter((a) => a['apply'])).toHaveLength(2));
    expect(s.of('CalendarEvent/itip').filter((a) => a['apply'])[1]).not.toHaveProperty('uids');
  });

  it('marks own events that overlap the invitation in the day agenda', async () => {
    const s = stub({
      'CalendarEvent/itip': () => view(),
      'CalendarEvent/expand': () => ({ accountId: 'a', list: [
        { eventId: 'x1', uid: 'other', title: 'Standup', calendarId: 'c2', showWithoutTime: false, instanceStart: '2026-11-03T10:30:00Z', instanceEnd: '2026-11-03T11:00:00Z', status: 'confirmed', freeBusyStatus: 'busy' },
        { eventId: 'x2', uid: 'other2', title: 'Lunch', calendarId: 'c2', showWithoutTime: false, instanceStart: '2026-11-03T11:00:00Z', instanceEnd: '2026-11-03T11:30:00Z', status: 'confirmed', freeBusyStatus: 'busy' },
        { eventId: 'self', uid: 'mtg@example.com', title: 'Weekly sync', calendarId: 'c1', showWithoutTime: false, instanceStart: '2026-11-03T10:00:00Z', instanceEnd: '2026-11-03T11:00:00Z', status: 'confirmed', freeBusyStatus: 'busy' },
      ] }),
    });
    render(() => <EventCard accountId="a" emailId="e" client={s} zone="UTC" />);
    expect(await screen.findByText('Conflicts with Standup')).toBeInTheDocument();
    const agenda = screen.getByRole('region', { name: /Your calendar on/ });
    expect(agenda.querySelectorAll('[data-agenda="conflict"]')).toHaveLength(1);
    expect(agenda.querySelectorAll('[data-agenda="invite"]')).toHaveLength(1);
    expect(within(agenda).getByText('Lunch')).toBeInTheDocument();
    expect(within(agenda).queryByText('Weekly sync')).not.toBeInTheDocument();
  });

  it('renders the description as text with only http(s) links, and collapses long attendee lists', async () => {
    const many = Object.fromEntries(Array.from({ length: 6 }, (_, i) => [`p${i}@example.net`, { name: `Person ${i}`, email: `p${i}@example.net`, role: 'attendee', participationStatus: 'accepted' }]));
    const s = stub({ 'CalendarEvent/itip': () => view({ event: { ...view().event, description: '<img src=x onerror=alert(1)> javascript:alert(1) https://example.com/ok', participants: { ...view().event.participants, ...many } } }) });
    render(() => <EventCard accountId="a" emailId="e" client={s} zone="UTC" />);
    await screen.findByRole('heading', { name: 'Weekly sync' });
    const desc = card().querySelector('[data-card="description"]')!;
    expect(desc.querySelector('img')).toBeNull();
    expect(desc.textContent).toContain('<img src=x onerror=alert(1)>');
    expect([...desc.querySelectorAll('a')].map((a) => a.getAttribute('href'))).toEqual(['https://example.com/ok']);
    const list = within(card()).getByRole('list', { name: 'Attendees' });
    expect(within(list).getAllByRole('listitem')).toHaveLength(4);
    fireEvent.click(within(card()).getByRole('button', { name: 'Show all 8' }));
    expect(within(list).getAllByRole('listitem')).toHaveLength(8);
  });

  it('renders nothing (or the fallback) for a part the server does not read as a calendar', async () => {
    const s = stub({ 'CalendarEvent/itip': () => ({ type: 'invalidArguments', description: 'the calendar part cannot be read' }) });
    render(() => <EventCard accountId="a" emailId="e" client={s} zone="UTC" fallback={<p>raw text</p>} />);
    expect(await screen.findByText('raw text')).toBeInTheDocument();
    expect(screen.queryByRole('region')).not.toBeInTheDocument();
  });

  it('names at most two conflicting events', () => {
    const inst = (title: string) => ({ eventId: title, uid: title, title, calendarId: 'c', start: new Date(0), end: new Date(1), allDay: false, status: 'confirmed', freeBusy: 'busy' });
    expect(conflictTitles([inst('A'), inst('B')])).toBe('A and B');
    expect(conflictTitles([inst('A'), inst('B'), inst('C'), inst('D')])).toBe('A, B, and 2 more');
  });
});

// t60: the mailbox route's <Suspense> wraps the whole shell. A card that reads a
// loading resource directly suspends it: the shell is swapped for "Loading…"
// and back, and every open message frame reloads (blank body, images fetched
// twice). The card must render inside a Suspense without ever triggering it.
describe('the event card under the mailbox Suspense', () => {
  it('never shows the Suspense fallback while its invitation and calendars load', async () => {
    let releaseCalendars: () => void = () => {};
    const calendarsGate = new Promise<void>((resolve) => { releaseCalendars = resolve; });
    const s = stub({
      'CalendarEvent/itip': () => view(),
      'Calendar/get': () => ({ accountId: 'a', list: [
        { id: 'c1', name: 'Work', color: '#2563eb', role: 'default', component: 'VEVENT' },
        { id: 'c2', name: 'Home', color: '#16a34a', role: null, component: 'VEVENT' },
      ] }),
    });
    const slow = { jmap: vi.fn(async (request: JmapRequest): Promise<JmapResponse> => {
      if (request.methodCalls[0]![0] === 'Calendar/get') await calendarsGate;
      return s.jmap(request);
    }) };
    const fallbacks: number[] = [];
    const Fallback = () => { fallbacks.push(Date.now()); return <p>Shell loading</p>; };
    render(() => <Suspense fallback={<Fallback />}><p>Shell</p><EventCard accountId="a" emailId="e" client={slow} zone="UTC" /></Suspense>);
    await waitFor(() => expect(card()).toBeInTheDocument());
    expect(screen.getByText('Shell')).toBeInTheDocument();
    releaseCalendars();
    await waitFor(() => expect(screen.getByRole('combobox')).toBeInTheDocument());
    expect(fallbacks).toEqual([]);
    expect(screen.queryByText('Shell loading')).toBeNull();
  });
});
