import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen } from '@solidjs/testing-library';
import { ConnectionToast } from './ConnectionToast.tsx';
import { RealtimeContext } from './context.ts';
import { createConnection } from './connection.ts';
import { createSubTabs } from './subTabs.ts';
import { createChangeReconciler } from './changes.ts';
import type { RealtimeController } from './controller.ts';

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
  it('shows an offline banner when the connection starts offline', () => {
    renderToast(makeController());
    expect(screen.getByRole('status')).toHaveTextContent(/offline/i);
  });

  it('shows nothing once the connection is online', () => {
    const controller = makeController();
    controller.connection.report('open', 'ws');
    renderToast(controller);
    expect(screen.queryByRole('status')).toBeNull();
  });

  it('announces a transient "Reconnected" when the socket recovers', () => {
    const controller = makeController();
    renderToast(controller); // starts offline
    controller.connection.report('open', 'ws');
    expect(screen.getByRole('status')).toHaveTextContent('Reconnected');
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

  it('surfaces an auth-expired banner with a reconnect action', () => {
    const controller = makeController();
    controller.connection.setAuthExpired();
    renderToast(controller);
    expect(screen.getByRole('status')).toHaveTextContent(/session expired/i);
    expect(screen.getByRole('button', { name: 'Reconnect' })).toBeInTheDocument();
  });
});
