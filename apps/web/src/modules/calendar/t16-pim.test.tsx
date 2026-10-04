// t16 PIM web (e16): quick-add (P3), category filter (P4), attachments (P5),
// webcal subscribe (P6), calendar sharing (P1). Drives the controller + editor +
// share dialog over the in-memory mock (the same surface e10 wires to the engine).

import { describe, it, expect } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { CalendarApp } from './index.tsx';
import { EventEditor } from './EventEditor.tsx';
import { ShareDialog } from './ShareDialog.tsx';
import { createCalendarController, type CalendarController } from './controller.ts';
import { createMockFeeds, createMockStore, createMockJmap, type MockStore } from './mock.ts';
import type { CalendarEventExt } from './types.ts';

function makeController(store: MockStore): CalendarController {
  return createCalendarController({
    jmap: createMockJmap(store),
    resolveAccount: () => Promise.resolve('acct-mock'),
    feeds: createMockFeeds(store),
  });
}

async function loaded(store = createMockStore()): Promise<CalendarController> {
  const c = makeController(store);
  await c.load();
  return c;
}

describe('calendar controller — quick add (P3)', () => {
  it('creates an event from a natural-language line', async () => {
    const c = await loaded();
    const before = c.masters().length;
    const id = await c.quickAdd('Dentist Tuesday 9am');
    expect(id).not.toBeNull();
    expect(c.masters().length).toBe(before + 1);
    expect(c.masters().some((m) => m.title === 'Dentist Tuesday 9am')).toBe(true);
  });

  it('reads the created id out of `created.id`', async () => {
    const store = createMockStore();
    const c = await loaded(store);
    const id = await c.quickAdd('Dentist');
    expect(typeof id).toBe('string');
    expect(store.events.some((e) => e.id === id)).toBe(true);
  });

  it('ignores an empty quick-add line', async () => {
    const c = await loaded();
    const before = c.masters().length;
    expect(await c.quickAdd('   ')).toBeNull();
    expect(c.masters().length).toBe(before);
  });
});

describe('calendar controller — category filter (P4)', () => {
  it('narrows the view to events carrying the category', async () => {
    const c = await loaded();
    await c.createEvent({ calendarId: 'cal-work', title: 'Tagged', start: '2026-07-20T10:00:00', categories: ['work'] });
    await c.createEvent({ calendarId: 'cal-work', title: 'Untagged', start: '2026-07-20T12:00:00' });

    c.setCategoryFilter('work');
    await c.load();

    expect(c.masters().some((m) => m.title === 'Tagged')).toBe(true);
    expect(c.masters().some((m) => m.title === 'Untagged')).toBe(false);
    expect(c.masters().every((m) => (m as CalendarEventExt).categories?.includes('work'))).toBe(true);

    // Clearing restores the full set.
    c.setCategoryFilter(null);
    await c.load();
    expect(c.masters().some((m) => m.title === 'Untagged')).toBe(true);
  });
});

describe('calendar controller — subscribe by URL (P6)', () => {
  it('adds a read-only overlay pinned to the source URL', async () => {
    const c = await loaded();
    const before = c.calendars().length;
    const id = await c.subscribeUrl('https://example.com/team.ics', 'Team');
    expect(typeof id).toBe('string');
    expect(c.calendars().length).toBe(before + 1);
    const added = c.calendars().find((x) => x.id === id)!;
    expect(added.name).toBe('Team');
    expect(added.isReadOnlyOverlay).toBe(true);
    expect(added.caldavUrl).toBe('https://example.com/team.ics');
  });

  it('fills the overlay from the fetched feed', async () => {
    const store = createMockStore();
    const feed = 'BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:f1\r\nSUMMARY:Feed event\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n';
    const c = createCalendarController({
      jmap: createMockJmap(store),
      resolveAccount: () => Promise.resolve('acct-mock'),
      feeds: createMockFeeds(store, () => feed),
    });
    await c.load();
    const id = await c.subscribeUrl('https://example.com/team.ics');
    expect(store.events.filter((e) => e.calendarId === id).map((e) => e.title)).toEqual(['Feed event']);
  });

  it('ignores an empty URL', async () => {
    const c = await loaded();
    const before = c.calendars().length;
    expect(await c.subscribeUrl('  ')).toBeNull();
    expect(c.calendars().length).toBe(before);
  });

  it('refresh replaces the overlay with the re-fetched feed', async () => {
    const store = createMockStore();
    let title = 'First';
    const c = createCalendarController({
      jmap: createMockJmap(store),
      resolveAccount: () => Promise.resolve('acct-mock'),
      feeds: createMockFeeds(
        store,
        () => `BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:f1\r\nSUMMARY:${title}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n`,
      ),
    });
    await c.load();
    const id = await c.subscribeUrl('https://example.com/team.ics');
    title = 'Second';
    await c.refreshSubscription(id!);
    expect(store.events.filter((e) => e.calendarId === id).map((e) => e.title)).toEqual(['Second']);
  });

  it('a feed the server cannot fetch rejects and adds no calendar', async () => {
    const store = createMockStore();
    const c = createCalendarController({
      jmap: createMockJmap(store),
      resolveAccount: () => Promise.resolve('acct-mock'),
      feeds: createMockFeeds(store, () => null),
    });
    await c.load();
    const before = store.calendars.length;
    await expect(c.subscribeUrl('https://example.com/gone.ics')).rejects.toThrow();
    expect(store.calendars.length).toBe(before);
  });
});

describe('EventEditor — categories + attachments (P4/P5)', () => {
  it('persists a category and an attachment on save', async () => {
    const c = await loaded();
    render(() => <EventEditor controller={c} event={null} onClose={() => {}} />);

    fireEvent.input(screen.getByLabelText('Title'), { target: { value: 'Kickoff' } });

    // The category input and its Add button share the accessible name "Add
    // category", so target the input by its placeholder.
    fireEvent.input(screen.getByPlaceholderText('Add a category'), { target: { value: 'project-x' } });
    fireEvent.keyDown(screen.getByPlaceholderText('Add a category'), { key: 'Enter' });
    expect(screen.getByText('project-x')).toBeInTheDocument();

    fireEvent.input(screen.getByLabelText('Attachment link'), { target: { value: 'https://drive/agenda' } });
    fireEvent.keyDown(screen.getByLabelText('Attachment link'), { key: 'Enter' });

    fireEvent.click(screen.getByRole('button', { name: 'Save' }));

    await waitFor(() => {
      const m = c.masters().find((x) => x.title === 'Kickoff') as CalendarEventExt | undefined;
      expect(m).toBeDefined();
      expect(m!.categories).toContain('project-x');
      expect(m!.attachments).toHaveLength(1);
      expect(m!.attachments![0]!.uri).toBe('https://drive/agenda');
    });
  });
});

describe('ShareDialog — calendar sharing (P1)', () => {
  it('lists an existing grant, adds one, and removes one', async () => {
    const c = await loaded();
    const workCal = c.calendars().find((x) => x.id === 'cal-work')!;
    render(() => <ShareDialog controller={c} calendar={workCal} onClose={() => {}} />);

    // Seeded grant.
    expect(screen.getByText('team@example.com')).toBeInTheDocument();

    // Add a person.
    fireEvent.input(screen.getByLabelText('Add a person'), { target: { value: 'sam@example.com' } });
    fireEvent.click(screen.getByRole('button', { name: 'Add' }));
    await waitFor(() => expect(screen.getByText('sam@example.com')).toBeInTheDocument());

    // Remove the seeded grant. The remove control's accessible name isolates the
    // (user-controlled) principal with bidi marks, so match by substring. Controls
    // disable while a share op is in flight, so wait for the row to settle first.
    const removeTeam = (): HTMLElement => screen.getByRole('button', { name: (n) => n.includes('team@example.com') });
    await waitFor(() => expect(removeTeam()).not.toBeDisabled());
    fireEvent.click(removeTeam());
    await waitFor(() => expect(screen.queryByText('team@example.com')).toBeNull());
  });
});

describe('CalendarApp — quick-add box wiring (P3)', () => {
  it('adds an event from the toolbar quick-add input', async () => {
    const store = createMockStore();
    const controller = makeController(store);
    render(() => <CalendarApp controller={controller} />);
    await screen.findByTitle('Design review');

    fireEvent.input(screen.getByLabelText('Quick add event'), { target: { value: 'Gym 6pm' } });
    fireEvent.keyDown(screen.getByLabelText('Quick add event'), { key: 'Enter' });

    await waitFor(() => expect(controller.masters().some((m) => m.title === 'Gym 6pm')).toBe(true));
  });
});
