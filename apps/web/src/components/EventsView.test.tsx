import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { AppContext } from '../state/context.ts';
import { createAppState } from '../state/store.ts';
import { makeClient, mkEmail } from './appHarness.tsx';
import { EventsNavEntry, EventsViewBar } from './EventsView.tsx';
import { EventHint, eventState, eventWhen } from './EventHint.tsx';
import { EVENTS_VIEW_ID, eventsViewCounts, listMailbox } from '../api/jmap.ts';
import { clearSettings, loadSettings, type Fetcher } from '../api/settings.ts';
import type { CalendarHint, JmapRequest, JmapResponse } from '../api/jmap-types.ts';

const json = (status: number, body: unknown): Response =>
  new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });

async function countSetting(value?: string): Promise<void> {
  const settings = value ? { 'mail.folderCounts': { source: 'default', locked: false, value } } : {};
  await loadSettings({ refresh: true, fetcher: (async () => json(200, { settings })) as Fetcher });
}

const invite: CalendarHint = { method: 'REQUEST', sequence: 0, title: 'Review', start: '2026-11-03T10:00:00', timeZone: 'Europe/Lisbon', startUtc: 1_793_700_000, endUtc: 1_793_703_600, events: 1 };

describe('event hint (t78)', () => {
  it('names what each calendar message does', () => {
    expect(eventState(invite)).toBe('invitation');
    expect(eventState({ ...invite, sequence: 3 })).toBe('updated');
    expect(eventState({ ...invite, method: 'CANCEL' })).toBe('cancelled');
    expect(eventState({ ...invite, status: 'cancelled' })).toBe('cancelled');
    expect(eventState({ ...invite, method: 'REPLY' })).toBe('reply');
    expect(eventState({ ...invite, method: 'COUNTER' })).toBe('counter');
    expect(eventState({ ...invite, method: 'PUBLISH' })).toBe('event');
    expect(eventState({ ...invite, method: null })).toBe('event');
  });

  it('shows an all-day event as its date, never shifted by a zone', () => {
    const when = eventWhen({ method: null, start: '2026-12-25', showWithoutTime: true });
    expect(when).toMatch(/25/);
    expect(when).not.toMatch(/\d{1,2}:\d{2}/);
    expect(eventWhen({ method: null })).toBe('');
  });

  it('shows a timed event with a time', () => {
    expect(eventWhen(invite)).toMatch(/\d{1,2}:\d{2}/);
  });

  it('renders nothing for a message without calendar content, a mark and the state otherwise', () => {
    const none = render(() => <span data-testid="row"><EventHint hint={null} />preview</span>);
    expect(none.getByTestId('row').textContent).toBe('preview');
    none.unmount();
    render(() => <EventHint hint={{ ...invite, method: 'CANCEL' }} />);
    const hint = screen.getByTestId('row-event-hint');
    expect(hint.dataset['eventState']).toBe('cancelled');
    expect(hint.querySelector('svg')).not.toBeNull();
  });
});

describe('Events view requests (t78)', () => {
  it('asks the list for the hint only in the Events view, with the upcoming bound when set', () => {
    const plain = listMailbox('a', 'inbox');
    expect((plain.methodCalls[1]![1] as { properties: string[] }).properties).not.toContain('mailwomanCalendar');
    const events = listMailbox('a', EVENTS_VIEW_ID, 50, {}, { calendarEndAfter: '2026-10-29T00:00:00.000Z' });
    expect(events.methodCalls[0]![1]).toMatchObject({ filter: { inMailbox: EVENTS_VIEW_ID, calendarEndAfter: '2026-10-29T00:00:00.000Z' } });
    expect((events.methodCalls[1]![1] as { properties: string[] }).properties).toContain('mailwomanCalendar');
  });

  it('counts with two id-less queries', () => {
    const req = eventsViewCounts('a');
    expect(req.methodCalls.map((c) => c[1])).toEqual([
      { accountId: 'a', filter: { inMailbox: EVENTS_VIEW_ID }, limit: 0, calculateTotal: true },
      { accountId: 'a', filter: { inMailbox: EVENTS_VIEW_ID, notKeyword: '$seen' }, limit: 0, calculateTotal: true },
    ]);
  });
});

function harness() {
  const client = makeClient({ emails: [mkEmail('e1', { mailwomanCalendar: invite, keywords: {} })] });
  const base = client.jmap;
  const sent: JmapRequest[] = [];
  client.jmap = vi.fn(async (body: JmapRequest): Promise<JmapResponse> => {
    sent.push(body);
    if (body.methodCalls[0]?.[2] === 'all') {
      return { methodResponses: [['Email/query', { accountId: 'acct1', ids: [], total: 7 }, 'all'], ['Email/query', { accountId: 'acct1', ids: [], total: 2 }, 'unread']], sessionState: 's' };
    }
    return base(body);
  }) as typeof client.jmap;
  const app = createAppState(client);
  const onSelect = vi.fn(() => void app.selectMailbox(EVENTS_VIEW_ID));
  render(() => <AppContext.Provider value={app}><EventsNavEntry active={false} onSelect={onSelect} /><EventsViewBar /></AppContext.Provider>);
  return { app, sent, onSelect };
}

describe('Events sidebar entry and filter (t78)', () => {
  beforeEach(() => { localStorage.clear(); clearSettings(); });
  afterEach(() => clearSettings());

  it('shows its counts as the folder count setting says', async () => {
    const { app } = harness();
    await app.login({ username: 'me@example.org', password: 'p' });
    await countSetting('both');
    await waitFor(() => expect(screen.getByTestId('sidebar-events').textContent).toContain('2'));
    expect(screen.getByTestId('sidebar-events').textContent).toContain('/ 7');
    await countSetting('total');
    await waitFor(() => expect(screen.getByTestId('sidebar-events').textContent).toMatch(/Events?7|7$/));
    await countSetting('none');
    await waitFor(() => expect(screen.getByTestId('sidebar-events').querySelector('.sidebar__badge')).toBeNull());
  });

  it('opens the view as a folder and keeps it across a folder refresh; Upcoming only re-queries with the bound', async () => {
    const { app, sent, onSelect } = harness();
    await app.login({ username: 'me@example.org', password: 'p' });
    expect(screen.queryByTestId('events-view-bar')).toBeNull();
    fireEvent.click(screen.getByTestId('sidebar-events'));
    expect(onSelect).toHaveBeenCalledOnce();
    await waitFor(() => expect(app.selectedMailboxId()).toBe(EVENTS_VIEW_ID));
    await waitFor(() => expect(app.messages().map((m) => m.id)).toEqual(['e1']));
    await app.refreshFolders();
    expect(app.selectedMailboxId()).toBe(EVENTS_VIEW_ID);

    const chip = screen.getByTestId('events-upcoming');
    expect(chip.getAttribute('aria-pressed')).toBe('false');
    sent.length = 0;
    fireEvent.click(chip);
    await waitFor(() => expect(chip.getAttribute('aria-pressed')).toBe('true'));
    await waitFor(() => {
      const query = sent.flatMap((r) => r.methodCalls).find((c) => c[0] === 'Email/query' && (c[1] as { filter?: { inMailbox?: string } }).filter?.inMailbox === EVENTS_VIEW_ID);
      expect((query?.[1] as { filter: Record<string, unknown> }).filter['calendarEndAfter']).toEqual(expect.any(String));
    });
  });
});
