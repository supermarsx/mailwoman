import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { createNewVersionNotice, MIN_CHECK_GAP_MS, POLL_MS, SNOOZE_MS, type NewVersionDeps } from './newVersionNotice.ts';

function memory(): Pick<Storage, 'getItem' | 'setItem'> {
  const m = new Map<string, string>();
  return { getItem: (k) => m.get(k) ?? null, setItem: (k, v) => void m.set(k, v) };
}

let clock = 0;
let server = 'v1';
let active = true;
let handlers: { reload: () => void; later: () => void } | null = null;
const show = vi.fn((h: { reload: () => void; later: () => void }) => { handlers = h; });
const hide = vi.fn();

function make(over: Partial<NewVersionDeps> = {}) {
  return createNewVersionNotice({
    fetchId: async () => server,
    active: () => active,
    show,
    hide,
    now: () => clock,
    random: () => 0.5,
    storage: memory(),
    ...over,
  });
}

beforeEach(() => { clock = 1_000_000; server = 'v1'; active = true; handlers = null; show.mockClear(); hide.mockClear(); });
afterEach(() => vi.useRealTimers());

it('records the first build and shows nothing while the server still serves it', async () => {
  const n = make();
  await n.check(true);
  expect(n.booted()).toBe('v1');
  clock += MIN_CHECK_GAP_MS;
  await n.check();
  expect(show).not.toHaveBeenCalled();
});

it('shows the toast when the served build differs, once', async () => {
  const n = make();
  await n.check(true);
  server = 'v2';
  clock += MIN_CHECK_GAP_MS;
  await n.check();
  clock += MIN_CHECK_GAP_MS;
  await n.check();
  await n.check(true);
  expect(show).toHaveBeenCalledTimes(1);
});

it('does nothing for an unauthenticated or opted-out session', async () => {
  const fetchId = vi.fn(async () => 'v1');
  const n = make({ fetchId, active: () => false });
  await n.check(true);
  expect(fetchId).not.toHaveBeenCalled();
  expect(n.booted()).toBeNull();
});

it('Later snoozes that build for an hour, then notifies again', async () => {
  const n = make();
  await n.check(true);
  server = 'v2';
  await n.check(true);
  handlers!.later();
  clock += SNOOZE_MS - 1;
  await n.check(true);
  expect(show).toHaveBeenCalledTimes(1);
  clock += 2;
  await n.check(true);
  expect(show).toHaveBeenCalledTimes(2);
});

it('a snooze is remembered per tab across a controller restart, and a newer build is not snoozed', async () => {
  const storage = memory();
  const a = make({ storage });
  await a.check(true);
  server = 'v2';
  await a.check(true);
  handlers!.later();
  const b = make({ storage });
  server = 'v1';
  await b.check(true);
  server = 'v2';
  await b.check(true);
  expect(show).toHaveBeenCalledTimes(1);
  server = 'v3';
  await b.check(true);
  expect(show).toHaveBeenCalledTimes(2);
});

it('survives unusable storage', async () => {
  const storage = { getItem: () => { throw new Error('blocked'); }, setItem: () => { throw new Error('blocked'); } };
  const n = make({ storage });
  await n.check(true);
  server = 'v2';
  await n.check(true);
  handlers!.later();
  expect(show).toHaveBeenCalledTimes(1);
});

it('removes the toast when the server goes back to the booted build', async () => {
  const n = make();
  await n.check(true);
  server = 'v2';
  await n.check(true);
  server = 'v1';
  await n.check(true);
  expect(hide).toHaveBeenCalledTimes(1);
});

it('ignores failed checks', async () => {
  const n = make({ fetchId: async () => null });
  await n.check(true);
  expect(n.booted()).toBeNull();
  expect(show).not.toHaveBeenCalled();
});

it('checks on focus, visibility and online, and on a jittered poll, until stopped', async () => {
  vi.useFakeTimers();
  const fetchId = vi.fn(async () => server);
  const n = make({ fetchId });
  n.start();
  await vi.advanceTimersByTimeAsync(0);
  expect(fetchId).toHaveBeenCalledTimes(1);
  server = 'v2';
  clock += MIN_CHECK_GAP_MS;
  window.dispatchEvent(new Event('focus'));
  await vi.advanceTimersByTimeAsync(0);
  expect(show).toHaveBeenCalledTimes(1);
  window.dispatchEvent(new Event('online'));
  await vi.advanceTimersByTimeAsync(POLL_MS);
  expect(fetchId.mock.calls.length).toBeGreaterThanOrEqual(4);
  n.stop();
  const calls = fetchId.mock.calls.length;
  window.dispatchEvent(new Event('focus'));
  await vi.advanceTimersByTimeAsync(POLL_MS * 2);
  expect(fetchId).toHaveBeenCalledTimes(calls);
  expect(hide).toHaveBeenCalled();
});
