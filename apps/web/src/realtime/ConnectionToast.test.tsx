import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, fireEvent, screen } from '@solidjs/testing-library';
import { ConnectionToast } from './ConnectionToast.tsx';
import { RealtimeContext } from './context.ts';
import { AppContext } from '../state/context.ts';
import type { AppState } from '../state/store.ts';
import { createConnection } from './connection.ts';
import { createSubTabs } from './subTabs.ts';
import { createChangeReconciler } from './changes.ts';
import { createRealtimeController, type RealtimeController } from './controller.ts';
import type { WebSocketLike } from './pushClient.ts';
import { resetToasts, toasts, TOAST_DURATIONS } from '../components/toast/index.ts';

/** The shared toast store's messages ("Reconnected" is a toast, not the banner). */
const toastMessages = (): string[] => toasts().map((i) => i.message);
afterEach(() => resetToasts());

function makeController(overrides: Partial<RealtimeController> = {}): RealtimeController {
  return {
    connection: createConnection(),
    subTabs: createSubTabs(),
    reconciler: createChangeReconciler(() => undefined),
    start: vi.fn(),
    stop: vi.fn(),
    reconnect: vi.fn(),
    onStateChange: () => () => undefined,
    ...overrides,
  };
}

function renderToast(controller: RealtimeController) {
  return render(() => (
    <RealtimeContext.Provider value={controller}>
      <ConnectionToast />
    </RealtimeContext.Provider>
  ));
}

describe('ConnectionToast', () => {
  it('shows an offline banner when a live connection loses the network', () => {
    const controller = makeController();
    controller.connection.report('open', 'ws');
    controller.connection.setOffline();
    renderToast(controller);
    expect(screen.getByRole('status')).toHaveTextContent(
      'You are offline — changes sync when you reconnect',
    );
  });

  it('shows nothing once the connection is online', () => {
    const controller = makeController();
    controller.connection.report('open', 'ws');
    renderToast(controller);
    expect(screen.queryByRole('status')).toBeNull();
  });

  it('raises a "Reconnected" success toast when the socket recovers, and drops the banner', () => {
    const controller = makeController();
    controller.connection.report('open', 'ws');
    controller.connection.setOffline();
    renderToast(controller);
    controller.connection.report('open', 'ws');
    expect(screen.queryByRole('status')).toBeNull();
    expect(toasts().map((i) => [i.tone, i.message])).toEqual([['success', 'Reconnected']]);
  });

  it('withdraws "Reconnected" when the connection drops again', () => {
    const controller = makeController();
    controller.connection.report('open', 'ws');
    controller.connection.setOffline();
    renderToast(controller);
    controller.connection.report('open', 'ws');
    expect(toastMessages()).toEqual(['Reconnected']);
    controller.connection.setOffline();
    expect(toastMessages()).toEqual([]);
    expect(screen.getByRole('status')).toHaveAttribute('data-state', 'offline');
  });

  it('offers a Reconnect action while degraded and invokes the controller', () => {
    const reconnect = vi.fn();
    const controller = makeController({ reconnect });
    controller.connection.report('degraded', 'poll');
    renderToast(controller);
    expect(screen.getByRole('status')).toHaveTextContent(/paused/i);
    fireEvent.click(screen.getByRole('button', { name: 'Reconnect' }));
    expect(reconnect).toHaveBeenCalledTimes(1);
  });

  it('offers "Sign in again" for an ended session, which signs the dead session out', () => {
    const logout = vi.fn(async () => undefined);
    const reconnect = vi.fn();
    const controller = makeController({ reconnect });
    controller.connection.report('open', 'ws');
    controller.connection.setAuthExpired();
    render(() => (
      <AppContext.Provider value={{ logout } as unknown as AppState}>
        <RealtimeContext.Provider value={controller}>
          <ConnectionToast />
        </RealtimeContext.Provider>
      </AppContext.Provider>
    ));
    expect(screen.getByRole('status')).toHaveTextContent(/session expired/i);
    // Reconnecting cannot revive an ended session, so it is not offered.
    expect(screen.queryByRole('button', { name: 'Reconnect' })).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Sign in again' }));
    expect(logout).toHaveBeenCalledTimes(1);
    expect(reconnect).not.toHaveBeenCalled();
  });

  it('shows the ended-session banner without an action when no store is in scope', () => {
    const controller = makeController();
    controller.connection.report('open', 'ws');
    controller.connection.setAuthExpired();
    renderToast(controller);
    expect(screen.getByRole('status')).toHaveTextContent(/session expired/i);
    expect(screen.queryByRole('button')).toBeNull();
  });
});

// The same component against the real controller and push client, driven the
// way App.tsx drives them: `start()` when a mail session exists, `stop()` when
// it does not. Only the socket is fake.
class FakeWs implements WebSocketLike {
  static last: FakeWs | undefined;
  onopen: ((ev: unknown) => void) | null = null;
  onmessage: ((ev: { data: unknown }) => void) | null = null;
  onerror: ((ev: unknown) => void) | null = null;
  onclose: ((ev: unknown) => void) | null = null;
  constructor(public url: string) {
    FakeWs.last = this;
  }
  send(): void {}
  close(): void {}
}

function realController(): RealtimeController {
  return createRealtimeController({ push: { WebSocketImpl: FakeWs, backoff: [50] } });
}

describe('ConnectionToast across sign-in and sign-out', () => {
  afterEach(() => {
    vi.useRealTimers();
    FakeWs.last = undefined;
  });

  it('claims nothing on a signed-out page, where a signed-in one would show the banner', () => {
    // Control: the banner does render for a session whose connection is lost.
    const signedIn = realController();
    signedIn.start();
    FakeWs.last?.onopen?.({});
    signedIn.connection.setOffline();
    const first = renderToast(signedIn);
    expect(screen.getByRole('status')).toHaveTextContent(/You are offline/);
    signedIn.stop();
    first.unmount();

    // Signed out: the controller exists (the store builds it at boot) but was
    // never started. A failed request on the login screen changes nothing.
    FakeWs.last = undefined;
    const signedOut = realController();
    renderToast(signedOut);
    expect(FakeWs.last).toBeUndefined();
    expect(screen.queryByRole('status')).toBeNull();
    signedOut.connection.setOffline();
    expect(screen.queryByRole('status')).toBeNull();
  });

  it('shows the banner when a signed-in connection is lost and clears it on reconnect', () => {
    vi.useFakeTimers();
    const controller = realController();
    renderToast(controller);

    controller.start();
    expect(screen.getByRole('status')).toHaveTextContent('Connecting…');
    FakeWs.last?.onopen?.({});
    // The first connect of a session is not a recovery.
    expect(screen.queryByRole('status')).toBeNull();
    expect(toastMessages()).toEqual([]);

    controller.connection.setOffline();
    expect(screen.getByRole('status')).toHaveAttribute('data-state', 'offline');

    // The socket drops too; the client retries and the new socket opens.
    const dropped = FakeWs.last;
    dropped?.onclose?.({});
    vi.advanceTimersByTime(50);
    expect(FakeWs.last).not.toBe(dropped);
    FakeWs.last?.onopen?.({});
    expect(screen.queryByRole('status')).toBeNull();
    expect(toastMessages()).toEqual(['Reconnected']);
    vi.advanceTimersByTime(TOAST_DURATIONS.success);
    expect(toastMessages()).toEqual([]);
    controller.stop();
  });

  it('drops the banner on sign-out and stays silent afterwards', () => {
    const controller = realController();
    renderToast(controller);
    controller.start();
    FakeWs.last?.onopen?.({});
    controller.connection.setOffline();
    expect(screen.getByRole('status')).toHaveTextContent(/You are offline/);

    controller.stop();
    expect(screen.queryByRole('status')).toBeNull();
    controller.connection.setOffline();
    expect(screen.queryByRole('status')).toBeNull();
  });
});
