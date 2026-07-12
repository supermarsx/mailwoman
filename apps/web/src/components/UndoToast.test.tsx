import { describe, it, expect, beforeEach } from 'vitest';
import { screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { UndoToast } from './UndoToast.tsx';
import { renderWithApp, mkEmail } from './appHarness.tsx';

describe('UndoToast', () => {
  beforeEach(() => localStorage.clear());

  it('renders nothing when no action is pending', () => {
    renderWithApp(() => <UndoToast />);
    expect(screen.queryByRole('status')).toBeNull();
  });

  it('shows the label + Undo button after a reversible action, and undo reverts it', async () => {
    const { app } = renderWithApp(() => <UndoToast />, { emails: [mkEmail('a'), mkEmail('b')] });
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });

    await app.pinMessage('b', true);
    expect(app.visibleMessages()[0]!.id).toBe('b');

    const toast = await screen.findByRole('status');
    expect(toast).toHaveTextContent('Pinned');
    const undo = screen.getByRole('button', { name: 'Undo' });

    fireEvent.click(undo);
    await waitFor(() => expect(app.visibleMessages().map((m) => m.id)).toEqual(['a', 'b']));
    await waitFor(() => expect(screen.queryByRole('status')).toBeNull());
  });

  it('labels the undo-send window Cancel', async () => {
    const { app } = renderWithApp(() => <UndoToast />, { emails: [mkEmail('a')] });
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });

    await app.sendMessage({ to: 'you@example.org', subject: 'Hi', htmlBody: '<p>x</p>', holdSeconds: 10 });
    expect(await screen.findByRole('button', { name: 'Cancel' })).toBeInTheDocument();
    expect(screen.getByRole('status')).toHaveTextContent('Message sent');
  });

  it('dismiss closes the toast without reverting', async () => {
    const { app } = renderWithApp(() => <UndoToast />, { emails: [mkEmail('a')] });
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
    await app.applyTag('a', 'work');

    fireEvent.click(screen.getByRole('button', { name: 'Dismiss' }));
    await waitFor(() => expect(screen.queryByRole('status')).toBeNull());
    // The tag stays applied — dismiss commits, it does not undo.
    expect(app.messages()[0]!.keywords?.['work']).toBe(true);
  });
});
