// Calendar feed subscriptions (webcal / ICS URL) — the client for the server's
// webcal sync driver (`crates/mw-server/src/import_routes.rs`).
//
// The engine's `Calendar/subscribe` and `Calendar/refreshSubscription` methods
// fetch nothing: they import a feed body the caller hands them as `blob`. The
// only thing that can fetch a feed is the server (a browser is stopped by CORS,
// and the server fetches through its SSRF-hardened, egress-routed fetcher), so
// a subscription has to go through these two routes — calling the JMAP method
// with a bare URL creates an overlay that stays empty.
//
//   POST /api/calendar/subscribe {url, name?}      (`calendar_subscribe`, :432-450)
//     → the `Calendar/subscribe` result: {accountId, created:{id}, url, imported}
//   POST /api/calendar/refresh   {calendarId, url} (`calendar_refresh`, :454-476)
//     → the `Calendar/refreshSubscription` result: {accountId, calendarId, imported}
//   Failure: a non-2xx with `{error}` — 502 when the fetch or the engine call
//   failed (`upstream`, :594-597), 501 outside engine mode (`auth_engine`, :551-564).
//
// Transport injectable so the controller unit-tests without a live server.

import { withBase } from '../../api/basePath.ts';
import type { Id } from '../../api/jmap-types.ts';
import type { CalendarRefreshResponse, CalendarSubscribeResponse } from './api.ts';

export type Fetcher = (input: string, init?: RequestInit) => Promise<Response>;

const defaultFetcher: Fetcher = (input, init) => fetch(input, { credentials: 'same-origin', ...init });

/** A feed request the server refused or could not complete. */
export class FeedError extends Error {
  constructor(
    readonly status: number,
    message: string,
  ) {
    super(message);
    this.name = 'FeedError';
  }
}

async function postJson<T>(fetcher: Fetcher, path: string, body: unknown): Promise<T> {
  const res = await fetcher(withBase(path), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    let detail = '';
    try {
      const parsed = (await res.json()) as { error?: unknown };
      if (typeof parsed.error === 'string') detail = parsed.error;
    } catch {
      /* a non-JSON error body carries nothing to add */
    }
    throw new FeedError(res.status, `calendar feed request failed: ${res.status} ${detail}`.trim());
  }
  return (await res.json()) as T;
}

/** The two feed operations the controller needs. */
export interface CalendarFeeds {
  /** Fetch `url` server-side and create a read-only overlay holding its events. */
  subscribe(url: string, name?: string): Promise<CalendarSubscribeResponse>;
  /** Re-fetch `url` server-side and replace the overlay's events. */
  refresh(calendarId: Id, url: string): Promise<CalendarRefreshResponse>;
}

/** The feed client over the server's sync-driver routes. */
export function createCalendarFeeds(fetcher: Fetcher = defaultFetcher): CalendarFeeds {
  return {
    subscribe(url, name) {
      const body: { url: string; name?: string } = { url };
      if (name !== undefined && name !== '') body.name = name;
      return postJson<CalendarSubscribeResponse>(fetcher, '/api/calendar/subscribe', body);
    },
    refresh(calendarId, url) {
      return postJson<CalendarRefreshResponse>(fetcher, '/api/calendar/refresh', { calendarId, url });
    },
  };
}
