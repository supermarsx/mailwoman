// The admin session bar: the countdown follows the SERVER's deadlines (limits come
// from the answer, never from this file), thresholds are announced once, and the
// bar never extends anything but through `extend()`.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@solidjs/testing-library';
import { loadCatalog } from '../../../i18n';
import { markAdminSessionActive, forgetAdminSession, observeAdminResponse, adminSessionGeneration } from '../../../api/adminSession.ts';
import { SessionBar } from './SessionBar.tsx';
import { createHttpSessionBarApi, formatClock, levelFor, parseTiming, parseSessionBarMode, splitLong, type SessionBarApi, type SessionBarController, type SessionBarMode, type SessionTiming } from './sessionBar.ts';

const T0 = 1_800_000_000_000;
const sec = (): number => Math.floor(Date.now() / 1000);

/** A stand-in for the server: it holds the two deadlines and answers with fresh timing. */
function fakeServer(idleLeft: number, absoluteLeft: number, opts: { idleTimeout?: number; skew?: number } = {}) {
  const idleTimeout = opts.idleTimeout ?? 1800;
  const absoluteTimeout = 43200;
  const skew = opts.skew ?? 0;
  const state = { idle: sec() + idleLeft, absolute: sec() + absoluteLeft, signedIn: sec() - 60, alive: true };
  const timing = (): SessionTiming => ({
    idleTimeoutSeconds: idleTimeout,
    absoluteTimeoutSeconds: absoluteTimeout,
    signedInAt: state.signedIn,
    idleExpiresAt: state.idle,
    absoluteExpiresAt: state.absolute,
    expiresAt: Math.min(state.idle, state.absolute),
    serverNow: sec() + skew,
  });
  const api: SessionBarApi = {
    status: vi.fn(async () => (state.alive ? timing() : null)),
    extend: vi.fn(async () => {
      if (!state.alive) return null;
      state.idle = Math.min(sec() + skew + idleTimeout, state.absolute);
      return timing();
    }),
  };
  return { state, api };
}

let controller: SessionBarController | undefined;
function mount(api: SessionBarApi, extra: { mode?: SessionBarMode; onExpired?: () => void; isHidden?: () => boolean } = {}) {
  const onExpired = extra.onExpired ?? vi.fn();
  render(() => (
    <SessionBar
      api={api}
      mode={extra.mode ?? 'always'}
      onExpired={onExpired}
      controller={(c) => { controller = c; }}
      {...(extra.isHidden ? { isHidden: extra.isHidden } : {})}
    />
  ));
  return onExpired;
}
const flush = (ms = 0) => vi.advanceTimersByTimeAsync(ms);
const text = (): string => screen.getByTestId('admin-session-bar-text').textContent ?? '';
const announce = (): string => screen.getByTestId('admin-session-announce').textContent ?? '';

beforeEach(async () => {
  vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout', 'setInterval', 'clearInterval', 'Date'] });
  vi.setSystemTime(T0);
  await loadCatalog('admin-shell');
  history.replaceState(null, '', '/admin#/users');
});
afterEach(() => {
  cleanup();
  controller = undefined;
  forgetAdminSession();
  vi.useRealTimers();
});

describe('clock helpers', () => {
  it('formats a countdown', () => {
    expect(formatClock(1721)).toBe('28:41');
    expect(formatClock(59)).toBe('0:59');
    expect(formatClock(3725)).toBe('1:02:05');
    expect(formatClock(-4)).toBe('0:00');
    expect(splitLong(41520)).toEqual({ hours: 11, minutes: 32 });
    expect(splitLong(3599)).toBeNull();
  });
  it('maps remaining seconds to levels at the 5 and 1 minute marks', () => {
    expect([301, 300, 299, 61, 60, 59, 1, 0].map(levelFor)).toEqual(['normal', 'normal', 'warning', 'warning', 'warning', 'critical', 'critical', 'expired']);
    // 300 s exactly is still normal (the warning is "less than 5 minutes"); 60 s is still warning.
    expect(levelFor(300)).toBe('normal');
    expect(levelFor(60)).toBe('warning');
  });
  it('accepts only complete server timing, and unknown modes fall back to always', () => {
    expect(parseTiming({})).toBeNull();
    expect(parseTiming(null)).toBeNull();
    expect(parseTiming({ idleTimeoutSeconds: 1 })).toBeNull();
    expect(parseSessionBarMode('warn-only')).toBe('warn-only');
    expect(parseSessionBarMode('sometimes')).toBe('always');
    expect(parseSessionBarMode(undefined)).toBe('always');
  });
});

describe('countdown', () => {
  it('shows the idle countdown when it ends first, and the absolute limit in the details', async () => {
    const server = fakeServer(1721, 41520);
    mount(server.api);
    await flush();
    expect(text()).toBe('Idle timeout in 28:41');
    expect(screen.getByTestId('admin-session-bar').dataset['primary']).toBe('idle');
    fireEvent.click(screen.getByRole('button', { name: /Idle timeout in|Session ends in/ }));
    const details = screen.getByTestId('admin-session-bar-details');
    expect(details).toHaveTextContent('Idle timeout in28:41');
    expect(details).toHaveTextContent('Session ends in11h 32m');
    expect(details).toHaveTextContent('Signed in');
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(screen.queryByTestId('admin-session-bar-details')).toBeNull();
  });

  it('shows the absolute limit when that ends first', async () => {
    const server = fakeServer(1700, 7800);
    mount(server.api);
    await flush();
    // Idle is sooner here, so flip it: the limit is nearer than the idle deadline.
    server.state.absolute = sec() + 1500;
    await controller!.resync();
    await flush();
    expect(text()).toBe('Session ends in 25:00');
    expect(screen.getByTestId('admin-session-bar').dataset['primary']).toBe('absolute');
    server.state.absolute = sec() + 7800;
    server.state.idle = sec() + 20000;
    await controller!.resync();
    await flush();
    expect(text()).toBe('Session ends in 2h 10m');
  });

  it('ticks once a second from the wall clock', async () => {
    mount(fakeServer(1721, 41520).api);
    await flush();
    expect(text()).toBe('Idle timeout in 28:41');
    await flush(1010);
    expect(text()).toBe('Idle timeout in 28:40');
    await flush(3010);
    expect(text()).toBe('Idle timeout in 28:37');
  });

  it('keeps a short label for a phone and the full label for everything else', async () => {
    mount(fakeServer(1721, 41520).api);
    await flush();
    const button = screen.getByRole('button', { name: /Idle timeout in|Session ends in/ });
    const short = button.querySelector('span[aria-hidden="true"]');
    expect(short).toHaveTextContent('28:41');
    // Two spans with different classes: the stylesheet hides one at phone width, the other elsewhere.
    expect(screen.getByTestId('admin-session-bar-text').className).not.toBe(short!.className);
  });
});

describe('thresholds', () => {
  it('announces the 5 minute, 1 minute and expiry thresholds once, never per tick', async () => {
    const server = fakeServer(303, 41520);
    mount(server.api);
    await flush();
    const bar = screen.getByTestId('admin-session-bar');
    expect(bar.dataset['level']).toBe('normal');
    expect(announce()).toBe('');
    expect(screen.queryByRole('button', { name: 'Extend' })).toBeNull();

    await flush(4010); // 299 s left
    expect(bar.dataset['level']).toBe('warning');
    expect(announce()).toBe('Your admin session ends in less than 5 minutes.');
    expect(screen.getByRole('button', { name: 'Extend' })).toBeInTheDocument();

    // The region's text must not change between thresholds: watch it for 200 ticks.
    const seen = new Set<string>();
    const watcher = new MutationObserver(() => seen.add(announce()));
    watcher.observe(screen.getByTestId('admin-session-announce'), { childList: true, characterData: true, subtree: true });
    await flush(200_000);
    expect(bar.dataset['level']).toBe('warning');
    expect(seen.size).toBe(0);

    await flush(40_000); // 59 s left
    expect(bar.dataset['level']).toBe('critical');
    expect(announce()).toBe('Your admin session ends in less than 1 minute.');
    await flush(60_000);
    expect(announce()).toBe('Your admin session has expired.');
    watcher.disconnect();
  });

  it('extends through the server only, and the countdown restarts from the server\'s answer', async () => {
    const server = fakeServer(200, 41520);
    mount(server.api);
    await flush();
    expect(screen.getByTestId('admin-session-bar').dataset['level']).toBe('warning');
    fireEvent.click(screen.getByRole('button', { name: 'Extend' }));
    await flush();
    expect(server.api.extend).toHaveBeenCalledTimes(1);
    expect(text()).toBe('Idle timeout in 30:00');
    expect(screen.getByTestId('admin-session-bar').dataset['level']).toBe('normal');
    expect(announce()).toBe('');
  });

  it('offers no extension when the absolute limit is the one about to end the session', async () => {
    const server = fakeServer(1500, 200);
    mount(server.api);
    await flush();
    expect(screen.getByTestId('admin-session-bar').dataset['level']).toBe('warning');
    expect(text()).toBe('Session ends in 3:20');
    expect(screen.queryByRole('button', { name: 'Extend' })).toBeNull();
  });

  it('never extends past the cap: the server clips it and the bar shows what it says', async () => {
    const server = fakeServer(100, 250);
    mount(server.api);
    await flush();
    fireEvent.click(screen.getByRole('button', { name: 'Extend' }));
    await flush();
    // The idle deadline moved to the cap (250 s), not to 30 minutes, and the cap is now what ends the session.
    expect(text()).toBe('Session ends in 4:10');
    expect(screen.queryByRole('button', { name: 'Extend' })).toBeNull();
  });
});

describe('server time and the local clock', () => {
  it('corrects for a local clock that is an hour fast through serverNow', async () => {
    // The server says 1800 s remain; this machine's clock is an hour ahead of the server's.
    const server = fakeServer(1800, 41520, { skew: -3600 });
    server.state.idle = sec() - 3600 + 1800;
    server.state.absolute = sec() - 3600 + 41520;
    mount(server.api);
    await flush();
    expect(text()).toBe('Idle timeout in 30:00');
    expect(screen.getByTestId('admin-session-bar').dataset['level']).toBe('normal');
    await flush(10_010);
    expect(text()).toBe('Idle timeout in 29:50');
  });

  it('survives a suspend: waking after the deadline asks the server and shows expiry', async () => {
    const server = fakeServer(1500, 41520);
    const onExpired = mount(server.api);
    await flush();
    server.state.alive = false; // the session ended while the laptop slept
    vi.setSystemTime(T0 + 1_700_000); // the clock jumps; no timer ran meanwhile
    await flush(1010);
    expect(server.api.status).toHaveBeenCalledTimes(2);
    expect(screen.getByText('Session expired — sign in again')).toBeInTheDocument();
    expect(onExpired).not.toHaveBeenCalled();
  });

  it('after a suspend the session another tab kept alive is not reported as expired', async () => {
    const server = fakeServer(1500, 41520);
    const onExpired = mount(server.api);
    await flush();
    vi.setSystemTime(T0 + 1_700_000);
    server.state.idle = sec() + 1700;
    server.state.absolute = sec() + 30000;
    await flush(1010);
    expect(screen.queryByText('Session expired — sign in again')).toBeNull();
    expect(text()).toMatch(/^Idle timeout in 28:/);
    expect(onExpired).not.toHaveBeenCalled();
  });

  it('counts an accepted admin request as activity (the server moved the idle deadline)', async () => {
    const server = fakeServer(600, 41520);
    mount(server.api);
    await flush();
    markAdminSessionActive();
    expect(text()).toBe('Idle timeout in 10:00');
    // The status read itself is not activity; an ordinary request is.
    observeAdminResponse('/admin/session/status', 200, adminSessionGeneration());
    expect(text()).toBe('Idle timeout in 10:00');
    observeAdminResponse('/admin/me', 200, adminSessionGeneration());
    await flush(1010);
    expect(text()).toBe('Idle timeout in 29:59');
  });
});

describe('hidden tab', () => {
  it('stops ticking while hidden and resyncs from the server when shown again', async () => {
    const server = fakeServer(1721, 41520);
    let hidden = false;
    mount(server.api, { isHidden: () => hidden });
    await flush();
    expect(text()).toBe('Idle timeout in 28:41');
    hidden = true;
    document.dispatchEvent(new Event('visibilitychange'));
    const calls = vi.mocked(server.api.status).mock.calls.length;
    await flush(10_000);
    expect(text()).toBe('Idle timeout in 28:41'); // no timer ran
    // Meanwhile another tab used the session.
    server.state.idle = sec() + 1800;
    hidden = false;
    document.dispatchEvent(new Event('visibilitychange'));
    await flush();
    expect(vi.mocked(server.api.status).mock.calls.length).toBe(calls + 1);
    expect(text()).toBe('Idle timeout in 30:00');
    await flush(1010);
        expect(text()).toBe('Idle timeout in 29:59');
  });
});

describe('expiry', () => {
  it('shows the expired message, then returns to sign-in while the address keeps the route', async () => {
    const server = fakeServer(3, 41520);
    const onExpired = mount(server.api);
    await flush();
    server.state.alive = false;
    await flush(4010);
    expect(screen.getByText('Session expired — sign in again')).toBeInTheDocument();
    expect(screen.getByTestId('admin-session-bar').dataset['level']).toBe('expired');
    expect(onExpired).not.toHaveBeenCalled();
    await flush(4100);
    expect(onExpired).toHaveBeenCalledTimes(1);
    // The sign-in gate takes the same address: the route to return to is still there.
    expect(location.hash).toBe('#/users');
  });

  it('the sign-in button leaves at once', async () => {
    const server = fakeServer(2, 41520);
    const onExpired = mount(server.api);
    await flush();
    server.state.alive = false;
    await flush(3010);
    fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));
    expect(onExpired).toHaveBeenCalledTimes(1);
  });
});

describe('preference modes', () => {
  it('warn-only draws nothing until the warning window, then draws the bar', async () => {
    const server = fakeServer(302, 41520);
    mount(server.api, { mode: 'warn-only' });
    await flush();
    expect(screen.queryByTestId('admin-session-bar')).toBeNull();
    await flush(3010);
    expect(screen.getByTestId('admin-session-bar').dataset['level']).toBe('warning');
    expect(announce()).toContain('less than 5 minutes');
  });

  it('hidden draws nothing and asks the server for nothing', async () => {
    const server = fakeServer(100, 41520);
    mount(server.api, { mode: 'hidden' });
    await flush(5000);
    expect(screen.queryByTestId('admin-session-bar')).toBeNull();
    expect(server.api.status).not.toHaveBeenCalled();
  });
});

describe('http client', () => {
  afterEach(() => vi.unstubAllGlobals());
  const body = { idleTimeoutSeconds: 1800, absoluteTimeoutSeconds: 43200, signedInAt: 1, idleExpiresAt: 2, absoluteExpiresAt: 3, expiresAt: 2, serverNow: 1 };
  it('reads the non-refreshing status route, and the ordinary session route to extend', async () => {
    const fetchMock = vi.fn(async () => new Response(JSON.stringify(body), { status: 200 }));
    vi.stubGlobal('fetch', fetchMock);
    const api = createHttpSessionBarApi('');
    expect((await api.status())?.idleTimeoutSeconds).toBe(1800);
    await api.extend();
    const urls = fetchMock.mock.calls.map((c) => String((c as unknown[])[0]));
    expect(urls[0]).toMatch(/\/admin\/session\/status$/);
    expect(urls[1]).toMatch(/\/admin\/session$/);
    // Only the look-without-touching read carries the peek header.
    const peeks = fetchMock.mock.calls.map((c) => new Headers(((c as unknown[])[1] as RequestInit).headers).get('x-mw-session-peek'));
    expect(peeks).toEqual(['1', null]);
  });
  it('answers null for an ended session and refuses a body without timing', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response('{}', { status: 401 })));
    expect(await createHttpSessionBarApi('').status()).toBeNull();
    vi.stubGlobal('fetch', vi.fn(async () => new Response('{"username":"a"}', { status: 200 })));
    await expect(createHttpSessionBarApi('').status()).rejects.toThrow();
  });
});
