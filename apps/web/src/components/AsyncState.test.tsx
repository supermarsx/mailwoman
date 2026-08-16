// Loading stability (t22-e10).
//
// Two properties here are easy to write a green test for and hard to write an
// HONEST one for, so each is paired with the assertion that fails when the
// property is removed:
//
//  * "no unbounded spinner" — a spinner that resolves in 200 ms and one that
//    spins until the tab closes are the same DOM. The bound is asserted by
//    advancing time PAST it and requiring the surface to have stopped claiming
//    progress, with a control just BEFORE it so a component that never shows a
//    pending state at all cannot pass.
//  * "retry recovers" — a button that re-invokes something guaranteed to fail
//    the same way is worse than no button. Solid's `lazy()` memoises the import
//    promise INCLUDING its rejection, so a retry that only calls `reset()`
//    replays the stored error without touching the network. The assertion is
//    therefore the LOADER CALL COUNT, not the presence of the button.

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@solidjs/testing-library';
import { AsyncEmpty, AsyncError, AsyncPending, createElapsed } from './AsyncState.tsx';
import { AsyncBoundary, LazyRoute } from './ErrorBoundary.tsx';
import { NetworkError } from '../api/client.ts';
import { createRoot, type JSX } from 'solid-js';

afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
});

describe('a pending state is bounded', () => {
  it('stops claiming to be loading once the bound elapses, and offers a way out', async () => {
    vi.useFakeTimers();
    const retry = vi.fn();
    render(() => <AsyncPending timeoutMs={5_000} onRetry={retry} />);

    expect(screen.getByTestId('async-pending')).toBeTruthy();

    await vi.advanceTimersByTimeAsync(5_000);

    // The whole point: past the bound it is no longer a spinner.
    expect(screen.queryByTestId('async-pending')).toBeNull();
    const alert = screen.getByRole('alert');
    expect(alert).toBeTruthy();

    fireEvent.click(screen.getByTestId('async-retry'));
    expect(retry).toHaveBeenCalledTimes(1);
  });

  it('is STILL loading just before the bound — the control', async () => {
    // Without this, "it becomes an error" also passes for a component that
    // renders the error immediately and never shows a pending state at all.
    vi.useFakeTimers();
    render(() => <AsyncPending timeoutMs={5_000} />);

    await vi.advanceTimersByTimeAsync(4_999);

    expect(screen.getByTestId('async-pending')).toBeTruthy();
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('offers no retry button when the caller gave it no way to retry', async () => {
    // A surface that genuinely cannot retry shows the failure and stops. An
    // inert button would read as a remedy.
    vi.useFakeTimers();
    render(() => <AsyncPending timeoutMs={1_000} />);
    await vi.advanceTimersByTimeAsync(1_000);

    expect(screen.getByRole('alert')).toBeTruthy();
    expect(screen.queryByTestId('async-retry')).toBeNull();
  });

  it('announces itself politely rather than swapping silently', async () => {
    render(() => <AsyncPending timeoutMs={60_000} />);
    const pending = screen.getByTestId('async-pending');
    expect(pending.getAttribute('role')).toBe('status');
    expect(pending.getAttribute('aria-live')).toBe('polite');
  });

  it('clears its timer on dispose, so an unmounted surface writes no signal', async () => {
    vi.useFakeTimers();
    const cleared = vi.spyOn(globalThis, 'clearTimeout');
    let dispose = (): void => undefined;
    createRoot((d) => {
      dispose = d;
      createElapsed(10_000);
    });
    dispose();
    expect(cleared).toHaveBeenCalled();
  });
});

describe('a failed surface says what kind of failure it was', () => {
  it('reads a transport failure as one', () => {
    render(() => <AsyncError error={new NetworkError('down')} />);
    expect(screen.getByRole('alert').textContent).toContain('Can’t reach the server');
  });

  it('falls back to the generic message for anything else', () => {
    render(() => <AsyncError error={new Error('boom')} />);
    expect(screen.getByRole('alert').textContent).toContain('Something went wrong');
  });

  it('an empty surface is neither pending nor failed', () => {
    render(() => <AsyncEmpty />);
    expect(screen.getByTestId('async-empty')).toBeTruthy();
    expect(screen.queryByRole('alert')).toBeNull();
    expect(screen.queryByTestId('async-pending')).toBeNull();
  });
});

describe('a lazy route that fails to import', () => {
  it('shows a failure, NOT a spinner that never resolves', async () => {
    const load = vi.fn(() => Promise.reject(new Error('chunk 404')));
    render(() => <LazyRoute load={load} />);

    await waitFor(() => expect(screen.getByRole('alert')).toBeTruthy());
    // The defect this lane exists to close: before, this stayed on screen for
    // the life of the tab.
    expect(screen.queryByTestId('async-pending')).toBeNull();
  });

  it('RETRY RE-IMPORTS — the loader is called again and the route recovers', async () => {
    let attempts = 0;
    const load = vi.fn(async () => {
      attempts += 1;
      if (attempts === 1) throw new Error('chunk 404');
      return { default: () => <p data-testid="arrived">module</p> };
    });

    render(() => <LazyRoute load={load} />);
    await waitFor(() => expect(screen.getByTestId('async-retry')).toBeTruthy());

    fireEvent.click(screen.getByTestId('async-retry'));

    await waitFor(() => expect(screen.getByTestId('arrived')).toBeTruthy());
    // THE load-bearing assertion. Solid memoises a lazy's rejection, so a retry
    // that only calls `reset()` leaves this at 1 and replays the same error —
    // a button that cannot succeed. Asserting the button exists, or that the
    // error re-renders, would pass for that broken version.
    expect(load).toHaveBeenCalledTimes(2);
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('renders the route without ceremony when the import succeeds', async () => {
    const load = vi.fn(async () => ({ default: () => <p data-testid="arrived">module</p> }));
    render(() => <LazyRoute load={load} />);
    await waitFor(() => expect(screen.getByTestId('arrived')).toBeTruthy());
    expect(load).toHaveBeenCalledTimes(1);
  });

  it('a fail-soft route renders nothing at all when it cannot load', async () => {
    // The UI-plugin tier's documented contract is to be invisible when it is
    // unavailable. An error card there would be the regression, not the fix.
    const load = vi.fn(() => Promise.reject(new Error('chunk 404')));
    const { container } = render(() => <LazyRoute load={load} failSoft />);

    await waitFor(() => expect(load).toHaveBeenCalled());
    await Promise.resolve();

    expect(screen.queryByRole('alert')).toBeNull();
    expect(screen.queryByTestId('async-pending')).toBeNull();
    expect(container.textContent).toBe('');
  });
});

describe('a boundary contains a surface that throws while rendering', () => {
  it('catches it instead of taking the parent down', () => {
    const Boom = (): never => {
      throw new Error('render exploded');
    };
    render(() => (
      <div>
        <p data-testid="sibling">still here</p>
        <AsyncBoundary>
          <Boom />
        </AsyncBoundary>
      </div>
    ));

    expect(screen.getByRole('alert')).toBeTruthy();
    // The failure is contained: everything outside the boundary still rendered.
    expect(screen.getByTestId('sibling')).toBeTruthy();
  });

  it('recovers when the cause is gone, and re-arms the caller first', () => {
    let broken = true;
    const onRetry = vi.fn(() => {
      broken = false;
    });
    const Sometimes = (): JSX.Element => {
      if (broken) throw new Error('not yet');
      return <p data-testid="recovered">ok</p>;
    };
    render(() => (
      <AsyncBoundary onRetry={onRetry}>
        <Sometimes />
      </AsyncBoundary>
    ));

    expect(screen.getByRole('alert')).toBeTruthy();
    fireEvent.click(screen.getByTestId('async-retry'));

    // `onRetry` must run BEFORE the subtree is rebuilt, or the rebuild replays
    // the same failure and the retry is decorative.
    expect(onRetry).toHaveBeenCalledTimes(1);
    expect(screen.getByTestId('recovered')).toBeTruthy();
  });

  it('a silent boundary contains the failure without drawing an error card', () => {
    const Boom = (): never => {
      throw new Error('quiet');
    };
    const { container } = render(() => (
      <AsyncBoundary fallback={() => null}>
        <Boom />
      </AsyncBoundary>
    ));
    expect(screen.queryByRole('alert')).toBeNull();
    expect(container.textContent).toBe('');
  });
});
