// t12 conformance (plan #7 + #15 web): the side-by-side conflict resolver, the
// distinct Schedule view, and the attendee ROLE/CUTYPE picker. These drive the
// real controller over the in-memory mock (the same discipline as calendar.test).

import { describe, it, expect } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { CalendarApp } from './index.tsx';
import { EventEditor } from './EventEditor.tsx';
import { createCalendarController, type CalendarController } from './controller.ts';
import { createMockStore, createMockJmap, type MockStore } from './mock.ts';
import { dateToLocal } from './datetime.ts';
import {
  attendeeRoleToRoles,
  participantRole,
  participantCutype,
  type ParticipantExt,
} from './types.ts';
import type { CalendarEvent } from '../../api/pim-types.ts';

function makeController(store: MockStore): CalendarController {
  return createCalendarController({
    jmap: createMockJmap(store),
    resolveAccount: () => Promise.resolve('acct-mock'),
  });
}

async function renderApp(store = createMockStore()): Promise<{ store: MockStore; controller: CalendarController }> {
  const controller = makeController(store);
  render(() => <CalendarApp controller={controller} />);
  await screen.findByTitle('Design review');
  return { store, controller };
}

/** A minimal full CalendarEvent for hand-built stores. */
function ev(over: Partial<CalendarEvent> & Pick<CalendarEvent, 'id' | 'calendarId' | 'title' | 'start'>): CalendarEvent {
  return {
    uid: over.id, description: '', locations: [], timeZone: 'Europe/London', duration: 'PT1H',
    showWithoutTime: false, recurrenceRules: [], recurrenceOverrides: {}, excludedRecurrenceDates: [],
    status: 'confirmed', priority: 0, freeBusyStatus: 'busy', participants: {}, alerts: {},
    sequence: 0, etag: null, ...over,
  };
}

describe('conflict resolver', () => {
  it('surfaces a resolve button when conflicts exist and opens a side-by-side dialog', async () => {
    const { controller } = await renderApp();
    // Lunch (12:00–13:00) and Design review (12:30–13:30) overlap → conflicts.
    await waitFor(() => expect(controller.conflicts().length).toBeGreaterThan(0));
    const openBtn = screen.getByRole('button', { name: /Resolve .* conflict/i });
    fireEvent.click(openBtn);
    const dialog = await screen.findByRole('dialog', { name: 'Resolve conflicts' });
    expect(dialog).toBeInTheDocument();
    // Two side panels compared side by side.
    expect(screen.getByTestId('resolver-earlier')).toBeInTheDocument();
    expect(screen.getByTestId('resolver-later')).toBeInTheDocument();
    // The free/busy grid (wiring queryFreeBusy) renders.
    expect(await screen.findByTestId('freebusy-grid')).toBeInTheDocument();
  });

  it('renders busy cells in the free/busy grid from queryFreeBusy', async () => {
    await renderApp();
    fireEvent.click(screen.getByRole('button', { name: /Resolve .* conflict/i }));
    await screen.findByTestId('freebusy-grid');
    // Design review's attendees are busy during it → at least one cell is Busy.
    await waitFor(() => expect(screen.getAllByLabelText(/: Busy$/).length).toBeGreaterThan(0));
  });

  it('reschedules the later event past the earlier one', async () => {
    const { controller } = await renderApp();
    fireEvent.click(screen.getByRole('button', { name: /Resolve .* conflict/i }));
    await screen.findByRole('dialog', { name: 'Resolve conflicts' });
    const before = controller.masterById('ev-review')!.start;
    fireEvent.click(screen.getByRole('button', { name: 'Reschedule later event' }));
    // Design review (later, 12:30) moves to Lunch's end (13:00).
    await waitFor(() => {
      const after = controller.masterById('ev-review')!.start;
      expect(after).not.toBe(before);
      expect(after.slice(11, 16)).toBe('13:00');
    });
  });

  it('double-book marks the later event free so it no longer counts as busy', async () => {
    const { controller } = await renderApp();
    fireEvent.click(screen.getByRole('button', { name: /Resolve .* conflict/i }));
    await screen.findByRole('dialog', { name: 'Resolve conflicts' });
    fireEvent.click(screen.getByRole('button', { name: 'Double-book (mark free)' }));
    await waitFor(() => expect(controller.masterById('ev-review')!.freeBusyStatus).toBe('free'));
  });
});

describe('schedule view (distinct from agenda)', () => {
  it('renders a distinct schedule feed, not the agenda list', async () => {
    const { controller } = await renderApp();
    fireEvent.click(screen.getByRole('tab', { name: 'Schedule' }));
    expect(controller.view()).toBe('schedule');
    // The schedule feed has its own container; the agenda list is NOT rendered.
    expect(await screen.findByTestId('schedule-view')).toBeInTheDocument();
    expect(screen.queryByRole('list', { name: 'Agenda' })).toBeNull();

    fireEvent.click(screen.getByRole('tab', { name: 'Agenda' }));
    expect(controller.view()).toBe('agenda');
    expect(await screen.findByRole('list', { name: 'Agenda' })).toBeInTheDocument();
    expect(screen.queryByTestId('schedule-view')).toBeNull();
  });

  it('shows a free-time gap row between spaced same-day events', async () => {
    const today = new Date();
    const y = today.getFullYear(), m = today.getMonth(), d = today.getDate();
    const store = createMockStore();
    store.events = [
      ev({ id: 'a', calendarId: 'cal-personal', title: 'Morning block', start: dateToLocal(new Date(y, m, d, 9, 0)) }),
      ev({ id: 'b', calendarId: 'cal-personal', title: 'Afternoon block', start: dateToLocal(new Date(y, m, d, 14, 0)) }),
    ];
    const controller = makeController(store);
    render(() => <CalendarApp controller={controller} />);
    await screen.findByRole('button', { name: 'New event' });
    fireEvent.click(screen.getByRole('tab', { name: 'Schedule' }));
    await screen.findByTestId('schedule-view');
    // 10:00 → 14:00 = a 4h free gap between the two blocks.
    await waitFor(() => expect(screen.getByTestId('schedule-gap')).toHaveTextContent(/free/i));
  });
});

describe('attendee role / cutype picker', () => {
  it('maps a UI role to the JSCalendar roles JSMap and round-trips it', () => {
    expect(attendeeRoleToRoles('chair')).toEqual({ chair: true, attendee: true });
    expect(attendeeRoleToRoles('optional')).toEqual({ attendee: true, optional: true });
    expect(participantRole({ name: '', email: '', role: 'attendee', participationStatus: 'needs-action', expectReply: true, roles: { chair: true } } as ParticipantExt)).toBe('chair');
    expect(participantCutype({ name: '', email: '', role: 'attendee', participationStatus: 'needs-action', expectReply: true, kind: 'room' } as ParticipantExt)).toBe('room');
    // Legacy fallback (no roles JSMap): organizer → chair.
    expect(participantRole({ name: '', email: '', role: 'organizer', participationStatus: 'accepted', expectReply: false } as ParticipantExt)).toBe('chair');
  });

  it('displays the role + cutype pickers bound to an existing attendee', async () => {
    const store = createMockStore();
    const controller = makeController(store);
    await controller.load();
    const event = ev({
      id: 'ev-r', calendarId: 'cal-work', title: 'Planning', start: dateToLocal(new Date()),
      participants: {
        a0: { name: 'Ann', email: 'ann@example.com', role: 'attendee', participationStatus: 'accepted', expectReply: true, roles: { chair: true, attendee: true }, kind: 'individual' } as ParticipantExt,
      },
    });
    render(() => <EventEditor controller={controller} event={event} onClose={() => {}} />);
    const roleSel = screen.getByLabelText(/^Role for/) as HTMLSelectElement;
    expect(roleSel.value).toBe('chair');
    const cutypeSel = screen.getByLabelText(/^Type for/) as HTMLSelectElement;
    expect(cutypeSel.value).toBe('individual');
    // Reply status badge is shown.
    expect(screen.getByText('Accepted')).toBeInTheDocument();
  });

  it('writes roles + kind when a new attendee is added and saved', async () => {
    const { store, controller } = await renderApp();
    fireEvent.click(screen.getByRole('button', { name: 'New event' }));
    await screen.findByRole('dialog', { name: 'New event' });
    fireEvent.input(screen.getByLabelText('Title'), { target: { value: 'Sync' } });
    fireEvent.input(screen.getByLabelText('Add attendee'), { target: { value: 'zed@example.com' } });
    fireEvent.click(screen.getByRole('button', { name: 'Add' }));
    fireEvent.change(screen.getByLabelText(/^Role for/), { target: { value: 'optional' } });
    fireEvent.change(screen.getByLabelText(/^Type for/), { target: { value: 'room' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(controller.masters().some((mm) => mm.title === 'Sync')).toBe(true));
    const created = store.events.find((e) => e.title === 'Sync')!;
    const part = Object.values(created.participants).find((p) => p.email === 'zed@example.com') as ParticipantExt;
    expect(part.roles).toEqual({ attendee: true, optional: true });
    expect(part.kind).toBe('room');
  });
});
