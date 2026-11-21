import {afterEach, beforeEach, expect, it, vi} from 'vitest';
import {apiFetch} from './http.ts';
import {ApiError} from './errors.ts';
import * as actions from './websocketActions.ts';
import * as configuration from './settings.ts';

const transport = {action: vi.fn()};

const json = (status: number, body: unknown) => new Response(JSON.stringify(body), {
  status, headers: {'content-type': 'application/json'},
});
beforeEach(() => {
  transport.action.mockReset();
  vi.spyOn(actions, 'websocketAction').mockImplementation(transport.action);
  vi.spyOn(configuration, 'settings').mockReturnValue({
    'realtime.actions.enabled': {value: true, source: 'global', locked: false},
    'realtime.actions.preferWebSocket': {value: true, source: 'global', locked: false},
  });
  document.cookie = 'mw_csrf=old; Path=/';
});
afterEach(() => {
  document.cookie = 'mw_csrf=; Max-Age=0; Path=/';
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

it('keeps a settings save on the action transport after a confirmed CSRF refusal', async () => {
  transport.action
    .mockResolvedValueOnce(json(403, {error: 'missing or invalid CSRF token'}))
    .mockResolvedValueOnce(json(200, {value: 'split'}));
  const fetch = vi.fn().mockImplementation(async () => {
    document.cookie = 'mw_csrf=fresh; Path=/';
    return json(200, {csrfToken: 'fresh'});
  });
  vi.stubGlobal('fetch', fetch);
  const response = await apiFetch('/api/settings/mail.reader.layout', {
    method: 'PUT', headers: {'content-type': 'application/json'}, body: '"split"',
  });
  expect(await response.json()).toEqual({value: 'split'});
  expect(transport.action).toHaveBeenCalledTimes(2);
  const retry = transport.action.mock.calls[1]!;
  expect(retry[0]).toBe('/api/settings/mail.reader.layout');
  expect(new Headers(retry[1].headers).get('x-csrf-token')).toBe('fresh');
  expect(retry[1].body).toBe('"split"');
  // The fresh session proof is the sole bootstrap HTTP read. The save itself
  // must not silently switch to fetch after its initial transport was chosen.
  expect(fetch).toHaveBeenCalledTimes(1);
  expect(fetch.mock.calls[0]![0]).toBe('/api/me');
});

it('does not fall back to HTTP after a sent action loses its acknowledgement', async () => {
  const unknown = new ApiError(503, 'Action outcome unknown', {code: 'outcomeUnknown'});
  transport.action.mockRejectedValue(unknown);
  const fetch = vi.fn();
  vi.stubGlobal('fetch', fetch);
  await expect(apiFetch('/api/account/drafts', {
    method: 'POST', headers: {'content-type': 'application/json'}, body: '{}',
  })).rejects.toMatchObject({code: 'outcomeUnknown'});
  expect(transport.action).toHaveBeenCalledTimes(1);
  expect(fetch).not.toHaveBeenCalled();
});
