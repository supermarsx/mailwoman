// The mail slice's undo window as a toast (replaces the old UndoToast tests).
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { ToastRegion, resetToasts, setToastPause, toasts } from './index.ts';
import { renderWithApp, mkEmail } from '../appHarness.tsx';

describe('undo toast', () => {
  beforeEach(() => {
    localStorage.clear();
    resetToasts();
  });
  afterEach(() => {
    resetToasts();
    vi.useRealTimers();
  });

  it('renders nothing when no action is pending', () => {
    renderWithApp(() => <ToastRegion />);
    expect(screen.queryByTestId('toast')).toBeNull();
  });

  it('shows the label, the seconds left and Undo after a reversible action, and Undo reverts it', async () => {
    const { app } = renderWithApp(() => <ToastRegion />, { emails: [mkEmail('a'), mkEmail('b')] });
    await app.login({ username: 'me@example.org', password: 'p' });

    await app.pinMessage('b', true);
    expect(app.visibleMessages()[0]!.id).toBe('b');

    const shown = await screen.findByTestId('toast');
    expect(shown).toHaveTextContent('Pinned');
    expect(shown).toHaveTextContent(/\d+ s/);
    expect(shown).toHaveAttribute('role', 'status');

    fireEvent.click(screen.getByRole('button', { name: 'Undo' }));
    await waitFor(() => expect(app.visibleMessages().map((m) => m.id)).toEqual(['a', 'b']));
    await waitFor(() => expect(screen.queryByTestId('toast')).toBeNull());
    expect(app.pendingUndo()).toBeNull();
  });

  it('labels the undo-send window Cancel', async () => {
    const { app } = renderWithApp(() => <ToastRegion />, { emails: [mkEmail('a')] });
    await app.login({ username: 'me@example.org', password: 'p' });

    await app.sendMessage({ to: 'you@example.org', subject: 'Hi', htmlBody: '<p>x</p>', holdSeconds: 10 });
    expect(await screen.findByRole('button', { name: 'Cancel' })).toBeInTheDocument();
    expect(screen.getByTestId('toast')).toHaveTextContent('Message sent');
  });

  it('closing commits: the toast goes and the action stays applied', async () => {
    const { app } = renderWithApp(() => <ToastRegion />, { emails: [mkEmail('a')] });
    await app.login({ username: 'me@example.org', password: 'p' });
    await app.applyTag('a', 'work');

    fireEvent.click(screen.getByRole('button', { name: 'Close notification' }));
    await waitFor(() => expect(screen.queryByTestId('toast')).toBeNull());
    expect(app.pendingUndo()).toBeNull();
    expect(app.messages()[0]!.keywords?.['work']).toBe(true);
  });

  it('keeps the undo window open while paused, but not the undo-send hold', async () => {
    const { app } = renderWithApp(() => <ToastRegion />, { emails: [mkEmail('a')] });
    await app.login({ username: 'me@example.org', password: 'p' });
    vi.useFakeTimers();

    await app.applyTag('a', 'work');
    setToastPause('hover', true);
    vi.advanceTimersByTime(60_000);
    expect(app.pendingUndo()?.label).toBe('Label added');
    setToastPause('hover', false);
    vi.advanceTimersByTime(10_000);
    expect(app.pendingUndo()).toBeNull();
    expect(toasts()).toEqual([]);

    setToastPause('hover', true);
    await app.sendMessage({ to: 'you@example.org', subject: 'Hi', htmlBody: '<p>x</p>', holdSeconds: 3 });
    expect(app.pendingUndo()?.actionLabel).toBe('Cancel');
    // The engine sends when its hold ends; Cancel must not outlive it.
    vi.advanceTimersByTime(3_000);
    expect(app.pendingUndo()).toBeNull();
    expect(toasts()).toEqual([]);
  });
});
