import { describe, it, expect } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { CalendarApp } from './index.tsx';
import { EventEditor } from './EventEditor.tsx';
import { createCalendarController, type CalendarController } from './controller.ts';
import { createMockStore, createMockJmap, type MockStore } from './mock.ts';
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
  // Wait for the initial load + first paint of the week grid.
  await screen.findByTitle('Design review');
  return { store, controller };
}

describe('CalendarApp', () => {
  it('renders the seeded week with events and flags overlaps as conflicts', async () => {
    await renderApp();
    expect(screen.getByTitle('Lunch')).toBeInTheDocument();
    expect(screen.getByTitle('Design review')).toBeInTheDocument();
    // Lunch (12:00–13:00) and Design review (12:30–13:30) overlap.
    expect(screen.getAllByText('conflict').length).toBeGreaterThanOrEqual(2);
  });

  it('switches views (week → month → year)', async () => {
    const { controller } = await renderApp();
    fireEvent.click(screen.getByRole('tab', { name: 'Month' }));
    expect(controller.view()).toBe('month');
    expect(screen.getByRole('tab', { name: 'Month', selected: true })).toBeInTheDocument();

    fireEvent.click(screen.getByRole('tab', { name: 'Year' }));
    expect(controller.view()).toBe('year');
    // A year view renders 12 mini-months.
    expect(await screen.findByText('January')).toBeInTheDocument();
  });

  it('opens an invite and lets the user accept it', async () => {
    const { store } = await renderApp();
    fireEvent.click(screen.getByTitle('Design review'));
    // The invite bar exposes the iTIP response controls.
    expect(await screen.findByRole('button', { name: 'Accept' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Decline' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Tentative' })).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Accept' }));
    await waitFor(() =>
      expect(store.events.find((e) => e.id === 'ev-review')!.participants['me']!.participationStatus).toBe('accepted'),
    );
  });

  it('creates an event through the editor', async () => {
    const { controller } = await renderApp();
    fireEvent.click(screen.getByRole('button', { name: '+ Event' }));
    expect(await screen.findByRole('dialog', { name: 'New event' })).toBeInTheDocument();
    fireEvent.input(screen.getByLabelText('Title'), { target: { value: 'Team sync' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(controller.masters().some((m) => m.title === 'Team sync')).toBe(true));
  });

  it('toggling a calendar off hides its events', async () => {
    await renderApp();
    fireEvent.click(screen.getByLabelText('Toggle Work'));
    // Design review lives on the Work calendar.
    await waitFor(() => expect(screen.queryByTitle('Design review')).toBeNull());
    // Lunch (Personal) is still visible.
    expect(screen.getByTitle('Lunch')).toBeInTheDocument();
  });

  it('imports a holiday pack into the visible calendar', async () => {
    const { controller } = await renderApp();
    const before = controller.masters().length;
    fireEvent.change(screen.getByLabelText('Subscribe to holidays'), { target: { value: 'uk' } });
    await waitFor(() => expect(controller.masters().length).toBeGreaterThan(before));
  });
});

function recurringInvite(): CalendarEvent {
  return {
    id: 'ev-x', calendarId: 'cal-work', uid: 'ev-x', title: 'Weekly sync', description: '', locations: [],
    start: '2026-07-13T09:00:00', timeZone: 'Europe/London', duration: 'PT1H', showWithoutTime: false,
    recurrenceRules: [{ frequency: 'weekly', byDay: ['mo'], count: 5 }], recurrenceOverrides: {},
    excludedRecurrenceDates: [], status: 'confirmed', priority: 0, freeBusyStatus: 'busy',
    participants: {}, alerts: {}, sequence: 0, etag: null,
  };
}

describe('EventEditor', () => {
  it('shows the recurrence editor prefilled from a recurring master', async () => {
    const store = createMockStore();
    const controller = makeController(store);
    await controller.load();
    render(() => <EventEditor controller={controller} event={recurringInvite()} onClose={() => {}} />);
    const repeats = screen.getByRole('checkbox', { name: 'Repeats' });
    expect((repeats as HTMLInputElement).checked).toBe(true);
    expect(screen.getByText('Every week on Mon, 5 times')).toBeInTheDocument();
  });

  it('does not show invite controls for a non-invite event', async () => {
    const store = createMockStore();
    const controller = makeController(store);
    await controller.load();
    render(() => <EventEditor controller={controller} event={recurringInvite()} onClose={() => {}} />);
    expect(screen.queryByRole('button', { name: 'Accept' })).toBeNull();
  });
});
