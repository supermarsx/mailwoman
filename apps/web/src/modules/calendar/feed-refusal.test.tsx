// A holiday pack fetched by URL with more than one location is refused by the
// server (t31-e11): `feed_text`, crates/mw-server/src/import_routes.rs, answers
// 422 with `code: "holMultipleLocations"`. The sidebar says so and points the
// user at Import, where the locations can be picked, instead of the message for
// a feed that could not be fetched. Any other refusal keeps that message.

import { describe, it, expect } from 'vitest';
import { render, fireEvent, screen, within } from '@solidjs/testing-library';
import { CalendarApp } from './index.tsx';
import { createCalendarController } from './controller.ts';
import { createCalendarFeeds, FeedError } from './feeds.ts';
import { MOCK_IDENTITY, createMockJmap, createMockStore } from './mock.ts';

const MANY = 'This holiday file lists more than one location. Download it, then use Import and pick the locations you want.';
const FETCH_FAILED = 'The calendar at that address could not be fetched, so it was not added.';
const REFRESH_FAILED = 'The calendar feed could not be fetched. Its events were left as they were.';

/** A feed client whose every request is answered with `status` and `body`. */
function answering(status: number, body: unknown) {
  return createCalendarFeeds(() => Promise.resolve(new Response(JSON.stringify(body), { status })));
}

const REFUSED = { error: 'the holiday file lists 2 locations; import the file and pick the locations', code: 'holMultipleLocations', locations: 2 };

async function renderWith(status: number, body: unknown): Promise<void> {
  const store = createMockStore();
  // A subscribed overlay, so the Refresh button is there.
  store.calendars.push({
    ...store.calendars[0]!,
    id: 'cal-feed',
    name: 'Feed',
    role: null,
    caldavUrl: 'https://example.com/holidays.hol',
    isReadOnlyOverlay: true,
  });
  const controller = createCalendarController({
    jmap: createMockJmap(store),
    resolveAccount: () => Promise.resolve('acct-mock'),
    resolveIdentity: () => Promise.resolve(MOCK_IDENTITY),
    feeds: answering(status, body),
  });
  render(() => <CalendarApp controller={controller} />);
  await screen.findByTitle('Design review');
}

async function subscribe(): Promise<HTMLElement> {
  fireEvent.input(screen.getByLabelText('Calendar URL'), { target: { value: 'https://example.com/holidays.hol' } });
  fireEvent.click(screen.getByRole('button', { name: 'Subscribe' }));
  return screen.findByTestId('calendar-feedback');
}

describe('a refused feed', () => {
  it('keeps the code of the refusal', async () => {
    const err = await answering(422, REFUSED)
      .subscribe('https://example.com/holidays.hol')
      .then(() => null, (e: unknown) => e);
    expect(err).toBeInstanceOf(FeedError);
    expect((err as FeedError).status).toBe(422);
    expect((err as FeedError).code).toBe('holMultipleLocations');
  });

  it('a pack with more than one location says to import the file and pick', async () => {
    await renderWith(422, REFUSED);
    expect(await subscribe()).toHaveTextContent(MANY);
  });

  it('a refresh of such a pack says the same', async () => {
    await renderWith(422, REFUSED);
    fireEvent.click(screen.getByRole('button', { name: 'Refresh Feed' }));
    expect(await screen.findByTestId('calendar-feedback')).toHaveTextContent(MANY);
  });

  it('any other refusal keeps the fetch-failed message', async () => {
    await renderWith(502, { error: 'upstream fetch failed' });
    const feedback = await subscribe();
    expect(feedback).toHaveTextContent(FETCH_FAILED);
    expect(within(feedback).queryByText(MANY)).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Refresh Feed' }));
    expect(await screen.findByText(REFRESH_FAILED)).toBeInTheDocument();
  });
});
